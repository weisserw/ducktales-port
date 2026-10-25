//! Source-backed B08 contracts; see trace.md for ranges/branches and CPU derivations.
use super::*;
use crate::{
    actors::{ACTIVE, ACTOR_SLOTS, ActorContact, VISIBLE},
    collision::{Extent, Position},
    player::Action,
    status::{BLANK, Health, HealthSegment},
};
use std::collections::VecDeque;
const SUBJECT: usize = 3;
const CHILD: usize = ACTOR_SLOTS - 1;
const CONTACT: u8 = 1;
const ACTOR_Y: u8 = 0x70;
const ACTOR_X: u16 = 0x0230;
const IDENTITY: u8 = 0x42;
const SOUND_REMOVE: u8 = 0x16;
const SOUND_REACT: u8 = 0x35;
const SOUND_DISPENSE: u8 = 0x25;
const AUDIO_EMPTY: u8 = 0x88;
const AUDIO_STOP: u8 = 0xff;
const FALL_SAMPLE: u8 = 0x2d;
const RISE_SAMPLE: u8 = 0x2f;
const RIGHT_SAMPLE: u8 = 0x29;
const LEFT_SAMPLE: u8 = 0x2b;
#[derive(Default)]
struct Boundary {
    calls: Vec<Work>,
    facts: VecDeque<u8>,
    contact: bool,
    target: Option<usize>,
    exits: VecDeque<bool>,
    post_velocity: Option<i16>,
    changed_at_replace: Vec<u8>,
}
impl World for Boundary {
    fn motion(
        &mut self,
        a: &mut ActorPool,
        _: &mut Player,
        s: usize,
        m: Movement,
        p: u8,
        n: u8,
    ) -> u8 {
        self.calls.push(Work::Motion(m, p, n));
        // Horizontal wrappers do not manufacture a vertical-velocity write.
        if matches!(
            m,
            Movement::Fall | Movement::Rise | Movement::PositiveY | Movement::NegativeY
        ) && let Some(v) = self.post_velocity
        {
            a[s].vertical_velocity = v;
        }
        self.facts.pop_front().unwrap_or(0)
    }
    fn integrate(&mut self, a: &mut ActorPool, s: usize, o: Integration) -> bool {
        self.calls.push(Work::Integrate(o));
        if matches!(o, Integration::FallingY | Integration::Gravity)
            && let Some(v) = self.post_velocity
        {
            a[s].vertical_velocity = v;
        }
        // SEC/SBC negative-X survives with carry set; preserve explicit exits.
        self.exits
            .pop_front()
            .unwrap_or(o == Integration::NegativeX)
    }
    fn player_contact(&mut self, _: &mut ActorPool, _: &mut Player, _: usize) -> bool {
        self.calls.push(Work::PlayerContact);
        self.contact
    }
    fn actor_contact(&mut self, _: &mut ActorPool, _: usize) -> Option<usize> {
        self.calls.push(Work::ActorContact);
        self.target
    }
    fn replace_terrain(&mut self, _: &mut ActorPool, s: usize, c: &mut u8) {
        self.calls.push(Work::ReplaceTerrain(s));
        self.changed_at_replace.push(*c);
        *c = c.wrapping_add(1);
    }
}
struct Fixture {
    actors: ActorPool,
    player: Player,
    random: RandomState,
    status: Status,
    bits: [u8; SUPPRESSION_BYTES],
    changed: u8,
    stage_bits: u8,
    flags: u8,
    extension: bool,
    latch: u8,
    dialogue: Option<u8>,
    stage: u8,
    late: bool,
    tick: u8,
    rule: TitleRule,
    audio: crate::audio::CommandQueue,
    catalog: Catalog,
    spawn: SpawnCatalog,
    world: Boundary,
}
impl Fixture {
    fn new(handler: u8) -> Self {
        let mut actors = [ActorContact::default(); ACTOR_SLOTS];
        actors[SUBJECT] = ActorContact {
            control: crate::actors::ActorControl::from_codes(ACTIVE, 0, 0, 0),
            render: VISIBLE,
            handler,
            animation: 0x1a,
            spawn_identity: Some(IDENTITY),
            position: Position {
                world_x: ACTOR_X,
                screen_y: ACTOR_Y,
                ..Position::default()
            },
            ..ActorContact::default()
        };
        actors[0].position = actors[SUBJECT].position;
        Self {
            actors,
            player: Player::default(),
            random: RandomState {
                words: [2, 4, 6, 8],
            },
            status: Status {
                reward: [BLANK; 6].into(),
                total: [BLANK; 7].into(),
                lives: 1,
                health: Health::from_overlay(
                    [HealthSegment {
                        y: 0,
                        tile: 0,
                        depleted: 0,
                        x: 0,
                    }; 5],
                    0x80,
                    0,
                ),
                countdown: [0; 3].into(),
                divider: 0,
                dirty: 0,
                ui_pending: 0,
            },
            bits: [0; SUPPRESSION_BYTES],
            changed: 0,
            stage_bits: 0,
            flags: 0,
            extension: false,
            latch: 0,
            dialogue: None,
            stage: 0,
            late: false,
            tick: 0,
            rule: TitleRule::OrdinaryDamage,
            audio: crate::audio::CommandQueue::default(),
            catalog: Catalog {
                motion: [
                    MotionPhase::Arc,
                    MotionPhase::FallThenSlide,
                    MotionPhase::ContactTransfer,
                    MotionPhase::Arc,
                    MotionPhase::Arc,
                    MotionPhase::Arc,
                    MotionPhase::SlidePose,
                    MotionPhase::Arc,
                    MotionPhase::TouchPose,
                ],
                launch_vertical: [0x0321; TERRAIN_LAUNCH_ENTRIES],
                launch_horizontal: [0x0123; TERRAIN_LAUNCH_ENTRIES],
                recovered: [0x23; 14],
                stage_rewards: [0; 15],
                transform_animation: [0x34; 20],
                transform_handler: [0x11; 20],
                transform_profile: [0x16; 20],
                transform_y_offset: [0x18; 20],
                secondary: [0; SECONDARY_HANDLERS],
                slide_nudge: [1, -1, -1, 1],
                dispenser_direction: [0, 1, 0, 2],
                dispenser_speed: [0, 0x0100, 0, 0x0100],
                reward_sound: [0x15, 0x15, 0x15, 0x15, 0x17, 0x15, 0x37],
            },
            spawn: SpawnCatalog {
                handlers: vec![],
                templates: vec![],
                levels: vec![],
                source_levels: vec![],
                extents: [Extent::default(); 32],
                vertical_presets: [0; 32],
                horizontal_presets: [0; 32],
                suppression: vec![],
            },
            world: Boundary::default(),
        }
    }
    fn actor(&self) -> &ActorContact {
        &self.actors[SUBJECT]
    }
    fn a(&mut self) -> &mut ActorContact {
        &mut self.actors[SUBJECT]
    }
    fn invoke(&mut self, entry: u8) -> Effects {
        let mut context = Context {
            player: &mut self.player,
            random: &mut self.random,
            status: &mut self.status,
            suppression: &mut self.bits,
            changed: &mut self.changed,
            stage_bits: &mut self.stage_bits,
            reward_flags: &mut self.flags,
            stage_one_extension: &mut self.extension,
            completion_latch: &mut self.latch,
            dialogue: &mut self.dialogue,
            stage: self.stage,
            late_mode: self.late,
            composition_tick: self.tick,
            rule: self.rule,
            audio_queue: &mut self.audio,
            catalog: &self.catalog,
            spawn: &self.spawn,
        };
        match entry {
            0 => step(&mut self.actors, SUBJECT, &mut context, &mut self.world),
            1 => reaction(&mut self.actors, SUBJECT, &mut context, &mut self.world),
            2 => collect(&mut self.actors, SUBJECT, &mut context),
            3 => secondary(&mut self.actors, SUBJECT, &mut context, false),
            4 => secondary(&mut self.actors, SUBJECT, &mut context, true),
            6 => activate_terrain(&mut self.actors, SUBJECT, &mut context, &mut self.world),
            _ => {
                launch_terrain(&mut self.actors, SUBJECT, &mut context);
                Effects::default()
            }
        }
    }
    fn run(&mut self) -> Effects {
        self.invoke(0)
    }
    fn swing(&mut self) {
        self.actors[0].animation = crate::player::SWING_ANIMATION;
        {
            let animation_value = SWING_FRAME;
            self.actors[0].playback.set_frame_code(animation_value);
        };
        self.world.contact = true;
    }
    fn revealed(&mut self) {
        self.a().animation = REACTION_ANIMATION;
        {
            let animation_value = 1;
            self.a().playback.set_frame_code(animation_value);
        };
        {
            let animation_value = REVEAL_TICK;
            self.a().playback.set_tick_code(animation_value);
        };
    }
}

#[test]
fn terrain_offscreen_culls_except_two_persistent_animations() {
    for animation in [0x1a, 0x1d, 0x1e] {
        let mut f = Fixture::new(TERRAIN_HANDLER);
        f.a().render = 0;
        f.a().animation = animation;
        f.run();
        assert_eq!(
            f.actor().state_code(),
            if animation == 0x1a { 0 } else { ACTIVE }
        );
    }
}

#[test]
fn swing_requires_animation_tick_frame_and_contact() {
    for gate in 0..4 {
        let mut f = Fixture::new(TERRAIN_HANDLER);
        f.swing();
        match gate {
            0 => {
                let animation_value = 1;
                f.actors[0].playback.set_tick_code(animation_value);
            }
            1 => {
                let animation_value = 2;
                f.actors[0].playback.set_frame_code(animation_value);
            }
            2 => f.actors[0].animation = 0,
            _ => f.world.contact = false,
        };
        f.run();
        assert_eq!(f.actor().handler, TERRAIN_HANDLER);
        assert_eq!(f.changed, 0);
    }
}

#[test]
fn swing_claims_frame_before_immediate_terrain_mutation() {
    let mut f = Fixture::new(TERRAIN_HANDLER);
    f.swing();
    {
        let actor_value = f.a().state_code() | (TERRAIN_REPRESENTATION);
        f.a().set_state_code(actor_value);
    };
    f.player.pose_direction = 2;
    f.run();
    assert_eq!(f.changed, 2);
    assert_eq!(f.actor().handler, MOVING_HANDLER);
    assert_eq!(f.actor().state_code(), ACTIVE);
    assert_eq!(f.actor().direction, 2);
    assert_eq!(f.actor().vertical_velocity, 0x0321);
    assert_eq!(f.world.calls.last(), Some(&Work::ReplaceTerrain(SUBJECT)));
}

#[test]
fn already_changed_swing_returns_without_hit_fallback() {
    let mut f = Fixture::new(TERRAIN_HANDLER);
    f.swing();
    f.changed = 1;
    f.a().hit_marker = 1;
    f.run();
    assert_eq!(f.actor().handler, TERRAIN_HANDLER);
    assert_eq!(f.actor().animation, 0x1a);
}

#[test]
fn special_swing_reward_bypasses_changed_gate_and_suppresses_identity() {
    let mut f = Fixture::new(TERRAIN_HANDLER);
    f.swing();
    f.changed = 1;
    f.a().animation = STAGE_REWARD_ANIMATION;
    f.run();
    assert_eq!(f.actor().handler, RECOVERY_HANDLER);
    assert_eq!(f.bits[8], 4);
    assert_eq!(f.changed, 1);
}

#[test]
fn hit_terrain_gate_defers_when_frame_claimed_and_clears_flag_after_mutation() {
    for changed in [0, 1] {
        let mut f = Fixture::new(TERRAIN_HANDLER);
        f.a().hit_marker = 1;
        {
            let actor_value = f.a().state_code() | (TERRAIN_REPRESENTATION);
            f.a().set_state_code(actor_value);
        };
        f.changed = changed;
        f.run();
        assert_eq!(
            f.actor().state_code() & TERRAIN_REPRESENTATION,
            if changed == 0 {
                0
            } else {
                TERRAIN_REPRESENTATION
            }
        );
        assert_eq!(
            f.actor().animation,
            if changed == 0 {
                REACTION_ANIMATION
            } else {
                0x1a
            }
        );
    }
}

#[test]
fn hit_animation_twenty_two_ignores_reaction() {
    let mut f = Fixture::new(TERRAIN_HANDLER);
    f.a().hit_marker = 1;
    f.a().animation = IGNORED_HIT_ANIMATION;
    let e = f.run();
    assert_eq!(f.actor().handler, TERRAIN_HANDLER);
    assert!(e.sounds.is_empty());
}

#[test]
fn ordinary_hit_terminal_effect_preserves_high_render_bits() {
    let mut f = Fixture::new(TERRAIN_HANDLER);
    f.a().hit_marker = 1;
    f.a().render = 0xe3;
    let e = f.run();
    assert_eq!(
        (
            f.actor().handler,
            f.actor().profile,
            f.actor().animation,
            f.actor().render
        ),
        (0, 0, REACTION_ANIMATION, 0xe8)
    );
    assert_eq!(e.sounds, vec![SOUND_REMOVE]);
}

#[test]
fn launch_shared_entry_bypasses_frame_claim_and_replacement() {
    let mut f = Fixture::new(TERRAIN_HANDLER);
    f.changed = 9;
    {
        let actor_value = 0xf0;
        f.a().set_state_code(actor_value);
    };
    f.a().animation = 0x1c;
    f.invoke(5);
    assert_eq!(f.actor().state_code(), 0xf2);
    assert_eq!(f.actor().handler, MOVING_HANDLER);
    assert_eq!(f.changed, 9);
    assert!(f.world.calls.is_empty());
}

#[test]
fn motion_arc_aliases_all_five_presets_and_rise_then_horizontal() {
    for phase in [0, 3, 4, 5, 7] {
        let mut f = Fixture::new(MOVING_HANDLER);
        {
            let actor_value = ACTIVE | phase;
            f.a().set_state_code(actor_value);
        };
        f.run();
        assert_eq!(
            f.world.calls,
            vec![
                Work::Motion(Movement::Rise, RISE_SAMPLE, 1),
                Work::Motion(Movement::NegativeX, LEFT_SAMPLE, 1),
                Work::ActorContact
            ]
        );
    }
}

#[test]
fn arc_apex_resets_velocity_without_same_call_fall() {
    let mut f = Fixture::new(MOVING_HANDLER);
    f.world.post_velocity = Some(-1);
    f.run();
    assert_eq!(f.actor().vertical_velocity, 0);
    assert_eq!(f.actor().motion_flags, 1);
    assert!(
        !f.world
            .calls
            .contains(&Work::Motion(Movement::Fall, FALL_SAMPLE, 1))
    );
}

#[test]
fn arc_falling_contact_combines_vertical_and_horizontal() {
    for facts in [[CONTACT, 0], [0, CONTACT], [0, 0]] {
        let mut f = Fixture::new(MOVING_HANDLER);
        f.a().motion_flags = 1;
        f.world.facts = facts.into();
        let e = f.run();
        assert_eq!(e.sounds.contains(&SOUND_REMOVE), facts != [0, 0]);
    }
}

#[test]
fn arc_actor_contact_enters_removal_at_animation_store_without_sound() {
    let mut f = Fixture::new(MOVING_HANDLER);
    f.world.target = Some(CHILD);
    let e = f.run();
    assert_eq!(f.actor().handler, 0);
    assert_eq!(f.actor().animation, 0x1a);
    assert_eq!(e.sounds, vec![SOUND_REMOVE]);
}

#[test]
fn fall_then_slide_clear_floor_latches_and_skips_horizontal() {
    let mut f = Fixture::new(MOVING_HANDLER);
    {
        let actor_value = f.a().state_code() | (1);
        f.a().set_state_code(actor_value);
    };
    f.run();
    assert_eq!(f.actor().motion_flags, 1);
    assert_eq!(
        f.world.calls,
        vec![
            Work::Motion(Movement::Fall, FALL_SAMPLE, 1),
            Work::ActorContact
        ]
    );
}

#[test]
fn fall_then_slide_floor_with_existing_latch_enters_special_reward() {
    let mut f = Fixture::new(MOVING_HANDLER);
    {
        let actor_value = f.a().state_code() | (1);
        f.a().set_state_code(actor_value);
    };
    f.a().motion_flags = 1;
    f.world.facts.push_back(CONTACT);
    f.run();
    assert_eq!(f.actor().handler, RECOVERY_HANDLER);
    assert_eq!(f.bits[8], 4);
}

#[test]
fn transfer_contact_activates_target_immediately_and_removes_source() {
    let mut f = Fixture::new(MOVING_HANDLER);
    {
        let actor_value = f.a().state_code() | (2);
        f.a().set_state_code(actor_value);
    };
    {
        let actor_value = TERRAIN_HANDLER;
        f.actors[CHILD].set_handler(actor_value);
    };
    f.actors[CHILD].animation = SPECIAL_TERRAIN_ANIMATION;
    {
        let actor_value = ACTIVE | TERRAIN_REPRESENTATION;
        f.actors[CHILD].set_state_code(actor_value);
    };
    f.world.target = Some(CHILD);
    f.run();
    assert_eq!(f.actors[CHILD].handler, MOVING_HANDLER);
    assert_eq!(
        f.actors[CHILD].state_code(),
        ACTIVE | TERRAIN_REPRESENTATION | 1
    );
    assert_eq!(f.actor().handler, 0);
}

#[test]
fn transfer_floor_or_wall_removes_before_actor_scan() {
    for facts in [[CONTACT, 0], [0, CONTACT]] {
        let mut f = Fixture::new(MOVING_HANDLER);
        {
            let actor_value = f.a().state_code() | (2);
            f.a().set_state_code(actor_value);
        };
        f.world.facts = facts.into();
        f.run();
        assert_eq!(f.actor().handler, 0);
        assert!(!f.world.calls.contains(&Work::ActorContact));
    }
}

#[test]
fn slide_pose_sets_twenty_hex_before_fall_and_reacts_to_latched_floor() {
    let mut f = Fixture::new(MOVING_HANDLER);
    {
        let actor_value = f.a().state_code() | (6);
        f.a().set_state_code(actor_value);
    };
    f.a().motion_flags = 1;
    f.world.facts.push_back(CONTACT);
    f.run();
    assert_eq!(f.actor().animation, REACTION_ANIMATION);
    assert_eq!(f.actor().handler, 0);
}

#[test]
fn touch_pose_contact_sets_animation_and_mutates_target_in_same_call() {
    let mut f = Fixture::new(MOVING_HANDLER);
    {
        let actor_value = f.a().state_code() | (8);
        f.a().set_state_code(actor_value);
    };
    f.world.facts.push_back(CONTACT);
    f.world.target = Some(CHILD);
    {
        let actor_value = TERRAIN_HANDLER;
        f.actors[CHILD].set_handler(actor_value);
    };
    {
        let actor_value = ACTIVE | TERRAIN_REPRESENTATION;
        f.actors[CHILD].set_state_code(actor_value);
    };
    f.changed = 0;
    f.run();
    assert_eq!(f.actor().animation, IGNORED_HIT_ANIMATION);
    assert_eq!(f.actors[CHILD].state_code() & TERRAIN_REPRESENTATION, 0);
    assert_eq!(f.changed, 1);
}

#[test]
fn touch_pose_floor_restores_handler_eight_only_without_contact_target() {
    let mut f = Fixture::new(MOVING_HANDLER);
    {
        let actor_value = f.a().state_code() | (8);
        f.a().set_state_code(actor_value);
    };
    f.a().motion_flags = 1;
    f.world.facts.push_back(CONTACT);
    f.run();
    assert_eq!(f.actor().handler, TERRAIN_HANDLER);
}

#[test]
fn proximity_fall_strict_four_or_thirty_two_threshold() {
    for (animation, distance, active) in [
        (0x20, 3, true),
        (0x20, 4, false),
        (0x87, 31, true),
        (0x87, 32, false),
    ] {
        let mut f = Fixture::new(FALLING_HANDLER);
        f.a().animation = animation;
        f.actors[0].position.world_x = ACTOR_X - distance;
        f.run();
        assert_eq!(f.actor().state_code() & 1, u8::from(active));
    }
}

#[test]
fn proximity_offscreen_culls_without_player_or_terrain_calls() {
    let mut f = Fixture::new(FALLING_HANDLER);
    f.a().render = 0;
    f.run();
    assert_eq!(f.actor().state_code(), 0);
    assert!(f.world.calls.is_empty());
}

#[test]
fn proximity_special_animation_skips_replacement_but_ordinary_changes_profile_above_player() {
    for animation in [0x20, 0x87] {
        let mut f = Fixture::new(FALLING_HANDLER);
        f.a().animation = animation;
        {
            let actor_value = f.a().state_code() | (TERRAIN_REPRESENTATION);
            f.a().set_state_code(actor_value);
        };
        f.actors[0].position.screen_y = ACTOR_Y + 1;
        f.run();
        assert_eq!(f.changed, if animation == 0x87 { 0 } else { 1 });
        assert_eq!(f.actor().profile, if animation == 0x87 { 0 } else { 0x10 });
    }
}

#[test]
fn proximity_player_contact_removes_before_fall() {
    let mut f = Fixture::new(FALLING_HANDLER);
    {
        let actor_value = f.a().state_code() | (1);
        f.a().set_state_code(actor_value);
    };
    f.world.contact = true;
    f.run();
    assert_eq!(f.actor().handler, 0);
    assert_eq!(f.world.calls, vec![Work::PlayerContact]);
}

#[test]
fn reaction_offscreen_clears_state_render_only_and_no_random() {
    let mut f = Fixture::new(TERRAIN_HANDLER);
    f.a().render = 0x40;
    let old = *f.actor();
    let e = f.invoke(1);
    assert_eq!(f.actor().state_code(), 0);
    assert_eq!(f.actor().render, 0);
    assert_eq!(f.actor().handler, old.handler);
    assert_eq!(f.random.words, [2, 4, 6, 8]);
    assert!(e.sounds.is_empty());
}

#[test]
fn reaction_queue_comparison_not_enqueue_success_supplies_rng_increment() {
    for (queued, expected) in [(AUDIO_EMPTY, 7), (SOUND_REMOVE, 6), (AUDIO_STOP, 7)] {
        let mut f = Fixture::new(TERRAIN_HANDLER);
        f.audio.slots[f.audio.producer] = queued;
        let e = f.invoke(1);
        assert_eq!(f.random.words, [2, expected, 6, 8]);
        assert_eq!(e.sounds, vec![SOUND_REACT]);
        assert_eq!(f.actor().handler, LAUNCHED_HANDLER);
    }
}

#[test]
fn reaction_initializes_same_call_arc_and_facing_equality_is_positive() {
    let mut f = Fixture::new(TERRAIN_HANDLER);
    {
        let actor_value = 0xff;
        f.a().set_state_code(actor_value);
    };
    f.invoke(1);
    assert_eq!(f.actor().state_code(), 0xf1);
    assert_eq!(f.actor().direction, 1);
    assert_eq!(f.actor().horizontal_magnitude, 0x80);
    assert_eq!(f.actor().playback.tick_code(), 0x80);
    assert_eq!(f.actor().profile, 0);
    assert_eq!(f.world.calls[0], Work::Integrate(Integration::FallingY));
}

#[test]
fn reaction_actor_left_of_player_uses_negative_facing() {
    let mut f = Fixture::new(TERRAIN_HANDLER);
    f.actors[0].position.world_x = ACTOR_X + 1;
    f.invoke(1);
    assert_eq!(f.actor().direction, 2);
}

#[test]
fn launched_apex_ignores_initial_y_boundary_and_resets_velocity() {
    let mut f = Fixture::new(LAUNCHED_HANDLER);
    f.world.exits.push_back(true);
    f.world.post_velocity = Some(-1);
    f.run();
    assert_ne!(f.actor().state_code(), 0);
    assert_eq!(f.actor().motion_flags, 1);
    assert_eq!(f.actor().vertical_velocity, 0);
}

#[test]
fn launched_falling_y_boundary_culls_before_horizontal() {
    let mut f = Fixture::new(LAUNCHED_HANDLER);
    {
        let actor_value = f.a().state_code() | (1);
        f.a().set_state_code(actor_value);
    };
    f.a().motion_flags = 1;
    f.world.exits = [false, true].into();
    f.run();
    assert_eq!(f.actor().state_code(), 0);
    assert_eq!(
        f.world.calls,
        vec![
            Work::Integrate(Integration::Gravity),
            Work::Integrate(Integration::FallingY)
        ]
    );
}

#[test]
fn launched_horizontal_boundary_polarity_differs_by_direction() {
    for (direction, result, removed) in [
        (1, true, true),
        (1, false, false),
        (2, false, true),
        (2, true, false),
    ] {
        let mut f = Fixture::new(LAUNCHED_HANDLER);
        {
            let actor_value = f.a().state_code() | (1);
            f.a().set_state_code(actor_value);
        };
        f.a().direction = direction;
        f.world.exits = [false, false, result].into();
        f.run();
        assert_eq!(f.actor().state_code() == 0, removed);
    }
}

#[test]
fn secondary_random_wrap_uses_only_low_two_bits_and_preserves_other_words() {
    let mut f = Fixture::new(TERRAIN_HANDLER);
    f.random.words = [0xff, 1, 0x55, 0xaa];
    f.invoke(3);
    assert_eq!(f.random.words, [0xff, 0, 0x55, 0xaa]);
}

#[test]
fn secondary_zero_mapping_and_full_pool_still_advance_random() {
    for full in [false, true] {
        let mut f = Fixture::new(TERRAIN_HANDLER);
        f.random.words = [2, 2, 6, 8];
        if full {
            for a in &mut f.actors[1..] {
                {
                    let actor_value = ACTIVE;
                    a.set_state_code(actor_value);
                };
            }
            f.catalog.secondary[TERRAIN_HANDLER as usize] = 0x23;
        }
        let e = f.invoke(3);
        assert_eq!(f.random.words[1], 4);
        assert!(e.spawned.is_empty());
    }
}

#[test]
fn secondary_success_preserves_source_and_copies_integer_position_only() {
    let mut f = Fixture::new(TERRAIN_HANDLER);
    f.random.words = [2, 2, 6, 8];
    f.catalog.secondary[TERRAIN_HANDLER as usize] = 0x23;
    f.actors[CHILD].position.x_fraction = 0xab;
    f.actors[CHILD].position.y_fraction = 0xcd;
    let old = *f.actor();
    let e = f.invoke(3);
    assert_eq!(*f.actor(), old);
    assert_eq!(e.spawned, vec![CHILD]);
    assert_eq!(f.actors[CHILD].position.world_x, ACTOR_X);
    assert_eq!(f.actors[CHILD].position.screen_y, ACTOR_Y);
    assert_eq!(
        (
            f.actors[CHILD].position.x_fraction,
            f.actors[CHILD].position.y_fraction
        ),
        (0xab, 0xcd)
    );
    assert_eq!(f.actors[CHILD].state_code(), 0xa1);
    assert_eq!(f.actors[CHILD].handler, RECOVERY_HANDLER);
}

#[test]
fn recovering_offscreen_twenty_nine_unsuppresses_only_own_bit() {
    for animation in [REAPPEARING_ANIMATION, 0x23] {
        let mut f = Fixture::new(RECOVERY_HANDLER);
        f.a().render = 0;
        f.a().animation = animation;
        f.bits[8] = 0xff;
        f.run();
        assert_eq!(
            f.bits[8],
            if animation == REAPPEARING_ANIMATION {
                0xfb
            } else {
                0xff
            }
        );
        assert_eq!(f.actor().state_code(), 0);
    }
}

#[test]
fn recovery_reveal_requires_nonzero_frame_exact_tick_five() {
    for (frame, tick, reveal) in [(0, 5, false), (1, 4, false), (1, 5, true), (2, 6, false)] {
        let mut f = Fixture::new(RECOVERY_HANDLER);
        f.a().animation = REACTION_ANIMATION;
        {
            let animation_value = frame;
            f.a().playback.set_frame_code(animation_value);
        };
        {
            let animation_value = tick;
            f.a().playback.set_tick_code(animation_value);
        };
        {
            let actor_value = 0x25;
            f.a().set_local_state_code(actor_value);
        };
        f.run();
        assert_eq!(
            f.actor().animation,
            if reveal { 0x25 } else { REACTION_ANIMATION }
        );
    }
}

#[test]
fn recovery_reveal_applies_option_remap_before_arc_same_call() {
    let mut f = Fixture::new(RECOVERY_HANDLER);
    f.revealed();
    {
        let actor_value = 0x25;
        f.a().set_local_state_code(actor_value);
    };
    f.rule = TitleRule::AlternateRewards;
    f.run();
    assert_eq!(f.actor().animation, 0x28);
    assert_eq!(f.actor().playback.tick_code(), 0);
    assert_eq!(f.world.calls[0], Work::Integrate(Integration::FallingY));
}

#[test]
fn recovery_rising_returns_before_player_collection() {
    let mut f = Fixture::new(RECOVERY_HANDLER);
    f.a().animation = 0x23;
    f.a().vertical_velocity = 0x0100;
    f.world.contact = true;
    f.run();
    assert_eq!(f.actor().state_code(), ACTIVE);
    assert!(!f.world.calls.contains(&Work::PlayerContact));
}

#[test]
fn recovery_apex_falls_same_call_landing_starts_ff_countdown() {
    let mut f = Fixture::new(RECOVERY_HANDLER);
    f.a().animation = 0x23;
    f.world.post_velocity = Some(-1);
    f.world.facts.push_back(CONTACT);
    f.run();
    assert_eq!(f.actor().state_code(), ACTIVE | 2);
    assert_eq!(f.actor().local_state_code(), EXPIRING_COUNTDOWN);
}

#[test]
fn recovery_countdown_only_below_twenty_nine_or_sixty_one() {
    for (animation, decrements) in [(0x28, true), (0x29, false), (0x61, true), (0x6f, false)] {
        let mut f = Fixture::new(RECOVERY_HANDLER);
        {
            let actor_value = f.a().state_code() | (2);
            f.a().set_state_code(actor_value);
        };
        f.a().animation = animation;
        {
            let actor_value = 3;
            f.a().set_local_state_code(actor_value);
        };
        f.run();
        assert_eq!(f.actor().local_state_code(), if decrements { 2 } else { 3 });
    }
}

#[test]
fn recovery_blinks_at_old_seventy_eight_and_expires_before_contact() {
    for count in [BLINK_COUNTDOWN, 0] {
        let mut f = Fixture::new(RECOVERY_HANDLER);
        {
            let actor_value = f.a().state_code() | (2);
            f.a().set_state_code(actor_value);
        };
        f.a().animation = 0x23;
        {
            let actor_value = count;
            f.a().set_local_state_code(actor_value);
        };
        f.run();
        if count == 0 {
            assert_eq!(f.actor().state_code(), 0);
            assert!(!f.world.calls.contains(&Work::PlayerContact));
        } else {
            assert_eq!(f.actor().render & 2, 2);
            assert_eq!(f.actor().local_state_code(), 0x77);
        }
    }
}

#[test]
fn recovery_damage_player_ignores_contact_death_removes_without_reward() {
    for action in [Action::Damage, Action::Death] {
        let mut f = Fixture::new(RECOVERY_HANDLER);
        {
            let actor_value = f.a().state_code() | (2);
            f.a().set_state_code(actor_value);
        };
        f.a().animation = 0x29;
        f.player.action = action;
        f.world.contact = true;
        let e = f.run();
        assert_eq!(
            f.actor().state_code(),
            if action == Action::Death {
                0
            } else {
                ACTIVE | 2
            }
        );
        assert!(e.reward_animation.is_none());
        assert!(!f.world.calls.contains(&Work::PlayerContact));
    }
}

#[test]
fn contact_recovery_requires_no_protection_and_actual_overlap() {
    for (protection, contact) in [(1, true), (0, false)] {
        let mut f = Fixture::new(CONTACT_RECOVERY_HANDLER);
        f.player.protection = protection;
        f.world.contact = contact;
        f.run();
        assert_eq!(f.actor().state_code(), ACTIVE);
        assert_eq!(f.bits[8], 0);
    }
}

#[test]
fn contact_recovery_initial_delay_decrements_in_same_call() {
    let mut f = Fixture::new(CONTACT_RECOVERY_HANDLER);
    f.world.contact = true;
    f.run();
    assert_eq!(f.actor().state_code(), ACTIVE | 1);
    assert_eq!(f.actor().local_counter_code(), CONTACT_DELAY - 1);
    assert_eq!(f.bits[8], 4);
}

#[test]
fn contact_delay_expiry_clears_terrain_saves_animation_without_restarting_clocks() {
    let mut f = Fixture::new(CONTACT_RECOVERY_HANDLER);
    {
        let actor_value = f.a().state_code() | (TERRAIN_REPRESENTATION | 1);
        f.a().set_state_code(actor_value);
    };
    {
        let actor_value = 1;
        f.a().set_local_counter_code(actor_value);
    };
    {
        let animation_value = 9;
        f.a().playback.set_tick_code(animation_value);
    };
    {
        let animation_value = 3;
        f.a().playback.set_frame_code(animation_value);
    };
    f.run();
    assert_eq!(f.actor().state_code(), ACTIVE | 2);
    assert_eq!(f.actor().local_state_code(), 0x1a);
    assert_eq!(f.actor().animation, REACTION_ANIMATION);
    assert_eq!(
        (
            f.actor().playback.tick_code(),
            f.actor().playback.frame_code()
        ),
        (9, 3)
    );
}

#[test]
fn contact_zero_delay_wraps_and_does_not_reveal() {
    let mut f = Fixture::new(CONTACT_RECOVERY_HANDLER);
    {
        let actor_value = f.a().state_code() | (1);
        f.a().set_state_code(actor_value);
    };
    {
        let actor_value = 0;
        f.a().set_local_counter_code(actor_value);
    };
    f.run();
    assert_eq!(f.actor().local_counter_code(), 0xff);
    assert_eq!(f.actor().state_code(), ACTIVE | 1);
}

#[test]
fn contact_restore_ordinary_resets_exact_recovery_fields() {
    let mut f = Fixture::new(CONTACT_RECOVERY_HANDLER);
    {
        let actor_value = f.a().state_code() | (2);
        f.a().set_state_code(actor_value);
    };
    f.revealed();
    {
        let actor_value = 0x26;
        f.a().set_local_state_code(actor_value);
    };
    f.rule = TitleRule::AlternateRewards;
    f.run();
    assert_eq!(f.actor().animation, 0x24);
    assert_eq!(
        (f.actor().handler, f.actor().state_code(), f.actor().profile),
        (RECOVERY_HANDLER, 0xa1, 0)
    );
    assert_eq!(f.actor().local_state_code(), 0);
}

#[test]
fn contact_restore_special_late_stage_four_prepares_palette_pair() {
    let mut f = Fixture::new(CONTACT_RECOVERY_HANDLER);
    {
        let actor_value = f.a().state_code() | (2);
        f.a().set_state_code(actor_value);
    };
    f.revealed();
    {
        let actor_value = SPECIAL_TERRAIN_ANIMATION;
        f.a().set_local_state_code(actor_value);
    };
    f.stage = 4;
    f.late = true;
    let e = f.run();
    assert_eq!(f.actor().handler, TERRAIN_HANDLER);
    assert_eq!(f.actor().profile, 0x80);
    assert_eq!(f.actor().local_state_code(), 10);
    assert_eq!(e.palette, Some([0x0f, 0x29]));
}

impl Fixture {
    fn run_with_floor(&mut self, floor: u8) -> Effects {
        self.world.facts.push_back(floor);
        self.run()
    }
}

#[test]
fn transform_offscreen_culls_before_profile_write() {
    let mut f = Fixture::new(TRANSFORM_HANDLER);
    f.a().render = 0;
    f.a().profile = 0x55;
    f.run();
    assert_eq!(f.actor().state_code(), 0);
    assert_eq!(f.actor().profile, 0x55);
}

#[test]
fn transform_swing_requires_tick_zero_and_contact_but_stage_gate_still_runs() {
    let mut f = Fixture::new(TRANSFORM_HANDLER);
    f.swing();
    {
        let animation_value = 1;
        f.actors[0].playback.set_tick_code(animation_value);
    };
    f.run();
    assert_eq!(f.actor().handler, TRANSFORM_HANDLER);
    assert_eq!(f.actor().profile, 0x81);
}

#[test]
fn transform_nonzero_mapping_increments_existing_handler_not_table_value() {
    let mut f = Fixture::new(TRANSFORM_HANDLER);
    f.swing();
    {
        let actor_value = 0xff;
        f.a().set_state_code(actor_value);
    };
    f.a().hit_marker = 1;
    f.run();
    assert_eq!(f.actor().handler, REVEAL_HANDLER);
    assert_eq!(f.actor().state_code(), 0xe0);
    assert_eq!(f.actor().position.screen_y, ACTOR_Y - 0x18);
    assert_eq!(f.actor().animation, REACTION_ANIMATION);
    assert_eq!(f.actor().hit_marker, 0);
    assert_eq!(f.bits[8], 4);
}

#[test]
fn transform_zero_handler_enters_reward_launch_with_fixed_saved_twenty_eight() {
    let mut f = Fixture::new(TRANSFORM_HANDLER);
    f.swing();
    f.catalog.transform_handler[0] = 0;
    f.run();
    assert_eq!(f.actor().handler, RECOVERY_HANDLER);
    assert_eq!(f.actor().local_state_code(), 0x28);
    assert_eq!(f.actor().vertical_velocity, 0x0400);
    assert_eq!(f.actor().animation, REACTION_ANIMATION);
}

#[test]
fn transform_index_six_enters_sliding_directly() {
    let mut f = Fixture::new(TRANSFORM_HANDLER);
    f.swing();
    f.stage = 1;
    {
        let actor_value = 2;
        f.a().set_local_state_code(actor_value);
    };
    f.run();
    assert_eq!(
        (f.actor().handler, f.actor().profile, f.actor().animation),
        (SLIDING_HANDLER, 0x10, 0x21)
    );
}

#[test]
fn stage_one_local_two_clears_low_state_without_swing_and_uses_absolute_y() {
    for (player_y, profile) in [
        (ACTOR_Y - 23, 0x82),
        (ACTOR_Y - 24, 0x83),
        (ACTOR_Y + 24, 0x83),
        (ACTOR_Y + 25, 0x83),
    ] {
        let mut f = Fixture::new(TRANSFORM_HANDLER);
        f.stage = 1;
        {
            let actor_value = 2;
            f.a().set_local_state_code(actor_value);
        };
        {
            let actor_value = 0xff;
            f.a().set_state_code(actor_value);
        };
        f.actors[0].position.screen_y = player_y;
        f.run();
        assert_eq!(
            (
                f.actor().state_code(),
                f.actor().animation,
                f.actor().profile
            ),
            (0xe0, 0x5b, profile)
        );
    }
}

#[test]
fn reveal_frame_tick_gates_precede_table_updates() {
    for (frame, tick) in [(0, 5), (1, 4), (2, 6)] {
        let mut f = Fixture::new(REVEAL_HANDLER);
        {
            let animation_value = frame;
            f.a().playback.set_frame_code(animation_value);
        };
        {
            let animation_value = tick;
            f.a().playback.set_tick_code(animation_value);
        };
        let old = *f.actor();
        f.run();
        assert_eq!(*f.actor(), old);
    }
}

#[test]
fn reveal_index_seven_y_add_includes_compare_carry() {
    let mut f = Fixture::new(REVEAL_HANDLER);
    f.revealed();
    f.stage = 1;
    {
        let actor_value = 3;
        f.a().set_local_state_code(actor_value);
    };
    f.run();
    assert_eq!(f.actor().position.screen_y, ACTOR_Y + 0x18);
    assert_eq!(
        (f.actor().animation, f.actor().handler, f.actor().profile),
        (0x34, 0x11, 0x16)
    );
}

#[test]
fn reveal_index_four_requests_sound_thirty_only() {
    for local in [0, 1] {
        let mut f = Fixture::new(REVEAL_HANDLER);
        f.revealed();
        f.stage = 1;
        {
            let actor_value = local;
            f.a().set_local_state_code(actor_value);
        };
        let e = f.run();
        assert_eq!(e.sounds, if local == 0 { vec![0x30] } else { vec![] });
    }
}

#[test]
fn sliding_initializes_and_applies_first_nudge_same_call() {
    let mut f = Fixture::new(SLIDING_HANDLER);
    f.tick = 0;
    f.run();
    assert_eq!(f.actor().horizontal_magnitude, 0x0200);
    assert_eq!(f.actor().local_state_code(), 1);
    assert_eq!(f.actor().position.world_x, ACTOR_X + 1);
    assert!(f.world.calls.is_empty());
}

#[test]
fn sliding_nudges_four_phase_signed_words_on_quarter_clock_only() {
    for (tick, local, x) in [
        (1, 1, ACTOR_X),
        (0, 1, ACTOR_X - 1),
        (0, 2, ACTOR_X - 1),
        (0, 3, ACTOR_X + 1),
    ] {
        let mut f = Fixture::new(SLIDING_HANDLER);
        {
            let actor_value = f.a().state_code() | (1);
            f.a().set_state_code(actor_value);
        };
        {
            let actor_value = local;
            f.a().set_local_state_code(actor_value);
        };
        f.tick = tick;
        f.run();
        assert_eq!(f.actor().position.world_x, x);
        assert_eq!(f.actor().local_state_code(), local + u8::from(tick == 0));
    }
}

#[test]
fn sliding_floor_friction_wraps_below_step_without_clamp() {
    let mut f = Fixture::new(SLIDING_HANDLER);
    {
        let actor_value = f.a().state_code() | (1);
        f.a().set_state_code(actor_value);
    };
    {
        let actor_value = 8;
        f.a().set_local_state_code(actor_value);
    };
    f.a().horizontal_magnitude = 8;
    f.world.facts = [CONTACT, 0].into();
    f.run();
    assert_eq!(f.actor().horizontal_magnitude, 0xfff8);
}

#[test]
fn sliding_friction_only_when_floor_and_nonzero() {
    for (bits, speed, expected) in [(0, 0x200, 0x200), (CONTACT, 0, 0), (CONTACT, 0x200, 0x1f0)] {
        let mut f = Fixture::new(SLIDING_HANDLER);
        {
            let actor_value = f.a().state_code() | (1);
            f.a().set_state_code(actor_value);
        };
        {
            let actor_value = 8;
            f.a().set_local_state_code(actor_value);
        };
        f.a().horizontal_magnitude = speed;
        f.world.facts = [bits, 0].into();
        f.run();
        assert_eq!(f.actor().horizontal_magnitude, expected);
    }
}

#[test]
fn sliding_player_overlap_runs_after_both_horizontal_paths() {
    for (direction, called) in [(1, true), (2, true)] {
        let mut f = Fixture::new(SLIDING_HANDLER);
        {
            let actor_value = f.a().state_code() | (1);
            f.a().set_state_code(actor_value);
        };
        {
            let actor_value = 8;
            f.a().set_local_state_code(actor_value);
        };
        f.a().direction = direction;
        f.a().horizontal_magnitude = 0x100;
        f.run();
        assert_eq!(f.world.calls.contains(&Work::PlayerContact), called);
    }
}

#[test]
fn sliding_ground_and_wall_or_stopped_restores_terrain_handler() {
    for (floor, wall, speed, settled) in [
        (CONTACT, CONTACT, 0x100, true),
        (CONTACT, 0, 0, true),
        (0, CONTACT, 0x100, false),
        (CONTACT, 0, 0x100, false),
    ] {
        let mut f = Fixture::new(SLIDING_HANDLER);
        {
            let actor_value = f.a().state_code() | (1);
            f.a().set_state_code(actor_value);
        };
        {
            let actor_value = 8;
            f.a().set_local_state_code(actor_value);
        };
        f.a().direction = 2;
        f.a().horizontal_magnitude = speed;
        f.world.facts = [floor, wall].into();
        f.run();
        assert_eq!(f.actor().handler == TERRAIN_HANDLER, settled);
    }
}

#[test]
fn released_swing_does_not_require_player_tick_zero_and_raises_wrapped_y() {
    let mut f = Fixture::new(RELEASED_HANDLER);
    f.swing();
    {
        let animation_value = 9;
        f.actors[0].playback.set_tick_code(animation_value);
    };
    f.a().position.screen_y = 8;
    f.run();
    assert_eq!(f.actor().position.screen_y, 0xf0);
    assert_eq!(f.actor().animation, REACTION_ANIMATION);
    assert_eq!(f.actor().profile, 0);
    assert_eq!(f.actor().state_code(), ACTIVE | 1);
}

#[test]
fn released_untouched_profile_uses_unsigned_difference_not_absolute() {
    for player_y in [ACTOR_Y - 23, ACTOR_Y - 24, ACTOR_Y + 1] {
        let mut f = Fixture::new(RELEASED_HANDLER);
        f.actors[0].animation = crate::player::SWING_ANIMATION;
        f.actors[0].position.screen_y = player_y;
        f.run();
        assert_eq!(
            f.actor().profile,
            if player_y == ACTOR_Y - 23 { 0x83 } else { 0x84 }
        );
    }
}

#[test]
fn released_reveal_option_remap_sets_speed_and_returns_before_fall() {
    let mut f = Fixture::new(RELEASED_HANDLER);
    {
        let actor_value = f.a().state_code() | (1);
        f.a().set_state_code(actor_value);
    };
    f.revealed();
    f.rule = TitleRule::AlternateRewards;
    f.run();
    assert_eq!(f.actor().animation, 0x28);
    assert_eq!(f.actor().horizontal_magnitude, 0x0120);
    assert!(f.world.calls.is_empty());
}

#[test]
fn released_no_floor_increments_latch_floor_zero_does_not() {
    for floor in [0, CONTACT] {
        let mut f = Fixture::new(RELEASED_HANDLER);
        {
            let actor_value = f.a().state_code() | (1);
            f.a().set_state_code(actor_value);
        };
        f.a().animation = 0x25;
        f.run_with_floor(floor);
        assert_eq!(f.actor().local_state_code(), u8::from(floor == 0));
    }
}

#[test]
fn released_floor_after_air_latch_enters_recovery_without_handler_rewrite() {
    let mut f = Fixture::new(RELEASED_HANDLER);
    {
        let actor_value = f.a().state_code() | (1);
        f.a().set_state_code(actor_value);
    };
    f.a().animation = 0x25;
    {
        let actor_value = 1;
        f.a().set_local_state_code(actor_value);
    };
    f.world.facts.push_back(CONTACT);
    f.run();
    assert_eq!(f.actor().state_code(), 0xa2);
    assert_eq!(f.actor().local_state_code(), 0xff);
    assert_eq!(f.actor().handler, RELEASED_HANDLER);
}

#[test]
fn released_collection_precedes_floor_latch_and_horizontal() {
    let mut f = Fixture::new(RELEASED_HANDLER);
    {
        let actor_value = f.a().state_code() | (1);
        f.a().set_state_code(actor_value);
    };
    f.a().animation = 0x28;
    f.world.contact = true;
    let e = f.run();
    assert_eq!(f.actor().state_code(), 0);
    assert_eq!(e.reward_animation, Some(0x28));
    assert_eq!(
        f.world.calls,
        vec![
            Work::Motion(Movement::Fall, FALL_SAMPLE, 1),
            Work::PlayerContact
        ]
    );
}

#[test]
fn dispenser_strict_horizontal_distance_four() {
    for distance in [3, 4] {
        let mut f = Fixture::new(DISPENSER_HANDLER);
        f.actors[0].position.world_x = ACTOR_X - distance;
        let e = f.run();
        assert_eq!(e.spawned.len(), usize::from(distance == 3));
    }
}

#[test]
fn dispenser_allocates_highest_free_child_and_four_sequence_values() {
    let mut f = Fixture::new(DISPENSER_HANDLER);
    f.a().animation = 0x48;
    for sequence in 0..DISPENSE_COUNT {
        {
            let actor_value = 0;
            f.a().set_local_counter_code(actor_value);
        };
        let e = f.run();
        let child = CHILD - sequence;
        assert_eq!(e.spawned, vec![child]);
        assert_eq!(
            f.actors[child].direction,
            f.catalog.dispenser_direction[sequence]
        );
        assert_eq!(
            f.actors[child].horizontal_magnitude,
            f.catalog.dispenser_speed[sequence]
        );
        assert_eq!(f.actors[child].position.screen_y, ACTOR_Y - 0x10);
        assert_eq!(f.actors[child].vertical_velocity, 0x02d4);
        assert_eq!(f.actors[child].state_code(), 0xa0);
        assert_eq!(f.actors[child].render, 0xa0);
        assert_eq!(f.actors[child].handler, DISPENSED_HANDLER);
        assert_eq!(e.sounds, vec![SOUND_DISPENSE]);
    }
    assert_eq!(f.actor().local_counter_code(), 0xff);
}

#[test]
fn dispenser_full_pool_does_not_advance_sequence_sound_or_animation() {
    let mut f = Fixture::new(DISPENSER_HANDLER);
    for a in &mut f.actors[1..] {
        {
            let actor_value = ACTIVE;
            a.set_state_code(actor_value);
        };
    }
    let old = *f.actor();
    let e = f.run();
    assert_eq!(f.actor().local_state_code(), 0);
    assert_eq!(f.actor().animation, old.animation);
    assert!(e.sounds.is_empty());
    assert!(e.spawned.is_empty());
}

#[test]
fn dispenser_countdown_one_reaches_zero_but_allocates_next_call() {
    let mut f = Fixture::new(DISPENSER_HANDLER);
    {
        let actor_value = f.a().state_code() | (1);
        f.a().set_state_code(actor_value);
    };
    {
        let actor_value = 1;
        f.a().set_local_counter_code(actor_value);
    };
    assert!(f.run().spawned.is_empty());
    assert_eq!(f.actor().local_counter_code(), 0);
    assert_eq!(f.run().spawned, vec![CHILD]);
}

#[test]
fn dispenser_negative_countdown_still_restores_animation_at_tick_seven() {
    let mut f = Fixture::new(DISPENSER_HANDLER);
    {
        let actor_value = f.a().state_code() | (1);
        f.a().set_state_code(actor_value);
    };
    {
        let actor_value = 0xff;
        f.a().set_local_counter_code(actor_value);
    };
    f.a().animation = 0x49;
    {
        let animation_value = 7;
        f.a().playback.set_tick_code(animation_value);
    };
    f.run();
    assert_eq!(f.actor().animation, 0x48);
    assert_eq!(f.actor().local_counter_code(), 0xff);
    assert_eq!(f.actor().playback.tick_code(), 0);
}

#[test]
fn dispensed_apex_resets_vertical_then_brakes_same_frame() {
    let mut f = Fixture::new(DISPENSED_HANDLER);
    f.a().direction = 1;
    f.a().horizontal_magnitude = 0x100;
    f.world.post_velocity = Some(-1);
    f.run();
    assert_eq!(f.actor().state_code(), ACTIVE | 1);
    assert_eq!(f.actor().horizontal_magnitude, 0xc0);
    assert_eq!(f.actor().vertical_velocity, 0);
}

#[test]
fn dispensed_negative_three_integer_skips_gravity_even_nonzero_fraction() {
    let mut f = Fixture::new(DISPENSED_HANDLER);
    {
        let actor_value = f.a().state_code() | (1);
        f.a().set_state_code(actor_value);
    };
    f.a().vertical_velocity = -0x280;
    f.run();
    assert!(
        !f.world
            .calls
            .contains(&Work::Integrate(Integration::Gravity))
    );
}

#[test]
fn dispensed_brake_zeroes_direction_only_exact_zero_and_wraps_underflow() {
    for (speed, expected, direction) in [(0x40, 0, 0), (0x20, 0xffe0, 1)] {
        let mut f = Fixture::new(DISPENSED_HANDLER);
        {
            let actor_value = f.a().state_code() | (1);
            f.a().set_state_code(actor_value);
        };
        f.a().direction = 1;
        f.a().horizontal_magnitude = speed;
        f.run();
        assert_eq!(f.actor().horizontal_magnitude, expected);
        assert_eq!(f.actor().direction, direction);
    }
}

#[test]
fn dispensed_contact_reward_precedes_f0_cull_and_retains_render() {
    let mut f = Fixture::new(DISPENSED_HANDLER);
    {
        let actor_value = f.a().state_code() | (1);
        f.a().set_state_code(actor_value);
    };
    f.a().position.screen_y = 0xf0;
    f.a().render = 0xa0;
    f.world.contact = true;
    f.status
        .health
        .edit_overlay(|segments| segments[0].depleted = 1);
    f.run();
    assert_eq!(f.actor().state_code(), 0);
    assert_eq!(f.actor().render, 0xa0);
    assert_eq!(f.status.health.overlay_segments()[0].depleted, 0);
}

#[test]
fn dispensed_y_cutoff_exact_f0_clears_state_and_render() {
    for y in [0xef, 0xf0] {
        let mut f = Fixture::new(DISPENSED_HANDLER);
        {
            let actor_value = f.a().state_code() | (1);
            f.a().set_state_code(actor_value);
        };
        f.a().position.screen_y = y;
        f.run();
        assert_eq!(f.actor().state_code() == 0, y == 0xf0);
        assert_eq!(f.actor().render == 0, y == 0xf0);
    }
}

#[test]
fn collection_removes_actor_before_all_reward_dispatch_classes() {
    for animation in [
        0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2b, 0x61, 0x6f, 0x74,
    ] {
        let mut f = Fixture::new(RECOVERY_HANDLER);
        f.a().animation = animation;
        f.invoke(2);
        assert_eq!(f.actor().state_code(), 0);
    }
}

#[test]
fn seven_ordinary_reward_classes_request_source_sound() {
    for (animation, sound) in [
        (0x23, 0x15),
        (0x24, 0x15),
        (0x25, 0x15),
        (0x26, 0x15),
        (0x27, 0x17),
        (0x28, 0x15),
        (0x29, 0x37),
    ] {
        let mut f = Fixture::new(RECOVERY_HANDLER);
        f.a().animation = animation;
        let e = f.invoke(2);
        assert_eq!(e.sounds, vec![sound]);
        assert_eq!(e.reward_animation, Some(animation));
    }
}

#[test]
fn reward_twenty_three_adds_two_at_digit_three_and_twenty_eight_adds_one_at_two() {
    for (animation, digit, amount) in [(0x23, 3, 2), (0x28, 2, 1), (0x61, 2, 5)] {
        let mut f = Fixture::new(RECOVERY_HANDLER);
        f.status.reward = [0; 6].into();
        f.a().animation = animation;
        f.invoke(2);
        assert_eq!(f.status.reward.display_codes()[digit], amount);
        assert_eq!(f.status.dirty, 1);
        assert_eq!(f.status.total.display_codes(), [BLANK; 7]);
    }
}

#[test]
fn reward_health_recovery_clears_alternation_and_one_depleted_segment() {
    let mut f = Fixture::new(RECOVERY_HANDLER);
    f.a().animation = 0x24;
    f.status.health.alternating = 1;
    f.status
        .health
        .edit_overlay(|segments| segments[0].depleted = 1);
    f.status
        .health
        .edit_overlay(|segments| segments[1].depleted = 1);
    f.invoke(2);
    assert_eq!(f.status.health.alternating, 0);
    assert_eq!(f.status.health.overlay_segments()[0].depleted, 0);
    assert_eq!(f.status.health.overlay_segments()[1].depleted, 1);
}

#[test]
fn reward_full_health_clears_all_flags() {
    let mut f = Fixture::new(RECOVERY_HANDLER);
    f.a().animation = 0x25;
    f.status.health.edit_overlay(|segments| {
        for segment in segments {
            segment.depleted = 1;
        }
    });
    f.status.health.alternating = 1;
    f.invoke(2);
    assert!(
        f.status
            .health
            .overlay_segments()
            .iter()
            .all(|s| s.depleted == 0)
    );
}

#[test]
fn reward_special_mode_clears_protection_and_sets_480_ticks() {
    let mut f = Fixture::new(RECOVERY_HANDLER);
    f.a().animation = 0x26;
    f.player.protection = 0x80;
    f.invoke(2);
    assert_eq!(
        (
            f.player.protection,
            f.player.special_mode,
            f.player.special_ticks
        ),
        (0, 1, 0x1e0)
    );
}

#[test]
fn reward_life_saturates_only_exact_nine_other_byte_wraps() {
    for (before, after) in [(8, 9), (9, 9), (0xff, 0)] {
        let mut f = Fixture::new(RECOVERY_HANDLER);
        f.a().animation = 0x27;
        f.status.lives = before;
        f.invoke(2);
        assert_eq!(f.status.lives, after);
    }
}

#[test]
fn reward_dialogue_stage_four_ors_four_others_two() {
    for (stage, flag, message) in [(0, 2, 0x0e), (4, 4, 0x10)] {
        let mut f = Fixture::new(RECOVERY_HANDLER);
        f.a().animation = 0x29;
        f.stage = stage;
        f.flags = 0x80;
        f.invoke(2);
        assert_eq!(f.flags, 0x80 | flag);
        assert_eq!(f.dialogue, Some(message));
        assert_eq!(f.player.action, Action::Dialogue);
    }
}

#[test]
fn extension_sets_stage_one_flag_even_when_both_extra_slots_unavailable() {
    let mut f = Fixture::new(RECOVERY_HANDLER);
    f.a().animation = 0x2b;
    f.stage = 1;
    f.invoke(2);
    assert!(f.extension);
    assert_eq!(f.status.health.overlay_segments()[3].y, 0);
    assert_eq!(f.status.health.overlay_segments()[4].y, 0);
}

#[test]
fn extension_uses_first_negative_extra_slot_then_restores_health() {
    for first_available in [true, false] {
        let mut f = Fixture::new(RECOVERY_HANDLER);
        f.a().animation = 0x2b;
        f.status
            .health
            .edit_overlay(|segments| segments[3].y = if first_available { 0x80 } else { 0x20 });
        f.status
            .health
            .edit_overlay(|segments| segments[4].y = 0x80);
        f.status.health.edit_overlay(|segments| {
            for segment in segments {
                segment.depleted = 1;
            }
        });
        f.invoke(2);
        let index = if first_available { 3 } else { 4 };
        assert_eq!(f.status.health.overlay_segments()[index].y, 0x20);
        assert!(
            f.status
                .health
                .overlay_segments()
                .iter()
                .all(|s| s.depleted == 0)
        );
    }
}

#[test]
fn treasure_stage_bits_reward_and_transition_split_at_seventy_four() {
    for animation in 0x6f..=0x76 {
        let mut f = Fixture::new(RECOVERY_HANDLER);
        f.a().animation = animation;
        f.status.reward = [0; 6].into();
        f.stage_bits = 0x80;
        f.latch = 9;
        let e = f.invoke(2);
        assert_eq!(f.stage_bits, 0x80 | (1 << (animation - 0x6f)));
        assert_eq!(f.status.reward.display_codes()[0], 1);
        assert_eq!(f.latch, 0);
        assert_eq!(
            f.player.action,
            if animation < 0x74 {
                Action::ConvertTimer
            } else {
                Action::Dialogue
            }
        );
        assert_eq!(f.dialogue, if animation < 0x74 { None } else { Some(0x0d) });
        assert_eq!(e.sounds, vec![0x37]);
    }
}

#[test]
fn terrain_horizontal_direction_positive_uses_named_right_sample() {
    let mut f = Fixture::new(MOVING_HANDLER);
    f.a().direction = 1;
    f.run();
    assert!(
        f.world
            .calls
            .contains(&Work::Motion(Movement::PositiveX, RIGHT_SAMPLE, 1))
    );
}
#[test]
fn specialized_stage_reward_fallback_and_nonzero_override_preserve_a0_flags() {
    for (mapped, expected) in [(0, 0x23), (0x74, 0x74)] {
        let mut f = Fixture::new(TERRAIN_HANDLER);
        f.a().hit_marker = 1;
        f.a().animation = STAGE_REWARD_ANIMATION;
        {
            let actor_value = 1;
            f.a().set_local_state_code(actor_value);
        };
        {
            let actor_value = 0xff;
            f.a().set_state_code(actor_value);
        };
        f.stage = 2;
        f.catalog.stage_rewards[7] = mapped;
        f.run();
        assert_eq!(f.actor().local_state_code(), expected);
        assert_eq!(f.actor().state_code(), 0xa0);
        assert_eq!(f.actor().vertical_velocity, 0x0400);
    }
}
#[test]
fn catalog_decodes_all_object_consumers_from_matching_local_rom() {
    let rom = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom"))
        .expect("local matching ROM");
    let c = Catalog::original(&rom).unwrap();
    const BANK_ORIGIN: usize = 0x18010;
    const CPU_ORIGIN: usize = 0x8000;
    let at = |cpu: usize| BANK_ORIGIN + cpu - CPU_ORIGIN;
    const RECOVERED: usize = 0x8fb8;
    const STAGE: usize = 0x9016;
    const TRANSFORM_ANIMATION: usize = 0x8fc6;
    const TRANSFORM_HANDLER: usize = 0x8fda;
    const TRANSFORM_PROFILE: usize = 0x8fee;
    const TRANSFORM_OFFSET: usize = 0x9002;
    const SECONDARY: usize = 0x9025;
    const REWARD_SOUND: usize = 0x8c0c;
    for (data, cpu) in [
        (&c.recovered[..], RECOVERED),
        (&c.stage_rewards[..], STAGE),
        (&c.transform_animation[..], TRANSFORM_ANIMATION),
        (&c.transform_handler[..], TRANSFORM_HANDLER),
        (&c.transform_profile[..], TRANSFORM_PROFILE),
        (&c.transform_y_offset[..], TRANSFORM_OFFSET),
        (&c.secondary[..], SECONDARY),
        (&c.reward_sound[..], REWARD_SOUND),
    ] {
        assert_eq!(data, &rom[at(cpu)..at(cpu) + data.len()]);
    }
    assert_eq!(
        c.motion,
        [
            MotionPhase::Arc,
            MotionPhase::FallThenSlide,
            MotionPhase::ContactTransfer,
            MotionPhase::Arc,
            MotionPhase::Arc,
            MotionPhase::Arc,
            MotionPhase::SlidePose,
            MotionPhase::Arc,
            MotionPhase::TouchPose
        ]
    );
    const LAUNCH: usize = 0x890e;
    const NUDGE_LOW: usize = 0x8ede;
    const NUDGE_HIGH: usize = 0x8ee2;
    const DIRECTION: usize = 0xa2ae;
    const SPEED_LOW: usize = 0xa2b2;
    const SPEED_HIGH: usize = 0xa2b6;
    for i in 0..MOTION_PRESETS {
        let base = at(LAUNCH) + i * 4;
        assert_eq!(
            c.launch_vertical[i],
            i16::from_le_bytes([rom[base], rom[base + 1]])
        );
        assert_eq!(
            c.launch_horizontal[i],
            u16::from_le_bytes([rom[base + 2], rom[base + 3]])
        );
    }
    for i in 0..DISPENSE_COUNT {
        assert_eq!(
            c.slide_nudge[i],
            i16::from_le_bytes([rom[at(NUDGE_LOW) + i], rom[at(NUDGE_HIGH) + i]])
        );
        assert_eq!(c.dispenser_direction[i], rom[at(DIRECTION) + i]);
        assert_eq!(
            c.dispenser_speed[i],
            u16::from_le_bytes([rom[at(SPEED_LOW) + i], rom[at(SPEED_HIGH) + i]])
        );
    }
}
#[test]
fn catalog_truncated_source_rejects_missing_tail_consumers() {
    assert!(Catalog::original(&[]).is_err());
}

// B09 independently traced literal JSR869D. These exercise the actual B08 owner,
// separately from existing86B5 bypass contracts and B09caller boundary fixtures.
#[test]
fn pushing_entry_869d_nonzero_changed_blocks_before_replacement_and_launch() {
    let mut f = Fixture::new(TERRAIN_HANDLER);
    {
        let actor_value = f.a().state_code() | (TERRAIN_REPRESENTATION);
        f.a().set_state_code(actor_value);
    };
    f.a().animation = STAGE_REWARD_ANIMATION;
    f.changed = 1;
    let old = f.actors;
    let e = f.invoke(6);
    assert_eq!(f.actors, old);
    assert_eq!(f.changed, 1);
    assert!(f.world.calls.is_empty());
    assert_eq!(e, Effects::default());
}
#[test]
fn pushing_entry_869d_bypasses_player_swing_frame_gate_and_launches_same_call() {
    let mut f = Fixture::new(TERRAIN_HANDLER);
    f.player.pose_direction = 2;
    f.actors[0].animation = 0;
    f.actors[0].playback.reset_frame();
    f.a().animation = 0x1a;
    f.a().motion_flags = 0xff;
    f.invoke(6);
    assert_eq!(f.changed, 1);
    assert_eq!(f.actor().handler, MOVING_HANDLER);
    assert_eq!(f.actor().direction, 2);
    assert_eq!(f.actor().motion_flags, 0);
    assert_eq!(
        (f.actor().vertical_velocity, f.actor().horizontal_magnitude),
        (0x0321, 0x0123)
    );
    assert!(f.world.calls.is_empty());
}
#[test]
fn pushing_entry_869d_clears_representation_and_commits_replacement_before_launch() {
    let mut f = Fixture::new(TERRAIN_HANDLER);
    {
        let actor_value = ACTIVE | TERRAIN_REPRESENTATION;
        f.a().set_state_code(actor_value);
    };
    f.a().animation = 0x1b;
    f.invoke(6);
    assert_eq!(f.world.calls, vec![Work::ReplaceTerrain(SUBJECT)]);
    // Boundary fixture increments changed for its committed replacement, after
    // entry's own INC. It deliberately proves entry updates happen before call.
    assert_eq!(f.changed, 2);
    assert_eq!(f.world.changed_at_replace, vec![1]);
    assert_eq!(f.actor().state_code() & TERRAIN_REPRESENTATION, 0);
    assert_eq!(f.actor().handler, MOVING_HANDLER);
    assert_eq!(f.actor().state_code() & 0x0f, 1);
}

#[test]
fn pushing_entry_869d_reward_animation_reads_extended_tuple_and_keeps_unmasked_phase_bits() {
    const REWARD_LAUNCH_INDEX: usize = 0x25;
    let mut f = Fixture::new(TERRAIN_HANDLER);
    f.a().animation = STAGE_REWARD_ANIMATION;
    f.catalog.launch_vertical[REWARD_LAUNCH_INDEX] = -0x0d5f;
    f.catalog.launch_horizontal[REWARD_LAUNCH_INDEX] = 0xb520;
    f.invoke(6);
    assert_eq!(f.changed, 1);
    assert_eq!(f.actor().handler, MOVING_HANDLER);
    assert_eq!(f.actor().state_code(), ACTIVE | REWARD_LAUNCH_INDEX as u8);
    assert_eq!(
        (f.actor().vertical_velocity, f.actor().horizontal_magnitude),
        (-0x0d5f, 0xb520)
    );
    assert_eq!(f.bits, [0; SUPPRESSION_BYTES]);
}
#[test]
fn catalog_pushing_reward_launch_consumes_bounded_adjacent_instruction_data() {
    use romatlas_port::assets::Rom;
    let b =
        std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../input/game.rom"))
            .expect("local ROM");
    Rom::from_bytes(b.clone(), crate::config::ROM_SHA256).expect("matching ROM");
    let c = Catalog::original(&b).unwrap();
    for index in 0..TERRAIN_LAUNCH_ENTRIES {
        let p = TERRAIN_LAUNCH_PHYSICAL + index * 4;
        assert_eq!(
            c.launch_vertical[index],
            i16::from_le_bytes([b[p], b[p + 1]])
        );
        assert_eq!(
            c.launch_horizontal[index],
            u16::from_le_bytes([b[p + 2], b[p + 3]])
        );
    }
}

#[test]
fn contact_recovery_nonreaction_pose_enters_special_without_ordinary_reset() {
    // 8C9B BNE8CE1 bypasses8CAF: only the special entry's fields change.
    let mut f = Fixture::new(CONTACT_RECOVERY_HANDLER);
    {
        let actor_value = 0xa2;
        f.a().set_state_code(actor_value);
    };
    f.a().animation = 0x24;
    {
        let animation_value = 7;
        f.a().playback.set_frame_code(animation_value);
    };
    {
        let animation_value = 9;
        f.a().playback.set_tick_code(animation_value);
    };
    f.a().vertical_velocity = 0x1234;
    {
        let actor_value = 4;
        f.a().set_local_state_code(actor_value);
    };
    {
        let actor_value = 5;
        f.a().set_local_counter_code(actor_value);
    };
    f.a().motion_flags = 8;
    f.run();
    assert_eq!(
        (f.actor().handler, f.actor().profile),
        (TERRAIN_HANDLER, 0x80)
    );
    assert_eq!((f.actor().state_code(), f.actor().animation), (0xa2, 0x24));
    assert_eq!(
        (
            f.actor().playback.frame_code(),
            f.actor().playback.tick_code(),
            f.actor().vertical_velocity
        ),
        (7, 9, 0x1234)
    );
    assert_eq!(
        (
            f.actor().local_state_code(),
            f.actor().local_counter_code(),
            f.actor().motion_flags
        ),
        (0, 0, 0)
    );
    assert!(f.world.calls.is_empty());
}

//! Independent source expectations: trace.md covers every family and branch.
//! Boundary fixtures supply contact facts/mutations, never implement game rules.
use super::*;
use crate::{
    actors::{ACTIVE, ACTOR_SLOTS, ActorContact, VISIBLE},
    collision::{Extent, Position, SampleOffset, TerrainResponse},
    player::Action,
    status::{BLANK, Health, HealthSegment},
};
use std::collections::VecDeque;
// Source selector identities are encodings of independently traced families.
const JUMP_WALKER: u8 = 0x01;
const BOUNCE_WALKER: u8 = 0x02;
const THROWER: u8 = 0x03;
const PROJECTILE: u8 = 0x04;
const RANDOM_POSE: u8 = 0x05;
const PROXIMITY_RISE: u8 = 0x06;
const CONTACT_POSE: u8 = 0x07;
const WAITING_JUMPER: u8 = 0x10;
const PRESET_FLIGHT: u8 = 0x11;
const PAIRED_CONTROLLER: u8 = 0x12;
const LEDGE_WALKER: u8 = 0x18;
const AMBUSH: u8 = 0x19;
const ERUPTION: u8 = 0x1a;
const REPEATING_JUMP: u8 = 0x1b;
const COMPANION_PATROL: u8 = 0x1c;
const FAST_LEDGE_WALKER: u8 = 0x1e;
const TIMED_PATROL: u8 = 0x1f;
const VERTICAL_PATROL: u8 = 0x20;
const TRIGGERED_JUMP: u8 = 0x22;
const LEFT_WALKER: u8 = 0x23;
const APPROACH: u8 = 0x25;
const FIRST_TIMED_STEALER: u8 = 0x30;
const SECOND_TIMED_STEALER: u8 = 0x31;
const JUMPING_STEALER: u8 = 0x32;
const HIT_ONLY: u8 = 0x33;
const PLAYER_LAUNCHER: u8 = 0x34;
const SUBJECT: usize = 3;
const CHILD: usize = ACTOR_SLOTS - 1;
const CONTACT: u8 = 1;
const SECONDARY: u8 = 2;
const READY: u8 = ACTIVE | 1;
const MOVING: u8 = ACTIVE | 2;
const THROWING: u8 = ACTIVE | 3;
const PLAYER_X: u16 = 0x0210;
const ACTOR_X: u16 = 0x0200;
const ACTOR_Y: u8 = 0x70;
const SYNTHETIC_UP_OFFSET: u8 = 0xdc;
const SYNTHETIC_DOWN_OFFSET: u8 = 0x1c;
const SYNTHETIC_UP_POSE: u8 = 0x2c;
const SYNTHETIC_DOWN_POSE: u8 = 0x3a;
const IDENTITY: u8 = 0x42;
#[derive(Default)]
struct Boundary {
    calls: Vec<Work>,
    facts: VecDeque<ContactFacts>,
    permission: bool,
    post_velocity: Option<i16>,
    post_direction: Option<u8>,
    post_y: Option<u8>,
    contact_action: Option<Action>,
    reaction_animation: Option<u8>,
}
impl World for Boundary {
    fn motion(
        &mut self,
        actors: &mut ActorPool,
        _: &mut Player,
        _: &mut u8,
        slot: usize,
        movement: Movement,
        start: u8,
        remaining: u8,
        _: u8,
    ) -> ContactFacts {
        self.calls.push(Work::Motion {
            movement,
            start,
            remaining,
        });
        if let Some(v) = self.post_velocity {
            actors[slot].vertical_velocity = v;
        }
        if let Some(v) = self.post_direction {
            actors[slot].direction = v;
        }
        if let Some(v) = self.post_y {
            actors[slot].position.screen_y = v;
        }
        self.facts.pop_front().unwrap_or_default()
    }
    fn integrate(&mut self, _: &mut ActorPool, _: usize, operation: Integration, _: u8) {
        self.calls.push(Work::Integrate(operation));
    }
    fn player_contact(&mut self, _: &mut ActorPool, p: &mut Player, _: usize) -> bool {
        self.calls.push(Work::PlayerContact);
        if let Some(a) = self.contact_action {
            p.action = a;
        }
        self.permission
    }
    fn reaction(&mut self, a: &mut ActorPool, _: &mut Player, _: &mut RandomState, slot: usize) {
        self.calls.push(Work::Reaction);
        self.reaction_animation = Some(a[slot].animation);
    }
    fn launch_terrain(&mut self, _: &mut ActorPool, slot: usize, _: u8) {
        self.calls.push(Work::LaunchTerrain(slot));
    }
}
struct Fixture {
    actors: ActorPool,
    player: Player,
    random: RandomState,
    status: Status,
    special: u8,
    geometry: Geometry,
    spawn: SpawnCatalog,
    catalog: Catalog,
    world: Boundary,
}
impl Fixture {
    fn new(selector: u8) -> Self {
        let mut actors = [ActorContact::default(); ACTOR_SLOTS];
        actors[0].position.world_x = PLAYER_X;
        actors[0].position.screen_y = ACTOR_Y;
        actors[SUBJECT] = ActorContact {
            control: crate::actors::ActorControl::from_codes(READY, 0, 0, selector),
            render: VISIBLE,
            handler: selector,
            direction: 1,
            position: Position {
                world_x: ACTOR_X,
                screen_y: ACTOR_Y,
                ..Position::default()
            },
            horizontal_magnitude: 0x0100,
            spawn_identity: Some(IDENTITY),
            ..ActorContact::default()
        };
        let segment = HealthSegment {
            y: 0,
            tile: 0,
            depleted: 0,
            x: 0,
        };
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
                health: Health::from_overlay([segment; 5], 0, 0),
                countdown: [0; 3].into(),
                divider: 0,
                dirty: 0,
                ui_pending: 0,
            },
            special: 0,
            geometry: Geometry {
                offsets: vec![SampleOffset::default(); 101],
                extents: [Extent::default(); 32],
                responses: [TerrainResponse::Solid; 16],
                velocity_presets: [0x0133, 0x07a9],
                particle_y: [0; 4],
                particle_x: [0; 4],
                particle_ticks: [0; 4],
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
            catalog: Catalog {
                attach_y: [SYNTHETIC_UP_OFFSET, SYNTHETIC_DOWN_OFFSET],
                attach_poses: [SYNTHETIC_UP_POSE, SYNTHETIC_DOWN_POSE],
                contact_profiles: [0x12, 0x13, 0x13],
                flight_sequence: [0; 14],
                companion_directions: [2, 1, 1, 2],
                companion_flips: [0, FLIP_X, FLIP_X, 0],
                companion_random_phases: [0, 4],
                companion_animations: [0x1d, 0x1e, 0x1d, 0x1e, 0x1d, 0x1e, 0x1d, 0x1e],
                companion_animation_flips: [FLIP_X, 0, FLIP_X, 0, FLIP_X, 0, FLIP_X, 0],
                ambush_sequence: [0; 14],
                ambush_flips: [0, FLIP_X],
                eruption_velocity: [0x0123, 0x0234, 0x0345],
                eruption_rise_event: [8, 9, 0],
                eruption_fall_event: [0x2a, 0x38, 0],
                fast_support: [0x1e, 0x20],
                stealer_intervals: [0x12, 0x09],
            },
            world: Boundary::default(),
        }
    }
    fn run(&mut self) -> Effects {
        step(
            &mut self.actors,
            SUBJECT,
            &mut Context {
                player: &mut self.player,
                random: &mut self.random,
                status: &mut self.status,
                special_spawn: &mut self.special,
                gravity: crate::actors::DEFAULT_GRAVITY,
                geometry: &self.geometry,
                spawn: &self.spawn,
                catalog: &self.catalog,
            },
            &mut self.world,
        )
    }
    fn facts(&mut self, bits: &[u8]) {
        self.world.facts = bits
            .iter()
            .map(|&bits| ContactFacts { bits, actor: None })
            .collect();
    }
    fn actor(&self) -> &ActorContact {
        &self.actors[SUBJECT]
    }
    fn actor_mut(&mut self) -> &mut ActorContact {
        &mut self.actors[SUBJECT]
    }
    fn init(&mut self) {
        {
            let actor_value = ACTIVE;
            self.actor_mut().set_state_code(actor_value);
        };
    }
    fn full(&mut self) {
        for a in &mut self.actors {
            {
                let actor_value = ACTIVE;
                a.set_state_code(actor_value);
            };
        }
    }
}
fn motion(movement: Movement, start: u8, remaining: u8) -> Work {
    Work::Motion {
        movement,
        start,
        remaining,
    }
}
fn original_rom() -> Vec<u8> {
    let bytes =
        std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../input/game.rom"))
            .expect("matching local ROM");
    romatlas_port::assets::Rom::from_bytes(bytes.clone(), crate::config::ROM_SHA256)
        .expect("ROM identity");
    bytes
}

#[test]
fn complete_selector_domain_matches_all_twentyfive_distinct_source_targets() {
    use Family::*;
    let expected = [
        (1, JumpWalker),
        (2, BounceWalker),
        (3, Thrower),
        (4, Projectile),
        (5, RandomPose),
        (6, ProximityRise),
        (7, ContactPose),
        (0x10, WaitingJumper),
        (0x11, PresetFlight),
        (0x12, PairedController),
        (0x18, LedgeWalker),
        (0x19, Ambush),
        (0x1a, Eruption),
        (0x1b, RepeatingJump),
        (0x1c, CompanionPatrol),
        (0x1e, FastLedgeWalker),
        (0x1f, TimedPatrol),
        (0x20, VerticalPatrol),
        (0x22, TriggeredJump),
        (0x23, LeftWalker),
        (0x25, Approach),
        (0x30, TimedStealer),
        (0x31, TimedStealer),
        (0x32, JumpingStealer),
        (0x33, HitOnly),
        (0x34, PlayerLauncher),
    ];
    for selector in 0..crate::actors::HANDLER_COUNT as u8 {
        assert_eq!(
            family(selector),
            expected
                .iter()
                .find(|(s, _)| *s == selector)
                .map(|(_, f)| *f),
            "selector {selector:02x}"
        );
    }
}
#[test]
fn original_family_tables_decode_all_bounded_consumers_from_matching_rom() {
    // Independent physical recipes follow source symbols; no original payload is embedded.
    const SWITCH_BIAS: usize = 0x10010;
    let rom = original_rom();
    let c = Catalog::original(&rom).unwrap();
    let entries: [(&[u8], usize); 14] = [
        (&c.attach_y, 0x8661),
        (&c.attach_poses, 0x8663),
        (&c.contact_profiles, 0x8665),
        (&c.flight_sequence, 0x9544),
        (&c.companion_directions, 0x96bb),
        (&c.companion_flips, 0x96bf),
        (&c.companion_random_phases, 0x96c3),
        (&c.companion_animations, 0x96c5),
        (&c.companion_animation_flips, 0x96cd),
        (&c.ambush_sequence, 0x9946),
        (&c.ambush_flips, 0x9954),
        (&c.eruption_rise_event, 0x9a7b),
        (&c.eruption_fall_event, 0x9a7e),
        (&c.fast_support, 0x9df5),
    ];
    for (values, cpu) in entries {
        assert_eq!(
            values,
            &rom[SWITCH_BIAS + cpu..SWITCH_BIAS + cpu + values.len()]
        );
    }
    assert_eq!(
        c.stealer_intervals,
        &rom[SWITCH_BIAS + 0xacbc..SWITCH_BIAS + 0xacbe]
    );
    for (i, actual) in c.eruption_velocity.iter().enumerate() {
        assert_eq!(
            *actual,
            i16::from_le_bytes([rom[SWITCH_BIAS + 0x9a75 + i], rom[SWITCH_BIAS + 0x9a78 + i]])
        );
    }
}
#[test]
fn truncated_original_tables_are_reported_as_errors_without_partial_catalog() {
    for end in [
        0, 0x18677, 0x19561, 0x196e4, 0x19965, 0x19a90, 0x19e06, 0x1accd,
    ] {
        assert!(Catalog::original(&vec![0; end]).is_err(), "length {end:x}");
    }
}
#[test]
fn each_reaction_prefix_keeps_its_exact_animation_and_retry_policy() {
    for (selector, animation, retry) in [
        (1, Some(0x0c), true),
        (2, Some(0x0c), false),
        (3, None, true),
        (5, Some(0x0f), false),
        (6, Some(0x11), false),
        (7, None, false),
        (0x18, None, false),
        (0x19, None, false),
        (0x1a, None, false),
        (0x1b, None, true),
        (0x1c, Some(0x56), false),
        (0x1e, None, false),
        (0x1f, None, false),
        (0x20, None, false),
        (0x25, None, true),
        (0x30, None, false),
        (0x32, None, true),
        (0x33, None, false),
    ] {
        let mut f = Fixture::new(selector);
        f.actor_mut().animation = 0x62;
        f.actor_mut().hit_marker = 1;
        f.run();
        assert_eq!(
            f.world.calls.last(),
            Some(&Work::Reaction),
            "selector{selector:x}"
        );
        assert_eq!(f.world.reaction_animation, Some(animation.unwrap_or(0x62)));
        assert_eq!(f.special, if retry { SHARED_REACTION_RETRY } else { 0 });
    }
}
#[test]
fn invisibility_reaction_gates_do_not_gain_hit_only_animation_rewrites() {
    for selector in [
        JUMP_WALKER,
        THROWER,
        AMBUSH,
        REPEATING_JUMP,
        APPROACH,
        JUMPING_STEALER,
    ] {
        let mut f = Fixture::new(selector);
        f.actor_mut().render = 0;
        f.actor_mut().animation = 0x62;
        f.run();
        assert_eq!(f.world.reaction_animation, Some(0x62));
    }
}
#[test]
fn direct_culling_preserves_identity_and_distinguishes_render_and_retry_writes() {
    for selector in [PROXIMITY_RISE, CONTACT_POSE, PRESET_FLIGHT] {
        let mut f = Fixture::new(selector);
        f.actor_mut().render = FLIP_X;
        f.run();
        assert_eq!(f.actor().state_code(), 0);
        assert_eq!(f.actor().spawn_identity, Some(IDENTITY));
        assert_eq!(
            f.actor().render,
            if selector == PRESET_FLIGHT { 0 } else { FLIP_X }
        );
        assert_eq!(
            f.special,
            if selector == PRESET_FLIGHT {
                SHARED_REACTION_RETRY
            } else {
                0
            }
        );
        assert!(f.world.calls.is_empty());
    }
}
#[test]
fn jumpwalker_first_call_initializes_faces_and_uses_horizontal_probe_for_launch() {
    let mut f = Fixture::new(JUMP_WALKER);
    f.init();
    f.facts(&[0, CONTACT]);
    f.run();
    assert_eq!(f.actor().state_code(), MOVING);
    assert_eq!(f.actor().vertical_velocity, BOUNCE_HEIGHT);
    assert_eq!(f.actor().horizontal_magnitude, 0x0100);
    assert_eq!(f.actor().direction, 1);
    assert_eq!(
        f.world.calls,
        vec![
            motion(Movement::Fall, 0x1e, 2),
            motion(Movement::PositiveX, 0x18, 2)
        ]
    );
}
#[test]
fn jumpwalker_floor_contact_alone_cannot_launch_and_nonzero_velocity_blocks_wall_launch() {
    for (bits, v) in [([CONTACT, 0], 0), ([CONTACT, CONTACT], 1)] {
        let mut f = Fixture::new(JUMP_WALKER);
        f.actor_mut().vertical_velocity = v;
        f.facts(&bits);
        f.run();
        assert_eq!(f.actor().state_code(), READY);
        assert_eq!(f.actor().vertical_velocity, v);
    }
}
#[test]
fn jumpwalker_apex_falls_same_call_but_nonnegative_rise_resets_animation() {
    for negative in [false, true] {
        let mut f = Fixture::new(JUMP_WALKER);
        {
            let actor_value = MOVING;
            f.actor_mut().set_state_code(actor_value);
        };
        {
            let animation_value = 4;
            f.actor_mut().playback.set_frame_code(animation_value);
        };
        {
            let animation_value = 5;
            f.actor_mut().playback.set_tick_code(animation_value);
        };
        f.world.post_velocity = Some(if negative { -1 } else { 1 });
        f.run();
        assert_eq!(f.actor().playback.frame_code(), 0);
        assert_eq!(f.actor().playback.tick_code(), 0);
        assert_eq!(f.world.calls[0], motion(Movement::Rise, 0x21, 2));
        assert_eq!(f.world.calls.len(), if negative { 3 } else { 2 });
        if negative {
            assert_eq!(f.world.calls[1], motion(Movement::Fall, 0x1e, 2));
            assert_eq!(f.actor().state_code(), READY);
        }
    }
}
#[test]
fn bouncewalker_initialization_reads_preset_zero_and_reverses_only_on_horizontal_contact() {
    let mut f = Fixture::new(BOUNCE_WALKER);
    f.init();
    f.geometry.velocity_presets[0] = 0x025a;
    f.facts(&[CONTACT, CONTACT]);
    f.run();
    assert_eq!(f.actor().horizontal_magnitude, 0x025a);
    assert_eq!(f.actor().direction, 2);
    assert_eq!(f.actor().vertical_velocity, 0);
    assert_eq!(f.actor().state_code(), READY);
}
#[test]
fn bouncewalker_airborne_clocks_reset_before_horizontal_and_floor_keeps_clocks() {
    for bits in [0, CONTACT] {
        let mut f = Fixture::new(BOUNCE_WALKER);
        {
            let animation_value = 4;
            f.actor_mut().playback.set_frame_code(animation_value);
        };
        {
            let animation_value = 5;
            f.actor_mut().playback.set_tick_code(animation_value);
        };
        f.facts(&[bits, 0]);
        f.run();
        assert_eq!(
            (
                f.actor().playback.frame_code(),
                f.actor().playback.tick_code()
            ),
            if bits == 0 { (0, 0) } else { (4, 5) }
        );
    }
}
#[test]
fn thrower_animation_child_inherits_selected_fields_and_preserves_stale_child_fields() {
    let mut f = Fixture::new(THROWER);
    {
        let actor_value = THROWING;
        f.actor_mut().set_state_code(actor_value);
    };
    {
        let animation_value = 1;
        f.actor_mut().playback.set_frame_code(animation_value);
    };
    {
        let actor_value = THROW_DURATION;
        f.actor_mut().set_local_state_code(actor_value);
    };
    f.actor_mut().position.screen_y = 4;
    f.actor_mut().render = VISIBLE | FLIP_X;
    f.actors[CHILD].direction = 0x55;
    f.actors[CHILD].hit_marker = 0x22;
    f.actors[CHILD].spawn_identity = Some(9);
    f.actors[CHILD].vertical_velocity = 0x1234;
    let e = f.run();
    let c = f.actors[CHILD];
    assert_eq!(
        (c.handler, c.animation, c.profile),
        (
            THROW_PROJECTILE_HANDLER,
            THROW_PROJECTILE_ANIMATION,
            THROW_PROJECTILE_PROFILE
        )
    );
    assert_eq!(
        (c.state_code(), c.render, c.position.world_x),
        (THROWING, VISIBLE | FLIP_X, ACTOR_X)
    );
    assert_eq!(c.position.screen_y, 0xfc);
    assert_eq!(c.horizontal_magnitude, 0x0200);
    assert_eq!((c.playback.tick_code(), c.playback.frame_code()), (0, 0));
    assert_eq!(
        (
            c.direction,
            c.hit_marker,
            c.spawn_identity,
            c.vertical_velocity
        ),
        (0x55, 0x22, Some(9), 0x1234)
    );
    assert_eq!(
        (f.actor().local_counter_code(), f.actor().local_state_code()),
        (1, THROW_DURATION - 1)
    );
    assert_eq!(e.sounds, vec![THROW_SOUND]);
    assert_eq!(e.spawned, vec![CHILD]);
}
#[test]
fn thrower_low_nibble_frame_gate_full_pool_retry_and_duplicate_latch() {
    for (frame, counter, full) in [
        (0, 0, false),
        (RESTART_FRAME, 0, false),
        (1, 1, false),
        (1, 0, true),
    ] {
        let mut f = Fixture::new(THROWER);
        if full {
            f.full();
        }
        {
            let actor_value = THROWING;
            f.actor_mut().set_state_code(actor_value);
        };
        {
            let animation_value = frame;
            f.actor_mut().playback.set_frame_code(animation_value);
        };
        {
            let actor_value = counter;
            f.actor_mut().set_local_counter_code(actor_value);
        };
        {
            let actor_value = 3;
            f.actor_mut().set_local_state_code(actor_value);
        };
        let e = f.run();
        assert!(e.spawned.is_empty());
        assert!(e.sounds.is_empty());
        assert_eq!(f.actor().local_counter_code(), counter);
        assert_eq!(f.actor().local_state_code(), 2);
    }
}
#[test]
fn thrower_countdown_zero_wrap_and_expiry_preserve_persistent_bits_and_child_latch() {
    for (old, new, phase, anim) in [(0, 0xff, 0xb3, 0x16), (1, 0, 0xb1, 0x14)] {
        let mut f = Fixture::new(THROWER);
        {
            let actor_value = 0xb3;
            f.actor_mut().set_state_code(actor_value);
        };
        f.actor_mut().animation = 0x16;
        {
            let actor_value = 1;
            f.actor_mut().set_local_counter_code(actor_value);
        };
        {
            let actor_value = old;
            f.actor_mut().set_local_state_code(actor_value);
        };
        f.run();
        assert_eq!(
            (
                f.actor().local_state_code(),
                f.actor().state_code(),
                f.actor().animation
            ),
            (new, phase, anim)
        );
        assert_eq!(f.actor().local_counter_code(), 1);
    }
}
#[test]
fn thrower_distance_gate_is_strict_and_writes_restart_metadata() {
    for (distance, starts) in [(0x50, true), (0x51, false)] {
        let mut f = Fixture::new(THROWER);
        f.actors[0].position.world_x = ACTOR_X + distance;
        f.facts(&[CONTACT, 0]);
        f.run();
        if starts {
            assert_eq!(
                (
                    f.actor().state_code(),
                    f.actor().animation,
                    f.actor().playback.frame_code(),
                    f.actor().playback.tick_code(),
                    f.actor().playback.restart_code(),
                    f.actor().local_state_code()
                ),
                (THROWING, 0x16, RESTART_FRAME, 0, 1, THROW_DURATION)
            );
        } else {
            assert_eq!(f.actor().state_code(), READY);
        }
    }
}
#[test]
fn thrower_horizontal_wall_precedes_proximity_and_prior_child_disables_new_throw() {
    let mut f = Fixture::new(THROWER);
    f.facts(&[CONTACT, CONTACT]);
    f.run();
    assert_eq!(f.actor().state_code(), MOVING);
    assert_eq!(f.actor().vertical_velocity, BOUNCE_HEIGHT);
    let mut f = Fixture::new(THROWER);
    {
        let actor_value = 1;
        f.actor_mut().set_local_counter_code(actor_value);
    };
    f.facts(&[CONTACT, 0]);
    f.run();
    assert_eq!(f.actor().state_code(), READY);
}
#[test]
fn thrower_rise_apex_falls_in_same_call_and_airborne_pose_resets_clocks() {
    let mut f = Fixture::new(THROWER);
    {
        let actor_value = MOVING;
        f.actor_mut().set_state_code(actor_value);
    };
    f.world.post_velocity = Some(-1);
    f.run();
    assert_eq!(
        f.world.calls[..2],
        [motion(Movement::Rise, 9, 2), motion(Movement::Fall, 6, 2)]
    );
    assert_eq!(
        (
            f.actor().animation,
            f.actor().playback.frame_code(),
            f.actor().playback.tick_code()
        ),
        (0x15, 0, 0)
    );
}
#[test]
fn projectile_uses_flip_motion_then_contact_but_only_motion_collision_removes() {
    for (flip, bits) in [(0, 0), (FLIP_X, CONTACT)] {
        let mut f = Fixture::new(PROJECTILE);
        f.actor_mut().render = VISIBLE | flip;
        f.facts(&[bits]);
        f.world.permission = true;
        f.world.contact_action = Some(Action::Damage);
        f.run();
        assert_eq!(
            f.world.calls,
            vec![
                motion(
                    if flip == 0 {
                        Movement::NegativeX
                    } else {
                        Movement::PositiveX
                    },
                    if flip == 0 { 0x25 } else { 0x24 },
                    0
                ),
                Work::PlayerContact
            ]
        );
        assert_eq!(f.player.action, Action::Damage);
        assert_eq!(f.actor().state_code(), if bits == 0 { READY } else { 0 });
        assert_eq!(
            f.actor().spawn_identity,
            if bits == 0 { Some(IDENTITY) } else { None }
        );
    }
}
#[test]
fn randompose_exact_frame_includes_compare_carry_and_mutates_only_third_random_word() {
    for (words, expected, pose, sound) in [
        ([0, 0, 9, 8], 1, 0x0e, true),
        ([0xff, 0, 9, 8], 0, 0x0d, false),
    ] {
        let mut f = Fixture::new(RANDOM_POSE);
        f.random.words = words;
        {
            let animation_value = 0x0c;
            f.actor_mut().playback.set_frame_code(animation_value);
        };
        let e = f.run();
        assert_eq!(f.random.words, [words[0], words[1], expected, words[3]]);
        assert_eq!(f.actor().animation, pose);
        assert_eq!(
            (
                f.actor().playback.tick_code(),
                f.actor().playback.frame_code()
            ),
            (0, 0)
        );
        assert_eq!(e.sounds, if sound { vec![0x2a] } else { vec![] });
    }
}
#[test]
fn randompose_neighbor_frames_skip_rng_and_full_world_comparison_handles_equality_and_pages() {
    for (x, flip) in [
        (PLAYER_X - 1, true),
        (PLAYER_X, false),
        (PLAYER_X + 0x100, false),
    ] {
        let mut f = Fixture::new(RANDOM_POSE);
        f.actor_mut().position.world_x = x;
        {
            let animation_value = 0x0b;
            f.actor_mut().playback.set_frame_code(animation_value);
        };
        let old = f.random;
        f.run();
        assert_eq!(f.random, old);
        assert_eq!(f.actor().render & FLIP_X != 0, flip);
        assert_eq!(f.actor().playback.frame_code(), 0x0b);
    }
}
#[test]
fn proximityrise_support_gate_controls_cooldown_and_strict_horizontal_threshold() {
    for (support, distance, timer, want, phase) in [
        (0, 1, 3, 3, READY),
        (CONTACT, 0x32, 3, 0, READY),
        (CONTACT, 0x31, 3, 2, READY),
        (CONTACT, 0x31, 0, 0, MOVING),
    ] {
        let mut f = Fixture::new(PROXIMITY_RISE);
        {
            let actor_value = timer;
            f.actor_mut().set_local_state_code(actor_value);
        };
        f.actors[0].position.world_x = ACTOR_X + distance;
        f.facts(&[support]);
        f.run();
        assert_eq!(f.actor().local_state_code(), want);
        assert_eq!(f.actor().state_code(), phase);
        assert_eq!(f.world.calls, vec![motion(Movement::NegativeY, 0x27, 0)]);
    }
}
#[test]
fn proximityrise_stop_boundaries_are_y_b0_or_vertical_distance_below_eighteen_hex() {
    for (y, dy, stops) in [(0xaf, 0x18, false), (0xaf, 0x17, true), (0xb0, 0x18, true)] {
        let mut f = Fixture::new(PROXIMITY_RISE);
        {
            let actor_value = MOVING;
            f.actor_mut().set_state_code(actor_value);
        };
        f.actor_mut().position.screen_y = y;
        f.actors[0].position.screen_y = y - dy;
        f.run();
        assert_eq!(f.world.calls, vec![Work::Integrate(Integration::PositiveY)]);
        assert_eq!(f.actor().vertical_velocity, 0x0300);
        assert_eq!(f.actor().state_code(), if stops { READY } else { MOVING });
        assert_eq!(f.actor().local_state_code(), if stops { 0x96 } else { 0 });
    }
}
#[test]
fn contactpose_player_death_precedes_invisible_hit_and_all_local_work() {
    let mut f = Fixture::new(CONTACT_POSE);
    f.player.action = Action::Death;
    f.actor_mut().render = 0;
    f.actor_mut().hit_marker = 1;
    let old = f.actors;
    f.run();
    assert_eq!(f.actors, old);
    assert!(f.world.calls.is_empty());
}
#[test]
fn contactpose_opening_gate_and_animation_twelve_relative_y_return() {
    for (distance, animation, actor_y, frame) in [
        (0x0c, 0x12, ACTOR_Y, 0),
        (0x0b, 0x12, ACTOR_Y, 1),
        (0x0b, 0x12, ACTOR_Y - 1, 0),
        (0x0b, 0x14, ACTOR_Y - 1, 1),
    ] {
        let mut f = Fixture::new(CONTACT_POSE);
        {
            let actor_value = ACTIVE;
            f.actor_mut().set_state_code(actor_value);
        };
        f.actor_mut().animation = animation;
        f.actor_mut().position.screen_y = actor_y;
        f.actors[0].position.world_x = ACTOR_X + distance;
        f.run();
        assert_eq!(f.actor().playback.frame_code(), frame);
        if animation == 0x12 && actor_y < ACTOR_Y {
            assert!(f.world.calls.is_empty());
        } else {
            assert_eq!(
                f.actor().profile,
                f.catalog.contact_profiles[frame as usize]
            );
        }
    }
}
#[test]
fn contactpose_frame_two_sound_precedes_permission_and_eight_pixel_attach_boundary() {
    for (distance, action) in [(7, Action::Protected), (8, Action::Damage)] {
        let mut f = Fixture::new(CONTACT_POSE);
        {
            let actor_value = ACTIVE;
            f.actor_mut().set_state_code(actor_value);
        };
        {
            let animation_value = 2;
            f.actor_mut().playback.set_frame_code(animation_value);
        };
        f.actor_mut().animation = 0x12;
        f.actors[0].position.world_x = ACTOR_X + distance;
        f.world.permission = true;
        let e = f.run();
        assert_eq!(e.sounds, vec![0x19]);
        assert_eq!(f.player.action, action);
        if distance == 7 {
            assert_eq!(f.player.protection, CONTACT_PROTECTION);
            assert_eq!(f.actors[0].position.world_x, ACTOR_X);
            assert_eq!(f.actors[0].position.screen_y, 0x4c);
            assert_eq!(f.actors[0].animation, SYNTHETIC_UP_POSE);
            assert_eq!(f.actors[0].horizontal_magnitude, 0x0300);
            assert_eq!(f.actors[0].vertical_velocity, 0);
            assert_eq!(
                (
                    f.actor().state_code(),
                    f.actor().playback.frame_code(),
                    f.actor().playback.tick_code()
                ),
                (READY, 2, FROZEN_TICK)
            );
        }
    }
}
#[test]
fn contactpose_denied_contact_never_attaches_even_when_geometrically_near() {
    let mut f = Fixture::new(CONTACT_POSE);
    {
        let actor_value = ACTIVE;
        f.actor_mut().set_state_code(actor_value);
    };
    {
        let animation_value = 1;
        f.actor_mut().playback.set_frame_code(animation_value);
    };
    f.actors[0].position.world_x = ACTOR_X;
    f.player.special_mode = 1;
    f.world.contact_action = Some(Action::Damage);
    f.run();
    assert_eq!(f.player.action, Action::Damage);
    assert_eq!(f.player.protection, 0);
    assert_eq!(f.actor().state_code(), ACTIVE);
}
#[test]
fn contactpose_second_offset_wraps_and_uses_second_pose_without_erasing_queue() {
    let mut f = Fixture::new(CONTACT_POSE);
    {
        let actor_value = ACTIVE;
        f.actor_mut().set_state_code(actor_value);
    };
    f.actor_mut().animation = 0x14;
    {
        let animation_value = 1;
        f.actor_mut().playback.set_frame_code(animation_value);
    };
    f.actor_mut().position.screen_y = 0xf0;
    f.actors[0].position.world_x = ACTOR_X;
    f.player.queued = Some(Action::TerrainPose);
    f.world.permission = true;
    f.run();
    assert_eq!(f.actors[0].position.screen_y, 0x0c);
    assert_eq!(f.actors[0].animation, SYNTHETIC_DOWN_POSE);
    assert_eq!(f.player.queued, Some(Action::TerrainPose));
}
#[test]
fn contactpose_release_exact_protection_eight_and_zero_have_different_effects() {
    for protection in [9, 8, 0] {
        let mut f = Fixture::new(CONTACT_POSE);
        f.actor_mut().animation = 0x12;
        {
            let animation_value = 2;
            f.actor_mut().playback.set_frame_code(animation_value);
        };
        f.actor_mut().playback.freeze_at_start();
        f.actor_mut().render = VISIBLE | FLIP_X | 3;
        f.actors[0].render = VISIBLE | 3;
        f.player.protection = protection;
        let e = f.run();
        match protection {
            8 => {
                assert_eq!(f.actor().render, (VISIBLE & 0xfc) ^ FLIP_X);
                assert_eq!(f.actor().animation, 0x13);
                assert_eq!(e.sounds, vec![0x1a]);
                assert_eq!(f.actor().state_code(), READY);
            }
            0 => {
                assert_eq!(f.actor().state_code(), ACTIVE);
                assert_eq!(f.actor().animation, 0x11);
                assert_eq!(f.actor().render, VISIBLE | 3);
            }
            _ => {
                assert_eq!(f.actor().playback.frame_code(), 2);
                assert!(e.sounds.is_empty());
            }
        }
    }
}
#[test]
fn waitingjumper_initial_wait_expires_without_same_call_start_then_exact_frame_three_starts() {
    let mut f = Fixture::new(WAITING_JUMPER);
    f.init();
    f.run();
    assert_eq!(f.actor().local_state_code(), 0x77);
    assert_eq!(f.actor().horizontal_magnitude, 0x0200);
    assert!(f.world.calls.is_empty());
    for (timer, frame, moves) in [(1, 3, false), (0, 2, false), (0, 3, true)] {
        let mut f = Fixture::new(WAITING_JUMPER);
        {
            let actor_value = timer;
            f.actor_mut().set_local_state_code(actor_value);
        };
        {
            let animation_value = frame;
            f.actor_mut().playback.set_frame_code(animation_value);
        };
        f.run();
        assert_eq!(!f.world.calls.is_empty(), moves);
        if moves {
            assert_eq!(f.actor().state_code(), MOVING);
            assert_eq!(f.actor().animation, 0x2d);
            assert_ne!(f.actor().profile & ACTIVE, 0);
            assert_eq!(f.world.calls[0], motion(Movement::Fall, 6, 2));
        }
    }
}
#[test]
fn waitingjumper_positive_counter_rises_then_stores_negative_apex_without_falling() {
    let mut f = Fixture::new(WAITING_JUMPER);
    {
        let actor_value = MOVING;
        f.actor_mut().set_state_code(actor_value);
    };
    {
        let actor_value = 1;
        f.actor_mut().set_local_counter_code(actor_value);
    };
    {
        let animation_value = 3;
        f.actor_mut().playback.set_frame_code(animation_value);
    };
    f.world.post_velocity = Some(-0x100);
    f.run();
    assert_eq!(f.world.calls, vec![motion(Movement::Rise, 9, 2)]);
    assert_eq!(f.actor().local_counter_code(), 0xff);
    assert_eq!(f.actor().vertical_velocity, 0);
    assert_eq!(f.actor().playback.tick_code(), 0);
}
#[test]
fn waitingjumper_falling_negative_counter_lands_in_wait_pose_before_horizontal_and_hit() {
    let mut f = Fixture::new(WAITING_JUMPER);
    {
        let actor_value = MOVING;
        f.actor_mut().set_state_code(actor_value);
    };
    {
        let actor_value = 0xff;
        f.actor_mut().set_local_counter_code(actor_value);
    };
    f.actor_mut().hit_marker = 1;
    f.facts(&[CONTACT]);
    f.run();
    assert_eq!(f.world.calls, vec![motion(Movement::Fall, 6, 2)]);
    assert_eq!(
        (
            f.actor().animation,
            f.actor().local_state_code(),
            f.actor().local_counter_code(),
            f.actor().state_code()
        ),
        (0x42, 0x1e, 0, READY)
    );
}
#[test]
fn waitingjumper_no_ground_suppresses_hit_but_grounded_tick_zero_sounds_before_reaction() {
    for support in [0, CONTACT] {
        let mut f = Fixture::new(WAITING_JUMPER);
        {
            let actor_value = MOVING;
            f.actor_mut().set_state_code(actor_value);
        };
        f.actor_mut().hit_marker = 1;
        f.facts(&[support, 0]);
        let e = f.run();
        if support == 0 {
            assert!(!f.world.calls.contains(&Work::Reaction));
            assert!(e.sounds.is_empty());
        } else {
            assert_eq!(f.world.calls.last(), Some(&Work::Reaction));
            assert_eq!(e.sounds, vec![0x31]);
        }
    }
}
#[test]
fn waitingjumper_obstruction_invokes_live_handler_eight_transition_and_launches() {
    let mut f = Fixture::new(WAITING_JUMPER);
    {
        let actor_value = MOVING;
        f.actor_mut().set_state_code(actor_value);
    };
    f.actor_mut().profile = 0x91;
    f.facts(&[CONTACT, CONTACT]);
    f.world.facts[1].actor = Some(7);
    {
        let actor_value = 8;
        f.actors[7].set_handler(actor_value);
    };
    let e = f.run();
    assert!(f.world.calls.contains(&Work::LaunchTerrain(7)));
    assert_eq!(
        (
            f.actor().animation,
            f.actor().vertical_velocity,
            f.actor().local_counter_code(),
            f.actor().direction,
            f.actor().profile
        ),
        (0x2e, PATROL_JUMP_SPEED, 1, 2, 0x11)
    );
    assert_eq!(e.sounds, vec![0x31, 0x1d]);
}
#[test]
fn presetflight_only_animation_eightyfive_honors_hit_and_skips_distance_audio() {
    for animation in [0x85, 0x40] {
        let mut f = Fixture::new(PRESET_FLIGHT);
        f.init();
        f.actor_mut().animation = animation;
        f.actor_mut().hit_marker = 1;
        f.actors[0].position.world_x = ACTOR_X + 0x60;
        let e = f.run();
        if animation == 0x85 {
            assert_eq!(f.world.calls, vec![Work::Reaction]);
        } else {
            assert!(f.world.calls.is_empty());
            assert!(e.sounds.is_empty());
            assert_eq!(f.actor().state_code(), ACTIVE);
        }
    }
}
#[test]
fn presetflight_activation_strict_distance_and_first_phase_preserves_existing_flip() {
    let mut f = Fixture::new(PRESET_FLIGHT);
    f.init();
    f.actor_mut().animation = 0x40;
    f.actor_mut().position.world_x = PLAYER_X + 0x5f;
    f.actor_mut().render = VISIBLE | FLIP_X;
    let e = f.run();
    assert_eq!(e.sounds, vec![0x30]);
    assert_eq!(f.actor().direction, 2);
    assert_eq!(f.actor().render, VISIBLE | FLIP_X);
    assert_eq!(f.actor().vertical_velocity, 0x0300);
    assert_eq!(
        f.world.calls,
        vec![Work::PlayerContact, Work::Integrate(Integration::PositiveY)]
    );
}
#[test]
fn presetflight_vertical_approach_uses_unsigned_difference_and_transitions_next_call() {
    for (actor_y, player_y, next) in [(0x70, 0x60, true), (0x70, 0x61, false), (0x60, 0x70, false)]
    {
        let mut f = Fixture::new(PRESET_FLIGHT);
        f.actor_mut().position.screen_y = actor_y;
        f.actors[0].position.screen_y = player_y;
        f.run();
        assert_eq!(f.actor().state_code(), if next { MOVING } else { READY });
        assert_eq!(
            f.world.calls.last(),
            Some(&Work::Integrate(Integration::PositiveY))
        );
    }
}
#[test]
fn presetflight_sequence_counter_wrap_cycle_parity_and_late_hold_durations() {
    for (counter, cycle, timer, next, aux) in [
        (6, 1, 3, 7, 1),
        (7, 0, 3, 8, 0),
        (7, 1, 8, 8, 1),
        (13, 1, 8, 0, 2),
    ] {
        let mut f = Fixture::new(PRESET_FLIGHT);
        {
            let actor_value = THROWING;
            f.actor_mut().set_state_code(actor_value);
        };
        {
            let actor_value = counter;
            f.actor_mut().set_local_counter_code(actor_value);
        };
        f.actor_mut().local_aux2 = cycle;
        f.spawn.vertical_presets[0] = 0x0100;
        f.spawn.horizontal_presets[0] = 0xff80;
        f.run();
        assert_eq!(f.actor().local_state_code(), timer - 1);
        assert_eq!(f.actor().local_counter_code(), next);
        assert_eq!(f.actor().local_aux2, aux);
        assert_eq!(f.actor().horizontal_magnitude, 0x0080);
    }
}
#[test]
fn presetflight_even_phase_holds_timer_until_separation_twenty_hex_and_adds_signed_y_directly() {
    for (dy, counts) in [(0x1f, false), (0x20, true)] {
        let mut f = Fixture::new(PRESET_FLIGHT);
        {
            let actor_value = MOVING;
            f.actor_mut().set_state_code(actor_value);
        };
        {
            let actor_value = 3;
            f.actor_mut().set_local_state_code(actor_value);
        };
        f.actor_mut().vertical_velocity = -0x0080;
        f.actor_mut().position.y_fraction = 0x40;
        f.actors[0].position.screen_y = ACTOR_Y + dy;
        f.run();
        assert_eq!(f.actor().local_state_code(), if counts { 2 } else { 3 });
        assert_eq!(
            f.actor().state_code(),
            if counts { THROWING } else { MOVING }
        );
        assert_eq!(
            (f.actor().position.screen_y, f.actor().position.y_fraction),
            (ACTOR_Y - 1, 0xc0)
        );
    }
}
#[test]
fn paired_init_reuses_highest_active_matching_animation_at_exact_world_x_ignoring_y() {
    let mut f = Fixture::new(PAIRED_CONTROLLER);
    f.init();
    for slot in [6, CHILD] {
        f.actors[slot] = ActorContact {
            control: crate::actors::ActorControl::from_codes(ACTIVE, 0, 0, 0),
            animation: 0x1d,
            position: Position {
                world_x: ACTOR_X,
                screen_y: 0xf0,
                ..Position::default()
            },
            ..ActorContact::default()
        };
    }
    let e = f.run();
    assert_eq!(f.actor().local_aux2, CHILD as u8);
    assert!(e.spawned.is_empty());
    assert_eq!(f.actor().state_code(), READY);
    assert!(f.world.calls.is_empty());
}
#[test]
fn paired_init_allocates_companion_preserving_render_identity_and_full_pool_links_player_zero() {
    let mut f = Fixture::new(PAIRED_CONTROLLER);
    f.init();
    f.actor_mut().position.screen_y = 0xfc;
    f.actors[CHILD].render = 0x55;
    f.actors[CHILD].spawn_identity = Some(9);
    let e = f.run();
    assert_eq!(e.spawned, vec![CHILD]);
    assert_eq!(f.actor().local_aux2, CHILD as u8);
    assert_eq!(
        (
            f.actors[CHILD].handler,
            f.actors[CHILD].state_code(),
            f.actors[CHILD].animation,
            f.actors[CHILD].profile,
            f.actors[CHILD].position.screen_y
        ),
        (8, 0xa0, 0x1e, 0x80, 4)
    );
    assert_eq!(
        (f.actors[CHILD].render, f.actors[CHILD].spawn_identity),
        (0x55, Some(9))
    );
    let mut f = Fixture::new(PAIRED_CONTROLLER);
    f.full();
    f.init();
    f.run();
    assert_eq!(f.actor().local_aux2, 0);
    assert_eq!(f.actor().state_code(), READY);
}
#[test]
fn paired_live_link_handler_nine_triggers_even_when_link_inactive_or_player_zero() {
    let mut f = Fixture::new(PAIRED_CONTROLLER);
    f.actor_mut().local_aux2 = 0;
    {
        let actor_value = 9;
        f.actors[0].set_handler(actor_value);
    };
    let e = f.run();
    assert_eq!(
        (
            f.actor().animation,
            f.actor().local_state_code(),
            f.actor().profile
        ),
        (0x31, 0x3c, 0)
    );
    assert_eq!(e.sounds, vec![0x2c]);
    assert!(f.world.calls.is_empty());
}
#[test]
fn paired_reaction_wait_old_one_does_not_launch_until_following_zero_call() {
    for timer in [1, 0] {
        let mut f = Fixture::new(PAIRED_CONTROLLER);
        f.actor_mut().animation = 0x31;
        {
            let actor_value = timer;
            f.actor_mut().set_local_state_code(actor_value);
        };
        f.run();
        if timer == 1 {
            assert_eq!(f.actor().local_state_code(), 0);
            assert!(f.world.calls.is_empty());
        } else {
            assert_eq!(f.actor().animation, 0x32);
            assert_eq!(f.world.calls, vec![Work::Reaction]);
        }
    }
}
#[test]
fn paired_all_eight_phases_update_eligible_companion_and_decrement_new_duration_same_call() {
    for phase in 0..8 {
        let mut f = Fixture::new(PAIRED_CONTROLLER);
        {
            let actor_value = phase;
            f.actor_mut().set_local_counter_code(actor_value);
        };
        f.actor_mut().local_aux2 = 7;
        {
            let actor_value = ACTIVE;
            f.actors[7].set_state_code(actor_value);
        };
        f.actors[7].animation = 0x1d;
        f.actors[7].render = VISIBLE | FLIP_X;
        let e = f.run();
        let i = phase as usize;
        assert_eq!(f.actors[7].animation, f.catalog.companion_animations[i]);
        assert_eq!(
            f.actors[7].render,
            (VISIBLE | FLIP_X) & !FLIP_X | f.catalog.companion_animation_flips[i]
        );
        assert_eq!(
            f.actor().local_state_code(),
            if phase & 1 == 0 {
                PAIR_WALK_DURATION - 1
            } else {
                PAIR_WAIT_DURATION - 1
            }
        );
        assert_eq!(
            f.actor().animation,
            if phase & 1 == 0 { 0x2f } else { 0x30 }
        );
        assert_eq!(f.world.calls.last(), Some(&Work::PlayerContact));
        assert!(e.sounds.is_empty());
    }
}
#[test]
fn paired_random_phase_selection_includes_lsr_carry_without_mutating_global_random() {
    let mut f = Fixture::new(PAIRED_CONTROLLER);
    {
        let actor_value = 3;
        f.actor_mut().set_local_counter_code(actor_value);
    };
    f.actor_mut().local_aux2 = 7;
    f.random.words = [0, 9, 0, 8];
    let old = f.random;
    f.run();
    assert_eq!(f.actor().local_counter_code(), 4);
    assert_eq!(f.random, old);
    assert_eq!(f.actor().direction, 0);
}
#[test]
fn paired_ineligible_link_is_not_reanimated_and_existing_countdown_does_not_advance_phase() {
    let mut f = Fixture::new(PAIRED_CONTROLLER);
    f.actor_mut().local_aux2 = 7;
    {
        let actor_value = 2;
        f.actor_mut().set_local_state_code(actor_value);
    };
    {
        let actor_value = 5;
        f.actor_mut().set_local_counter_code(actor_value);
    };
    f.actors[7].animation = 0x62;
    let old = f.actors[7];
    f.run();
    assert_eq!(f.actors[7], old);
    assert_eq!(
        (f.actor().local_counter_code(), f.actor().local_state_code()),
        (5, 1)
    );
}
#[test]
fn ledgewalkers_initialize_distinct_speed_and_apply_both_ledge_and_wall_reversals() {
    for selector in [LEDGE_WALKER, FAST_LEDGE_WALKER] {
        for (floor, wall, want) in [
            (CONTACT, 0, 1),
            (0, 0, 2),
            (0, CONTACT, 1),
            (CONTACT, CONTACT, 2),
        ] {
            let mut f = Fixture::new(selector);
            f.init();
            f.facts(&[floor, wall]);
            f.run();
            assert_eq!(f.actor().direction, want);
            assert_eq!(
                f.actor().horizontal_magnitude,
                if selector == LEDGE_WALKER {
                    0x0080
                } else {
                    0x0133
                }
            );
            let offset = usize::from(selector == FAST_LEDGE_WALKER);
            if offset > 0 {
                assert_eq!(f.world.calls[0], Work::PlayerContact);
            }
            assert_eq!(
                f.world.calls[offset],
                motion(
                    Movement::Fall,
                    if selector == LEDGE_WALKER { 0x2e } else { 0x20 },
                    0
                )
            );
            assert_eq!(
                f.world.calls[offset + 1],
                motion(
                    if floor == 0 {
                        Movement::NegativeX
                    } else {
                        Movement::PositiveX
                    },
                    if selector == LEDGE_WALKER {
                        if floor == 0 { 0x2b } else { 0x29 }
                    } else if floor == 0 {
                        0x1c
                    } else {
                        0x19
                    },
                    0
                )
            );
        }
    }
}
#[test]
fn ambush_wait_strict_twelve_pixel_gate_and_pose_thirtynine_skips_contact() {
    for (distance, starts) in [(0x0b, true), (0x0c, false)] {
        let mut f = Fixture::new(AMBUSH);
        f.init();
        f.actor_mut().animation = 0x39;
        f.actors[0].position.world_x = ACTOR_X + distance;
        f.run();
        assert!(f.world.calls.is_empty());
        assert_eq!(f.actor().state_code(), if starts { READY } else { ACTIVE });
        if starts {
            assert_eq!(
                (
                    f.actor().local_state_code(),
                    f.actor().local_counter_code(),
                    f.actor().vertical_velocity
                ),
                (0x1e, 0, 0x0280)
            );
        }
    }
}
#[test]
fn ambush_phase_one_countdown_expiry_returns_before_terrain_and_exact_pose_tick_changes() {
    let mut f = Fixture::new(AMBUSH);
    {
        let actor_value = 1;
        f.actor_mut().set_local_state_code(actor_value);
    };
    f.run();
    assert_eq!(f.actor().animation, 0x3c);
    assert_eq!(f.world.calls, vec![Work::PlayerContact]);
    let mut f = Fixture::new(AMBUSH);
    f.actor_mut().animation = 0x3c;
    {
        let animation_value = 8;
        f.actor_mut().playback.set_tick_code(animation_value);
    };
    f.actors[0].position.screen_y = ACTOR_Y + 0x28;
    f.run();
    assert_eq!(f.actor().animation, 0x3b);
    assert_eq!(f.actor().state_code(), READY);
    assert_eq!(
        f.world.calls,
        vec![Work::PlayerContact, motion(Movement::PositiveY, 0x40, 1)]
    );
}
#[test]
fn ambush_phase_one_contact_or_wrapped_difference_below_twenty_eight_advances() {
    for (bits, player_y, advances) in [
        (0, ACTOR_Y + 0x27, true),
        (0, ACTOR_Y + 0x28, false),
        (0, ACTOR_Y - 1, false),
        (CONTACT, ACTOR_Y - 1, true),
    ] {
        let mut f = Fixture::new(AMBUSH);
        f.actor_mut().animation = 0x3b;
        f.actors[0].position.screen_y = player_y;
        f.facts(&[bits]);
        f.run();
        assert_eq!(
            f.actor().state_code(),
            if advances { MOVING } else { READY }
        );
    }
}
#[test]
fn ambush_phase_two_and_three_share_preset_body_with_signed_words_and_directional_ranges() {
    for phase in [MOVING, THROWING] {
        let mut f = Fixture::new(AMBUSH);
        {
            let actor_value = phase;
            f.actor_mut().set_state_code(actor_value);
        };
        f.actor_mut().direction = 2;
        f.actors[0].position.world_x = ACTOR_X + 0x40;
        f.spawn.vertical_presets[0] = -0x0180;
        f.spawn.horizontal_presets[0] = 0xff80;
        f.run();
        assert_eq!(f.actor().motion_flags, 8);
        assert_eq!(f.actor().vertical_velocity, 0x0180);
        assert_eq!(f.actor().horizontal_magnitude, 0x0080);
        assert_eq!(
            f.world.calls,
            vec![
                Work::PlayerContact,
                motion(Movement::NegativeY, 0x42, 1),
                motion(Movement::NegativeX, 0x3d, 2)
            ]
        );
        assert_eq!(
            f.actor().local_state_code(),
            if phase == MOVING { 3 } else { 2 }
        );
    }
}
#[test]
fn ambush_vertical_and_horizontal_gate_can_advance_two_phases_in_same_call() {
    let mut f = Fixture::new(AMBUSH);
    {
        let actor_value = MOVING;
        f.actor_mut().set_state_code(actor_value);
    };
    {
        let actor_value = 3;
        f.actor_mut().set_local_state_code(actor_value);
    };
    f.actors[0].position.screen_y = ACTOR_Y + 0x50;
    f.actors[0].position.world_x = ACTOR_X + 0x0b;
    let e = f.run();
    assert_eq!(f.actor().state_code(), ACTIVE | 4);
    assert_eq!(f.actor().local_state_code(), 2);
    assert_eq!(f.actor().animation, 0x3c);
    assert_eq!(f.actor().vertical_velocity, 0x0280);
    assert_eq!(e.sounds, vec![0x32, 0x1f]);
}
#[test]
fn ambush_preset_counter_wrap_and_early_timer_holds_preserve_phase_two() {
    let mut f = Fixture::new(AMBUSH);
    {
        let actor_value = MOVING;
        f.actor_mut().set_state_code(actor_value);
    };
    {
        let actor_value = 13;
        f.actor_mut().set_local_counter_code(actor_value);
    };
    f.actors[0].position.world_x = ACTOR_X + 0x40;
    f.actors[0].position.screen_y = ACTOR_Y + 0x4f;
    f.run();
    assert_eq!(f.actor().local_counter_code(), 0);
    assert_eq!(f.actor().local_state_code(), 3);
    assert_eq!(f.actor().state_code(), MOVING);
}
#[test]
fn ambush_attack_fall_landing_protects_profile_and_rest_expiry_restores_phase_two() {
    let mut f = Fixture::new(AMBUSH);
    {
        let actor_value = ACTIVE | 4;
        f.actor_mut().set_state_code(actor_value);
    };
    f.actor_mut().animation = 0x3c;
    {
        let animation_value = 8;
        f.actor_mut().playback.set_tick_code(animation_value);
    };
    f.facts(&[CONTACT]);
    let e = f.run();
    assert_eq!(
        (
            f.actor().animation,
            f.actor().local_state_code(),
            f.actor().profile
        ),
        (0x39, 0x78, ACTIVE)
    );
    assert_eq!(e.sounds, vec![0x20]);
    let mut f = Fixture::new(AMBUSH);
    {
        let actor_value = 0xb4;
        f.actor_mut().set_state_code(actor_value);
    };
    f.actor_mut().animation = 0x39;
    f.actor_mut().profile = 0x91;
    {
        let actor_value = 1;
        f.actor_mut().set_local_state_code(actor_value);
    };
    f.run();
    assert_eq!(
        (
            f.actor().state_code(),
            f.actor().animation,
            f.actor().profile,
            f.actor().local_counter_code()
        ),
        (0xb2, 0x3b, 0x11, 0)
    );
    assert!(f.world.calls.is_empty());
}
#[test]
fn ambush_zero_rest_timer_wraps_and_unsupported_tick_eight_switches_to_five_c_pose() {
    let mut f = Fixture::new(AMBUSH);
    {
        let actor_value = ACTIVE | 4;
        f.actor_mut().set_state_code(actor_value);
    };
    f.actor_mut().animation = 0x39;
    f.run();
    assert_eq!(f.actor().local_state_code(), 0xff);
    let mut f = Fixture::new(AMBUSH);
    {
        let actor_value = ACTIVE | 4;
        f.actor_mut().set_state_code(actor_value);
    };
    f.actor_mut().animation = 0x3c;
    {
        let animation_value = 8;
        f.actor_mut().playback.set_tick_code(animation_value);
    };
    f.run();
    assert_eq!(f.actor().animation, 0x5c);
    assert_eq!(f.actor().state_code(), ACTIVE | 4);
}
#[test]
fn eruption_profile_uses_old_y_and_proximity_gate_leaves_distant_actor_inactive() {
    for (y, profile) in [(0, 0), (1, 0x91), (0xcf, 0x91), (0xd0, 0), (0xff, 0)] {
        let mut f = Fixture::new(ERUPTION);
        f.init();
        f.actor_mut().position.screen_y = y;
        f.actors[0].position.world_x = ACTOR_X + 0x20;
        f.run();
        assert_eq!(f.actor().profile, profile);
        assert_eq!(f.actor().state_code(), ACTIVE);
        assert!(f.world.calls.is_empty());
    }
}
#[test]
fn eruption_activation_uses_every_local_selector_and_integrates_before_gravity() {
    for selector in 0..3 {
        let mut f = Fixture::new(ERUPTION);
        f.init();
        {
            let actor_value = selector;
            f.actor_mut().set_local_state_code(actor_value);
        };
        f.actor_mut().position.screen_y = 1;
        f.actors[0].position.world_x = ACTOR_X + 0x1f;
        f.run();
        assert_eq!(f.actor().position.screen_y, 0xef);
        assert_eq!(f.actor().profile, 0x91);
        assert_eq!(
            f.actor().vertical_velocity,
            f.catalog.eruption_velocity[selector as usize]
        );
        assert_eq!(
            (
                f.actor().state_code(),
                f.actor().local_aux2,
                f.actor().playback.tick_code()
            ),
            (READY, 1, FROZEN_TICK)
        );
        assert_eq!(
            f.world.calls,
            vec![
                Work::Integrate(Integration::FallingY),
                Work::Integrate(Integration::Gravity)
            ]
        );
    }
}
#[test]
fn eruption_negative_apex_zeroes_velocity_and_selects_frame_one() {
    let mut f = Fixture::new(ERUPTION);
    f.actor_mut().vertical_velocity = -1;
    f.run();
    assert_eq!(f.actor().motion_flags, 1);
    assert_eq!(f.actor().vertical_velocity, 0);
    assert_eq!(f.actor().playback.frame_code(), 1);
}
#[test]
fn eruption_rise_event_changes_palette_and_sound_even_when_full_pool() {
    for full in [false, true] {
        let mut f = Fixture::new(ERUPTION);
        if full {
            f.full();
            {
                let actor_value = READY;
                f.actor_mut().set_state_code(actor_value);
            };
        }
        {
            let actor_value = 0;
            f.actor_mut().set_local_state_code(actor_value);
        };
        f.actor_mut().local_aux2 = 7;
        let e = f.run();
        assert_eq!(e.palette_pair, Some([0x19, 0x27]));
        assert!(e.palette_upload);
        assert_eq!(e.sounds, vec![0x21]);
        assert_eq!(e.spawned, if full { vec![] } else { vec![CHILD] });
        if !full {
            let c = f.actors[CHILD];
            assert_eq!(
                (
                    c.state_code(),
                    c.render,
                    c.handler,
                    c.animation,
                    c.profile,
                    c.position.world_x,
                    c.position.screen_y
                ),
                (0xa8, 0xa8, 0, 0x37, 0, ACTOR_X, 0xbc)
            );
        }
    }
}
#[test]
fn eruption_fall_gravity_precedes_integration_y_f0_starts_pause_and_preserves_profile_until_next_call()
 {
    let mut f = Fixture::new(ERUPTION);
    f.actor_mut().motion_flags = 1;
    f.actor_mut().position.screen_y = 0xf0;
    f.actor_mut().position.y_fraction = 0x55;
    f.run();
    assert_eq!(
        f.world.calls,
        vec![
            Work::Integrate(Integration::Gravity),
            Work::Integrate(Integration::FallingY)
        ]
    );
    assert_eq!(
        (
            f.actor().position.screen_y,
            f.actor().position.y_fraction,
            f.actor().local_counter_code()
        ),
        (0, 0, 0x78)
    );
}
#[test]
fn eruption_events_can_fire_during_pause_but_expiry_skips_event_and_preserves_persistent_state() {
    for counter in [2, 1] {
        let mut f = Fixture::new(ERUPTION);
        {
            let actor_value = 0xb1;
            f.actor_mut().set_state_code(actor_value);
        };
        f.actor_mut().motion_flags = 1;
        {
            let actor_value = counter;
            f.actor_mut().set_local_counter_code(actor_value);
        };
        f.actor_mut().local_aux2 = 0x29;
        let e = f.run();
        assert!(f.world.calls.is_empty());
        assert_eq!(f.actor().local_aux2, 0x2a);
        assert_eq!(
            f.actor().state_code(),
            if counter == 1 { 0xb0 } else { 0xb1 }
        );
        assert_eq!(
            e.palette_pair,
            if counter == 1 {
                None
            } else {
                Some([0x21, 0x17])
            }
        );
    }
}
#[test]
fn eruption_zero_event_selector_and_aux_wrap_never_spuriously_emit_palette_event() {
    let mut f = Fixture::new(ERUPTION);
    {
        let actor_value = 2;
        f.actor_mut().set_local_state_code(actor_value);
    };
    f.actor_mut().local_aux2 = 0xff;
    let e = f.run();
    assert_eq!(f.actor().local_aux2, 0);
    assert_eq!(e.palette_pair, None);
}
#[test]
fn repeatingjump_initial_or_two_preserves_velocity_and_landing_launches_exact_fraction() {
    let mut f = Fixture::new(REPEATING_JUMP);
    f.init();
    f.actor_mut().vertical_velocity = 0x0134;
    f.facts(&[CONTACT, 0]);
    let e = f.run();
    assert_eq!(f.actor().state_code(), READY);
    assert_eq!(f.actor().vertical_velocity, REPEAT_JUMP_SPEED);
    assert_eq!(
        (
            f.actor().playback.tick_code(),
            f.actor().playback.frame_code()
        ),
        (0, 0)
    );
    assert!(e.sounds.is_empty());
    assert_eq!(
        f.world.calls,
        vec![
            Work::PlayerContact,
            motion(Movement::Fall, 0x1e, 2),
            motion(Movement::PositiveX, 0x18, 2)
        ]
    );
}
#[test]
fn repeatingjump_rise_freezes_nonzero_frame_and_apex_advances_without_fall_same_call() {
    let mut f = Fixture::new(REPEATING_JUMP);
    {
        let animation_value = 1;
        f.actor_mut().playback.set_frame_code(animation_value);
    };
    f.world.post_velocity = Some(-1);
    f.run();
    assert_eq!(f.actor().playback.tick_code(), FROZEN_TICK);
    assert_eq!(f.actor().state_code(), MOVING);
    assert_eq!(
        f.world.calls,
        vec![
            Work::PlayerContact,
            motion(Movement::Rise, 0x21, 2),
            motion(Movement::PositiveX, 0x18, 2)
        ]
    );
}
#[test]
fn companionpatrol_activation_strict_fifty_and_clears_state_bit_ten_hex() {
    for distance in [0x4f, 0x50] {
        let mut f = Fixture::new(COMPANION_PATROL);
        {
            let actor_value = 0xb0;
            f.actor_mut().set_state_code(actor_value);
        };
        f.actors[0].position.world_x = ACTOR_X + distance;
        f.facts(&[CONTACT, 0]);
        f.run();
        if distance == 0x50 {
            assert_eq!(f.actor().state_code(), 0xb0);
            assert!(f.world.calls.is_empty());
        } else {
            assert_eq!(f.actor().state_code(), 0xa1);
            assert_eq!(f.actor().horizontal_magnitude, 0x0133);
            assert_eq!(f.actor().local_state_code(), 0x4f);
        }
    }
}
#[test]
fn companionpatrol_support_tests_all_bits_and_adds_y_two_before_horizontal() {
    let mut f = Fixture::new(COMPANION_PATROL);
    {
        let actor_value = 3;
        f.actor_mut().set_local_state_code(actor_value);
    };
    f.facts(&[SECONDARY, 0]);
    f.run();
    assert_eq!(f.actor().position.screen_y, ACTOR_Y + 2);
    assert_eq!(f.actor().state_code(), READY);
    assert_eq!(f.actor().local_state_code(), 2);
    assert_eq!(
        f.world.calls,
        vec![
            Work::PlayerContact,
            motion(Movement::Fall, 0x2e, 0),
            motion(Movement::PositiveX, 0x29, 0)
        ]
    );
}
#[test]
fn companionpatrol_floor_loss_wall_or_timer_expiry_launch_and_full_pool_keeps_parent_commit() {
    for (facts, timer) in [
        (vec![0], 5),
        (vec![CONTACT, CONTACT], 5),
        (vec![CONTACT, 0], 1),
    ] {
        let mut f = Fixture::new(COMPANION_PATROL);
        f.full();
        {
            let actor_value = READY;
            f.actor_mut().set_state_code(actor_value);
        };
        {
            let actor_value = timer;
            f.actor_mut().set_local_state_code(actor_value);
        };
        f.facts(&facts);
        let e = f.run();
        assert_eq!(
            (
                f.actor().state_code(),
                f.actor().direction,
                f.actor().local_state_code(),
                f.actor().vertical_velocity,
                f.actor().animation,
                f.actor().playback.tick_code()
            ),
            (MOVING, 2, 0x50, BOUNCE_HEIGHT, 0x56, FROZEN_TICK)
        );
        assert_eq!(e.sounds, vec![0x34]);
        assert!(e.spawned.is_empty());
    }
}
#[test]
fn companionpatrol_effect_child_copies_post_adjusted_parent_y_and_preserves_stale_identity() {
    let mut f = Fixture::new(COMPANION_PATROL);
    {
        let actor_value = 1;
        f.actor_mut().set_local_state_code(actor_value);
    };
    f.actors[CHILD].spawn_identity = Some(9);
    f.facts(&[CONTACT, 0]);
    let e = f.run();
    let c = f.actors[CHILD];
    assert_eq!(e.spawned, vec![CHILD]);
    assert_eq!(
        (
            c.state_code(),
            c.render,
            c.animation,
            c.handler,
            c.profile,
            c.position.screen_y
        ),
        (0xa8, 0xa8, 0x57, 0, 0, ACTOR_Y + 6)
    );
    assert_eq!(c.spawn_identity, Some(9));
}
#[test]
fn companionpatrol_arc_apex_advances_frame_and_landing_returns_two_states_with_y_one() {
    let mut f = Fixture::new(COMPANION_PATROL);
    {
        let actor_value = MOVING;
        f.actor_mut().set_state_code(actor_value);
    };
    {
        let animation_value = 1;
        f.actor_mut().playback.set_frame_code(animation_value);
    };
    f.world.post_velocity = Some(-1);
    f.run();
    assert_eq!(
        (
            f.actor().state_code(),
            f.actor().playback.frame_code(),
            f.actor().vertical_velocity
        ),
        (THROWING, 2, 0)
    );
    assert_eq!(
        f.world.calls,
        vec![Work::PlayerContact, motion(Movement::Rise, 0x2f, 1)]
    );
    let mut f = Fixture::new(COMPANION_PATROL);
    {
        let actor_value = THROWING;
        f.actor_mut().set_state_code(actor_value);
    };
    f.facts(&[CONTACT]);
    f.run();
    assert_eq!(
        (
            f.actor().state_code(),
            f.actor().animation,
            f.actor().position.screen_y
        ),
        (READY, 0x55, ACTOR_Y + 1)
    );
    assert_eq!(
        f.world.calls,
        vec![Work::PlayerContact, motion(Movement::Fall, 0x2d, 1)]
    );
}
#[test]
fn timedpatrol_first_interval_and_motion_before_reversal_differ_from_vertical_flip_policy() {
    for selector in [TIMED_PATROL, VERTICAL_PATROL] {
        let mut f = Fixture::new(selector);
        f.init();
        f.run();
        assert_eq!(f.actor().local_state_code(), 0x15);
        assert_eq!(f.actor().state_code(), READY);
        assert_eq!(f.world.calls[0], Work::PlayerContact);
        assert_eq!(
            f.world.calls[1],
            Work::Integrate(if selector == TIMED_PATROL {
                Integration::PositiveX
            } else {
                Integration::NegativeY
            })
        );
        assert_ne!(f.actor().render & FLIP_X, 0);
    }
}
#[test]
fn patrol_expiry_moves_old_direction_then_changes_direction_and_zero_timer_wraps() {
    for selector in [TIMED_PATROL, VERTICAL_PATROL] {
        for timer in [1, 0] {
            let mut f = Fixture::new(selector);
            {
                let actor_value = timer;
                f.actor_mut().set_local_state_code(actor_value);
            };
            f.run();
            assert_eq!(f.actor().direction, if timer == 1 { 2 } else { 1 });
            assert_eq!(
                f.actor().local_state_code(),
                if timer == 1 {
                    if selector == TIMED_PATROL { 0x20 } else { 0x2c }
                } else {
                    0xff
                }
            );
            assert_eq!(
                f.world.calls[1],
                Work::Integrate(if selector == TIMED_PATROL {
                    Integration::PositiveX
                } else {
                    Integration::NegativeY
                })
            );
        }
    }
}
#[test]
fn verticalpatrol_does_not_resynchronize_flip_after_first_initialization() {
    let mut f = Fixture::new(VERTICAL_PATROL);
    f.actor_mut().direction = 2;
    f.actor_mut().render = VISIBLE | FLIP_X;
    {
        let actor_value = 2;
        f.actor_mut().set_local_state_code(actor_value);
    };
    f.run();
    assert_eq!(f.actor().render, VISIBLE | FLIP_X);
    assert_eq!(f.world.calls[1], Work::Integrate(Integration::PositiveY));
}
#[test]
fn triggeredjump_distant_init_freezes_animation_and_clears_flip_without_initializing() {
    let mut f = Fixture::new(TRIGGERED_JUMP);
    f.init();
    f.actor_mut().render = VISIBLE | FLIP_X;
    {
        let animation_value = 7;
        f.actor_mut().playback.set_tick_code(animation_value);
    };
    f.actors[0].position.world_x = ACTOR_X + 0x32;
    f.run();
    assert_eq!(f.actor().state_code(), ACTIVE);
    assert_eq!(f.actor().render, VISIBLE);
    assert_eq!(f.actor().playback.tick_code(), 0);
    assert!(f.world.calls.is_empty());
}
#[test]
fn triggeredjump_initial_direction_one_means_negative_horizontal_and_clear_path_advances() {
    let mut f = Fixture::new(TRIGGERED_JUMP);
    f.init();
    f.actors[0].position.world_x = ACTOR_X + 0x31;
    f.run();
    assert_eq!(f.actor().state_code(), MOVING);
    assert_eq!(f.actor().vertical_velocity, PATROL_JUMP_SPEED);
    assert_eq!(f.actor().horizontal_magnitude, 0x0180);
    assert_eq!(
        f.world.calls,
        vec![Work::PlayerContact, motion(Movement::NegativeX, 0x39, 0)]
    );
}
#[test]
fn triggeredjump_wall_change_freezes_tick_and_keeps_patrol_phase() {
    let mut f = Fixture::new(TRIGGERED_JUMP);
    f.facts(&[CONTACT]);
    f.actor_mut().render = VISIBLE | FLIP_X;
    f.run();
    assert_eq!(f.actor().state_code(), READY);
    assert_eq!(f.actor().direction, 2);
    assert_eq!(f.actor().render, VISIBLE);
    assert_eq!(f.actor().playback.tick_code(), 0);
}
#[test]
fn triggeredjump_rise_apex_and_landing_keep_source_phase_three_without_invented_patrol_reset() {
    let mut f = Fixture::new(TRIGGERED_JUMP);
    {
        let actor_value = MOVING;
        f.actor_mut().set_state_code(actor_value);
    };
    f.world.post_velocity = Some(-1);
    f.run();
    assert_eq!(f.actor().state_code(), THROWING);
    assert_eq!(f.actor().vertical_velocity, 0);
    assert_eq!(
        f.world.calls,
        vec![Work::PlayerContact, motion(Movement::Rise, 0x53, 0)]
    );
    let mut f = Fixture::new(TRIGGERED_JUMP);
    {
        let actor_value = THROWING;
        f.actor_mut().set_state_code(actor_value);
    };
    f.facts(&[CONTACT, 0]);
    let e = f.run();
    assert_eq!(f.actor().state_code(), THROWING);
    assert_eq!(e.sounds, vec![0x27]);
    assert_eq!(
        f.world.calls,
        vec![
            Work::PlayerContact,
            motion(Movement::Fall, 0x31, 2),
            motion(Movement::NegativeX, 0x39, 0)
        ]
    );
}
#[test]
fn leftwalker_initializes_and_moves_only_on_floor_with_sound_only_tick_zero() {
    for (floor, tick) in [(0, 0), (CONTACT, 0), (CONTACT, 1)] {
        let mut f = Fixture::new(LEFT_WALKER);
        f.init();
        {
            let animation_value = tick;
            f.actor_mut().playback.set_tick_code(animation_value);
        };
        f.facts(&[floor]);
        let e = f.run();
        assert_eq!(f.actor().horizontal_magnitude, 0x0180);
        assert_eq!(f.actor().state_code(), READY);
        assert_eq!(
            f.world.calls[..2],
            [Work::PlayerContact, motion(Movement::Fall, 0x31, 2)]
        );
        assert_eq!(f.world.calls.len(), if floor == 0 { 2 } else { 3 });
        if floor != 0 {
            assert_eq!(f.world.calls[2], Work::Integrate(Integration::NegativeX));
        }
        assert_eq!(
            e.sounds,
            if floor != 0 && tick == 0 {
                vec![0x27]
            } else {
                vec![]
            }
        );
    }
}
#[test]
fn approach_init_preserves_vertical_and_far_phase_only_moves_horizontally() {
    let mut f = Fixture::new(APPROACH);
    f.init();
    f.actor_mut().vertical_velocity = 0x1234;
    f.actors[0].position.world_x = ACTOR_X + 0x40;
    f.run();
    assert_eq!(f.actor().vertical_velocity, 0x1234);
    assert_eq!(f.actor().horizontal_magnitude, 0x01cc);
    assert_eq!(f.actor().state_code(), READY);
    assert_eq!(f.world.calls, vec![motion(Movement::PositiveX, 0x18, 2)]);
}
#[test]
fn approach_proximity_starts_downward_phase_and_contact_initializes_jump_pose() {
    for contact in [0, CONTACT] {
        let mut f = Fixture::new(APPROACH);
        f.actor_mut().horizontal_magnitude = 0x01cc;
        f.facts(&[contact]);
        let e = f.run();
        assert_eq!(f.actor().state_code(), MOVING);
        assert_eq!(f.world.calls, vec![motion(Movement::PositiveY, 0x1e, 2)]);
        if contact == 0 {
            assert_eq!(f.actor().vertical_velocity, 0x01cc);
            assert!(e.sounds.is_empty());
        } else {
            assert_eq!(
                (
                    f.actor().animation,
                    f.actor().motion_flags,
                    f.actor().vertical_velocity,
                    f.actor().horizontal_magnitude
                ),
                (0x4d, 8, PATROL_JUMP_SPEED, 0x0100)
            );
            assert_eq!(e.sounds, vec![0x24]);
        }
    }
}
#[test]
fn approach_motion_eight_waits_exact_frame_two_and_saves_horizontal_bits_across_rise() {
    for (frame, wall, next_state, next_motion) in [
        (1, CONTACT, MOVING, 8),
        (2, 0, MOVING, 4),
        (2, CONTACT, THROWING, 0),
    ] {
        let mut f = Fixture::new(APPROACH);
        {
            let actor_value = MOVING;
            f.actor_mut().set_state_code(actor_value);
        };
        f.actor_mut().motion_flags = 8;
        f.actor_mut().animation = 0x4d;
        {
            let animation_value = frame;
            f.actor_mut().playback.set_frame_code(animation_value);
        };
        f.world.post_velocity = Some(-1);
        f.facts(&[wall, 0]);
        f.run();
        assert_eq!(f.actor().state_code(), next_state);
        assert_eq!(f.actor().motion_flags, next_motion);
        if frame == 1 {
            assert!(f.world.calls.is_empty());
        } else {
            assert_eq!(
                f.world.calls,
                vec![
                    motion(Movement::PositiveX, 0x18, 2),
                    motion(Movement::Rise, 0x21, 2)
                ]
            );
            if wall == 0 {
                assert_eq!(f.actor().vertical_velocity, 0);
                assert_eq!(f.actor().playback.tick_code(), FROZEN_TICK);
            } else {
                assert_eq!(f.actor().vertical_velocity, 0x01cc);
                assert_eq!(f.actor().animation, 0x4c);
            }
        }
    }
}
#[test]
fn approach_non_four_d_pose_skips_frame_gate_but_nonnegative_rise_keeps_motion_eight() {
    let mut f = Fixture::new(APPROACH);
    {
        let actor_value = MOVING;
        f.actor_mut().set_state_code(actor_value);
    };
    f.actor_mut().motion_flags = 8;
    f.actor_mut().animation = 0x4c;
    f.actor_mut().direction = 2;
    f.world.post_velocity = Some(1);
    f.run();
    assert_eq!(
        f.world.calls,
        vec![
            motion(Movement::NegativeX, 0x1b, 2),
            motion(Movement::Rise, 0x21, 2)
        ]
    );
    assert_eq!(f.actor().motion_flags, 8);
}
#[test]
fn approach_falling_without_floor_moves_horizontal_but_landing_waits_for_pose_completion() {
    for (floor, animation, frame, tick, waiting) in [
        (0, 0x4d, 0, 0, false),
        (CONTACT, 0x4d, 0, 0, true),
        (CONTACT, 0x4e, 1, 0, true),
        (CONTACT, 0x4e, 0, 1, true),
    ] {
        let mut f = Fixture::new(APPROACH);
        {
            let actor_value = MOVING;
            f.actor_mut().set_state_code(actor_value);
        };
        f.actor_mut().motion_flags = 4;
        f.actor_mut().animation = animation;
        {
            let animation_value = frame;
            f.actor_mut().playback.set_frame_code(animation_value);
        };
        {
            let animation_value = tick;
            f.actor_mut().playback.set_tick_code(animation_value);
        };
        f.facts(&[floor, 0]);
        let e = f.run();
        assert_eq!(f.world.calls.len(), if waiting { 1 } else { 2 });
        if floor != 0 && animation != 0x4e {
            assert_eq!(
                (f.actor().animation, f.actor().playback.tick_code()),
                (0x4e, 1)
            );
            assert_eq!(e.sounds, vec![0x24]);
        }
        assert_eq!(f.actor().state_code(), MOVING);
    }
}
#[test]
fn approach_landing_facing_comparison_restores_old_direction_and_branches_on_temporary_result() {
    for (old, changed) in [(1, false), (2, true)] {
        let mut f = Fixture::new(APPROACH);
        {
            let actor_value = MOVING;
            f.actor_mut().set_state_code(actor_value);
        };
        f.actor_mut().motion_flags = 4;
        f.actor_mut().animation = 0x4e;
        f.actor_mut().direction = old;
        f.facts(&[CONTACT]);
        f.run();
        assert_eq!(f.actor().direction, old);
        if changed {
            assert_eq!(
                (
                    f.actor().state_code(),
                    f.actor().motion_flags,
                    f.actor().animation,
                    f.actor().vertical_velocity,
                    f.actor().horizontal_magnitude
                ),
                (THROWING, 0, 0x4c, 0x01cc, 0x01cc)
            );
        } else {
            assert_eq!(
                (
                    f.actor().state_code(),
                    f.actor().motion_flags,
                    f.actor().animation,
                    f.actor().playback.frame_code(),
                    f.actor().vertical_velocity
                ),
                (MOVING, 8, 0x4d, 2, PATROL_JUMP_SPEED)
            );
        }
    }
}
#[test]
fn approach_state_three_height_gate_is_strict_eighty_eight_and_contact_wins() {
    for (y, bits, advance) in [(0x88, 0, false), (0x87, 0, true), (0x88, CONTACT, true)] {
        let mut f = Fixture::new(APPROACH);
        {
            let actor_value = THROWING;
            f.actor_mut().set_state_code(actor_value);
        };
        f.actor_mut().position.screen_y = y;
        f.facts(&[bits]);
        f.run();
        assert_eq!(f.actor().motion_flags, u8::from(advance));
        assert_eq!(f.world.calls, vec![motion(Movement::NegativeY, 0x21, 2)]);
    }
}
#[test]
fn approach_state_three_horizontal_wall_requests_extra_upward_probe_without_gravity() {
    for wall in [0, CONTACT] {
        let mut f = Fixture::new(APPROACH);
        {
            let actor_value = THROWING;
            f.actor_mut().set_state_code(actor_value);
        };
        f.actor_mut().motion_flags = 1;
        f.facts(&[wall, 0]);
        f.run();
        assert_eq!(f.world.calls[0], motion(Movement::PositiveX, 0x18, 2));
        assert_eq!(f.world.calls.len(), if wall == 0 { 1 } else { 2 });
        if wall != 0 {
            assert_eq!(f.world.calls[1], motion(Movement::NegativeY, 0x21, 2));
        }
    }
}
#[test]
fn timedstealer_each_selector_loads_own_interval_and_pause_one_consumes_whole_call() {
    for selector in [FIRST_TIMED_STEALER, SECOND_TIMED_STEALER] {
        let mut f = Fixture::new(selector);
        f.init();
        f.run();
        assert_eq!(
            f.actor().local_aux2,
            f.catalog.stealer_intervals[usize::from(selector - 0x30)]
        );
        assert_eq!(f.actor().local_state_code(), f.actor().local_aux2 - 1);
        assert_eq!(f.actor().horizontal_magnitude, 0x0100);
    }
    let mut f = Fixture::new(FIRST_TIMED_STEALER);
    {
        let actor_value = 1;
        f.actor_mut().set_local_counter_code(actor_value);
    };
    f.world.permission = true;
    f.run();
    assert_eq!(f.actor().local_counter_code(), 0);
    assert!(f.world.calls.is_empty());
    assert_eq!(f.status.dirty, 0);
}
#[test]
fn timedstealer_contact_mutates_upper_three_digits_before_same_actor_timer_decrement() {
    let mut f = Fixture::new(FIRST_TIMED_STEALER);
    {
        let actor_value = 5;
        f.actor_mut().set_local_state_code(actor_value);
    };
    f.actor_mut().local_aux2 = 4;
    f.status.reward = [BLANK, 1, 0, 0, 0, 0].into();
    f.world.permission = true;
    let e = f.run();
    assert_eq!(f.status.reward.display_codes(), [BLANK, BLANK, 9, 0, 0, 0]);
    assert_eq!(f.status.dirty, 1);
    assert_eq!(f.actor().local_state_code(), 5);
    assert_eq!(f.actor().local_counter_code(), 0);
    assert_eq!(e.thefts, 1);
    assert_eq!(
        f.world.calls,
        vec![Work::Integrate(Integration::PositiveX), Work::PlayerContact]
    );
}
#[test]
fn timedstealer_repeated_permission_steals_repeatedly_and_underflow_still_delays_countdown() {
    let mut f = Fixture::new(FIRST_TIMED_STEALER);
    {
        let actor_value = 1;
        f.actor_mut().set_local_state_code(actor_value);
    };
    f.status.reward = [BLANK; 6].into();
    f.world.permission = true;
    f.run();
    f.run();
    assert_eq!(f.status.reward.display_codes(), [BLANK; 6]);
    assert_eq!(f.status.dirty, 2);
    assert_eq!(f.actor().local_state_code(), 1);
    assert_eq!(f.actor().local_counter_code(), 0);
}
#[test]
fn timedstealer_denied_contact_can_reverse_and_double_interval_with_byte_wrap() {
    let mut f = Fixture::new(SECOND_TIMED_STEALER);
    {
        let actor_value = 1;
        f.actor_mut().set_local_state_code(actor_value);
    };
    f.actor_mut().local_aux2 = 0x80;
    f.actor_mut().direction = 2;
    let e = f.run();
    assert_eq!(
        f.world.calls,
        vec![Work::Integrate(Integration::NegativeX), Work::PlayerContact]
    );
    assert_eq!(
        (
            f.actor().direction,
            f.actor().local_counter_code(),
            f.actor().local_state_code()
        ),
        (1, THIEF_PAUSE, 0)
    );
    assert_eq!(f.actor().render & FLIP_X, 0);
    assert_eq!(f.status.dirty, 0);
    assert_eq!(e.thefts, 0);
}
#[test]
fn jumpingstealer_initialization_uses_named_selector_one_recipe_not_normal_walker_preset() {
    let mut f = Fixture::new(JUMPING_STEALER);
    f.init();
    f.geometry.velocity_presets = [0x025a, 0x07a9];
    f.run();
    assert_eq!(f.actor().horizontal_magnitude, 0x07a9);
    assert_eq!(f.actor().local_state_code(), 0);
    assert_eq!(f.actor().direction, 1);
    assert_eq!(f.actor().state_code(), READY);
}
#[test]
fn jumpingstealer_permission_latches_same_actor_and_skips_further_contact_and_theft() {
    let mut f = Fixture::new(JUMPING_STEALER);
    f.status.reward = [BLANK, 1, 0, 0, 0, 0].into();
    f.world.permission = true;
    let first = f.run();
    assert_eq!(f.actor().local_state_code(), 1);
    assert_eq!(f.status.reward.display_codes(), [BLANK, BLANK, 9, 0, 0, 0]);
    assert_eq!(first.thefts, 1);
    f.world.calls.clear();
    let second = f.run();
    assert!(!f.world.calls.contains(&Work::PlayerContact));
    assert_eq!(f.status.dirty, 1);
    assert_eq!(second.thefts, 0);
}
#[test]
fn jumpingstealer_uses_horizontal_contact_for_jump_after_theft_without_direction_reversal() {
    let mut f = Fixture::new(JUMPING_STEALER);
    f.world.permission = true;
    f.facts(&[0, CONTACT]);
    f.run();
    assert_eq!(f.actor().state_code(), MOVING);
    assert_eq!(f.actor().vertical_velocity, BOUNCE_HEIGHT);
    assert_eq!(f.actor().direction, 1);
    assert_eq!(f.actor().local_state_code(), 1);
    assert_eq!(
        f.world.calls,
        vec![
            motion(Movement::Fall, 0x1e, 2),
            motion(Movement::PositiveX, 0x18, 2),
            Work::PlayerContact
        ]
    );
}
#[test]
fn jumpingstealer_nonnegative_rise_clears_clocks_and_apex_falls_same_call() {
    for negative in [false, true] {
        let mut f = Fixture::new(JUMPING_STEALER);
        {
            let actor_value = MOVING;
            f.actor_mut().set_state_code(actor_value);
        };
        {
            let actor_value = 1;
            f.actor_mut().set_local_state_code(actor_value);
        };
        {
            let animation_value = 7;
            f.actor_mut().playback.set_tick_code(animation_value);
        };
        {
            let animation_value = 2;
            f.actor_mut().playback.set_frame_code(animation_value);
        };
        f.world.post_velocity = Some(if negative { -1 } else { 1 });
        f.run();
        assert_eq!(f.world.calls[0], motion(Movement::Rise, 0x21, 2));
        assert_eq!(f.world.calls.len(), if negative { 3 } else { 2 });
        if negative {
            assert_eq!(f.world.calls[1], motion(Movement::Fall, 0x1e, 2));
            assert_eq!(f.actor().state_code(), READY);
        }
        assert_eq!(
            (
                f.actor().playback.tick_code(),
                f.actor().playback.frame_code()
            ),
            (0, 0)
        );
    }
}
#[test]
fn hitonly_without_marker_has_no_contact_motion_allocation_or_state_effect() {
    let mut f = Fixture::new(HIT_ONLY);
    let old = f.actors;
    let status = f.status.clone();
    let e = f.run();
    assert_eq!(f.actors, old);
    assert_eq!(f.status, status);
    assert!(f.world.calls.is_empty());
    assert!(e.spawned.is_empty());
}
#[test]
fn launcher_excluded_player_states_precede_all_visibility_hit_and_animation_work() {
    for action in [Action::Damage, Action::Death] {
        let mut f = Fixture::new(PLAYER_LAUNCHER);
        f.player.action = action;
        f.init();
        f.actor_mut().render = 0;
        f.actor_mut().hit_marker = 1;
        {
            let animation_value = 4;
            f.actor_mut().playback.set_tick_code(animation_value);
        };
        let old = f.actors;
        f.run();
        assert_eq!(f.actors, old);
        assert!(f.world.calls.is_empty());
    }
}
#[test]
fn launcher_permission_overrides_immediate_damage_and_keeps_queued_reaction_protection() {
    let mut f = Fixture::new(PLAYER_LAUNCHER);
    f.init();
    f.player.protection = 9;
    f.player.queued = Some(Action::TerrainPose);
    f.world.permission = true;
    f.world.contact_action = Some(Action::Damage);
    let e = f.run();
    assert_eq!(f.player.action, Action::ExternalLaunch);
    assert_eq!(f.player.protection, 9);
    assert_eq!(f.player.queued, Some(Action::TerrainPose));
    assert_eq!(
        (
            f.actors[0].horizontal_magnitude,
            f.actors[0].vertical_velocity,
            f.actors[0].motion_flags
        ),
        (LAUNCH_HORIZONTAL, LAUNCH_SPEED, 0)
    );
    assert_eq!(
        (f.actor().state_code(), f.actor().playback.tick_code()),
        (READY, HELD_LAUNCH_TICKS)
    );
    assert_eq!(e.sounds, vec![0x28]);
}
#[test]
fn launcher_denied_contact_only_freezes_tick_and_used_actor_only_holds_finished_animation() {
    let mut f = Fixture::new(PLAYER_LAUNCHER);
    f.init();
    {
        let animation_value = 4;
        f.actor_mut().playback.set_tick_code(animation_value);
    };
    f.run();
    assert_eq!(f.actor().playback.tick_code(), 0);
    assert_eq!(f.actor().state_code(), ACTIVE);
    for (frame, tick, want) in [(0, 0, FROZEN_TICK), (1, 0, 0), (0, 1, 1)] {
        let mut f = Fixture::new(PLAYER_LAUNCHER);
        {
            let animation_value = frame;
            f.actor_mut().playback.set_frame_code(animation_value);
        };
        {
            let animation_value = tick;
            f.actor_mut().playback.set_tick_code(animation_value);
        };
        f.run();
        assert_eq!(f.actor().playback.tick_code(), want);
        assert!(f.world.calls.is_empty());
    }
}

#[test]
fn contactpose_release_animation_ff_wrap_falls_through_to_phase_decrement() {
    let mut f = Fixture::new(CONTACT_POSE);
    f.player.protection = 8;
    f.actor_mut().animation = 0xff;
    f.actors[0].render = VISIBLE;
    let e = f.run();
    assert_eq!(
        (
            f.actor().state_code(),
            f.actor().animation,
            f.actor().playback.frame_code(),
            f.actor().playback.tick_code()
        ),
        (ACTIVE, 0xff, 0, 0)
    );
    assert_eq!(f.actor().render & FLIP_X, 0);
    assert_eq!(e.sounds, vec![0x1a]);
}

#[test]
fn proximity_invisible_hit_culls_before_hit_animation_and_shared_reaction() {
    // 84E5 visibility branch precedes84F0 hit inspection.
    let mut f = Fixture::new(6);
    f.actor_mut().render = 0x40;
    f.actor_mut().hit_marker = 1;
    f.actor_mut().animation = 0x55;
    let effects = f.run();
    assert_eq!(
        (
            f.actor().state_code(),
            f.actor().render,
            f.actor().animation
        ),
        (0, 0x40, 0x55)
    );
    assert!(effects.work.is_empty());
}

#[test]
fn ambush_tracking_vertical_gate_matches_all_byte_height_pairs() {
    // 9885..989E subtracts playerY-actorY without borrow then compares its
    // high nibble against50. Over byte heights this is exactly a gap >=80.
    for actor_y in [0u8, 0x60, u8::MAX] {
        for player_y in 0..=u8::MAX {
            let mut f = Fixture::new(AMBUSH);
            f.actor_mut().set_state_code(MOVING);
            f.actor_mut().set_local_state_code(3);
            f.actor_mut().position.screen_y = actor_y;
            {
                let animation_value = 1;
                f.actor_mut().playback.set_tick_code(animation_value);
            };
            f.actors[0].position.screen_y = player_y;
            f.actors[0].position.world_x = ACTOR_X + 0x40;
            f.run();
            let advances = player_y >= actor_y && (player_y - actor_y) & 0xf0 >= 0x50;
            assert_eq!(
                f.actor().state_code(),
                if advances { THROWING } else { MOVING }
            );
            assert_eq!(f.actor().local_state_code(), if advances { 2 } else { 3 });
        }
    }
}

//! Whole B11 source-derived contracts; independent branch evidence in trace.md.
//! Fixtures inject owner boundary facts, never implement encounter algorithms.
use super::*;
use crate::{
    actors::{ACTIVE, ACTOR_SLOTS, ActorContact, ActorPool, VISIBLE},
    collision::Position,
};
use std::collections::VecDeque;
const SUBJECT: usize = 3;
const SPARE: usize = 15;
const READY: u8 = ACTIVE | 1;
const SECOND: u8 = ACTIVE | 2;
const THIRD: u8 = ACTIVE | 3;
const FOURTH: u8 = ACTIVE | 4;
const START_X: u16 = 0x0480;
const START_Y: u8 = 0x80;
const SOLID: u8 = 1;
const TRIPLET_SOUND: u8 = 0x1c;
const LAND_SOUND: u8 = 0x1e;
const BOUNCE_SOUND: u8 = 0x22;
const TURN_SOUND: u8 = 0x23;
const ATTACK_Y_CELL: usize = 0x6f;
const IMPACT_Y_CELL: usize = 0x2f;
const PERIMETER_Y_CELL: usize = 0x67;
const WAVE_Y_CELL: usize = 0x20;
#[derive(Default)]
struct Boundary {
    calls: Vec<Work>,
    facts: VecDeque<MotionFacts>,
    post_velocity: Option<i16>,
    post_y: Option<u8>,
    post_x: Option<u16>,
    integrated: Option<Position>,
    continuation: bool,
    preset_y: i16,
    preset_x: u16,
    reaction_seen: Vec<ActorContact>,
    contact_seen: Vec<ActorContact>,
}
impl World for Boundary {
    fn motion(
        &mut self,
        a: &mut ActorPool,
        _: &mut Player,
        _: &WorldCache,
        s: usize,
        m: Movement,
        p: u8,
        n: u8,
    ) -> MotionFacts {
        self.calls.push(Work::Motion(m, p, n));
        // Horizontal wrappers do not inject a vertical-velocity update.
        if matches!(
            m,
            Movement::Fall | Movement::Rise | Movement::PositiveY | Movement::NegativeY
        ) && let Some(v) = self.post_velocity
        {
            a[s].vertical_velocity = v;
        }
        if let Some(y) = self.post_y {
            a[s].position.screen_y = y;
        }
        if let Some(x) = self.post_x {
            a[s].position.world_x = x;
        }
        self.facts.pop_front().unwrap_or_default()
    }
    fn integrate(&mut self, a: &mut ActorPool, s: usize, o: Integration) -> bool {
        self.calls.push(Work::Integrate(o));
        if let Some(p) = self.integrated {
            a[s].position = p;
        }
        self.continuation
    }
    fn player_contact(&mut self, a: &mut ActorPool, _: &mut Player, s: usize) {
        self.calls.push(Work::PlayerContact);
        self.contact_seen.push(a[s]);
    }
    fn preset(&mut self, a: &mut ActorPool, s: usize, p: u8) {
        self.calls.push(Work::Preset(p));
        a[s].vertical_velocity = self.preset_y;
        a[s].horizontal_magnitude = self.preset_x;
    }
    fn reaction(&mut self, a: &mut ActorPool, _: &mut Player, _: &mut RandomState, s: usize) {
        self.calls.push(Work::Reaction);
        self.reaction_seen.push(a[s]);
    }
}
fn catalog() -> Catalog {
    // Authored fixture values; exact original recipes separately tested below.
    Catalog {
        attack_entries: [
            AttackPhase::Opening,
            AttackPhase::Height,
            AttackPhase::Preset,
            AttackPhase::Land,
            AttackPhase::Fire,
        ],
        heights: [0x40, 0x50, 0x60, 0x70],
        delays: [4, 6],
        directions: [1, 2],
        targets: [0x20, 0x40, 0x60, 0x80, 0xa0, 0xc0, 0xe0, 0xf0],
        motion: [
            0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
        ],
        child_offsets: [0xf0, 0x10],
        attack_facing: [0, 0x40],
        child_vertical: [0x100, 0x26a, 0],
        child_horizontal: [0x200, 0x16a, 0x200],
        jump_delays: [3, 5, 7, 9, 11, 13, 15, 17],
        split_offsets: [0xc0, 0xe0],
        perimeter: perimeter_program(
            [
                5, 9, 5, 9, 5, 9, 10, 9, 10, 9, 10, 6, 10, 6, 10, 6, 5, 6, 5, 6,
            ],
            [1, 0, 1, 0, 1, 2, 0, 2, 0, 2, 1, 0, 1, 0, 1, 2, 0, 2, 0, 2],
        ),
        corrections: [4, -4],
        wave: wave_program(
            [
                0x70, 0xd8, 0x48, 0xa8, 0x18, 0x88, 0, 0x68, 0x98, 0x28, 0xd8, 0x48, 0x88, 0,
            ],
            [0, 8, 24, 24, 32, 48, 0, 0, 8, 24, 24, 40, 40, 0],
        ),
        sequences: [0, 7],
        shake_x: [0, 4],
        center_facing: [0, 0x40],
        center_speeds: [0x3c0, 0x1cc],
    }
}
struct Fixture {
    actors: ActorPool,
    player: Player,
    random: RandomState,
    hp: u8,
    impact: u8,
    camera: Camera,
    terrain: WorldCache,
    split_low: u8,
    split_control: u8,
    tick: u8,
    increment: bool,
    catalog: Catalog,
    world: Boundary,
}
impl Fixture {
    fn new(handler: u8) -> Self {
        let mut actors = [ActorContact::default(); ACTOR_SLOTS];
        actors[SUBJECT] = ActorContact {
            control: crate::actors::ActorControl::from_codes(READY, 0, 0, 0),
            handler,
            render: VISIBLE,
            position: Position {
                world_x: START_X,
                screen_y: START_Y,
                ..Default::default()
            },
            animation: ATTACK_IDLE,
            ..Default::default()
        };
        Self {
            actors,
            player: Player::default(),
            random: RandomState::default(),
            hp: 5,
            impact: 0,
            camera: Camera {
                x: 0x04a5,
                ..Default::default()
            },
            terrain: WorldCache::default(),
            split_low: 0xc0,
            split_control: 0,
            tick: 0,
            increment: false,
            catalog: catalog(),
            world: Boundary::default(),
        }
    }
    fn actor(&mut self) -> &mut ActorContact {
        &mut self.actors[SUBJECT]
    }
    fn run(&mut self) -> Effects {
        step(
            &mut self.actors,
            SUBJECT,
            &mut Context {
                player: &mut self.player,
                random: &mut self.random,
                hit_points: &mut self.hp,
                impact: &mut self.impact,
                camera: &mut self.camera,
                terrain: &mut self.terrain,
                split_low: &mut self.split_low,
                split_control: &mut self.split_control,
                composition_tick: self.tick,
                random_increment: self.increment,
                catalog: &self.catalog,
            },
            &mut self.world,
        )
    }
    fn solid(&mut self) {
        self.world.facts.push_back(MotionFacts {
            bits: SOLID,
            ..Default::default()
        });
    }
}
fn motion(m: Movement, p: u8, n: u8) -> Work {
    Work::Motion(m, p, n)
}
fn rom() -> Vec<u8> {
    let b =
        std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../input/game.rom"))
            .expect("local matching ROM");
    romatlas_port::assets::Rom::from_bytes(b.clone(), crate::config::ROM_SHA256)
        .expect("matching ROM identity");
    b
}
const BANK_PHYSICAL: usize = 0x18010;
const BANK_CPU: usize = 0x8000;
fn bytes(b: &[u8], cpu: usize, n: usize) -> &[u8] {
    &b[BANK_PHYSICAL + cpu - BANK_CPU..BANK_PHYSICAL + cpu - BANK_CPU + n]
}
#[test]
fn original_catalog_resolves_actual_phase_entries_after_preceding_return_labels() {
    let b = rom();
    let c = Catalog::original(&b).unwrap();
    assert_eq!(
        c.attack_entries,
        [
            AttackPhase::Opening,
            AttackPhase::Height,
            AttackPhase::Preset,
            AttackPhase::Land,
            AttackPhase::Fire
        ]
    );
    const LOW: usize = 0xa585;
    const HIGH: usize = 0xa58a;
    // Source label binds preceding RTS; these are true indirect dispatch starts.
    for (i, expected) in [0xa357, 0xa386, 0xa3ec, 0xa495, 0xa4c7]
        .into_iter()
        .enumerate()
    {
        assert_eq!(
            usize::from(bytes(&b, LOW, 5)[i]) | (usize::from(bytes(&b, HIGH, 5)[i]) << 8),
            expected
        );
    }
    assert_eq!(&c.heights[..], bytes(&b, 0xa58f, 4));
    assert_eq!(&c.delays[..], bytes(&b, 0xa593, 2));
    assert_eq!(&c.directions[..], bytes(&b, 0xa595, 2));
    assert_eq!(&c.targets[..], bytes(&b, 0xa597, 8));
    assert_eq!(&c.motion[..], bytes(&b, 0xa5a0, MOTION_STEPS));
    assert_eq!(&c.child_offsets[..], bytes(&b, 0xa5ae, 2));
    assert_eq!(&c.attack_facing[..], bytes(&b, 0xa5b0, 2));
    for i in 0..3 {
        assert_eq!(
            c.child_vertical[i],
            i16::from_le_bytes([bytes(&b, 0xa5b2, 3)[i], bytes(&b, 0xa5b5, 3)[i]])
        );
        assert_eq!(
            c.child_horizontal[i],
            u16::from_le_bytes([bytes(&b, 0xa5b8, 3)[i], bytes(&b, 0xa5bb, 3)[i]])
        );
    }
}
#[test]
fn original_catalog_decodes_all_impact_path_wave_and_center_parameters() {
    let b = rom();
    let c = Catalog::original(&b).unwrap();
    assert_eq!(&c.jump_delays[..], bytes(&b, 0xa6c9, 8));
    assert_eq!(&c.split_offsets[..], bytes(&b, 0xa6d1, 2));
    assert_eq!(
        &c.perimeter.map(|step| step.source_path_code())[..],
        bytes(&b, 0xa8ac, PATH_STEPS)
    );
    assert_eq!(
        &c.perimeter.map(|step| step.gate.source_code())[..],
        bytes(&b, 0xa8c0, PATH_STEPS)
    );
    assert_eq!(
        c.corrections,
        [bytes(&b, 0xa8d4, 2)[0] as i8, bytes(&b, 0xa8d4, 2)[1] as i8]
    );
    for index in 0..WAVE_ENTRIES {
        let x = bytes(&b, 0xaaa8, WAVE_ENTRIES)[index];
        let at_tick = bytes(&b, 0xaab6, WAVE_ENTRIES)[index];
        assert_eq!(
            c.wave[index],
            if x == 0 {
                WaveStep::AwaitLastChild
            } else {
                WaveStep::Spawn { x, at_tick }
            }
        );
    }
    assert_eq!(&c.sequences[..], bytes(&b, 0xaac4, 2));
    assert_eq!(&c.shake_x[..], bytes(&b, 0xaac6, 2));
    assert_eq!(&c.center_facing[..], bytes(&b, 0xac37, 2));
    for i in 0..2 {
        assert_eq!(
            c.center_speeds[i],
            u16::from_le_bytes([bytes(&b, 0xac39, 2)[i], bytes(&b, 0xac3b, 2)[i]])
        );
    }
}
#[test]
fn bounded_catalog_rejects_missing_data_in_every_required_recipe() {
    let b = rom();
    for cpu in [0xa585, 0xa58f, 0xa5bd, 0xa6d2, 0xa8d5, 0xaac7, 0xac3c] {
        assert!(Catalog::original(&b[..BANK_PHYSICAL + cpu - BANK_CPU]).is_err());
    }
}
#[test]
fn first_marked_hit_decrements_shared_hp_and_starts_flash_without_clearing_marker() {
    let mut f = Fixture::new(ATTACK);
    f.actors[SUBJECT].hit_marker = 1;
    f.actors[SUBJECT].render = 0xa4;
    let e = hit(&mut f.actors, SUBJECT, &mut f.hp, 4);
    assert_eq!(f.hp, 4);
    assert_eq!(f.actors[SUBJECT].boss_protection, HIT_FLASH);
    assert_eq!(f.actors[SUBJECT].hit_marker, 1);
    assert_eq!(f.actors[SUBJECT].render, 0xa6);
    assert!(!e.defeated);
    assert_eq!(e.treasure, None);
}
#[test]
fn shared_hp_zero_wraps_to_ff_and_is_surviving_hit() {
    let mut f = Fixture::new(ATTACK);
    f.hp = 0;
    let e = hit(&mut f.actors, SUBJECT, &mut f.hp, 4);
    assert_eq!(f.hp, 0xff);
    assert_eq!(f.actors[SUBJECT].boss_protection, HIT_FLASH);
    assert!(!e.defeated);
}
#[test]
fn protected_marked_hit_only_decrements_interval_and_preserves_hp_marker_and_other_flags() {
    for interval in [2, HIT_FLASH, 0xff] {
        let mut f = Fixture::new(PERIMETER);
        f.actors[SUBJECT].boss_protection = interval;
        f.actors[SUBJECT].hit_marker = 7;
        f.actors[SUBJECT].render = 0xfe;
        let e = hit(&mut f.actors, SUBJECT, &mut f.hp, 4);
        assert_eq!(f.hp, 5);
        assert_eq!(f.actors[SUBJECT].boss_protection, interval - 1);
        assert_eq!(f.actors[SUBJECT].hit_marker, 7);
        assert_eq!(f.actors[SUBJECT].render, 0xfe);
        assert!(!e.defeated);
    }
}
#[test]
fn final_protected_invocation_clears_marker_and_only_flash_render_bit() {
    let mut f = Fixture::new(ATTACK);
    f.actors[SUBJECT].boss_protection = 1;
    f.actors[SUBJECT].hit_marker = 4;
    f.actors[SUBJECT].render = 0xff;
    hit(&mut f.actors, SUBJECT, &mut f.hp, 4);
    assert_eq!(f.hp, 5);
    assert_eq!(f.actors[SUBJECT].boss_protection, 0);
    assert_eq!(f.actors[SUBJECT].hit_marker, 0);
    assert_eq!(f.actors[SUBJECT].render, 0xfd);
}
#[test]
fn unmarked_boss_does_not_tick_shared_protection() {
    let mut f = Fixture::new(ATTACK);
    f.actors[SUBJECT].boss_protection = HIT_FLASH;
    {
        let actor_value = ACTIVE;
        f.actors[SUBJECT].set_state_code(actor_value);
    };
    f.run();
    assert_eq!(f.actors[SUBJECT].boss_protection, HIT_FLASH);
    assert_eq!(f.hp, 5);
}
#[test]
fn last_hit_allocates_treasure_then_reports_defeat_without_starting_flash() {
    let mut f = Fixture::new(ATTACK);
    f.hp = 1;
    f.actors[SUBJECT].hit_marker = 1;
    let e = hit(&mut f.actors, SUBJECT, &mut f.hp, 4);
    assert_eq!(f.hp, 0);
    assert!(e.defeated);
    assert_eq!(e.treasure, Some(SPARE));
    assert_eq!(f.actors[SUBJECT].boss_protection, 0);
    assert_eq!(f.actors[SPARE].animation, TREASURE_ANIMATION);
}
#[test]
fn treasure_uses_highest_inactive_sign_independent_of_visibility() {
    let mut f = Fixture::new(ATTACK);
    {
        let actor_value = 0x7f;
        f.actors[SPARE].set_state_code(actor_value);
    };
    f.actors[SPARE].render = 0xff;
    let e = treasure(&mut f.actors, SUBJECT, 7);
    assert_eq!(e.treasure, Some(SPARE));
    assert!(e.defeated);
    assert_eq!(f.actors[SPARE].position.world_x, 0x0780);
    assert_eq!(f.actors[SPARE].position.screen_y, TREASURE_Y);
    assert_eq!(f.actors[SPARE].state_code(), 0xb1);
    assert_eq!(f.actors[SPARE].render, 0xb0);
    assert_eq!(f.actors[SPARE].handler, TREASURE_HANDLER);
    assert_eq!(f.actors[SPARE].local_counter_code(), TREASURE_WAIT);
}
#[test]
fn treasure_clears_only_source_fields_and_retains_fractions_motion_identity_and_locals() {
    let mut f = Fixture::new(ATTACK);
    let prior = ActorContact {
        playback: crate::presentation::AnimationPlayback::from_codes(6, 7, 8),
        control: crate::actors::ActorControl::from_codes(0x10, 0x34, 0x56, 0),
        render: 0xff,
        position: Position {
            world_x: 0x1234,
            screen_y: 5,
            x_fraction: 0x23,
            y_fraction: 0x45,
        },
        horizontal_magnitude: 0x6789,
        vertical_velocity: -123,
        local_aux2: 0x78,
        sequence_cursor: 0x9a,
        boss_protection: 0xbc,
        hit_marker: 5,
        profile: 9,
        spawn_identity: Some(0x42),
        spawn_y: 0x43,
        motion_flags: 0x44,
        direction: 0x45,
        ..Default::default()
    };
    f.actors[SPARE] = prior;
    treasure(&mut f.actors, SUBJECT, 2);
    let a = f.actors[SPARE];
    assert_eq!((a.position.x_fraction, a.position.y_fraction), (0x23, 0x45));
    assert_eq!(
        (a.horizontal_magnitude, a.vertical_velocity),
        (0x6789, -123)
    );
    assert_eq!(
        (a.local_state_code(), a.local_aux2, a.sequence_cursor),
        (0x34, 0x78, 0x9a)
    );
    assert_eq!(
        (
            a.hit_marker,
            a.playback.tick_code(),
            a.playback.frame_code(),
            a.profile,
            a.boss_protection
        ),
        (0, 0, 0, 0, 0)
    );
    assert_eq!(
        (
            a.playback.restart_code(),
            a.spawn_identity,
            a.spawn_y,
            a.motion_flags,
            a.direction
        ),
        (8, Some(0x42), 0x43, 0x44, 0x45)
    );
}
#[test]
fn full_treasure_pool_overwrites_slot_one_and_preserves_player() {
    let mut f = Fixture::new(ATTACK);
    for a in &mut f.actors[1..] {
        {
            let actor_value = ACTIVE;
            a.set_state_code(actor_value);
        };
    }
    let player = f.actors[0];
    let e = treasure(&mut f.actors, SUBJECT, 4);
    assert_eq!(e.treasure, Some(1));
    assert_eq!(f.actors[1].animation, TREASURE_ANIMATION);
    assert_eq!(f.actors[0], player);
    assert_eq!(f.actors[SUBJECT].handler, ATTACK);
}
#[test]
fn final_animation84_bypasses_allocation_and_preserves_entire_pool() {
    let mut f = Fixture::new(ATTACK);
    f.actors[SUBJECT].animation = FINAL_DEFEAT_ANIMATION;
    let before = f.actors;
    let e = treasure(&mut f.actors, SUBJECT, 4);
    assert!(e.defeated);
    assert_eq!(e.treasure, None);
    assert_eq!(f.actors, before);
}
#[test]
fn every_land_defeat_delegates_reaction_before_local_initialization() {
    for handler in [ATTACK, IMPACT, PERIMETER, WAVE, CENTER] {
        let mut f = Fixture::new(handler);
        f.actors[SUBJECT].hit_marker = 1;
        {
            let actor_value = ACTIVE;
            f.actors[SUBJECT].set_state_code(actor_value);
        };
        // Distinct from discarded LDA5E: proves no animation rewrite.
        f.actors[SUBJECT].animation = CENTER_IDLE;
        f.hp = 1;
        f.split_low = 0xe0;
        f.split_control = 1;
        let animation = f.actors[SUBJECT].animation;
        let e = f.run();
        assert!(e.defeated);
        assert_eq!(e.treasure, Some(SPARE));
        assert_eq!(f.world.calls, [Work::Reaction]);
        assert_eq!(
            f.world.reaction_seen[0].animation, animation,
            "LDA5E is overwritten by shared visibility helper"
        );
        assert_eq!(f.actors[SUBJECT].state_code(), ACTIVE);
        if handler == IMPACT {
            assert_eq!((f.split_low, f.split_control), (0xc0, 0));
        } else {
            assert_eq!((f.split_low, f.split_control), (0xe0, 1));
        }
    }
}
#[test]
fn same_slot_one_full_pool_treasure_alias_is_visible_to_following_reaction() {
    let mut f = Fixture::new(ATTACK);
    for a in &mut f.actors[1..] {
        {
            let actor_value = ACTIVE;
            a.set_state_code(actor_value);
        };
    }
    f.actors[1] = f.actors[SUBJECT];
    f.actors[1].hit_marker = 1;
    f.hp = 1;
    let e = step(
        &mut f.actors,
        1,
        &mut Context {
            player: &mut f.player,
            random: &mut f.random,
            hit_points: &mut f.hp,
            impact: &mut f.impact,
            camera: &mut f.camera,
            terrain: &mut f.terrain,
            split_low: &mut f.split_low,
            split_control: &mut f.split_control,
            composition_tick: 0,
            random_increment: false,
            catalog: &f.catalog,
        },
        &mut f.world,
    );
    assert_eq!(e.treasure, Some(1));
    assert_eq!(f.world.reaction_seen[0].animation, TREASURE_ANIMATION);
    assert_eq!(f.world.reaction_seen[0].state_code(), 0xb1);
    assert_eq!(f.world.reaction_seen[0].profile, 0);
}
#[test]
fn attack_opening_writes_collision_cell_and_advances_only_on_sixtieth_increment() {
    for (prior, next, state) in [
        (0, 1, ACTIVE),
        (58, 59, ACTIVE),
        (59, 60, READY),
        (60, 61, ACTIVE),
        (0xff, 0, ACTIVE),
    ] {
        let mut f = Fixture::new(ATTACK);
        {
            let actor_value = ACTIVE;
            f.actors[SUBJECT].set_state_code(actor_value);
        };
        {
            let actor_value = prior;
            f.actors[SUBJECT].set_local_state_code(actor_value);
        };
        {
            let animation_value = 9;
            f.actors[SUBJECT].playback.set_tick_code(animation_value);
        };
        {
            let animation_value = 8;
            f.actors[SUBJECT].playback.set_frame_code(animation_value);
        };
        f.run();
        assert_eq!(f.terrain.collision[ATTACK_Y_CELL], 0x44);
        assert_eq!(f.terrain.changed_this_frame, 0);
        assert_eq!(f.actors[SUBJECT].local_state_code(), next);
        assert_eq!(f.actors[SUBJECT].state_code(), state);
        assert_eq!(f.actors[SUBJECT].playback.tick_code(), 0);
        if state == READY {
            assert_eq!(f.actors[SUBJECT].animation, REVEAL);
            assert_eq!(f.actors[SUBJECT].playback.frame_code(), 0);
            assert_eq!(f.actors[SUBJECT].vertical_velocity, 0x0380);
        } else {
            assert_eq!(f.actors[SUBJECT].playback.frame_code(), 8);
        }
    }
}
#[test]
fn attack_reveal_requires_both_nonzero_frame_and_exact_tick_five() {
    for (frame, tick) in [(0, 5), (1, 4), (1, 6)] {
        let mut f = Fixture::new(ATTACK);
        f.actors[SUBJECT].animation = REVEAL;
        {
            let animation_value = frame;
            f.actors[SUBJECT].playback.set_frame_code(animation_value);
        };
        {
            let animation_value = tick;
            f.actors[SUBJECT].playback.set_tick_code(animation_value);
        };
        let before = f.actors;
        f.run();
        assert_eq!(f.actors, before);
        assert!(f.world.calls.is_empty());
    }
}
#[test]
fn attack_reveal_selects_all_four_heights_with_compare_carry_increment_and_byte_wrap() {
    for word in [0, 1, 2, 3, 0xff] {
        let mut f = Fixture::new(ATTACK);
        f.actors[SUBJECT].animation = REVEAL;
        {
            let animation_value = 1;
            f.actors[SUBJECT].playback.set_frame_code(animation_value);
        };
        {
            let animation_value = 5;
            f.actors[SUBJECT].playback.set_tick_code(animation_value);
        };
        f.random.words = [word, 0, 0, 0];
        f.run();
        let sum = word.wrapping_add(1);
        assert_eq!(f.random.words, [word, sum, 0, 0]);
        assert_eq!(
            f.actors[SUBJECT].local_state_code(),
            f.catalog.heights[usize::from(sum & 3)]
        );
        assert_eq!(
            (
                f.actors[SUBJECT].animation,
                f.actors[SUBJECT].playback.tick_code(),
                f.actors[SUBJECT].playback.frame_code()
            ),
            (ATTACK_IDLE, 0, 0)
        );
        assert!(f.world.calls.is_empty());
    }
}
#[test]
fn attack_rise_integrates_before_strict_height_comparison_and_does_not_fall_through_motion() {
    for y in [0x60, 0x61] {
        let mut f = Fixture::new(ATTACK);
        {
            let actor_value = 0x60;
            f.actors[SUBJECT].set_local_state_code(actor_value);
        };
        f.world.integrated = Some(Position {
            screen_y: y,
            ..f.actors[SUBJECT].position
        });
        f.run();
        assert_eq!(f.actors[SUBJECT].state_code(), READY);
        assert_eq!(f.world.calls, [Work::Integrate(Integration::FallingY)]);
    }
}
#[test]
fn attack_crossed_height_selects_every_target_parity_and_returns_before_preset() {
    for index in 0u8..8 {
        let mut f = Fixture::new(ATTACK);
        {
            let actor_value = 0x61;
            f.actors[SUBJECT].set_local_state_code(actor_value);
        };
        f.actors[SUBJECT].position.screen_y = 0x60;
        f.actors[SUBJECT].local_aux2 = 9;
        f.actors[SUBJECT].sequence_cursor = 8;
        f.random.words = [0, index, 0, 0];
        f.run();
        assert_eq!(f.random.words[2], index);
        assert_eq!(
            f.actors[SUBJECT].local_counter_code(),
            f.catalog.targets[usize::from(index)]
        );
        assert_eq!(
            f.actors[SUBJECT].local_state_code(),
            f.catalog.delays[usize::from(index & 1)]
        );
        assert_eq!(
            f.actors[SUBJECT].direction,
            f.catalog.directions[usize::from(index & 1)]
        );
        assert_eq!(
            (
                f.actors[SUBJECT].local_aux2,
                f.actors[SUBJECT].sequence_cursor,
                f.actors[SUBJECT].state_code()
            ),
            (0, 0, SECOND)
        );
        assert_eq!(f.world.calls, [Work::Integrate(Integration::FallingY)]);
    }
}
#[test]
fn actual_preset_phase_executes_every_fourteen_entry_and_normalizes_signed_horizontal() {
    for cursor in 0..MOTION_STEPS {
        let mut f = Fixture::new(ATTACK);
        {
            let actor_value = SECOND;
            f.actors[SUBJECT].set_state_code(actor_value);
        };
        {
            let actor_value = 10;
            f.actors[SUBJECT].set_local_state_code(actor_value);
        };
        f.actors[SUBJECT].sequence_cursor = cursor as u8;
        f.actors[SUBJECT].direction = 1;
        f.world.preset_x = 0xfe80;
        f.world.preset_y = -0x0180;
        f.actors[SUBJECT].position.y_fraction = 0x40;
        let e = f.run();
        assert_eq!(
            f.world.calls,
            [
                Work::Preset(f.catalog.motion[cursor]),
                motion(Movement::PositiveX, 0x18, 2)
            ]
        );
        assert_eq!(f.actors[SUBJECT].horizontal_magnitude, 0x0180);
        assert_eq!(
            (
                f.actors[SUBJECT].position.screen_y,
                f.actors[SUBJECT].position.y_fraction
            ),
            (0x7e, 0xc0)
        );
        assert_eq!(
            f.actors[SUBJECT].sequence_cursor,
            ((cursor + 1) % MOTION_STEPS) as u8
        );
        assert_eq!(f.actors[SUBJECT].local_aux2, 2);
        assert_eq!(f.actors[SUBJECT].local_state_code(), 9);
        assert!(e.sounds.is_empty());
    }
}
#[test]
fn preset_hold_reloads_only_after_three_calls_and_direction_bit_one_selects_left() {
    let mut f = Fixture::new(ATTACK);
    {
        let actor_value = SECOND;
        f.actors[SUBJECT].set_state_code(actor_value);
    };
    {
        let actor_value = 10;
        f.actors[SUBJECT].set_local_state_code(actor_value);
    };
    f.actors[SUBJECT].direction = 2;
    for _ in 0..3 {
        f.run();
    }
    assert_eq!(
        f.world
            .calls
            .iter()
            .filter(|w| matches!(w, Work::Preset(_)))
            .count(),
        1
    );
    assert_eq!(f.actors[SUBJECT].local_aux2, 0);
    assert_eq!(
        f.world.calls.last(),
        Some(&motion(Movement::NegativeX, 0x1b, 2))
    );
    f.run();
    assert_eq!(
        f.world
            .calls
            .iter()
            .filter(|w| matches!(w, Work::Preset(_)))
            .count(),
        2
    );
}
#[test]
fn preset_horizontal_nonnegative_zero_and_signed_min_magnitude_preserve_width() {
    for (raw, expected) in [(0, 0), (0x1234, 0x1234), (0x8000, 0x8000), (0xffff, 1)] {
        let mut f = Fixture::new(ATTACK);
        {
            let actor_value = SECOND;
            f.actors[SUBJECT].set_state_code(actor_value);
        };
        {
            let actor_value = 9;
            f.actors[SUBJECT].set_local_state_code(actor_value);
        };
        f.world.preset_x = raw;
        f.run();
        assert_eq!(f.actors[SUBJECT].horizontal_magnitude, expected);
    }
}
#[test]
fn preset_distance_is_low_byte_and_strict_four_only_after_delay_expires() {
    for (x, delay, advanced) in [
        (0x0183, 1, true),
        (0x0184, 1, false),
        (0x017d, 0, true),
        (0x017c, 0, false),
        (0x0680, 2, false),
        (0x0680, 0, true),
    ] {
        let mut f = Fixture::new(ATTACK);
        {
            let actor_value = SECOND;
            f.actors[SUBJECT].set_state_code(actor_value);
        };
        {
            let actor_value = delay;
            f.actors[SUBJECT].set_local_state_code(actor_value);
        };
        f.actors[SUBJECT].local_aux2 = 2;
        {
            let actor_value = 0x80;
            f.actors[SUBJECT].set_local_counter_code(actor_value);
        };
        f.actors[SUBJECT].position.world_x = x;
        f.run();
        assert_eq!(
            f.actors[SUBJECT].state_code(),
            if advanced { THIRD } else { SECOND }
        );
        if advanced {
            assert_eq!(f.actors[SUBJECT].vertical_velocity, 0x0380);
        }
        assert_eq!(f.world.calls.len(), 1);
    }
}
#[test]
fn preset_wall_reversal_requires_expired_hold_and_retains_same_call_phase_advance() {
    for (hold, reverse) in [(1, true), (2, false)] {
        let mut f = Fixture::new(ATTACK);
        {
            let actor_value = SECOND;
            f.actors[SUBJECT].set_state_code(actor_value);
        };
        f.actors[SUBJECT].local_aux2 = hold;
        {
            let actor_value = 0x80;
            f.actors[SUBJECT].set_local_counter_code(actor_value);
        };
        f.actors[SUBJECT].direction = 1;
        f.solid();
        f.run();
        assert_eq!(f.actors[SUBJECT].direction, if reverse { 2 } else { 1 });
        assert_eq!(f.actors[SUBJECT].state_code(), THIRD);
    }
}
#[test]
fn actual_land_phase_waits_floor_then_reveals_and_faces_full_world_x() {
    let mut miss = Fixture::new(ATTACK);
    {
        let actor_value = THIRD;
        miss.actor().set_state_code(actor_value);
    };
    miss.run();
    assert_eq!(miss.actor().state_code(), THIRD);
    assert_eq!(miss.world.calls, [motion(Movement::PositiveY, 0x1e, 2)]);
    for (player_x, direction, flip) in [(0x0480, 2, 0), (0x0580, 1, 0x40), (0x0380, 2, 0)] {
        let mut f = Fixture::new(ATTACK);
        {
            let actor_value = THIRD;
            f.actors[SUBJECT].set_state_code(actor_value);
        };
        f.actors[0].position.world_x = player_x;
        f.actors[SUBJECT].render = 0xff;
        f.solid();
        f.run();
        assert_eq!(f.actors[SUBJECT].state_code(), FOURTH);
        assert_eq!(f.actors[SUBJECT].animation, REVEAL);
        assert_eq!(f.actors[SUBJECT].direction, direction);
        assert_eq!(f.actors[SUBJECT].render, 0xbf | flip);
    }
}
#[test]
fn actual_fire_phase_reveal_gate_and_fifteen_tick_hold_do_not_spawn_on_expiry_call() {
    let mut f = Fixture::new(ATTACK);
    {
        let actor_value = FOURTH;
        f.actors[SUBJECT].set_state_code(actor_value);
    };
    f.actors[SUBJECT].animation = REVEAL;
    {
        let animation_value = 1;
        f.actors[SUBJECT].playback.set_frame_code(animation_value);
    };
    {
        let animation_value = 5;
        f.actors[SUBJECT].playback.set_tick_code(animation_value);
    };
    f.run();
    assert_eq!(
        (
            f.actors[SUBJECT].animation,
            f.actors[SUBJECT].local_state_code(),
            f.actors[SUBJECT].local_counter_code()
        ),
        (ATTACK_FIRE, SHORT_DELAY, 0)
    );
    for _ in 0..SHORT_DELAY {
        let e = f.run();
        assert!(e.children.is_empty());
        assert_eq!(f.actors[SUBJECT].playback.tick_code(), 0);
    }
    assert_eq!(f.actors[SUBJECT].local_state_code(), 0);
}
#[test]
fn fire_reveal_mismatch_and_nonattack_frame_do_not_allocate() {
    for (animation, frame, tick) in [
        (REVEAL, 0, 5),
        (REVEAL, 1, 4),
        (ATTACK_FIRE, 1, 0),
        (ATTACK_FIRE, 3, 0),
    ] {
        let mut f = Fixture::new(ATTACK);
        {
            let actor_value = FOURTH;
            f.actors[SUBJECT].set_state_code(actor_value);
        };
        f.actors[SUBJECT].animation = animation;
        {
            let animation_value = frame;
            f.actors[SUBJECT].playback.set_frame_code(animation_value);
        };
        {
            let animation_value = tick;
            f.actors[SUBJECT].playback.set_tick_code(animation_value);
        };
        let e = f.run();
        assert!(e.children.is_empty());
        assert!(e.sounds.is_empty());
    }
}
#[test]
fn fire_event_overwrites_reserved_slots_in_reverse_order_with_all_three_speed_pairs() {
    let mut f = Fixture::new(ATTACK);
    {
        let actor_value = FOURTH;
        f.actors[SUBJECT].set_state_code(actor_value);
    };
    f.actors[SUBJECT].animation = ATTACK_FIRE;
    {
        let animation_value = 2;
        f.actors[SUBJECT].playback.set_frame_code(animation_value);
    };
    f.actors[SUBJECT].direction = 1;
    f.actors[SUBJECT].position.world_x = 0x04f8;
    for s in FIRST_CHILD..=LAST_TRIPLET {
        f.actors[s] = ActorContact {
            control: crate::actors::ActorControl::from_codes(0xff, 9, 0, 0),
            hit_marker: 7,
            boss_protection: 8,
            spawn_identity: Some(10),
            position: Position {
                x_fraction: 11,
                y_fraction: 12,
                ..Default::default()
            },
            ..Default::default()
        };
    }
    let e = f.run();
    assert_eq!(e.children, [10, 9, 8]);
    assert_eq!(e.sounds, [TRIPLET_SOUND]);
    for (i, s) in (FIRST_CHILD..=LAST_TRIPLET).enumerate() {
        let a = f.actors[s];
        assert_eq!(
            a.position.world_x, 0x0408,
            "low-byte offset ignores carry into page"
        );
        assert_eq!(a.position.screen_y, START_Y);
        assert_eq!(
            (a.state_code(), a.render, a.handler, a.animation, a.profile),
            (0xa0, 0xa0, PROJECTILE, ATTACK_CHILD_ANIMATION, 0x1d)
        );
        assert_eq!(
            (a.vertical_velocity, a.horizontal_magnitude),
            (f.catalog.child_vertical[i], f.catalog.child_horizontal[i])
        );
        assert_eq!(
            (
                a.hit_marker,
                a.boss_protection,
                a.local_state_code(),
                a.spawn_identity
            ),
            (7, 8, 9, Some(10))
        );
        assert_eq!((a.position.x_fraction, a.position.y_fraction), (11, 12));
    }
    assert_eq!(
        (
            f.actors[SUBJECT].local_state_code(),
            f.actors[SUBJECT].local_counter_code()
        ),
        (IMPACT_TICKS, 1)
    );
}
#[test]
fn attack_fire_counter_restarts_reveal_tail_without_opening_cache_write() {
    let mut f = Fixture::new(ATTACK);
    {
        let actor_value = FOURTH;
        f.actors[SUBJECT].set_state_code(actor_value);
    };
    f.actors[SUBJECT].animation = ATTACK_FIRE;
    {
        let actor_value = 1;
        f.actors[SUBJECT].set_local_counter_code(actor_value);
    };
    {
        let actor_value = 0;
        f.actors[SUBJECT].set_local_state_code(actor_value);
    };
    {
        let animation_value = 4;
        f.actors[SUBJECT].playback.set_frame_code(animation_value);
    };
    f.run();
    assert_eq!(f.actors[SUBJECT].state_code(), READY);
    assert_eq!(f.actors[SUBJECT].animation, REVEAL);
    assert_eq!(f.actors[SUBJECT].vertical_velocity, 0x0380);
    assert_eq!(f.terrain.collision[ATTACK_Y_CELL], 0);
    assert_eq!(f.actors[SUBJECT].local_counter_code(), 1);
}
#[test]
fn projectile_contact_precedes_integration_and_both_facing_branches() {
    for (direction, horizontal) in [
        (1, Integration::PositiveX),
        (2, Integration::NegativeX),
        (3, Integration::PositiveX),
    ] {
        let mut f = Fixture::new(PROJECTILE);
        f.actors[SUBJECT].direction = direction;
        f.world.continuation = true;
        f.run();
        assert_eq!(
            f.world.calls,
            [
                Work::PlayerContact,
                Work::Integrate(Integration::FallingY),
                Work::Integrate(horizontal)
            ]
        );
        assert_eq!(f.actors[SUBJECT].state_code(), READY);
    }
}
#[test]
fn projectile_vertical_terminal_clears_state_only_after_contact() {
    let mut f = Fixture::new(PROJECTILE);
    f.actors[SUBJECT].render = 0xa8;
    f.run();
    assert_eq!(f.actors[SUBJECT].state_code(), 0);
    assert_eq!(f.actors[SUBJECT].render, 0xa8);
    assert_eq!(
        f.world.calls,
        [Work::PlayerContact, Work::Integrate(Integration::FallingY)]
    );
}
#[test]
fn impact_initializes_then_moves_and_decrements_delay_in_same_call() {
    let mut f = Fixture::new(IMPACT);
    {
        let actor_value = ACTIVE;
        f.actors[SUBJECT].set_state_code(actor_value);
    };
    {
        let actor_value = 7;
        f.actors[SUBJECT].set_local_counter_code(actor_value);
    };
    f.run();
    assert_eq!(
        (
            f.actors[SUBJECT].position.screen_y,
            f.actors[SUBJECT].horizontal_magnitude,
            f.actors[SUBJECT].direction
        ),
        (0xac, 0x0100, 1)
    );
    assert_eq!(
        (
            f.actors[SUBJECT].state_code(),
            f.actors[SUBJECT].local_state_code(),
            f.actors[SUBJECT].local_counter_code()
        ),
        (READY, 29, 0)
    );
    assert_eq!(f.terrain.collision[IMPACT_Y_CELL], 0x44);
    assert_eq!(f.terrain.changed_this_frame, 0);
    assert_eq!(f.world.calls, [motion(Movement::PositiveX, 0x61, 0)]);
}
#[test]
fn impact_delay_expiry_faces_after_horizontal_and_then_obeys_collision_reversal() {
    for (bits, expected_direction) in [(0, 1), (SOLID, 2)] {
        let mut f = Fixture::new(IMPACT);
        {
            let actor_value = 1;
            f.actors[SUBJECT].set_local_state_code(actor_value);
        };
        f.actors[SUBJECT].direction = 2;
        f.actors[0].position.world_x = 0x0580;
        f.world.post_x = Some(0x0481);
        f.world.facts.push_back(MotionFacts {
            bits,
            ..Default::default()
        });
        f.run();
        assert_eq!(f.actors[SUBJECT].vertical_velocity, 0x06b7);
        assert_eq!(f.actors[SUBJECT].local_state_code(), 0);
        assert_eq!(f.actors[SUBJECT].direction, expected_direction);
        assert_eq!(f.world.calls, [motion(Movement::NegativeX, 0x62, 0)]);
    }
}
#[test]
fn impact_rising_sign_transition_clears_velocity_and_moves_horizontally() {
    for (velocity, phase) in [(0, READY), (-1, SECOND)] {
        let mut f = Fixture::new(IMPACT);
        f.actors[SUBJECT].direction = 1;
        f.world.post_velocity = Some(velocity);
        f.run();
        assert_eq!(f.actors[SUBJECT].state_code(), phase);
        assert_eq!(
            f.world.calls,
            [
                motion(Movement::Rise, 0x64, 0),
                motion(Movement::PositiveX, 0x61, 0)
            ]
        );
        if velocity < 0 {
            assert_eq!(f.actors[SUBJECT].vertical_velocity, 0);
        }
    }
}
#[test]
fn impact_fall_miss_moves_but_landing_starts_shake_at_twenty_nine_same_call() {
    let mut miss = Fixture::new(IMPACT);
    {
        let actor_value = SECOND;
        miss.actor().set_state_code(actor_value);
    };
    miss.run();
    assert_eq!(
        miss.world.calls,
        [
            motion(Movement::Fall, 0x63, 0),
            motion(Movement::NegativeX, 0x62, 0)
        ]
    );
    let mut f = Fixture::new(IMPACT);
    {
        let actor_value = SECOND;
        f.actors[SUBJECT].set_state_code(actor_value);
    };
    f.impact = 0xff;
    f.tick = 4;
    f.solid();
    let e = f.run();
    assert_eq!(e.sounds, [LAND_SOUND]);
    assert_eq!(f.impact, 0);
    assert_eq!(f.actors[SUBJECT].local_counter_code(), 29);
    assert_eq!((f.split_low, f.split_control), (0xe0, 1));
    assert_eq!(f.world.calls, [motion(Movement::Fall, 0x63, 0)]);
}
#[test]
fn impact_shake_uses_composition_tick_bit_two_and_never_moves() {
    for (tick, control, low) in [
        (0, 0, 0xc0),
        (3, 0, 0xc0),
        (4, 1, 0xe0),
        (7, 1, 0xe0),
        (8, 0, 0xc0),
        (0xff, 1, 0xe0),
    ] {
        let mut f = Fixture::new(IMPACT);
        {
            let actor_value = SECOND;
            f.actors[SUBJECT].set_state_code(actor_value);
        };
        {
            let actor_value = 2;
            f.actors[SUBJECT].set_local_counter_code(actor_value);
        };
        f.tick = tick;
        f.run();
        assert_eq!((f.split_control, f.split_low), (control, low));
        assert_eq!(f.actors[SUBJECT].local_counter_code(), 1);
        assert!(f.world.calls.is_empty());
    }
}
#[test]
fn impact_shake_expiry_selects_all_eight_delays_and_preserves_caller_specific_increment() {
    for increment in [false, true] {
        for seed in 0u8..8 {
            let mut f = Fixture::new(IMPACT);
            {
                let actor_value = SECOND;
                f.actors[SUBJECT].set_state_code(actor_value);
            };
            {
                let actor_value = 1;
                f.actors[SUBJECT].set_local_counter_code(actor_value);
            };
            f.random.words = [seed, 0x44, 0x55, 0];
            f.increment = increment;
            f.impact = 3;
            f.split_low = 0xe0;
            f.split_control = 1;
            f.run();
            let expected = seed + u8::from(increment);
            assert_eq!(f.random.words, [expected, 0x44, 0x55, 0]);
            assert_eq!(
                f.actors[SUBJECT].local_state_code(),
                f.catalog.jump_delays[usize::from(expected & 7)]
            );
            assert_eq!(f.actors[SUBJECT].state_code(), READY);
            assert_eq!((f.impact, f.split_low, f.split_control), (0, 0xc0, 0));
            assert!(f.world.calls.is_empty());
        }
    }
}
#[test]
fn marked_surviving_hit_clears_inherited_random_increment_before_impact_expiry() {
    let mut f = Fixture::new(IMPACT);
    {
        let actor_value = SECOND;
        f.actors[SUBJECT].set_state_code(actor_value);
    };
    {
        let actor_value = 1;
        f.actors[SUBJECT].set_local_counter_code(actor_value);
    };
    f.actors[SUBJECT].hit_marker = 1;
    f.actors[SUBJECT].boss_protection = 2;
    f.increment = true;
    f.random.words = [1, 0, 0, 2];
    f.run();
    assert_eq!(f.random.words[0], 3);
    assert_eq!(f.hp, 5);
    assert_eq!(f.actors[SUBJECT].boss_protection, 1);
}
#[test]
fn perimeter_reveal_gate_then_profile_enable_returns_without_opening_motion() {
    for (frame, tick, accepted) in [(0, 5, false), (1, 4, false), (1, 5, true)] {
        let mut f = Fixture::new(PERIMETER);
        {
            let actor_value = ACTIVE;
            f.actors[SUBJECT].set_state_code(actor_value);
        };
        f.actors[SUBJECT].animation = REVEAL;
        {
            let animation_value = frame;
            f.actors[SUBJECT].playback.set_frame_code(animation_value);
        };
        {
            let animation_value = tick;
            f.actors[SUBJECT].playback.set_tick_code(animation_value);
        };
        f.actors[SUBJECT].profile = 0x12;
        f.run();
        assert_eq!(
            f.actors[SUBJECT].profile,
            if accepted { 0x92 } else { 0x12 }
        );
        assert_eq!(
            f.actors[SUBJECT].animation,
            if accepted { PERIMETER_IDLE } else { REVEAL }
        );
        assert_eq!(f.actors[SUBJECT].state_code(), ACTIVE);
        assert!(f.world.calls.is_empty());
    }
}
#[test]
fn perimeter_initialization_preserves_page_fractions_and_falls_same_call() {
    let mut f = Fixture::new(PERIMETER);
    {
        let actor_value = ACTIVE;
        f.actors[SUBJECT].set_state_code(actor_value);
    };
    f.actors[SUBJECT].animation = PERIMETER_IDLE;
    f.actors[SUBJECT].position.world_x = 0x09aa;
    f.actors[SUBJECT].position.x_fraction = 7;
    f.actors[SUBJECT].position.y_fraction = 8;
    f.run();
    assert_eq!(
        (
            f.actors[SUBJECT].position.world_x,
            f.actors[SUBJECT].position.screen_y
        ),
        (0x0980, 0xb0)
    );
    assert_eq!(
        (
            f.actors[SUBJECT].position.x_fraction,
            f.actors[SUBJECT].position.y_fraction
        ),
        (7, 8)
    );
    assert_eq!(f.terrain.collision[PERIMETER_Y_CELL], 0x55);
    assert_eq!(f.world.calls, [motion(Movement::Fall, 0x31, 2)]);
    assert_eq!(f.actors[SUBJECT].state_code(), READY);
}
#[test]
fn perimeter_first_two_landed_bounces_launch_without_path_transition() {
    for previous in [0, 1] {
        let mut f = Fixture::new(PERIMETER);
        {
            let actor_value = previous;
            f.actors[SUBJECT].set_local_state_code(actor_value);
        };
        f.solid();
        let e = f.run();
        assert_eq!(e.sounds, [BOUNCE_SOUND]);
        assert_eq!(f.actors[SUBJECT].local_state_code(), previous + 1);
        assert_eq!(f.actors[SUBJECT].motion_flags, 1);
        assert_eq!(f.actors[SUBJECT].vertical_velocity, 0x02d4);
        assert_eq!(f.actors[SUBJECT].state_code(), READY);
    }
}
#[test]
fn perimeter_third_bounce_selects_both_orientations_with_compare_increment_and_node_zero() {
    for (seed, word, direction, motion_flags) in [(0, 1, 2, 4), (1, 2, 1, 4), (0xff, 0, 1, 4)] {
        let mut f = Fixture::new(PERIMETER);
        {
            let actor_value = 2;
            f.actors[SUBJECT].set_local_state_code(actor_value);
        };
        f.random.words = [seed, 0, 0, 0];
        f.solid();
        f.run();
        assert_eq!(f.random.words[1], word);
        assert_eq!(f.actors[SUBJECT].state_code(), SECOND);
        assert_eq!(
            f.actors[SUBJECT].local_state_code(),
            if word & 1 == 0 { 1 } else { 2 }
        );
        assert_eq!(
            (f.actors[SUBJECT].direction, f.actors[SUBJECT].motion_flags),
            (direction, motion_flags)
        );
        assert_eq!(f.actors[SUBJECT].local_counter_code(), 0);
        assert_eq!(f.actors[SUBJECT].local_aux2, 0);
        assert_eq!(
            (
                f.actors[SUBJECT].horizontal_magnitude,
                f.actors[SUBJECT].vertical_velocity
            ),
            (0x0398, 0x0398)
        );
        assert_eq!(f.actors[SUBJECT].animation, REVEAL);
    }
}
#[test]
fn perimeter_upward_bounce_clears_sign_crossing_without_falling_same_call() {
    for (v, flag) in [(0, 1), (-1, 0)] {
        let mut f = Fixture::new(PERIMETER);
        f.actors[SUBJECT].motion_flags = 1;
        f.world.post_velocity = Some(v);
        f.run();
        assert_eq!(f.actors[SUBJECT].motion_flags, flag);
        assert_eq!(f.world.calls, [motion(Movement::Rise, 0x52, 2)]);
        if v < 0 {
            assert_eq!(f.actors[SUBJECT].vertical_velocity, 0);
        }
    }
}
#[test]
fn perimeter_moving_reveal_gate_then_contact_vertical_horizontal_order_and_profile_clear() {
    let mut f = Fixture::new(PERIMETER);
    {
        let actor_value = SECOND;
        f.actors[SUBJECT].set_state_code(actor_value);
    };
    f.actors[SUBJECT].animation = REVEAL;
    {
        let animation_value = 1;
        f.actors[SUBJECT].playback.set_frame_code(animation_value);
    };
    {
        let animation_value = 5;
        f.actors[SUBJECT].playback.set_tick_code(animation_value);
    };
    f.actors[SUBJECT].profile = 0x92;
    f.actors[SUBJECT].direction = 1;
    f.actors[SUBJECT].motion_flags = 4;
    {
        let actor_value = 0;
        f.actors[SUBJECT].set_local_counter_code(actor_value);
    };
    f.run();
    assert_eq!(f.actors[SUBJECT].animation, PERIMETER_MOVING);
    assert_eq!(f.actors[SUBJECT].profile, 0x12);
    assert_eq!(
        f.world.calls,
        [
            Work::PlayerContact,
            motion(Movement::PositiveY, 0x31, 2),
            motion(Movement::PositiveX, 0x34, 2)
        ]
    );
    assert_eq!(f.world.contact_seen[0].profile, 0x12);
}
#[test]
fn perimeter_all_twenty_path_records_advance_with_masks_zero_correction_and_mirroring() {
    for orientation in [1, 2] {
        for cursor in 0..PATH_STEPS {
            let mut f = Fixture::new(PERIMETER);
            {
                let actor_value = SECOND;
                f.actors[SUBJECT].set_state_code(actor_value);
            };
            f.actors[SUBJECT].animation = PERIMETER_MOVING;
            {
                let actor_value = orientation;
                f.actors[SUBJECT].set_local_state_code(actor_value);
            };
            {
                let actor_value = cursor as u8;
                f.actors[SUBJECT].set_local_counter_code(actor_value);
            };
            f.actors[SUBJECT].motion_flags = 8;
            f.actors[SUBJECT].position.screen_y = 1;
            let mask = f.catalog.perimeter[cursor].gate.source_code();
            f.world.facts.push_back(MotionFacts {
                bits: if mask & 2 != 0 { 1 } else { 0 },
                ..Default::default()
            });
            f.world.facts.push_back(MotionFacts {
                bits: mask & 1,
                ..Default::default()
            });
            f.run();
            let next = (cursor + 1) % PATH_STEPS;
            let record = f.catalog.perimeter[next].source_path_code();
            let expected_direction = if orientation == 1 {
                record & 3
            } else {
                (record & 3) ^ 3
            };
            assert_eq!(
                (
                    f.actors[SUBJECT].local_counter_code(),
                    f.actors[SUBJECT].direction,
                    f.actors[SUBJECT].motion_flags
                ),
                (next as u8, expected_direction, record & 0x0c)
            );
            assert_eq!(
                f.actors[SUBJECT].position.screen_y,
                if mask == 0 { 0xfd } else { 1 }
            );
            assert_eq!(f.actors[SUBJECT].local_aux2, if next == 0 { 1 } else { 0 });
        }
    }
}
#[test]
fn perimeter_nonzero_mask_ignores_unrequested_axis_but_zero_mask_requires_neither() {
    let mut f = Fixture::new(PERIMETER);
    {
        let actor_value = SECOND;
        f.actors[SUBJECT].set_state_code(actor_value);
    };
    {
        let actor_value = 1;
        f.actors[SUBJECT].set_local_state_code(actor_value);
    };
    f.actors[SUBJECT].animation = PERIMETER_MOVING;
    {
        let actor_value = 0;
        f.actors[SUBJECT].set_local_counter_code(actor_value);
    };
    f.solid();
    f.solid();
    f.run();
    assert_eq!(f.actors[SUBJECT].local_counter_code(), 1);
    let mut g = Fixture::new(PERIMETER);
    {
        let actor_value = SECOND;
        g.actor().set_state_code(actor_value);
    };
    g.actor().animation = PERIMETER_MOVING;
    {
        let actor_value = 1;
        g.actor().set_local_counter_code(actor_value);
    };
    g.solid();
    g.run();
    assert_eq!(g.actor().local_counter_code(), 1);
}
#[test]
fn perimeter_positive_zero_mask_correction_wraps_integer_y_only() {
    let mut f = Fixture::new(PERIMETER);
    {
        let actor_value = SECOND;
        f.actors[SUBJECT].set_state_code(actor_value);
    };
    f.actors[SUBJECT].animation = PERIMETER_MOVING;
    {
        let actor_value = 1;
        f.actors[SUBJECT].set_local_counter_code(actor_value);
    };
    {
        let actor_value = 1;
        f.actors[SUBJECT].set_local_state_code(actor_value);
    };
    f.actors[SUBJECT].position.screen_y = 0xfe;
    f.actors[SUBJECT].position.y_fraction = 9;
    f.run();
    assert_eq!(
        (
            f.actors[SUBJECT].position.screen_y,
            f.actors[SUBJECT].position.y_fraction
        ),
        (2, 9)
    );
}
#[test]
fn perimeter_return_restart_requires_completed_circuit_strict_three_and_mask_mismatch() {
    for (x, circuit, restart) in [
        (0x1282, 1, true),
        (0x127e, 1, true),
        (0x1283, 1, false),
        (0x127d, 1, false),
        (0x1280, 0, false),
    ] {
        let mut f = Fixture::new(PERIMETER);
        {
            let actor_value = SECOND;
            f.actors[SUBJECT].set_state_code(actor_value);
        };
        f.actors[SUBJECT].animation = PERIMETER_MOVING;
        f.actors[SUBJECT].local_aux2 = circuit;
        f.actors[SUBJECT].position.world_x = x;
        f.run();
        assert_eq!(
            f.actors[SUBJECT].state_code(),
            if restart { ACTIVE } else { SECOND }
        );
        if restart {
            assert_eq!(
                (
                    f.actors[SUBJECT].animation,
                    f.actors[SUBJECT].playback.frame_code(),
                    f.actors[SUBJECT].playback.tick_code()
                ),
                (REVEAL, 0, 0)
            );
        }
    }
}
#[test]
fn wave_opening_initializes_cache_and_velocity_then_frame_zero_waits() {
    let mut f = Fixture::new(WAVE);
    {
        let actor_value = ACTIVE;
        f.actors[SUBJECT].set_state_code(actor_value);
    };
    f.actors[SUBJECT].animation = WAVE_MOVING;
    f.run();
    assert_eq!(f.actors[SUBJECT].state_code(), READY);
    assert_eq!(
        (
            f.actors[SUBJECT].horizontal_magnitude,
            f.actors[SUBJECT].vertical_velocity
        ),
        (0x01cc, 0x06ee)
    );
    assert_eq!(f.actors[SUBJECT].direction, 2);
    assert_eq!(f.terrain.collision[WAVE_Y_CELL], 0x11);
    assert_eq!(f.terrain.changed_this_frame, 0);
    assert!(f.world.calls.is_empty());
}
#[test]
fn wave_countdown_freezes_tick_and_advances_frame_only_on_expiry_without_motion() {
    for (counter, frame) in [(2, 4), (1, 5)] {
        let mut f = Fixture::new(WAVE);
        {
            let actor_value = counter;
            f.actors[SUBJECT].set_local_counter_code(actor_value);
        };
        {
            let animation_value = 7;
            f.actors[SUBJECT].playback.set_tick_code(animation_value);
        };
        {
            let animation_value = 4;
            f.actors[SUBJECT].playback.set_frame_code(animation_value);
        };
        f.run();
        assert_eq!(f.actors[SUBJECT].local_counter_code(), counter - 1);
        assert_eq!(f.actors[SUBJECT].playback.tick_code(), 0);
        assert_eq!(f.actors[SUBJECT].playback.frame_code(), frame);
        assert!(f.world.calls.is_empty());
    }
}
#[test]
fn wave_rise_requires_frame_two_and_sign_crossing_returns_before_horizontal() {
    let mut blocked = Fixture::new(WAVE);
    {
        let animation_value = 1;
        blocked.actor().playback.set_frame_code(animation_value);
    };
    blocked.run();
    assert!(blocked.world.calls.is_empty());
    for (v, phase, steps) in [(0, 0, 2), (-1, 1, 1)] {
        let mut f = Fixture::new(WAVE);
        {
            let animation_value = 2;
            f.actors[SUBJECT].playback.set_frame_code(animation_value);
        };
        f.world.post_velocity = Some(v);
        f.run();
        assert_eq!(f.actors[SUBJECT].local_state_code(), phase);
        assert_eq!(f.actors[SUBJECT].playback.tick_code(), 0);
        assert_eq!(f.world.calls.len(), steps);
        assert_eq!(f.world.calls[0], motion(Movement::Rise, 0x52, 2));
        if v < 0 {
            assert_eq!(f.actors[SUBJECT].vertical_velocity, 0);
        }
    }
}
#[test]
fn wave_falling_holds_frame_two_and_no_floor_continues_horizontal() {
    let mut f = Fixture::new(WAVE);
    {
        let actor_value = 1;
        f.actors[SUBJECT].set_local_state_code(actor_value);
    };
    {
        let animation_value = 2;
        f.actors[SUBJECT].playback.set_frame_code(animation_value);
    };
    {
        let animation_value = 6;
        f.actors[SUBJECT].playback.set_tick_code(animation_value);
    };
    f.actors[SUBJECT].direction = 1;
    f.run();
    assert_eq!(f.actors[SUBJECT].playback.tick_code(), 0);
    assert_eq!(
        f.world.calls,
        [
            motion(Movement::Fall, 0x31, 2),
            motion(Movement::PositiveX, 0x34, 2)
        ]
    );
}
#[test]
fn wave_landing_sets_frame_one_and_sound_depends_on_tick_zero_even_when_frame_changes() {
    for frame in [0, 1, 2, 3] {
        let mut f = Fixture::new(WAVE);
        {
            let actor_value = 1;
            f.actors[SUBJECT].set_local_state_code(actor_value);
        };
        {
            let animation_value = frame;
            f.actors[SUBJECT].playback.set_frame_code(animation_value);
        };
        f.actors[SUBJECT].playback.restart_clock();
        f.solid();
        let e = f.run();
        assert_eq!(f.actors[SUBJECT].playback.frame_code(), 1);
        assert_eq!(e.sounds, [LAND_SOUND]);
        assert_eq!(f.world.calls, [motion(Movement::Fall, 0x31, 2)]);
    }
}
#[test]
fn wave_landing_nonseven_tick_returns_without_horizontal_or_relaunch() {
    for tick in [1, 6, 8] {
        let mut f = Fixture::new(WAVE);
        {
            let actor_value = 1;
            f.actors[SUBJECT].set_local_state_code(actor_value);
        };
        {
            let animation_value = 1;
            f.actors[SUBJECT].playback.set_frame_code(animation_value);
        };
        {
            let animation_value = tick;
            f.actors[SUBJECT].playback.set_tick_code(animation_value);
        };
        f.solid();
        let e = f.run();
        assert!(e.sounds.is_empty());
        assert_eq!(f.actors[SUBJECT].local_counter_code(), 0);
        assert_eq!(f.world.calls, [motion(Movement::Fall, 0x31, 2)]);
    }
}
#[test]
fn wave_landing_tick_seven_relaunches_and_wall_contact_enters_turn_same_call() {
    for wall in [false, true] {
        let mut f = Fixture::new(WAVE);
        {
            let actor_value = 1;
            f.actors[SUBJECT].set_local_state_code(actor_value);
        };
        {
            let animation_value = 1;
            f.actors[SUBJECT].playback.set_frame_code(animation_value);
        };
        {
            let animation_value = 7;
            f.actors[SUBJECT].playback.set_tick_code(animation_value);
        };
        f.actors[SUBJECT].direction = 1;
        f.solid();
        f.world.facts.push_back(MotionFacts {
            bits: u8::from(wall),
            ..Default::default()
        });
        let e = f.run();
        assert_eq!(f.actors[SUBJECT].local_counter_code(), IMPACT_TICKS);
        assert_eq!(f.actors[SUBJECT].vertical_velocity, 0x06ee);
        assert_eq!(f.actors[SUBJECT].local_state_code(), 0);
        assert_eq!(
            f.actors[SUBJECT].state_code(),
            if wall { SECOND } else { READY }
        );
        assert_eq!(f.actors[SUBJECT].direction, if wall { 2 } else { 1 });
        assert_eq!(e.sounds, if wall { vec![TURN_SOUND] } else { vec![] });
        if wall {
            assert_eq!(
                (
                    f.actors[SUBJECT].animation,
                    f.actors[SUBJECT].playback.frame_code(),
                    f.actors[SUBJECT].playback.tick_code()
                ),
                (WAVE_TURN, 0, 0)
            );
        }
    }
}
#[test]
fn wave_horizontal_wall_without_relaunch_counter_does_not_trigger_wave() {
    let mut f = Fixture::new(WAVE);
    {
        let animation_value = 2;
        f.actors[SUBJECT].playback.set_frame_code(animation_value);
    };
    f.actors[SUBJECT].direction = 2;
    f.world.facts.push_back(MotionFacts::default());
    f.solid();
    let e = f.run();
    assert_eq!(f.actors[SUBJECT].state_code(), READY);
    assert_eq!(f.actors[SUBJECT].direction, 2);
    assert!(e.sounds.is_empty());
}
#[test]
fn wave_turn_reveal_waits_both_conditions_before_rng_and_spawning() {
    for (frame, tick) in [(0, 7), (1, 6), (1, 8)] {
        let mut f = Fixture::new(WAVE);
        {
            let actor_value = SECOND;
            f.actors[SUBJECT].set_state_code(actor_value);
        };
        f.actors[SUBJECT].animation = WAVE_TURN;
        {
            let animation_value = frame;
            f.actors[SUBJECT].playback.set_frame_code(animation_value);
        };
        {
            let animation_value = tick;
            f.actors[SUBJECT].playback.set_tick_code(animation_value);
        };
        let before = f.actors;
        f.run();
        assert_eq!(f.actors, before);
        assert_eq!(f.random.words, [0; 4]);
    }
}
#[test]
fn wave_turn_completion_rng_compare_increment_selects_both_sequences_and_spawns_immediately() {
    for (seed, word, start_x, cursor) in [(0, 1, 0x68, 8), (1, 2, 0x70, 1), (0xff, 0, 0x70, 1)] {
        let mut f = Fixture::new(WAVE);
        {
            let actor_value = SECOND;
            f.actors[SUBJECT].set_state_code(actor_value);
        };
        f.actors[SUBJECT].animation = WAVE_TURN;
        {
            let animation_value = 1;
            f.actors[SUBJECT].playback.set_frame_code(animation_value);
        };
        {
            let animation_value = 7;
            f.actors[SUBJECT].playback.set_tick_code(animation_value);
        };
        f.random.words = [seed, 0, 0, 0];
        let e = f.run();
        assert_eq!(f.random.words[1], word);
        assert_eq!(e.children, [FIRST_CHILD]);
        assert_eq!(f.actors[FIRST_CHILD].position.world_x, 0x0400 | start_x);
        assert_eq!(f.actors[SUBJECT].local_counter_code(), cursor);
        assert_eq!(f.actors[SUBJECT].local_state_code(), 1);
        assert_eq!(f.actors[SUBJECT].local_aux2, 9);
        assert_eq!(f.actors[SUBJECT].animation, WAVE_MOVING);
    }
}
#[test]
fn wave_emission_uses_composition_tick_shake_preserving_camera_page_and_byte_tick_wrap() {
    for (tick, x) in [(0, 0), (3, 0), (4, 4), (7, 4), (8, 0)] {
        let mut f = Fixture::new(WAVE);
        {
            let actor_value = SECOND;
            f.actors[SUBJECT].set_state_code(actor_value);
        };
        f.actors[SUBJECT].animation = WAVE_MOVING;
        {
            let actor_value = 0xff;
            f.actors[SUBJECT].set_local_state_code(actor_value);
        };
        {
            let actor_value = 1;
            f.actors[SUBJECT].set_local_counter_code(actor_value);
        };
        f.actors[SUBJECT].local_aux2 = 9;
        f.tick = tick;
        f.run();
        assert_eq!(f.camera.x, 0x0400 | x);
        assert_eq!(f.actors[SUBJECT].local_state_code(), 0);
        assert_eq!(f.actors[SUBJECT].playback.tick_code(), 0);
    }
}
#[test]
fn equal_time_wave_entries_spawn_both_children_same_call_with_selective_overwrite() {
    let mut f = Fixture::new(WAVE);
    {
        let actor_value = SECOND;
        f.actors[SUBJECT].set_state_code(actor_value);
    };
    f.actors[SUBJECT].animation = WAVE_MOVING;
    {
        let actor_value = 2;
        f.actors[SUBJECT].set_local_counter_code(actor_value);
    };
    {
        let actor_value = 24;
        f.actors[SUBJECT].set_local_state_code(actor_value);
    };
    f.actors[SUBJECT].local_aux2 = 10;
    for s in [10, 11] {
        f.actors[s] = ActorContact {
            control: crate::actors::ActorControl::from_codes(0xff, 7, 6, 0),
            hit_marker: 9,
            boss_protection: 8,
            horizontal_magnitude: 0x1234,
            direction: 2,
            spawn_identity: Some(5),
            position: Position {
                x_fraction: 4,
                y_fraction: 3,
                ..Default::default()
            },
            ..Default::default()
        };
    }
    let e = f.run();
    assert_eq!(e.children, [10, 11]);
    assert_eq!(f.actors[SUBJECT].local_counter_code(), 4);
    assert_eq!(f.actors[SUBJECT].local_aux2, 12);
    assert_eq!(f.actors[SUBJECT].local_state_code(), 25);
    for (s, x) in [(10, 0x48), (11, 0xa8)] {
        let a = f.actors[s];
        assert_eq!(
            (a.position.world_x, a.position.screen_y),
            (0x0400 | x, 0x38)
        );
        assert_eq!(
            (
                a.state_code(),
                a.render,
                a.animation,
                a.handler,
                a.profile,
                a.vertical_velocity
            ),
            (0xa0, 0xa0, 0x47, WAVE_CHILD, 0x10, 0x0180)
        );
        assert_eq!(
            (
                a.hit_marker,
                a.boss_protection,
                a.local_state_code(),
                a.local_counter_code(),
                a.horizontal_magnitude,
                a.direction,
                a.spawn_identity
            ),
            (9, 8, 7, 6, 0x1234, 2, Some(5))
        );
        assert_eq!((a.position.x_fraction, a.position.y_fraction), (4, 3));
    }
}
#[test]
fn both_full_wave_sequences_emit_exactly_six_children_in_reserved_slots() {
    for sequence in [0, 7] {
        let mut f = Fixture::new(WAVE);
        {
            let actor_value = SECOND;
            f.actors[SUBJECT].set_state_code(actor_value);
        };
        f.actors[SUBJECT].animation = WAVE_MOVING;
        {
            let actor_value = sequence;
            f.actors[SUBJECT].set_local_counter_code(actor_value);
        };
        f.actors[SUBJECT].local_aux2 = FIRST_CHILD as u8;
        let mut children = vec![];
        for _ in 0..49 {
            let e = f.run();
            children.extend(e.children);
        }
        assert_eq!(children, [8, 9, 10, 11, 12, 13]);
        assert_eq!(f.actors[SUBJECT].local_aux2, 14);
        assert_eq!(
            f.actors[SUBJECT].local_counter_code(),
            if sequence == 0 { 6 } else { 13 }
        );
    }
}
#[test]
fn wave_sentinel_waits_on_last_child_any_nonzero_state_not_protection_or_earlier_children() {
    for state in [1, 0x7f, ACTIVE, 0xff] {
        let mut f = Fixture::new(WAVE);
        {
            let actor_value = SECOND;
            f.actors[SUBJECT].set_state_code(actor_value);
        };
        f.actors[SUBJECT].animation = WAVE_MOVING;
        {
            let actor_value = 6;
            f.actors[SUBJECT].set_local_counter_code(actor_value);
        };
        f.actors[SUBJECT].local_aux2 = 14;
        {
            let actor_value = state;
            f.actors[13].set_state_code(actor_value);
        };
        f.actors[13].boss_protection = 0;
        f.run();
        assert_eq!(f.actors[SUBJECT].state_code(), SECOND);
        assert_eq!(f.actors[SUBJECT].local_counter_code(), 6);
    }
}
#[test]
fn wave_last_child_zero_resets_even_if_earlier_child_active_and_retains_camera_page() {
    let mut f = Fixture::new(WAVE);
    {
        let actor_value = SECOND;
        f.actors[SUBJECT].set_state_code(actor_value);
    };
    f.actors[SUBJECT].animation = WAVE_MOVING;
    {
        let actor_value = 6;
        f.actors[SUBJECT].set_local_counter_code(actor_value);
    };
    f.actors[SUBJECT].local_aux2 = 14;
    {
        let actor_value = 49;
        f.actors[SUBJECT].set_local_state_code(actor_value);
    };
    {
        let actor_value = ACTIVE;
        f.actors[8].set_state_code(actor_value);
    };
    {
        let actor_value = 0;
        f.actors[13].set_state_code(actor_value);
    };
    f.actors[13].boss_protection = 0xff;
    f.tick = 4;
    f.run();
    assert_eq!(f.actors[SUBJECT].state_code(), READY);
    assert_eq!(
        (
            f.actors[SUBJECT].local_state_code(),
            f.actors[SUBJECT].local_counter_code(),
            f.actors[SUBJECT].local_aux2
        ),
        (0, 0, 0)
    );
    assert_eq!(f.actors[SUBJECT].vertical_velocity, 0x06ee);
    assert_eq!(f.camera.x, 0x0400);
    assert_eq!(f.actors[8].state_code(), ACTIVE);
}
#[test]
fn wave_child_floor_clear_is_immediate_before_player_contact_and_miss_preserves_actor() {
    for floor in [false, true] {
        let mut f = Fixture::new(WAVE_CHILD);
        if floor {
            f.solid();
        }
        f.run();
        assert_eq!(
            f.world.calls,
            [motion(Movement::PositiveY, 0x2d, 0), Work::PlayerContact]
        );
        let seen = f.world.contact_seen[0];
        assert_eq!(
            (seen.state_code(), seen.render),
            if floor { (0, 0) } else { (READY, VISIBLE) }
        );
    }
}
#[test]
fn center_initialization_delays_and_faces_by_low_x_only_preserving_page_and_fractions() {
    let mut f = Fixture::new(CENTER);
    {
        let actor_value = ACTIVE;
        f.actors[SUBJECT].set_state_code(actor_value);
    };
    f.actors[SUBJECT].position.world_x = 0x09ff;
    f.actors[SUBJECT].position.x_fraction = 7;
    f.actors[SUBJECT].render = 0xff;
    f.actors[0].position.world_x = 0x0181;
    f.run();
    assert_eq!(f.actors[SUBJECT].position.world_x, 0x0980);
    assert_eq!(f.actors[SUBJECT].position.x_fraction, 7);
    assert_eq!(f.actors[SUBJECT].local_aux2, 14);
    assert_eq!(f.actors[SUBJECT].animation, CENTER_IDLE);
    assert_eq!(f.actors[SUBJECT].render, 0xff);
    assert_eq!(f.actors[SUBJECT].vertical_velocity, 0);
    assert_eq!(f.terrain.collision[IMPACT_Y_CELL], 0x44);
    assert!(f.world.calls.is_empty());
}
#[test]
fn center_delay_faces_both_low_byte_sides_and_expiry_still_returns_without_rng() {
    for (player_x, render) in [(0x0180, 0xbf), (0x0181, 0xff), (0x017f, 0xbf)] {
        let mut f = Fixture::new(CENTER);
        f.actors[SUBJECT].local_aux2 = 1;
        f.actors[SUBJECT].render = 0xff;
        f.actors[0].position.world_x = player_x;
        f.run();
        assert_eq!(f.actors[SUBJECT].local_aux2, 0);
        assert_eq!(f.actors[SUBJECT].render, render);
        assert_eq!(f.random.words, [0; 4]);
        assert!(f.world.calls.is_empty());
    }
}
#[test]
fn center_ground_random_selection_uses_incoming_increment_then_first_addition_carry() {
    for (words, increment, expected, direction) in [
        ([0, 0, 0, 0], false, [0, 0, 0, 0], 1),
        ([0xff, 0, 0xff, 0], true, [0xff, 0, 0xff, 0], 1),
        ([0xff, 1, 1, 0], false, [0xff, 2, 1, 0], 1),
    ] {
        let mut f = Fixture::new(CENTER);
        f.random.words = words;
        f.increment = increment;
        f.run();
        assert_eq!(f.random.words, expected);
        assert_eq!(f.actors[SUBJECT].direction, direction);
        assert_eq!(f.actors[SUBJECT].local_state_code(), 0);
        assert_eq!(
            f.actors[SUBJECT].horizontal_magnitude,
            f.catalog.center_speeds[0]
        );
        assert_eq!(f.actors[SUBJECT].animation, CENTER_MOVING);
        assert_eq!(f.actors[SUBJECT].state_code(), SECOND);
        assert!(matches!(
            f.world.calls.last(),
            Some(Work::Motion(Movement::PositiveX, 0x55, 2))
        ));
    }
}
#[test]
fn center_jump_selection_uses_second_sum_parity_and_rises_same_call() {
    let mut f = Fixture::new(CENTER);
    f.random.words = [0, 0, 1, 0];
    f.run();
    assert_eq!(f.random.words, [0, 1, 1, 0]);
    assert_eq!(f.actors[SUBJECT].local_state_code(), 2);
    assert_eq!(
        f.actors[SUBJECT].horizontal_magnitude,
        f.catalog.center_speeds[1]
    );
    assert_eq!(f.actors[SUBJECT].vertical_velocity, 0x06ee);
    assert_eq!(f.actors[SUBJECT].animation, CENTER_JUMP);
    assert_eq!(
        f.world.calls,
        [
            motion(Movement::Rise, 0x5e, 2),
            motion(Movement::PositiveX, 0x55, 2)
        ]
    );
}
#[test]
fn center_marked_survivor_zeros_unseeded_random_increment_and_still_selects_motion() {
    let mut f = Fixture::new(CENTER);
    f.actors[SUBJECT].hit_marker = 1;
    f.actors[SUBJECT].boss_protection = 2;
    f.increment = true;
    f.random.words = [0, 0, 0, 0];
    f.run();
    assert_eq!(f.random.words, [0; 4]);
    assert_eq!(f.actors[SUBJECT].animation, CENTER_MOVING);
    assert_eq!(f.actors[SUBJECT].boss_protection, 1);
    assert_eq!(f.hp, 5);
}
#[test]
fn center_rise_negative_transitions_to_odd_phase_then_horizontal_without_fall() {
    let mut f = Fixture::new(CENTER);
    {
        let actor_value = SECOND;
        f.actors[SUBJECT].set_state_code(actor_value);
    };
    {
        let actor_value = 2;
        f.actors[SUBJECT].set_local_state_code(actor_value);
    };
    f.actors[SUBJECT].direction = 2;
    f.world.post_velocity = Some(-1);
    f.run();
    assert_eq!(f.actors[SUBJECT].local_state_code(), 1);
    assert_eq!(f.actors[SUBJECT].vertical_velocity, 0);
    assert_eq!(
        f.world.calls,
        [
            motion(Movement::Rise, 0x5e, 2),
            motion(Movement::NegativeX, 0x58, 2)
        ]
    );
}
#[test]
fn center_first_landing_relaunches_once_reverses_and_returns_before_horizontal() {
    let mut f = Fixture::new(CENTER);
    {
        let actor_value = SECOND;
        f.actors[SUBJECT].set_state_code(actor_value);
    };
    {
        let actor_value = 1;
        f.actors[SUBJECT].set_local_state_code(actor_value);
    };
    f.actors[SUBJECT].direction = 1;
    f.solid();
    f.run();
    assert_eq!(
        (
            f.actors[SUBJECT].local_state_code(),
            f.actors[SUBJECT].local_counter_code(),
            f.actors[SUBJECT].direction
        ),
        (2, 1, 2)
    );
    assert_eq!(f.actors[SUBJECT].vertical_velocity, 0x0548);
    assert_eq!(f.world.calls, [motion(Movement::Fall, 0x5b, 2)]);
}
#[test]
fn center_second_landing_goes_idle_immediately_and_preserves_local_phase_for_next_selection() {
    let mut f = Fixture::new(CENTER);
    {
        let actor_value = SECOND;
        f.actors[SUBJECT].set_state_code(actor_value);
    };
    {
        let actor_value = 1;
        f.actors[SUBJECT].set_local_state_code(actor_value);
    };
    {
        let actor_value = 1;
        f.actors[SUBJECT].set_local_counter_code(actor_value);
    };
    f.actors[SUBJECT].position.world_x = 0x05c0;
    f.solid();
    f.run();
    assert_eq!(f.actors[SUBJECT].position.world_x, 0x0580);
    assert_eq!(
        (
            f.actors[SUBJECT].state_code(),
            f.actors[SUBJECT].local_counter_code(),
            f.actors[SUBJECT].local_state_code(),
            f.actors[SUBJECT].local_aux2
        ),
        (READY, 0, 1, IMPACT_TICKS)
    );
    assert_eq!(f.actors[SUBJECT].animation, CENTER_IDLE);
    assert_eq!(f.world.calls, [motion(Movement::Fall, 0x5b, 2)]);
}
#[test]
fn center_horizontal_wall_reverses_increments_counter_and_returns_without_center_reset() {
    let mut f = Fixture::new(CENTER);
    {
        let actor_value = SECOND;
        f.actors[SUBJECT].set_state_code(actor_value);
    };
    f.actors[SUBJECT].direction = 1;
    {
        let actor_value = 0xff;
        f.actors[SUBJECT].set_local_counter_code(actor_value);
    };
    f.solid();
    f.run();
    assert_eq!(f.actors[SUBJECT].direction, 2);
    assert_eq!(f.actors[SUBJECT].local_counter_code(), 0);
    assert_eq!(f.actors[SUBJECT].state_code(), SECOND);
    assert_eq!(f.world.calls, [motion(Movement::PositiveX, 0x55, 2)]);
}
#[test]
fn center_ground_return_requires_counter_and_strict_low_byte_four_after_movement() {
    for (x, counter, idle) in [
        (0x1283, 1, true),
        (0x127d, 1, true),
        (0x1284, 1, false),
        (0x127c, 1, false),
        (0x1280, 0, false),
    ] {
        let mut f = Fixture::new(CENTER);
        {
            let actor_value = SECOND;
            f.actors[SUBJECT].set_state_code(actor_value);
        };
        {
            let actor_value = counter;
            f.actors[SUBJECT].set_local_counter_code(actor_value);
        };
        f.world.post_x = Some(x);
        f.run();
        assert_eq!(
            f.actors[SUBJECT].state_code(),
            if idle { READY } else { SECOND }
        );
        if idle {
            assert_eq!(f.actors[SUBJECT].position.world_x, 0x1280);
            assert_eq!(f.actors[SUBJECT].local_aux2, IMPACT_TICKS);
            assert_eq!(f.actors[SUBJECT].local_counter_code(), 0);
        }
    }
}
#[test]
fn center_airborne_even_without_contact_does_not_idle_at_center() {
    let mut f = Fixture::new(CENTER);
    {
        let actor_value = SECOND;
        f.actors[SUBJECT].set_state_code(actor_value);
    };
    {
        let actor_value = 1;
        f.actors[SUBJECT].set_local_state_code(actor_value);
    };
    {
        let actor_value = 1;
        f.actors[SUBJECT].set_local_counter_code(actor_value);
    };
    f.run();
    assert_eq!(f.actors[SUBJECT].state_code(), SECOND);
    assert_eq!(f.actors[SUBJECT].local_aux2, 0);
    assert_eq!(
        f.world.calls,
        [
            motion(Movement::Fall, 0x5b, 2),
            motion(Movement::NegativeX, 0x58, 2)
        ]
    );
}
#[test]
fn original_spawn_routes_join_all_five_land_boss_templates_without_folklore_identity() {
    let b = rom();
    let c = crate::actors::SpawnCatalog::original(&b).unwrap();
    // Independent parallel main arrays and BE16 templates establish exact routes.
    for (stage, row, identity, x, y, template, handler, animation, profile) in [
        (0, 1, 11, 0x0428, 0xac, 80, IMPACT, 0x60, 0x9f),
        (1, 3, 27, 0x0528, 0xb0, 81, ATTACK, ATTACK_FIRE, 0x96),
        (2, 2, 55, 0x0530, 0x90, 82, PERIMETER, PERIMETER_IDLE, 0x95),
        (3, 2, 32, 0x04d8, 0xb0, 83, WAVE, WAVE_MOVING, 0x95),
        (4, 5, 159, 0x0430, 0xb4, 84, CENTER, CENTER_MOVING, 0x9e),
    ] {
        let level = c.source(crate::identifiers::StageId(stage)).unwrap();
        let record = level.rows[row]
            .iter()
            .find(|r| r.identity == identity)
            .unwrap();
        assert_eq!(
            (record.world_x, record.screen_y, record.template),
            (x, y, template)
        );
        let t = c.templates[template];
        assert_eq!(
            (t.handler, t.animation, t.profile),
            (handler, animation, profile)
        );
    }
    for (selector, label, cpu) in [
        (ATTACK, "Bank06_FivePhaseAttackActor", 0xa335),
        (PROJECTILE, "Bank06_UpdateAttackProjectile", 0xa5be),
        (IMPACT, "Bank06_JumpingImpactActor", 0xa5db),
        (PERIMETER, "Bank06_PerimeterPathActor", 0xa6d3),
        (WAVE, "Bank06_BouncingWaveActor", 0xa8d6),
        (WAVE_CHILD, "Bank06_UpdateWaveChild", 0xaac8),
        (CENTER, "Bank06_CenterReturningActor", 0xaae1),
    ] {
        let entry = &c.handlers[usize::from(selector)];
        assert_eq!(entry.owner, crate::actors::HandlerOwner::Bosses);
        assert_eq!(entry.label, label);
        const HANDLER_LOW: usize = 0x802c;
        const HANDLER_HIGH: usize = 0x806f;
        let i = usize::from(selector);
        assert_eq!(
            usize::from(bytes(&b, HANDLER_LOW, 67)[i])
                | (usize::from(bytes(&b, HANDLER_HIGH, 67)[i]) << 8),
            cpu
        );
    }
}
#[test]
fn preset_positive_fraction_carry_and_integer_wrap_do_not_advance_land_same_call() {
    let mut f = Fixture::new(ATTACK);
    {
        let actor_value = SECOND;
        f.actors[SUBJECT].set_state_code(actor_value);
    };
    f.actors[SUBJECT].local_aux2 = 1;
    {
        let actor_value = 9;
        f.actors[SUBJECT].set_local_state_code(actor_value);
    };
    f.actors[SUBJECT].position.screen_y = 0xff;
    f.actors[SUBJECT].position.y_fraction = 0xc0;
    f.actors[SUBJECT].vertical_velocity = 0x0180;
    f.run();
    assert_eq!(
        (
            f.actors[SUBJECT].position.screen_y,
            f.actors[SUBJECT].position.y_fraction
        ),
        (1, 0x40)
    );
    assert_eq!(f.world.calls.len(), 1);
}
#[test]
fn impact_no_delay_horizontal_collision_does_not_gain_delayed_phase_reversal() {
    let mut f = Fixture::new(IMPACT);
    f.actors[SUBJECT].direction = 1;
    f.world.facts.push_back(MotionFacts::default());
    f.solid();
    f.run();
    assert_eq!(f.actors[SUBJECT].direction, 1);
    assert_eq!(f.actors[SUBJECT].local_state_code(), 0);
    assert_eq!(f.world.calls.len(), 2);
}
#[test]
fn perimeter_moving_reveal_mismatch_freezes_all_contact_and_motion() {
    for (frame, tick) in [(0, 5), (1, 4), (1, 6)] {
        let mut f = Fixture::new(PERIMETER);
        {
            let actor_value = SECOND;
            f.actors[SUBJECT].set_state_code(actor_value);
        };
        f.actors[SUBJECT].animation = REVEAL;
        {
            let animation_value = frame;
            f.actors[SUBJECT].playback.set_frame_code(animation_value);
        };
        {
            let animation_value = tick;
            f.actors[SUBJECT].playback.set_tick_code(animation_value);
        };
        let prior = f.actors;
        f.run();
        assert_eq!(f.actors, prior);
        assert!(f.world.calls.is_empty());
    }
}
#[test]
fn perimeter_circuit_counter_wraps_without_forcing_immediate_opening_reset() {
    let mut f = Fixture::new(PERIMETER);
    {
        let actor_value = SECOND;
        f.actors[SUBJECT].set_state_code(actor_value);
    };
    f.actors[SUBJECT].animation = PERIMETER_MOVING;
    {
        let actor_value = 1;
        f.actors[SUBJECT].set_local_state_code(actor_value);
    };
    {
        let actor_value = 19;
        f.actors[SUBJECT].set_local_counter_code(actor_value);
    };
    f.actors[SUBJECT].local_aux2 = 0xff;
    f.solid();
    f.world.facts.push_back(MotionFacts::default());
    f.run();
    assert_eq!(f.actors[SUBJECT].local_aux2, 0);
    assert_eq!(f.actors[SUBJECT].local_counter_code(), 0);
    assert_eq!(f.actors[SUBJECT].state_code(), SECOND);
}

#[test]
fn perimeter_contact_gate_preserves_every_mask_and_both_axis_results() {
    // A82C..A839: zero requires neither contact; nonzero requires all selected
    // bits. Masks selecting nonexistent bits can never advance the path.
    for mask in 0..=u8::MAX {
        for contacts in 0..4u8 {
            let mut f = Fixture::new(PERIMETER);
            f.actors[SUBJECT].set_state_code(SECOND);
            f.actors[SUBJECT].animation = PERIMETER_MOVING;
            f.actors[SUBJECT].set_local_state_code(1);
            f.actors[SUBJECT].set_local_counter_code(0);
            f.catalog.perimeter[0].gate = ContactGate::from_source(mask);
            f.world.facts.push_back(MotionFacts {
                bits: (contacts >> 1) & 1,
                ..Default::default()
            });
            f.world.facts.push_back(MotionFacts {
                bits: contacts & 1,
                ..Default::default()
            });
            f.run();
            let advances = if mask == 0 {
                contacts == 0
            } else {
                contacts & mask == mask
            };
            assert_eq!(
                f.actors[SUBJECT].local_counter_code(),
                u8::from(advances),
                "mask {mask}, contacts {contacts}"
            );
        }
    }
}

#[test]
fn decoded_perimeter_records_preserve_used_source_bits_and_mirrored_heading() {
    // A889 reads only low direction bits and0C motion bits. Upper source bits
    // have no movement consumer and are intentionally absent from native steps.
    for path in 0..=u8::MAX {
        for mask in [0, 1, 2, 3, 4, 0xff] {
            let step = PerimeterStep::from_source(path, mask);
            assert_eq!(step.source_path_code(), path & 0x0f);
            assert_eq!(step.horizontal.mirrored().direction_code(), (path & 3) ^ 3);
            assert_eq!(step.motion_code(), path & 0x0c);
            assert_eq!(step.gate.source_code(), mask);
        }
    }
}

#[test]
fn decoded_wave_endpoint_ignores_time_and_spawn_retains_exact_byte_timestamp() {
    // EmitTimedWaveChildren checks X0 before its time-table read; spawn records use
    // equality against the byte tick (includingFF); no >= or duration inference.
    for tick in 0..=u8::MAX {
        assert_eq!(WaveStep::from_source(0, tick), WaveStep::AwaitLastChild);
        for x in [1, 0x80, 0xff] {
            assert_eq!(
                WaveStep::from_source(x, tick),
                WaveStep::Spawn { x, at_tick: tick }
            );
        }
    }
}

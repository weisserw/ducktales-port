//! Executable B06 native policies. Read adjacent trace.md for instruction/caller
//! derivations; table expectations use matching local ROM, never copied assets.
use super::*;
use crate::{
    actors::{ACTOR_SLOTS, ActorContact},
    collision::{DESCENDING_MOTION, FACE_POSITIVE, Position, SOLID_CONTACT, Snap},
    config,
    runtime::{Buttons, PortInput},
    status::HealthSegment,
};
use romatlas_port::assets::Rom;
const TEST_X: u16 = 0x01f8;
const TEST_Y: u8 = 0x68;
const TEST_FRACTION: u8 = 0x55;
const TEST_RENDER: u8 = 0xa3;
const TEST_TICK: u8 = 3;
const TEST_FRAME: u8 = 4;
const CLIMB_CLASS: u8 = 2;
const RISING_MOTION: u8 = 8;
const HELD_ACTOR: u8 = 0xa0;
const LAUNCH_ACTOR: u8 = 0xe0;
const NEXT_DOWN_Y: u8 = 0xe6;
// Source E8BF loads E8E7+2 = E8E9; the historical test name is retained.
const UP_SLOT_Y: u8 = 0x32;
const IMPACT_FRAME: u8 = 2;
const FINISH_SWING_FRAME: u8 = 6;
const ORDINARY_THRESHOLD: u16 = 0x042d;
const POGO_THRESHOLD: u16 = 0x05d2;
fn actor() -> ActorContact {
    ActorContact {
        position: Position {
            world_x: TEST_X,
            screen_y: TEST_Y,
            x_fraction: TEST_FRACTION,
            y_fraction: TEST_FRACTION,
        },
        state: HELD_ACTOR,
        render: TEST_RENDER,
        horizontal_magnitude: NORMAL_SPEED,
        animation_tick: TEST_TICK,
        animation_frame: TEST_FRAME,
        ..Default::default()
    }
}
fn player(action: Action) -> Player {
    Player {
        action,
        gravity: PLAYER_GRAVITY,
        animation_delay: ANIMATION_DELAY,
        jump_velocity: 0x0610,
        ..Default::default()
    }
}
fn input(held: Buttons, pressed: Buttons) -> PortInput {
    PortInput { held, pressed }
}
fn health() -> Health {
    Health {
        segments: std::array::from_fn(|i| HealthSegment {
            y: if i == 3 { 0xff } else { 0x20 },
            tile: 0,
            depleted: 0,
            x: 0x20,
        }),
        end: 0xff,
        alternating: 0,
    }
}
fn data() -> PlayerData {
    PlayerData {
        handlers: [
            Handler::Standing,
            Handler::Airborne,
            Handler::ForcedJump,
            Handler::Climbing,
            Handler::Swing,
            Handler::Protected,
            Handler::Damage,
            Handler::TerrainPose,
            Handler::FallOut,
            Handler::WaitA,
            Handler::Carried,
            Handler::Carried,
            Handler::ExternalLaunch,
            Handler::Death,
            Handler::Hold,
            Handler::Dialogue,
            Handler::ConvertTimer,
        ],
        facing: [0, FACE_POSITIVE],
        column_starts: [0x20, 0x1f],
        climb_offsets: [0, 0, 8, 8],
        rise_thresholds: [ORDINARY_THRESHOLD, POGO_THRESHOLD],
        vertical_samples: [9, 6],
        transition: [0, 0, 0xff, 1, 0xe6, 1, 0x32, 0xff],
    }
}
fn pool() -> ActorPool {
    let mut a = [ActorContact::default(); ACTOR_SLOTS];
    a[0] = actor();
    a
}
fn room() -> RoomContext {
    RoomContext {
        camera: 0x0100,
        camera_room: 5,
        terrain_room: 6,
        row: 1,
        rows: 3,
        columns: 4,
        special_spawn: 7,
        ..Default::default()
    }
}
fn rooms() -> Vec<u8> {
    vec![0; 32]
}
#[derive(Clone, Debug, Eq, PartialEq)]
enum Call {
    Gravity(u8),
    Y,
    X(Horizontal),
    Scan(bool),
    Probe(u8, u8),
    Snap(Snap),
}
/// This fixture is a B05 boundary: it records calls and injects independent
/// contact/coordinate facts. It implements no player rules or integration math.
#[derive(Default)]
struct World {
    calls: Vec<Call>,
    horizontal_entry_animations: Vec<u8>,
    returns: Vec<ReturnOperation>,
    scan: CollisionFacts,
    horizontal_actor: Option<usize>,
    horizontal_immediate: Option<Action>,
    probe: CollisionFacts,
    probe_sequence: std::collections::VecDeque<CollisionFacts>,
    y: Option<u8>,
    x: Option<u16>,
    velocity: Option<i16>,
    immediate: Option<Action>,
    queued: Option<Action>,
}
impl ContactWorld for World {
    fn returned(&mut self, operation: ReturnOperation) {
        self.returns.push(operation);
    }
    fn gravity(&mut self, a: &mut ActorContact, s: u8) {
        self.calls.push(Call::Gravity(s));
        if let Some(v) = self.velocity {
            a.vertical_velocity = v;
        }
    }
    fn integrate_y(&mut self, a: &mut ActorContact) {
        self.calls.push(Call::Y);
        if let Some(y) = self.y {
            a.position.screen_y = y;
        }
    }
    fn integrate_x(&mut self, a: &mut ActorContact, d: Horizontal) {
        self.calls.push(Call::X(d));
        if let Some(x) = self.x {
            a.position.world_x = x;
        }
    }
    fn scan(&mut self, actors: &mut ActorPool, p: &mut Player, v: bool) -> CollisionFacts {
        self.calls.push(Call::Scan(v));
        if let Some(s) = self.immediate {
            p.action = s;
        }
        let mut facts = self.scan;
        if !v {
            self.horizontal_entry_animations.push(actors[0].animation);
            facts.actor = self.horizontal_actor.or(facts.actor);
            if let Some(action) = self.horizontal_immediate {
                p.action = action;
            }
        }
        facts
    }
    fn probe(&mut self, actors: &mut ActorPool, p: &mut Player, s: u8, r: u8) -> CollisionFacts {
        self.calls.push(Call::Probe(s, r));
        let facts = self.probe_sequence.pop_front().unwrap_or(self.probe);
        actors[0].render |= facts.render;
        if let Some(action) = self.queued {
            p.queued = Some(action);
        }
        facts
    }
    fn snap(&mut self, _: &mut ActorContact, d: Snap) {
        self.calls.push(Call::Snap(d));
    }
}
#[test]
fn original_all_seventeen_targets_and_every_bounded_parameter_consumer_decode() {
    let bytes =
        std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../input/game.rom"))
            .expect("matching local ROM");
    Rom::from_bytes(bytes.clone(), config::ROM_SHA256).expect("validated identity");
    let d = PlayerData::original(&bytes).unwrap();
    assert_eq!(d, data());
}
macro_rules! dispatch_case {
    ($name:ident,$action:ident,$handler:ident) => {
        #[test]
        fn $name() {
            let p = player(Action::$action);
            assert_eq!(p.dispatch(&data()), Handler::$handler);
        }
    };
}
dispatch_case!(standing_selector_dispatches_standing, Standing, Standing);
dispatch_case!(airborne_selector_dispatches_airborne, Airborne, Airborne);
dispatch_case!(
    forced_selector_dispatches_held_mirror,
    ForcedJump,
    ForcedJump
);
dispatch_case!(climb_selector_dispatches_climb, Climbing, Climbing);
dispatch_case!(swing_selector_dispatches_impact_windows, Swing, Swing);
dispatch_case!(
    protected_selector_dispatches_hold_and_recoil,
    Protected,
    Protected
);
dispatch_case!(damage_selector_dispatches_initial_arc, Damage, Damage);
dispatch_case!(
    terrain_pose_selector_dispatches_provisional_pose,
    TerrainPose,
    TerrainPose
);
dispatch_case!(fall_selector_dispatches_life_loss, FallOut, FallOut);
dispatch_case!(wait_selector_dispatches_edge_release, WaitA, WaitA);
dispatch_case!(
    carried_locked_selector_uses_shared_target,
    CarriedLocked,
    Carried
);
dispatch_case!(
    carried_jumpable_selector_uses_shared_target,
    CarriedJumpable,
    Carried
);
dispatch_case!(
    launch_selector_dispatches_external_arc,
    ExternalLaunch,
    ExternalLaunch
);
dispatch_case!(death_selector_dispatches_two_phase_trajectory, Death, Death);
dispatch_case!(hold_selector_dispatches_tick_only, Hold, Hold);
dispatch_case!(
    dialogue_selector_delegates_b15_without_owning_text,
    Dialogue,
    Dialogue
);
dispatch_case!(
    timer_conversion_selector_delegates_b14,
    ConvertTimer,
    ConvertTimer
);
#[test]
fn protection_predecrements_only_positive_controls_and_always_installs_player_gravity() {
    for (before, after) in [(0, 0), (1, 0), (0x7f, 0x7e), (0x80, 0x80), (0xff, 0xff)] {
        let mut p = player(Action::Hold);
        p.gravity = 0;
        p.protection = before;
        let e = p.pre_dispatch(0, 2);
        assert_eq!((p.gravity, p.protection), (PLAYER_GRAVITY, after));
        assert!(e.stage_sounds.is_empty());
    }
}
#[test]
fn positive_special_timer_borrows_and_does_not_requeue_already_active_audio() {
    let mut p = player(Action::Hold);
    p.special_mode = 1;
    p.special_ticks = 0x0100;
    let e = p.pre_dispatch(SPECIAL_AUDIO, 2);
    assert_eq!(p.special_ticks, 0x00ff);
    assert_eq!(p.special_mode, 1);
    assert!(e.stage_sounds.is_empty());
}
#[test]
fn special_expiry_can_queue_special_then_stage_in_same_update() {
    let mut p = player(Action::Hold);
    p.special_mode = 1;
    p.special_ticks = 1;
    let e = p.pre_dispatch(3, 4);
    assert_eq!(e.stage_sounds, [SPECIAL_AUDIO, 4]);
    assert_eq!((p.special_mode, p.special_ticks), (0, 0));
}
#[test]
fn special_zero_duration_wraps_instead_of_expiring() {
    let mut p = player(Action::Hold);
    p.special_mode = 1;
    let e = p.pre_dispatch(0, 2);
    assert_eq!(p.special_ticks, u16::MAX);
    assert_eq!(p.special_mode, 1);
    assert_eq!(e.stage_sounds, [SPECIAL_AUDIO]);
}
#[test]
fn negative_and_zero_special_controls_leave_timer_and_audio_untouched() {
    for mode in [0, 0x80, 0xff] {
        let mut p = player(Action::Hold);
        p.special_mode = mode;
        p.special_ticks = 1;
        let e = p.pre_dispatch(0, 2);
        assert_eq!((p.special_mode, p.special_ticks), (mode, 1));
        assert!(e.stage_sounds.is_empty());
    }
}
#[test]
fn standing_after_immediate_vertical_reaction_skips_select_and_jump() {
    let mut p = player(Action::Damage);
    let mut a = actor();
    let e = p.standing_controls(&mut a, input(Buttons::NONE, Buttons::SELECT | Buttons::A));
    assert_eq!(p.action, Action::Damage);
    assert_eq!(e.work, [Work::HorizontalInput]);
    assert_eq!(a.animation_frame, TEST_FRAME);
}
#[test]
fn select_cost_precedes_ready_b_swing_and_b_precedes_a() {
    let mut p = player(Action::Standing);
    let mut a = actor();
    a.animation = STRIKE_READY_ANIMATION;
    p.momentum_direction = Buttons::LEFT.0;
    let e = p.standing_controls(
        &mut a,
        input(Buttons::LEFT, Buttons::SELECT | Buttons::B | Buttons::A),
    );
    assert_eq!(p.action, Action::Swing);
    assert_eq!(
        (a.animation, a.animation_frame, a.animation_tick),
        (SWING_ANIMATION, 0, 0)
    );
    assert_eq!(
        (p.momentum_direction, a.horizontal_magnitude),
        (0, NORMAL_SPEED)
    );
    assert_eq!(e.work, [Work::CostOverlay, Work::HorizontalInput]);
}
#[test]
fn jump_edge_copies_configured_velocity_and_enters_airborne_same_update() {
    let mut p = player(Action::Standing);
    let mut a = actor();
    let e = p.standing_controls(&mut a, input(Buttons::NONE, Buttons::A));
    assert_eq!(
        (p.action, a.vertical_velocity),
        (Action::Airborne, p.jump_velocity)
    );
    assert_eq!(e.work, [Work::VerticalRise]);
}
#[test]
fn held_a_without_edge_does_not_jump_and_up_only_is_idle() {
    let mut p = player(Action::Standing);
    let mut a = actor();
    a.animation = WALK_ANIMATION;
    let e = p.standing_controls(&mut a, input(Buttons::A | Buttons::UP, Buttons::NONE));
    assert_eq!(p.action, Action::Standing);
    assert_eq!(
        (
            a.animation,
            a.animation_frame,
            a.animation_tick,
            p.animation_delay
        ),
        (IDLE_ANIMATION, 0, 0, ANIMATION_DELAY)
    );
    assert_eq!(e.work, [Work::HorizontalInput]);
}
#[test]
fn standing_keeps_landing_pose_without_movement() {
    let mut p = player(Action::Standing);
    let mut a = actor();
    a.animation = LAND_ANIMATION;
    p.standing_controls(&mut a, input(Buttons::NONE, Buttons::NONE));
    assert_eq!(
        (a.animation, a.animation_tick, a.animation_frame),
        (LAND_ANIMATION, TEST_TICK, TEST_FRAME)
    );
}
#[test]
fn down_dominates_horizontal_and_restarts_crouch_only_on_selector_change() {
    let mut p = player(Action::Standing);
    let mut a = actor();
    let e = p.standing_controls(&mut a, input(Buttons::DOWN | Buttons::LEFT, Buttons::NONE));
    assert_eq!(
        (
            a.animation,
            a.animation_tick,
            a.animation_frame,
            a.animation_restart
        ),
        (CROUCH_ANIMATION, 0, RESTART_FRAME, 1)
    );
    assert!(e.work.is_empty());
    a.animation_tick = TEST_TICK;
    p.standing_controls(&mut a, input(Buttons::DOWN, Buttons::NONE));
    assert_eq!(a.animation_tick, TEST_TICK);
}
#[test]
fn blocked_walk_delay_selects_ready_pose_at_zero_and_captures_direction() {
    let mut p = player(Action::Standing);
    p.animation_delay = 1;
    let mut a = actor();
    p.finish_walk(&mut a, input(Buttons::LEFT, Buttons::NONE), SOLID_CONTACT);
    assert_eq!(
        (
            a.animation,
            a.animation_frame,
            a.animation_tick,
            p.pose_direction
        ),
        (STRIKE_READY_ANIMATION, 0, 0, Buttons::LEFT.0)
    );
}
#[test]
fn blocked_walk_zero_delay_wraps_and_ready_pose_does_not_decrement() {
    let mut p = player(Action::Standing);
    p.animation_delay = 0;
    let mut a = actor();
    p.finish_walk(&mut a, input(Buttons::RIGHT, Buttons::NONE), SOLID_CONTACT);
    assert_eq!(p.animation_delay, 0xff);
    assert_eq!(a.animation, WALK_ANIMATION);
    a.animation = STRIKE_READY_ANIMATION;
    p.finish_walk(&mut a, input(Buttons::RIGHT, Buttons::NONE), SOLID_CONTACT);
    assert_eq!(p.animation_delay, 0xff);
}
#[test]
fn free_walk_resets_delay_and_preserves_existing_walk_animation_progress() {
    let mut p = player(Action::Standing);
    p.animation_delay = 1;
    let mut a = actor();
    a.animation = WALK_ANIMATION;
    p.finish_walk(&mut a, input(Buttons::LEFT, Buttons::NONE), 0);
    assert_eq!(
        (p.animation_delay, a.animation_tick, a.animation_frame),
        (ANIMATION_DELAY, TEST_TICK, TEST_FRAME)
    );
}
#[test]
fn air_and_pogo_animations_bypass_momentum_and_reset_speed() {
    for animation in [AIR_ANIMATION, POGO_ANIMATION] {
        let mut p = player(Action::Airborne);
        p.momentum_control = 1;
        p.momentum_direction = Buttons::LEFT.0;
        let mut a = actor();
        a.animation = animation;
        a.horizontal_magnitude = MOMENTUM_CAP;
        let plan = p.horizontal_plan(&mut a, Buttons::RIGHT.0);
        assert_eq!(
            (a.horizontal_magnitude, p.momentum_direction),
            (NORMAL_SPEED, 0)
        );
        assert_eq!(plan.direction, Some(Horizontal::Positive));
    }
}
#[test]
fn momentum_first_direction_accelerates_and_release_without_direction_does_nothing() {
    let mut p = player(Action::Standing);
    p.momentum_control = 1;
    let mut a = actor();
    assert_eq!(p.horizontal_plan(&mut a, 0).direction, None);
    assert_eq!(a.horizontal_magnitude, NORMAL_SPEED);
    p.momentum_control = 1;
    let plan = p.horizontal_plan(&mut a, Buttons::RIGHT.0);
    assert_eq!(
        (
            p.momentum_control,
            p.momentum_direction,
            a.horizontal_magnitude
        ),
        (0, Buttons::RIGHT.0, NORMAL_SPEED + MOMENTUM_ACCELERATION)
    );
    assert_eq!(plan.direction, Some(Horizontal::Positive));
}
#[test]
fn momentum_cap_applies_only_to_exact_integer_two() {
    for (before, after) in [(0x0278, MOMENTUM_CAP), (0x02f8, 0x0308)] {
        let mut p = player(Action::Standing);
        p.momentum_direction = Buttons::RIGHT.0;
        let mut a = actor();
        a.horizontal_magnitude = before;
        p.horizontal_plan(&mut a, Buttons::RIGHT.0);
        assert_eq!(a.horizontal_magnitude, after);
    }
}
#[test]
fn momentum_release_floor_clears_direction_and_returns_without_motion() {
    let mut p = player(Action::Standing);
    p.momentum_direction = Buttons::RIGHT.0;
    let mut a = actor();
    a.horizontal_magnitude = NORMAL_SPEED;
    let plan = p.horizontal_plan(&mut a, 0);
    assert_eq!(
        (a.horizontal_magnitude, p.momentum_direction),
        (NORMAL_SPEED, 0)
    );
    assert_eq!(plan.direction, None);
}
#[test]
fn momentum_release_continues_in_remembered_direction_above_floor() {
    let mut p = player(Action::Standing);
    p.momentum_direction = Buttons::LEFT.0;
    let mut a = actor();
    a.horizontal_magnitude = 0x0200;
    let plan = p.horizontal_plan(&mut a, 0);
    assert_eq!(a.horizontal_magnitude, 0x01fa);
    assert_eq!(plan.direction, Some(Horizontal::Negative));
}
#[test]
fn momentum_reverse_decelerates_old_direction_but_updates_facing_from_new_input() {
    let mut p = player(Action::Standing);
    p.momentum_direction = Buttons::LEFT.0;
    let mut a = actor();
    a.horizontal_magnitude = MOMENTUM_CAP;
    let plan = p.horizontal_plan(&mut a, Buttons::RIGHT.0);
    assert_eq!(a.horizontal_magnitude, 0x0270);
    assert_eq!(p.momentum_direction, Buttons::LEFT.0);
    assert_eq!(plan.direction, Some(Horizontal::Negative));
    assert!(plan.update_facing_after);
}
#[test]
fn momentum_reverse_floor_does_not_capture_opposite_direction() {
    let mut p = player(Action::Standing);
    p.momentum_direction = Buttons::LEFT.0;
    let mut a = actor();
    let plan = p.horizontal_plan(&mut a, Buttons::RIGHT.0);
    assert_eq!(
        (a.horizontal_magnitude, p.momentum_direction),
        (NORMAL_SPEED, 0)
    );
    assert_eq!(plan.direction, None);
    assert!(plan.update_facing_after);
}
#[test]
fn external_latch_consumes_once_and_positive_bonus_is_independent_of_left_input() {
    let mut p = player(Action::Standing);
    p.external_motion = 2;
    let mut a = actor();
    let plan = p.horizontal_plan(&mut a, Buttons::LEFT.0);
    assert_eq!(p.external_motion, 0);
    assert!(plan.external_positive_step);
    assert_eq!(plan.direction, Some(Horizontal::Negative));
    assert!(
        !p.horizontal_plan(&mut a, Buttons::LEFT.0)
            .external_positive_step
    );
}
#[test]
fn external_latch_without_held_horizontal_enters_direct_positive_path() {
    let mut p = player(Action::Standing);
    p.external_motion = 1;
    let mut a = actor();
    let plan = p.horizontal_plan(&mut a, 0);
    assert_eq!(p.external_motion, 0);
    assert!(!plan.external_positive_step);
    assert_eq!(plan.direction, Some(Horizontal::Positive));
}
#[test]
fn horizontal_actor_penetration_caps_twelve_and_actual_direction_selects_terrain_side() {
    let mut p = player(Action::Standing);
    let mut a = pool();
    let mut w = World {
        scan: CollisionFacts {
            bits: SOLID_CONTACT,
            penetration: Some(20),
            ..Default::default()
        },
        x: Some(TEST_X + 2),
        ..Default::default()
    };
    let e = p.horizontal(&mut a, Buttons::RIGHT.0, &mut w);
    assert_eq!(a[0].position.world_x, TEST_X - 10);
    assert_eq!(a[0].position.x_fraction, TEST_FRACTION);
    assert_eq!(e.spawn, Some(1));
    assert_eq!(e.collision_bits, SOLID_CONTACT);
    assert!(w.calls.contains(&Call::Probe(0, 2)));
    assert_ne!(a[0].render & FACE_POSITIVE, 0);
}
#[test]
fn negative_horizontal_contact_adds_capped_push_across_world_page() {
    let mut p = player(Action::Standing);
    let mut a = pool();
    let mut w = World {
        scan: CollisionFacts {
            bits: 2,
            penetration: Some(20),
            ..Default::default()
        },
        x: Some(0x01ff),
        probe: CollisionFacts {
            bits: SOLID_CONTACT,
            ..Default::default()
        },
        ..Default::default()
    };
    let e = p.horizontal(&mut a, Buttons::LEFT.0, &mut w);
    assert_eq!(a[0].position.world_x, 0x020b);
    assert_eq!(e.spawn, Some(2));
    assert_eq!(e.collision_bits, 3);
    assert!(w.calls.contains(&Call::Probe(3, 2)));
    assert!(w.calls.contains(&Call::Snap(Snap::NegativeX)));
    assert_eq!(a[0].render & FACE_POSITIVE, 0);
}
#[test]
fn unchanged_horizontal_position_retains_selected_spawn_direction() {
    for (held, spawn, start) in [(Buttons::RIGHT, 1, 0), (Buttons::LEFT, 2, 3)] {
        let mut p = player(Action::Standing);
        let mut a = pool();
        let mut w = World::default();
        let e = p.horizontal(&mut a, held.0, &mut w);
        assert_eq!(e.spawn, Some(spawn));
        assert_eq!(w.calls.last(), Some(&Call::Probe(start, 2)));
    }
}
#[test]
fn horizontal_terrain_solid_snaps_positive_and_merges_actor_bits() {
    let mut p = player(Action::Standing);
    let mut a = pool();
    let mut w = World {
        scan: CollisionFacts {
            bits: 2,
            ..Default::default()
        },
        probe: CollisionFacts {
            bits: SOLID_CONTACT,
            ..Default::default()
        },
        ..Default::default()
    };
    let e = p.horizontal(&mut a, Buttons::RIGHT.0, &mut w);
    assert_eq!(e.collision_bits, 3);
    assert_eq!(w.calls.last(), Some(&Call::Snap(Snap::PositiveX)));
}
#[test]
fn rising_threshold_equality_continues_and_strictly_lower_without_a_cuts_jump() {
    for (action, threshold) in [
        (Action::Airborne, ORDINARY_THRESHOLD),
        (Action::ForcedJump, POGO_THRESHOLD),
    ] {
        let p = player(action);
        let mut a = actor();
        a.vertical_velocity = threshold as i16;
        assert_eq!(p.rise_policy(&a, 0, &data()), RisePolicy::Rise);
        a.vertical_velocity = (threshold - 1) as i16;
        assert_eq!(p.rise_policy(&a, 0, &data()), RisePolicy::StopThenFall);
        assert_eq!(p.rise_policy(&a, Buttons::A.0, &data()), RisePolicy::Rise);
    }
}
#[test]
fn rising_uses_integer_sign_and_higher_states_skip_release_cutoff() {
    let mut a = actor();
    a.vertical_velocity = -1;
    assert_eq!(
        player(Action::Airborne).rise_policy(&a, Buttons::A.0, &data()),
        RisePolicy::Fall
    );
    a.vertical_velocity = 1;
    for state in [
        Action::Protected,
        Action::Damage,
        Action::ExternalLaunch,
        Action::Death,
    ] {
        assert_eq!(player(state).rise_policy(&a, 0, &data()), RisePolicy::Rise);
    }
}
#[test]
fn falling_gravity_precedes_integration_and_contact_penetration_is_uncapped() {
    let mut p = player(Action::Standing);
    let mut a = pool();
    let mut w = World {
        y: Some(TEST_Y + 2),
        scan: CollisionFacts {
            bits: SOLID_CONTACT,
            penetration: Some(20),
            actor_bits: 2,
            ..Default::default()
        },
        ..Default::default()
    };
    let e = p.vertical(&mut a, Vertical::Falling, 0, &data(), &mut w);
    assert_eq!(
        &w.calls[..3],
        &[Call::Gravity(PLAYER_GRAVITY), Call::Y, Call::Scan(true)]
    );
    assert_eq!(a[0].position.screen_y, TEST_Y - 18);
    assert_eq!(a[0].vertical_velocity, 0);
    assert_eq!(a[0].position.y_fraction, TEST_FRACTION);
    assert_eq!(e.collision_bits, SOLID_CONTACT);
    assert_eq!(p.actor_contacts, 2);
}
#[test]
fn terminal_fall_integer_f4_skips_gravity_for_every_fraction() {
    for fraction in [0, 0x55, 0xff] {
        let mut p = player(Action::Airborne);
        let mut a = pool();
        a[0].vertical_velocity = (0xf400u16 | fraction) as i16;
        let mut w = World::default();
        p.vertical(&mut a, Vertical::Falling, 0, &data(), &mut w);
        assert_eq!(w.calls[0], Call::Y);
        assert!(!w.calls.iter().any(|c| matches!(c, Call::Gravity(_))));
    }
}
#[test]
fn unsupported_standing_becomes_airborne_but_existing_immediate_damage_is_retained() {
    for (initial, immediate, expected) in [
        (Action::Standing, None, Action::Airborne),
        (Action::Standing, Some(Action::Damage), Action::Damage),
    ] {
        let mut p = player(initial);
        let mut a = pool();
        let mut w = World {
            immediate,
            ..Default::default()
        };
        p.vertical(&mut a, Vertical::Falling, 0, &data(), &mut w);
        assert_eq!(p.action, expected);
        assert_eq!(a[0].animation, AIR_ANIMATION);
    }
}
#[test]
fn rising_actor_correction_adds_penetration_plus_one_then_gravity_after_probe() {
    let mut p = player(Action::Airborne);
    let mut a = pool();
    a[0].vertical_velocity = 0x0600;
    let mut w = World {
        scan: CollisionFacts {
            penetration: Some(4),
            bits: 2,
            ..Default::default()
        },
        y: Some(TEST_Y - 2),
        ..Default::default()
    };
    p.vertical(&mut a, Vertical::Rising, Buttons::A.0, &data(), &mut w);
    assert_eq!(a[0].position.screen_y, TEST_Y + 3);
    assert_eq!(
        &w.calls[..3],
        &[Call::Y, Call::Scan(true), Call::Probe(6, 2)]
    );
    assert_eq!(w.calls.last(), Some(&Call::Gravity(PLAYER_GRAVITY)));
}
#[test]
fn rising_terrain_solid_stops_then_runs_fall_in_same_call_and_clears_fraction() {
    let mut p = player(Action::Airborne);
    let mut a = pool();
    a[0].vertical_velocity = 0x0600;
    let mut w = World {
        probe: CollisionFacts {
            bits: SOLID_CONTACT,
            ..Default::default()
        },
        ..Default::default()
    };
    p.vertical(&mut a, Vertical::Rising, Buttons::A.0, &data(), &mut w);
    assert_eq!(a[0].position.y_fraction, 0);
    assert_eq!(a[0].motion_flags, DESCENDING_MOTION);
    assert_eq!(w.calls.iter().filter(|c| **c == Call::Y).count(), 2);
    assert!(w.calls.contains(&Call::Snap(Snap::NegativeY)));
    assert!(w.calls.contains(&Call::Snap(Snap::PositiveY)));
}
#[test]
fn release_cutoff_clears_y_fraction_before_falling_integration() {
    let mut p = player(Action::Airborne);
    let mut a = pool();
    a[0].vertical_velocity = 1;
    let mut w = World::default();
    p.vertical(&mut a, Vertical::Rising, 0, &data(), &mut w);
    assert_eq!(a[0].position.y_fraction, 0);
    assert_eq!(w.calls[0], Call::Gravity(PLAYER_GRAVITY));
    assert_eq!(a[0].motion_flags, DESCENDING_MOTION);
}
#[test]
fn vertical_probe_uses_actual_byte_direction_and_restores_render_while_oring_contacts() {
    for (new_y, start, snap) in [
        (TEST_Y - 1, 9, Snap::NegativeY),
        (TEST_Y + 1, 6, Snap::PositiveY),
    ] {
        let mut p = player(Action::Airborne);
        let mut a = pool();
        let mut w = World {
            y: Some(new_y),
            scan: CollisionFacts {
                bits: 2,
                actor_bits: 2,
                ..Default::default()
            },
            probe: CollisionFacts {
                bits: SOLID_CONTACT,
                actor_bits: SOLID_CONTACT,
                render: 4,
                ..Default::default()
            },
            ..Default::default()
        };
        let e = p.vertical(&mut a, Vertical::Falling, 0, &data(), &mut w);
        assert!(w.calls.contains(&Call::Probe(start, 2)));
        assert!(w.calls.contains(&Call::Snap(snap)));
        assert_eq!(
            (e.collision_bits, p.actor_contacts, a[0].render),
            (3, 3, TEST_RENDER)
        );
    }
}
#[test]
fn equal_y_vertical_probe_keeps_requested_motion_direction() {
    for (direction, velocity, start) in [(Vertical::Falling, -1, 6), (Vertical::Rising, 0x0600, 9)]
    {
        let mut p = player(Action::Airborne);
        let mut a = pool();
        a[0].vertical_velocity = velocity;
        let mut w = World::default();
        p.vertical(&mut a, direction, Buttons::A.0, &data(), &mut w);
        assert!(w.calls.contains(&Call::Probe(start, 2)));
    }
}
#[test]
fn pogo_state_overrides_swing_guard_but_swing_state_preserves_animation_under_down_b() {
    let mut a = actor();
    player(Action::ForcedJump).airborne_animation(&mut a, 0);
    assert_eq!(
        (a.animation, a.animation_tick, a.animation_frame),
        (POGO_ANIMATION, FROZEN_TICK, 0)
    );
    a = actor();
    player(Action::Swing).airborne_animation(&mut a, Buttons::DOWN.0 | Buttons::B.0);
    assert_eq!(
        (a.animation, a.animation_tick, a.animation_frame),
        (0, TEST_TICK, TEST_FRAME)
    );
}
#[test]
fn already_pogo_retains_progress_and_ready_pose_requires_exact_direction() {
    let mut p = player(Action::Airborne);
    let mut a = actor();
    a.animation = POGO_ANIMATION;
    p.airborne_animation(&mut a, Buttons::DOWN.0 | Buttons::B.0);
    assert_eq!(a.animation_tick, TEST_TICK);
    a.animation = STRIKE_READY_ANIMATION;
    p.pose_direction = Buttons::LEFT.0;
    p.airborne_animation(&mut a, Buttons::LEFT.0);
    assert_eq!(a.animation, STRIKE_READY_ANIMATION);
    p.airborne_animation(&mut a, Buttons::RIGHT.0);
    assert_eq!(
        (a.animation, a.animation_tick, a.animation_frame),
        (AIR_ANIMATION, 0, 0)
    );
}
#[test]
fn ordinary_air_animation_resets_progress_even_if_selector_already_two() {
    let p = player(Action::Airborne);
    let mut a = actor();
    a.animation = AIR_ANIMATION;
    p.airborne_animation(&mut a, 0);
    assert_eq!((a.animation_frame, a.animation_tick), (0, 0));
}
#[test]
fn airborne_ready_b_edge_enters_swing_and_returns_before_landing_or_horizontal() {
    let mut p = player(Action::Airborne);
    let mut a = actor();
    a.animation = STRIKE_READY_ANIMATION;
    a.motion_flags = DESCENDING_MOTION;
    let e = p.landing(
        &mut a,
        input(Buttons::NONE, Buttons::B),
        CollisionFacts {
            bits: SOLID_CONTACT,
            ..Default::default()
        },
    );
    assert_eq!((p.action, a.animation), (Action::Swing, SWING_ANIMATION));
    assert_eq!((a.animation_frame, a.animation_tick), (0, 0));
    assert!(e.work.is_empty());
}
#[test]
fn landing_requires_both_descending_and_final_solid_bit() {
    for (motion, bits) in [(RISING_MOTION, SOLID_CONTACT), (DESCENDING_MOTION, 0)] {
        let mut p = player(Action::Airborne);
        let mut a = actor();
        a.motion_flags = motion;
        let e = p.landing(
            &mut a,
            input(Buttons::NONE, Buttons::NONE),
            CollisionFacts {
                bits,
                ..Default::default()
            },
        );
        assert_eq!(p.action, Action::Airborne);
        assert!(e.sounds.is_empty());
        assert_eq!(e.work, [Work::HorizontalInput]);
    }
}
#[test]
fn ordinary_landing_sound_animation_restart_and_action_are_exact() {
    let mut p = player(Action::Airborne);
    let mut a = actor();
    a.motion_flags = DESCENDING_MOTION;
    let e = p.landing(
        &mut a,
        input(Buttons::NONE, Buttons::NONE),
        CollisionFacts {
            bits: SOLID_CONTACT,
            ..Default::default()
        },
    );
    assert_eq!(p.action, Action::Standing);
    assert_eq!(
        (
            a.animation,
            a.animation_frame,
            a.animation_tick,
            a.animation_restart
        ),
        (LAND_ANIMATION, RESTART_FRAME, 0, 1)
    );
    assert_eq!(e.sounds, [LAND_SOUND]);
}
#[test]
fn pogo_requires_actor_contact_held_b_and_down_unless_already_forced() {
    for (action, held, contacts, want) in [
        (Action::Airborne, Buttons::DOWN | Buttons::B, 1, true),
        (Action::ForcedJump, Buttons::B, 1, true),
        (Action::Airborne, Buttons::B, 1, false),
        (Action::Airborne, Buttons::DOWN | Buttons::B, 0, false),
        (Action::ForcedJump, Buttons::DOWN, 1, false),
    ] {
        let mut p = player(action);
        let mut a = actor();
        a.motion_flags = DESCENDING_MOTION;
        let e = p.landing(
            &mut a,
            input(held, Buttons::NONE),
            CollisionFacts {
                bits: SOLID_CONTACT,
                actor_bits: contacts,
                ..Default::default()
            },
        );
        assert_eq!(
            p.action,
            if want {
                Action::ForcedJump
            } else {
                Action::Standing
            }
        );
        if want {
            assert_eq!(
                (
                    a.vertical_velocity,
                    a.animation,
                    a.animation_frame,
                    a.animation_restart
                ),
                (POGO_SPEED, POGO_ANIMATION, POGO_START_FRAME, 2)
            );
            assert_eq!(e.sounds, [POGO_SOUND]);
        }
    }
}
#[test]
fn optional_pogo_height_gate_is_strict_below_a0() {
    for (control, y, want) in [
        (1, POGO_HEIGHT_LIMIT - 1, true),
        (1, POGO_HEIGHT_LIMIT, false),
        (0, POGO_HEIGHT_LIMIT, true),
    ] {
        let mut p = player(Action::ForcedJump);
        p.pogo_height_control = control;
        let mut a = actor();
        a.motion_flags = DESCENDING_MOTION;
        a.position.screen_y = y;
        p.landing(
            &mut a,
            input(Buttons::B, Buttons::NONE),
            CollisionFacts {
                bits: SOLID_CONTACT,
                actor_bits: 1,
                ..Default::default()
            },
        );
        assert_eq!(
            p.action,
            if want {
                Action::ForcedJump
            } else {
                Action::Standing
            }
        );
    }
}
#[test]
fn swing_or_ready_landing_clears_action_without_landing_sound_or_animation() {
    for (state, animation) in [
        (Action::Swing, SWING_ANIMATION),
        (Action::Airborne, STRIKE_READY_ANIMATION),
    ] {
        let mut p = player(state);
        let mut a = actor();
        a.animation = animation;
        a.motion_flags = DESCENDING_MOTION;
        let e = p.landing(
            &mut a,
            input(Buttons::NONE, Buttons::NONE),
            CollisionFacts {
                bits: SOLID_CONTACT,
                ..Default::default()
            },
        );
        assert_eq!(p.action, Action::Standing);
        assert_eq!(a.animation, animation);
        assert!(e.sounds.is_empty());
    }
}
#[test]
fn climb_alignment_descending_sample_priority_and_right_rounds_next_cell() {
    let mut p = player(Action::Airborne);
    let mut a = actor();
    a.position.world_x = 0x01f8;
    let d = p.climb_alignment(&mut a, [2; 4], 0, &data());
    assert_eq!(
        (d, a.position.world_x, p.action),
        (Some(Horizontal::Positive), 0x0200, Action::Climbing)
    );
    assert_eq!(
        (a.animation, a.animation_frame, a.animation_tick),
        (CLIMB_ANIMATION, 0, 0)
    );
    assert_eq!(a.position.screen_y, TEST_Y);
}
#[test]
fn climb_alignment_left_rounding_and_unequal_even_sample_snap_y_midpoint() {
    let mut p = player(Action::Airborne);
    let mut a = actor();
    a.position.world_x = 0x0123;
    a.position.screen_y = 0x65;
    let result = p.climb_alignment(&mut a, [2, 1, 0, 0], 0, &data());
    assert_eq!(result, Some(Horizontal::Negative));
    assert_eq!((a.position.world_x, a.position.screen_y), (0x0120, 0x68));
}
#[test]
fn climb_alignment_odd_sample_unequal_pair_wraps_y_up_one_cell() {
    let mut p = player(Action::Airborne);
    let mut a = actor();
    a.position.world_x = 0x0123;
    a.position.screen_y = 0x05;
    let result = p.climb_alignment(&mut a, [0, 2, 0, 0], 0, &data());
    assert_eq!(result, Some(Horizontal::Negative));
    assert_eq!(a.position.screen_y, 0xf8);
}
#[test]
fn climb_alignment_right_pair_reads_results_two_three_not_contact_boolean() {
    let mut p = player(Action::Airborne);
    let mut a = actor();
    a.position.world_x = 0x0128;
    a.position.screen_y = 0x65;
    let result = p.climb_alignment(&mut a, [0, 0, 2, 0], 0, &data());
    assert_eq!(result, Some(Horizontal::Positive));
    assert_eq!((a.position.world_x, a.position.screen_y), (0x0130, 0x68));
}
#[test]
fn climb_alignment_rejects_solid_and_nibble_eight_boundary() {
    for (bits, x) in [(SOLID_CONTACT, 0x0120), (0, 0x0128)] {
        let mut p = player(Action::Airborne);
        let mut a = actor();
        a.position.world_x = x;
        let before = a;
        assert_eq!(p.climb_alignment(&mut a, [2, 2, 0, 0], bits, &data()), None);
        assert_eq!(a, before);
    }
}
#[test]
fn forced_jump_mirrors_held_b_into_a_without_changing_pressed_or_other_port() {
    let mut p = player(Action::ForcedJump);
    let mut c = Controllers::default();
    c.ports[0] = input(Buttons::B | Buttons::LEFT, Buttons::SELECT);
    c.ports[1] = input(Buttons::UP, Buttons::START);
    let before = c.ports[1];
    let e = p.forced_jump(&mut c);
    assert_eq!(c.ports[0].held, Buttons::A | Buttons::B | Buttons::LEFT);
    assert_eq!(c.ports[0].pressed, Buttons::SELECT);
    assert_eq!(c.ports[1], before);
    assert_eq!(p.action, Action::ForcedJump);
    assert_eq!(e.work, [Work::VerticalRise]);
}
#[test]
fn forced_jump_release_changes_action_one_before_shared_airborne() {
    let mut p = player(Action::ForcedJump);
    let mut c = Controllers::default();
    p.forced_jump(&mut c);
    assert_eq!(p.action, Action::Airborne);
    assert_eq!(p.animation_delay, ANIMATION_DELAY);
}
#[test]
fn climbing_idle_freezes_tick_and_pressed_a_leaves_with_zero_velocity() {
    let mut p = player(Action::Climbing);
    let mut a = actor();
    a.vertical_velocity = 0x0123;
    p.climbing(&mut a, input(Buttons::NONE, Buttons::NONE), CLIMB_CLASS);
    assert_eq!(a.animation_tick, TEST_TICK | FROZEN_TICK);
    assert_eq!(p.action, Action::Climbing);
    p.climbing(&mut a, input(Buttons::A, Buttons::A), CLIMB_CLASS);
    assert_eq!((p.action, a.vertical_velocity), (Action::Standing, 0));
}
#[test]
fn climb_direction_copies_horizontal_magnitude_and_sound_only_unfrozen_tick_zero() {
    for (tick, sounds) in [
        (FROZEN_TICK, vec![CLIMB_SOUND]),
        (FROZEN_TICK | TEST_TICK, vec![]),
    ] {
        let mut p = player(Action::Climbing);
        let mut a = actor();
        a.animation_tick = tick;
        let e = p.climbing(&mut a, input(Buttons::UP, Buttons::NONE), CLIMB_CLASS);
        assert_eq!(a.vertical_velocity, NORMAL_SPEED as i16);
        assert_eq!(a.animation_tick, tick & !FROZEN_TICK);
        assert_eq!(e.sounds, sounds);
        assert_eq!(
            e.work,
            [Work::Terrain {
                start: 0x14,
                remaining: 3
            }]
        );
    }
}
#[test]
fn climb_down_and_conflicting_game_written_direction_use_lower_samples() {
    for held in [Buttons::DOWN, Buttons::DOWN | Buttons::UP] {
        let mut p = player(Action::Climbing);
        let mut a = actor();
        let e = p.climbing(&mut a, input(held, Buttons::A), CLIMB_CLASS);
        assert_eq!(p.action, Action::Climbing);
        assert_eq!(
            e.work,
            [Work::Terrain {
                start: 0x10,
                remaining: 3
            }]
        );
    }
}
#[test]
fn climb_wrong_final_class_leaves_and_clears_velocity_before_boundary() {
    let mut p = player(Action::Climbing);
    let mut a = actor();
    a.position.screen_y = BOTTOM_ROOM_Y;
    let e = p.climbing(&mut a, input(Buttons::DOWN, Buttons::NONE), 1);
    assert_eq!((p.action, a.vertical_velocity), (Action::Standing, 0));
    assert!(!e.work.contains(&Work::VerticalTransition));
}
#[test]
fn climb_transition_thresholds_are_upper_strict_and_lower_inclusive() {
    for (held, y, want) in [
        (Buttons::UP, TOP_CLIMB_Y - 1, true),
        (Buttons::UP, TOP_CLIMB_Y, false),
        (Buttons::DOWN, BOTTOM_ROOM_Y - 1, false),
        (Buttons::DOWN, BOTTOM_ROOM_Y, true),
    ] {
        let mut p = player(Action::Climbing);
        let mut a = actor();
        a.position.screen_y = y;
        let e = p.climbing(&mut a, input(held, Buttons::NONE), CLIMB_CLASS);
        assert_eq!(e.work.contains(&Work::VerticalTransition), want);
    }
}
#[test]
fn late_route_bottom_message_precedes_camera_legality() {
    let mut p = player(Action::Climbing);
    let mut a = pool();
    a[0].position.screen_y = LATE_ROUTE_Y;
    let mut r = room();
    r.late_route = 1;
    r.camera = 0x0023;
    let (out, e) = p.room_transition(&mut a, Buttons::DOWN.0, &mut r, &rooms(), &data());
    assert_eq!(
        (out, p.action, e.message),
        (
            RoomResult::Message,
            Action::Dialogue,
            Some(LATE_ROUTE_MESSAGE)
        )
    );
    assert!(!e.transition);
}
#[test]
fn late_route_below_c0_still_requires_camera_low_zero_and_high_odd() {
    for camera in [0x0101, 0x0000, 0x0200] {
        let mut p = player(Action::Climbing);
        let mut a = pool();
        let mut r = room();
        r.late_route = 1;
        r.camera = camera;
        let before = r;
        assert_eq!(
            p.room_transition(&mut a, Buttons::UP.0, &mut r, &rooms(), &data())
                .0,
            RoomResult::Rejected
        );
        assert_eq!(r, before);
    }
}
#[test]
fn downward_transition_commits_row_both_cursors_and_player_y_e6() {
    let mut p = player(Action::Climbing);
    let mut a = pool();
    let mut r = room();
    let (out, e) = p.room_transition(&mut a, Buttons::DOWN.0, &mut r, &rooms(), &data());
    assert_eq!(out, RoomResult::Committed);
    assert_eq!((r.row, r.camera_room, r.terrain_room), (2, 9, 10));
    assert_eq!(
        (a[0].position.screen_y, a[2].position.screen_y),
        (NEXT_DOWN_Y, 0)
    );
    assert_eq!(a[0].render, TEST_RENDER & !3);
    assert_eq!(r.special_spawn, 0);
    assert!(e.transition);
    assert_eq!(e.work, [Work::VerticalTransition]);
}
#[test]
fn upward_transition_writes_slot_two_y_one_and_preserves_player_y() {
    let mut p = player(Action::Climbing);
    let mut a = pool();
    a[2].position.screen_y = 0x77;
    let mut r = room();
    let (out, e) = p.room_transition(&mut a, Buttons::UP.0, &mut r, &rooms(), &data());
    assert_eq!(out, RoomResult::Committed);
    assert_eq!((r.row, r.camera_room, r.terrain_room), (0, 1, 2));
    assert_eq!(
        (a[0].position.screen_y, a[2].position.screen_y),
        (TEST_Y, UP_SLOT_Y)
    );
    assert!(e.transition);
}
#[test]
fn missing_room_and_row_bounds_reject_without_partial_cursor_commit() {
    for (row, held, missing) in [
        (0, Buttons::UP, false),
        (2, Buttons::DOWN, false),
        (1, Buttons::DOWN, true),
    ] {
        let mut p = player(Action::Climbing);
        let mut a = pool();
        let mut r = room();
        r.row = row;
        let before = r;
        let mut entries = rooms();
        if missing {
            entries[9] = 0xff;
        }
        assert_eq!(
            p.room_transition(&mut a, held.0, &mut r, &entries, &data())
                .0,
            RoomResult::Rejected
        );
        assert_eq!(r, before);
        assert_eq!(a[0].position.screen_y, TEST_Y);
    }
}
#[test]
fn room_stride_and_cursor_arithmetic_wrap_as_bytes_including_zero_width_adc_carry() {
    let mut p = player(Action::Climbing);
    let mut a = pool();
    let mut r = room();
    r.columns = 0;
    r.camera_room = 5;
    r.terrain_room = 6;
    let (out, _) = p.room_transition(&mut a, Buttons::UP.0, &mut r, &rooms(), &data());
    assert_eq!(out, RoomResult::Committed);
    assert_eq!((r.camera_room, r.terrain_room), (6, 6));
}
#[test]
fn airborne_downward_boundary_uses_port_two_up_gate_and_final_row() {
    for (port2, row, want) in [
        (Buttons::UP, 1, false),
        (Buttons::NONE, 2, false),
        (Buttons::NONE, 1, true),
    ] {
        let mut p = player(Action::Airborne);
        let mut a = pool();
        a[0].motion_flags = DESCENDING_MOTION;
        a[0].position.screen_y = BOTTOM_ROOM_Y;
        let mut c = Controllers::default();
        c.ports[1].held = port2;
        let mut r = room();
        r.row = row;
        let e = p.airborne_boundaries(&mut a, &mut c, &mut r, &rooms(), &data());
        assert_eq!(e.transition, want);
        assert_eq!(e.work.first(), Some(&Work::UpwardProbe));
        if want {
            assert_eq!(c.ports[0].held, Buttons::DOWN);
        }
    }
}
#[test]
fn airborne_descend_rounds_camera_at_eighty_and_selects_column_from_previous_scroll() {
    for (low, high, previous, column, want_high) in [(0x7f, 1, 0, 0x20, 1), (0x80, 0, 1, 0x1f, 1)] {
        let mut p = player(Action::Airborne);
        let mut a = pool();
        a[0].motion_flags = DESCENDING_MOTION;
        a[0].position.screen_y = BOTTOM_ROOM_Y;
        let mut c = Controllers::default();
        c.ports[0].pressed = Buttons::B;
        let mut r = room();
        r.camera = ((high as u16) << 8) | low;
        r.previous_scroll = previous;
        let e = p.airborne_boundaries(&mut a, &mut c, &mut r, &rooms(), &data());
        assert!(e.transition);
        assert_eq!(
            (r.camera, r.terrain_column),
            ((want_high as u16) << 8, column)
        );
        assert_eq!(c.ports[0].pressed, Buttons::B);
    }
}
#[test]
fn airborne_descend_requires_motion_bit_and_y_e0_inclusive() {
    for (motion, y) in [
        (RISING_MOTION, BOTTOM_ROOM_Y),
        (DESCENDING_MOTION, BOTTOM_ROOM_Y - 1),
    ] {
        let mut p = player(Action::Airborne);
        let mut a = pool();
        a[0].motion_flags = motion;
        a[0].position.screen_y = y;
        let mut c = Controllers::default();
        let mut r = room();
        let e = p.airborne_boundaries(&mut a, &mut c, &mut r, &rooms(), &data());
        assert!(!e.transition);
        assert_eq!(r.row, 1);
    }
}
#[test]
fn swing_impact_only_tick_zero_frame_two_and_all_three_exempt_handlers() {
    for handler in [0x08, 0x15, 0x42] {
        let mut p = player(Action::Swing);
        let mut a = actor();
        a.animation = SWING_ANIMATION;
        a.animation_frame = IMPACT_FRAME;
        a.animation_tick = 0;
        let e = p.swing(&mut a, 0, Some(handler));
        assert_eq!(a.animation, SWING_ANIMATION);
        assert_eq!(e.sounds, [SWING_SOUND]);
        assert_eq!(p.action, Action::Swing);
    }
}
#[test]
fn swing_no_contact_and_nonexempt_contact_both_enter_blocked_pose() {
    for contact in [None, Some(0x09)] {
        let mut p = player(Action::Swing);
        let mut a = actor();
        a.animation = SWING_ANIMATION;
        a.animation_frame = IMPACT_FRAME;
        a.animation_tick = 0;
        let e = p.swing(&mut a, 0, contact);
        assert_eq!(
            (
                a.animation,
                a.animation_frame,
                a.animation_tick,
                a.local_state
            ),
            (BLOCKED_SWING_ANIMATION, 0, 0, BLOCKED_CYCLES)
        );
        assert_eq!(e.sounds, [SWING_SOUND, BLOCKED_SOUND]);
    }
}
#[test]
fn swing_nonimpact_frames_or_nonzero_ticks_do_not_scan_or_emit_sound() {
    for (frame, tick) in [
        (IMPACT_FRAME, 1),
        (IMPACT_FRAME - 1, 0),
        (IMPACT_FRAME + 1, 0),
    ] {
        let mut p = player(Action::Swing);
        let mut a = actor();
        a.animation = SWING_ANIMATION;
        a.animation_frame = frame;
        a.animation_tick = tick;
        let e = p.swing(&mut a, 0, None);
        assert!(e.sounds.is_empty());
        assert!(!e.work.contains(&Work::ActorScan));
    }
}
#[test]
fn swing_frame_six_directional_match_restores_ready_pose_otherwise_resets_delay() {
    for (held, animation, delay) in [
        (Buttons::LEFT, STRIKE_READY_ANIMATION, 1),
        (Buttons::RIGHT, SWING_ANIMATION, ANIMATION_DELAY),
    ] {
        let mut p = player(Action::Swing);
        p.pose_direction = Buttons::LEFT.0;
        p.animation_delay = 1;
        let mut a = actor();
        a.animation = SWING_ANIMATION;
        a.animation_frame = FINISH_SWING_FRAME;
        let e = p.swing(&mut a, held.0, None);
        assert_eq!(p.action, Action::Standing);
        assert_eq!((a.animation, p.animation_delay), (animation, delay));
        assert!(e.sounds.is_empty());
    }
}
#[test]
fn blocked_swing_counts_only_cycle_boundary_and_finishes_at_fifth_decrement() {
    let mut p = player(Action::Swing);
    let mut a = actor();
    a.animation = BLOCKED_SWING_ANIMATION;
    a.local_state = 1;
    a.animation_frame = 0;
    a.animation_tick = 1;
    p.swing(&mut a, 0, None);
    assert_eq!(a.local_state, 1);
    a.animation_tick = 0;
    p.swing(&mut a, 0, None);
    assert_eq!(
        (a.local_state, p.action, p.animation_delay),
        (0, Action::Standing, ANIMATION_DELAY)
    );
}
#[test]
fn blocked_swing_zero_counter_wraps_instead_of_finishing() {
    let mut p = player(Action::Swing);
    let mut a = actor();
    a.animation = BLOCKED_SWING_ANIMATION;
    a.local_state = 0;
    a.animation_tick = 0;
    a.animation_frame = 0;
    p.swing(&mut a, 0, None);
    assert_eq!((a.local_state, p.action), (0xff, Action::Swing));
}
#[test]
fn protected_motion_holds_at_eight_or_above_including_negative_controls() {
    for ticks in [RECOIL_THRESHOLD, 0x7f, 0x80, 0xff] {
        let mut p = player(Action::Protected);
        p.protection = ticks;
        let mut a = actor();
        let before = a;
        let e = p.protected(&mut a, &mut health(), TitleRule::OrdinaryDamage);
        assert_eq!(a, before);
        assert!(e.work.is_empty());
        assert_eq!(p.action, Action::Protected);
    }
}
#[test]
fn protected_near_expiry_uses_opposite_facing_recoil_then_restores_facing() {
    for render in [TEST_RENDER, TEST_RENDER | FACE_POSITIVE] {
        let mut p = player(Action::Protected);
        p.protection = RECOIL_THRESHOLD - 1;
        let mut a = actor();
        a.render = render;
        let previous_animation = a.animation;
        let e = p.protected(&mut a, &mut health(), TitleRule::OrdinaryDamage);
        assert_eq!(e.work, [Work::VerticalFall, Work::ProtectedContinuation]);
        // Pose/facing selection belongs after the pending vertical callback.
        assert_eq!(a.animation, previous_animation);
        assert_eq!(a.render & FACE_POSITIVE, render & FACE_POSITIVE);
    }
}

#[test]
fn protected_expiry_restores_normal_speed_then_applies_health_rule() {
    let mut p = player(Action::Protected);
    let mut a = actor();
    a.horizontal_magnitude = 0x0300;
    let mut h = health();
    let e = p.protected(&mut a, &mut h, TitleRule::OrdinaryDamage);
    assert_eq!(
        (p.action, a.horizontal_magnitude),
        (Action::Standing, NORMAL_SPEED)
    );
    assert_eq!(e.work, [Work::DamageHealth]);
    assert_eq!(h.segments[2].depleted, 1);
}
#[test]
fn protected_expiry_exhaustion_enters_death_with_phase_cleared() {
    let mut p = player(Action::Protected);
    let mut a = actor();
    a.state = 0xa7;
    let mut h = health();
    h.segments[1].depleted = 1;
    let e = p.protected(&mut a, &mut h, TitleRule::OrdinaryDamage);
    assert_eq!(p.action, Action::Death);
    assert_eq!(a.state & 0xf0, 0xa0);
    assert_eq!(a.state & 0x0f, 0);
    assert_eq!(e.work, [Work::DamageHealth, Work::DeathTrajectory]);
}
#[test]
fn damage_first_entry_initializes_arc_once_and_applies_alternating_rule() {
    let mut p = player(Action::Damage);
    let mut a = actor();
    a.horizontal_magnitude = 0x0300;
    let mut h = health();
    let e = p.damage(&mut a, &mut h, TitleRule::AlternatingDamage);
    assert_eq!(
        (p.protection, a.vertical_velocity, a.horizontal_magnitude),
        (DAMAGE_PROTECTION, DAMAGE_SPEED, 0x0300)
    );
    assert_eq!((a.animation_frame, a.animation_tick), (0, 0));
    assert_eq!(e.sounds, [DAMAGE_SOUND]);
    assert_eq!(h.alternating, 1);
    assert_eq!(h.segments[0].depleted, 0);
    assert_eq!(e.work.first(), Some(&Work::DamageHealth));
    assert!(e.work.contains(&Work::VerticalRise));
    let again = p.damage(&mut a, &mut h, TitleRule::AlternatingDamage);
    assert!(again.sounds.is_empty());
    assert_eq!(h.alternating, 1);
}
#[test]
fn damage_release_at_exact_three_c_resets_fraction_only_and_action() {
    for ticks in [DAMAGE_RELEASE - 1, DAMAGE_RELEASE, DAMAGE_RELEASE + 1] {
        let mut p = player(Action::Damage);
        p.protection = ticks;
        let mut a = actor();
        a.horizontal_magnitude = 0x0300;
        a.vertical_velocity = -1;
        let e = p.damage(&mut a, &mut health(), TitleRule::OrdinaryDamage);
        assert_eq!(e.work.first(), Some(&Work::VerticalFall));
        // The policy schedules the sequel; the complete dispatch regression below
        // proves the release and animation occur after actual vertical physics.
        assert_eq!(p.action, Action::Damage);
        assert_eq!(a.horizontal_magnitude, 0x0300);
        assert!(e.work.contains(&Work::DamageContinuation));
    }
}
#[test]
fn damage_zero_health_initializes_protection_then_immediately_delegates_death() {
    let mut p = player(Action::Damage);
    let mut a = actor();
    a.state = 0xaf;
    let mut h = health();
    h.segments[1].depleted = 1;
    let e = p.damage(&mut a, &mut h, TitleRule::OrdinaryDamage);
    assert_eq!((p.action, p.protection), (Action::Death, DAMAGE_PROTECTION));
    assert_eq!(a.state & 0x0f, 0);
    assert_eq!(e.sounds, [DAMAGE_SOUND]);
    assert!(!e.work.contains(&Work::Horizontal(Horizontal::Positive)));
}
#[test]
fn provisional_terrain_pose_applies_four_pixel_wrap_only_on_animation_change() {
    let mut p = player(Action::TerrainPose);
    let mut a = actor();
    a.position.screen_y = 0xfe;
    p.pose(&mut a, input(Buttons::NONE, Buttons::NONE));
    assert_eq!(
        (
            a.position.screen_y,
            a.animation,
            a.animation_frame,
            a.animation_tick
        ),
        (2, POSE_ANIMATION, 0, 0)
    );
    a.animation_tick = TEST_TICK;
    p.pose(&mut a, input(Buttons::A, Buttons::NONE));
    assert_eq!((a.position.screen_y, a.animation_tick), (2, TEST_TICK));
    assert_eq!(p.action, Action::TerrainPose);
}
#[test]
fn terrain_pose_pressed_a_enters_configured_jump() {
    let mut p = player(Action::TerrainPose);
    let mut a = actor();
    let e = p.pose(&mut a, input(Buttons::NONE, Buttons::A));
    assert_eq!(
        (p.action, a.vertical_velocity),
        (Action::Airborne, p.jump_velocity)
    );
    assert_eq!(e.work, [Work::VerticalRise]);
}
#[test]
fn fall_out_before_d0_continues_horizontal_without_life_loss() {
    let mut p = player(Action::FallOut);
    let mut a = actor();
    a.position.screen_y = LIFE_LOSS_Y - 1;
    let e = p.fall_out(&mut a, 2, 1, &mut health());
    assert_eq!(e.reload, None);
    assert_eq!(e.work, [Work::VerticalFall, Work::HorizontalInput]);
    assert_eq!(p.action, Action::FallOut);
}
#[test]
fn fall_out_at_d0_requests_reload_and_clears_all_transients_without_touching_position() {
    let mut p = player(Action::FallOut);
    p.queued = Some(Action::Damage);
    let mut a = actor();
    a.position.screen_y = LIFE_LOSS_Y;
    let mut h = health();
    h.alternating = 7;
    let e = p.fall_out(&mut a, 2, 1, &mut h);
    assert_eq!(e.reload, Some(0x8a));
    assert_eq!(
        (p.action, p.queued, a.state, h.alternating),
        (Action::Standing, None, 0, 0)
    );
    assert_eq!(a.position.screen_y, LIFE_LOSS_Y);
    assert!(!e.work.contains(&Work::HorizontalInput));
}
#[test]
fn life_loss_reload_uses_carry_from_stage_plus_eighty_seven_in_second_adc() {
    for (stage, variant, expected) in [
        (0, 0, 0x87),
        (0x78, 0, 0xff),
        (0x79, 0, 1),
        (0xff, 0, 0x87),
        (0xff, 0xff, 0x86),
    ] {
        let mut p = player(Action::FallOut);
        let mut a = actor();
        a.position.screen_y = LIFE_LOSS_Y;
        assert_eq!(
            p.fall_out(&mut a, stage, variant, &mut health()).reload,
            Some(expected)
        );
    }
}
#[test]
fn wait_a_uses_edge_bit_and_always_clears_tick_without_restarting_frame() {
    for (pressed, held, want) in [
        (Buttons::NONE, Buttons::A, Action::WaitA),
        (Buttons::A | Buttons::B, Buttons::NONE, Action::Standing),
    ] {
        let mut p = player(Action::WaitA);
        let mut a = actor();
        p.wait_a(&mut a, input(held, pressed));
        assert_eq!(
            (p.action, a.animation_tick, a.animation_frame),
            (want, 0, TEST_FRAME)
        );
    }
}
#[test]
fn carried_down_hides_player_and_bypasses_probe_and_jump_even_on_a_edge() {
    for action in [Action::CarriedLocked, Action::CarriedJumpable] {
        let mut p = player(action);
        let mut a = actor();
        let e = p.carried(
            &mut a,
            input(Buttons::DOWN, Buttons::A),
            CollisionFacts {
                bits: SOLID_CONTACT,
                ..Default::default()
            },
        );
        assert_eq!(a.state, 0);
        assert_eq!(p.action, action);
        assert_eq!((e.scroll, e.spawn), (Some(2), Some(2)));
        assert!(e.work.is_empty());
    }
}
#[test]
fn only_jumpable_carried_state_accepts_exact_a_edge_byte() {
    for (action, pressed, jump) in [
        (Action::CarriedLocked, Buttons::A, false),
        (Action::CarriedJumpable, Buttons::A, true),
        (Action::CarriedJumpable, Buttons::A | Buttons::B, false),
        (Action::CarriedJumpable, Buttons::NONE, false),
    ] {
        let mut p = player(action);
        let mut a = actor();
        let e = p.carried(
            &mut a,
            input(Buttons::NONE, pressed),
            CollisionFacts::default(),
        );
        assert_eq!(p.action, if jump { Action::Airborne } else { action });
        assert_eq!(a.state, HELD_ACTOR);
        assert_eq!(
            e.work,
            if jump {
                vec![Work::VerticalRise]
            } else {
                vec![Work::Terrain {
                    start: 3,
                    remaining: 2,
                }]
            }
        );
        if jump {
            assert_eq!(a.vertical_velocity, p.jump_velocity);
        }
    }
}
#[test]
fn carried_upper_solid_adds_eight_and_enters_damage_immediately() {
    let mut p = player(Action::CarriedLocked);
    let mut a = actor();
    a.position.screen_y = 0xfc;
    let e = p.carried(
        &mut a,
        input(Buttons::NONE, Buttons::NONE),
        CollisionFacts {
            bits: SOLID_CONTACT,
            ..Default::default()
        },
    );
    assert_eq!((p.action, a.position.screen_y), (Action::Damage, 4));
    assert_eq!(
        e.work,
        [
            Work::Terrain {
                start: 3,
                remaining: 2
            },
            Work::DamageHealth
        ]
    );
}
#[test]
fn external_launch_phase_zero_below_ceiling_skips_vertical_and_always_resets_air_pose() {
    let mut p = player(Action::ExternalLaunch);
    let mut a = pool();
    a[0].position.screen_y = LAUNCH_CEILING - 1;
    let mut w = World::default();
    let e = p.launch(&mut a, &mut w);
    assert!(!w.calls.contains(&Call::Y));
    assert!(!w.calls.iter().any(|c| matches!(c, Call::Gravity(_))));
    assert_eq!((e.scroll, e.spawn), (Some(1), Some(1)));
    assert_eq!(
        (a[0].animation, a[0].animation_frame, a[0].animation_tick),
        (AIR_ANIMATION, 0, 0)
    );
}
#[test]
fn external_launch_phase_zero_integrates_before_gravity_at_ceiling() {
    let mut p = player(Action::ExternalLaunch);
    let mut a = pool();
    a[0].position.screen_y = LAUNCH_CEILING;
    let mut w = World::default();
    p.launch(&mut a, &mut w);
    assert_eq!(&w.calls[..2], &[Call::Y, Call::Gravity(PLAYER_GRAVITY)]);
    assert!(w.calls.contains(&Call::Probe(0, 2)));
}
#[test]
fn external_launch_fall_phase_sets_e0_and_gravity_before_y() {
    let mut p = player(Action::ExternalLaunch);
    let mut a = pool();
    a[0].motion_flags = 1;
    let mut w = World::default();
    p.launch(&mut a, &mut w);
    assert_eq!(a[0].state, LAUNCH_ACTOR);
    assert_eq!(&w.calls[..2], &[Call::Gravity(PLAYER_GRAVITY), Call::Y]);
}
#[test]
fn external_launch_page_seven_changes_phase_after_horizontal_and_clears_velocity() {
    for (x, phase) in [(0x06ff, 0), (0x0700, 1), (0x07ff, 1), (0x0800, 0)] {
        let mut p = player(Action::ExternalLaunch);
        let mut a = pool();
        a[0].vertical_velocity = 0x0728;
        let mut w = World {
            x: Some(x),
            ..Default::default()
        };
        p.launch(&mut a, &mut w);
        assert_eq!(a[0].motion_flags, phase);
        if phase != 0 {
            assert_eq!(a[0].vertical_velocity, 0);
        }
    }
}
#[test]
fn launch_terrain_obstruction_restores_speed_and_ordinary_action() {
    let mut p = player(Action::ExternalLaunch);
    let mut a = pool();
    a[0].horizontal_magnitude = 0x0400;
    let mut w = World {
        probe: CollisionFacts {
            bits: SOLID_CONTACT,
            ..Default::default()
        },
        ..Default::default()
    };
    p.launch(&mut a, &mut w);
    assert_eq!(
        (p.action, a[0].horizontal_magnitude),
        (Action::Standing, NORMAL_SPEED)
    );
    assert_eq!(a[0].animation, AIR_ANIMATION);
}
#[test]
fn death_first_phase_initializes_arc_once_preserving_high_flags() {
    let mut p = player(Action::Death);
    let mut a = pool();
    let mut w = World::default();
    p.death(&mut a, &mut w, 2, 0, &mut health());
    assert_eq!(
        (
            a[0].state,
            a[0].motion_flags,
            a[0].animation,
            a[0].animation_frame,
            a[0].direction
        ),
        (HELD_ACTOR | 1, 0, HURT_ANIMATION, 0, 1)
    );
    assert_eq!(
        (a[0].vertical_velocity, a[0].horizontal_magnitude),
        (DEATH_SPEED, DEATH_DRIFT)
    );
    assert_eq!(
        w.calls,
        [
            Call::Y,
            Call::Gravity(PLAYER_GRAVITY),
            Call::X(Horizontal::Positive)
        ]
    );
}
#[test]
fn death_apex_changes_phase_and_returns_before_horizontal_drift() {
    let mut p = player(Action::Death);
    let mut a = pool();
    a[0].state |= 1;
    let mut w = World {
        velocity: Some(-1),
        ..Default::default()
    };
    p.death(&mut a, &mut w, 2, 0, &mut health());
    assert_eq!((a[0].motion_flags, a[0].vertical_velocity), (1, 0));
    assert_eq!(w.calls, [Call::Y, Call::Gravity(PLAYER_GRAVITY)]);
}
#[test]
fn death_falling_gravity_before_y_then_drift_until_e0() {
    let mut p = player(Action::Death);
    let mut a = pool();
    a[0].state |= 1;
    a[0].motion_flags = 1;
    let mut w = World {
        y: Some(BOTTOM_ROOM_Y - 1),
        ..Default::default()
    };
    let e = p.death(&mut a, &mut w, 2, 0, &mut health());
    assert_eq!(e.reload, None);
    assert_eq!(
        w.calls,
        [
            Call::Gravity(PLAYER_GRAVITY),
            Call::Y,
            Call::X(Horizontal::Positive)
        ]
    );
}
#[test]
fn death_at_e0_uses_common_reload_and_returns_without_horizontal() {
    let mut p = player(Action::Death);
    p.queued = Some(Action::Damage);
    let mut a = pool();
    a[0].state |= 1;
    a[0].motion_flags = 1;
    let mut h = health();
    h.alternating = 1;
    let mut w = World {
        y: Some(BOTTOM_ROOM_Y),
        ..Default::default()
    };
    let e = p.death(&mut a, &mut w, 2, 1, &mut h);
    assert_eq!(e.reload, Some(0x8a));
    assert_eq!(
        (p.action, p.queued, a[0].state, h.alternating),
        (Action::Standing, None, 0, 0)
    );
    assert_eq!(w.calls, [Call::Gravity(PLAYER_GRAVITY), Call::Y]);
}
#[test]
fn held_state_changes_tick_only_and_preserves_motion_animation_and_action() {
    let p = player(Action::Hold);
    let mut a = actor();
    let mut expected = a;
    expected.animation_tick = 0;
    p.hold(&mut a);
    assert_eq!(a, expected);
    assert_eq!(p.action, Action::Hold);
}
#[test]
fn complete_dialogue_and_conversion_calls_tick_player_timers_before_owner_delegation() {
    for (action, expected) in [
        (Action::Dialogue, Work::Dialogue),
        (Action::ConvertTimer, Work::ConvertTimer),
    ] {
        let mut p = player(action);
        p.protection = 2;
        p.special_mode = 1;
        p.special_ticks = 2;
        let mut actors = pool();
        let mut inputs = Controllers::default();
        let mut r = room();
        let entries = rooms();
        let mut h = health();
        let mut w = World::default();
        let mut c = StepContext {
            room: &mut r,
            rooms: &entries,
            health: &mut h,
            title_rule: TitleRule::OrdinaryDamage,
            stage: 3,
            life_loss_variant: 0,
            current_stage_audio: SPECIAL_AUDIO,
        };
        let e = p.step(&mut actors, &mut inputs, &mut c, &data(), &mut w);
        assert_eq!((p.protection, p.special_ticks), (1, 1));
        assert_eq!(e.work, [expected]);
        assert!(w.calls.is_empty());
    }
}
#[test]
fn complete_standing_call_observes_immediate_vertical_damage_before_input_controls() {
    let mut p = player(Action::Standing);
    let mut actors = pool();
    let mut inputs = Controllers::default();
    inputs.ports[0] = input(Buttons::A, Buttons::A | Buttons::SELECT);
    let mut r = room();
    let entries = rooms();
    let mut h = health();
    let mut w = World {
        immediate: Some(Action::Damage),
        ..Default::default()
    };
    let mut c = StepContext {
        room: &mut r,
        rooms: &entries,
        health: &mut h,
        title_rule: TitleRule::OrdinaryDamage,
        stage: 0,
        life_loss_variant: 0,
        current_stage_audio: 0,
    };
    let e = p.step(&mut actors, &mut inputs, &mut c, &data(), &mut w);
    assert_eq!(p.action, Action::Damage);
    assert!(!e.work.contains(&Work::CostOverlay));
    assert!(!e.work.contains(&Work::VerticalRise));
    assert_eq!(actors[0].vertical_velocity, 0);
}
#[test]
fn complete_forced_jump_mirror_precedes_rising_cutoff_and_never_resamples_input() {
    let mut p = player(Action::ForcedJump);
    let mut actors = pool();
    actors[0].vertical_velocity = 1;
    let mut inputs = Controllers::default();
    inputs.ports[0] = input(Buttons::B, Buttons::SELECT);
    let mut r = room();
    let entries = rooms();
    let mut h = health();
    let mut w = World::default();
    let mut c = StepContext {
        room: &mut r,
        rooms: &entries,
        health: &mut h,
        title_rule: TitleRule::OrdinaryDamage,
        stage: 0,
        life_loss_variant: 0,
        current_stage_audio: 0,
    };
    p.step(&mut actors, &mut inputs, &mut c, &data(), &mut w);
    assert_eq!(inputs.ports[0].held, Buttons::B | Buttons::A);
    assert_eq!(inputs.ports[0].pressed, Buttons::SELECT);
    assert_eq!(w.calls.first(), Some(&Call::Y));
    assert_eq!(actors[0].position.y_fraction, TEST_FRACTION);
}
#[test]
fn complete_committed_climb_transition_does_not_resume_remaining_player_action() {
    let mut p = player(Action::Climbing);
    let mut actors = pool();
    actors[0].position.screen_y = BOTTOM_ROOM_Y;
    let mut inputs = Controllers::default();
    inputs.ports[0] = input(Buttons::DOWN, Buttons::NONE);
    let mut r = room();
    let entries = rooms();
    let mut h = health();
    let mut w = World {
        probe: CollisionFacts::default(),
        ..Default::default()
    };
    let mut c = StepContext {
        room: &mut r,
        rooms: &entries,
        health: &mut h,
        title_rule: TitleRule::OrdinaryDamage,
        stage: 0,
        life_loss_variant: 0,
        current_stage_audio: 0,
    };
    // The final class comes from result3 of the four-sample climb probe. A
    // separate fixture field carries sample facts; it does not choose action.
    w.probe.classes[3] = CLIMB_CLASS;
    let e = p.step(&mut actors, &mut inputs, &mut c, &data(), &mut w);
    assert!(e.transition);
    assert_eq!(r.row, 2);
    assert_eq!(e.work.last(), Some(&Work::VerticalTransition));
    assert!(!e.work.contains(&Work::HorizontalInput));
}
#[test]
fn upward_probe_requires_held_up_and_uses_four_source_samples_before_alignment() {
    for held in [Buttons::NONE, Buttons::UP] {
        let mut p = player(Action::Airborne);
        let mut a = pool();
        a[0].position.world_x = 0x0123;
        let mut w = World {
            probe: CollisionFacts {
                classes: [2, 2, 0, 0],
                ..Default::default()
            },
            ..Default::default()
        };
        p.upward_probe(&mut a, held.0, &data(), &mut w);
        if held == Buttons::NONE {
            assert!(w.calls.is_empty());
            assert_eq!(p.action, Action::Airborne);
        } else {
            assert_eq!(w.calls, [Call::Probe(0x0c, 3)]);
            assert_eq!(
                (p.action, a[0].position.world_x),
                (Action::Climbing, 0x0120)
            );
        }
    }
}
#[test]
fn upward_probe_solid_bit_prevents_alignment_even_with_climb_samples() {
    let mut p = player(Action::Airborne);
    let mut a = pool();
    let before = a[0];
    let mut w = World {
        probe: CollisionFacts {
            bits: SOLID_CONTACT,
            classes: [2; 4],
            ..Default::default()
        },
        ..Default::default()
    };
    p.upward_probe(&mut a, Buttons::UP.0, &data(), &mut w);
    assert_eq!(a[0], before);
    assert_eq!(p.action, Action::Airborne);
}
#[test]
fn ready_airborne_pose_loses_pose_only_when_horizontal_solid_contact_is_absent() {
    for (animation, bits, want) in [
        (STRIKE_READY_ANIMATION, 0, AIR_ANIMATION),
        (
            STRIKE_READY_ANIMATION,
            SOLID_CONTACT,
            STRIKE_READY_ANIMATION,
        ),
        (SWING_ANIMATION, 0, SWING_ANIMATION),
    ] {
        let p = player(Action::Airborne);
        let mut a = actor();
        a.animation = animation;
        p.airborne_horizontal_exit(&mut a, bits);
        assert_eq!(a.animation, want);
        if want == AIR_ANIMATION {
            assert_eq!((a.animation_frame, a.animation_tick), (0, 0));
        } else {
            assert_eq!(
                (a.animation_frame, a.animation_tick),
                (TEST_FRAME, TEST_TICK)
            );
        }
    }
}
#[test]
fn external_bonus_adds_one_and_half_with_fraction_and_world_page_carry_before_left_step() {
    let mut p = player(Action::Standing);
    p.external_motion = 1;
    let mut a = pool();
    a[0].position.world_x = 0x01ff;
    a[0].position.x_fraction = 0xc0;
    let mut w = World::default();
    p.horizontal(&mut a, Buttons::LEFT.0, &mut w);
    assert_eq!(
        (a[0].position.world_x, a[0].position.x_fraction),
        (0x0201, 0x40)
    );
    assert_eq!(w.calls.first(), Some(&Call::X(Horizontal::Negative)));
    assert_eq!(p.external_motion, 0);
}
#[test]
fn complete_carried_upper_contact_initializes_damage_and_health_before_return() {
    let mut p = player(Action::CarriedLocked);
    let mut actors = pool();
    let mut inputs = Controllers::default();
    let mut r = room();
    let entries = rooms();
    let mut h = health();
    let mut w = World {
        probe: CollisionFacts {
            bits: SOLID_CONTACT,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut c = StepContext {
        room: &mut r,
        rooms: &entries,
        health: &mut h,
        title_rule: TitleRule::OrdinaryDamage,
        stage: 0,
        life_loss_variant: 0,
        current_stage_audio: 0,
    };
    let e = p.step(&mut actors, &mut inputs, &mut c, &data(), &mut w);
    assert_eq!(
        (p.action, p.protection),
        (Action::Damage, DAMAGE_PROTECTION)
    );
    assert_eq!(h.segments[2].depleted, 1);
    assert_eq!(e.sounds, [DAMAGE_SOUND]);
    assert!(w.calls.contains(&Call::Probe(3, 2)));
    assert!(w.calls.contains(&Call::Y));
}
#[test]
fn complete_protected_expiry_exhaustion_runs_death_initializer_in_same_call() {
    let mut p = player(Action::Protected);
    let mut actors = pool();
    actors[0].state = 0xa7;
    let mut inputs = Controllers::default();
    let mut r = room();
    let entries = rooms();
    let mut h = health();
    h.segments[1].depleted = 1;
    let mut w = World::default();
    let mut c = StepContext {
        room: &mut r,
        rooms: &entries,
        health: &mut h,
        title_rule: TitleRule::OrdinaryDamage,
        stage: 0,
        life_loss_variant: 0,
        current_stage_audio: 0,
    };
    p.step(&mut actors, &mut inputs, &mut c, &data(), &mut w);
    assert_eq!((p.action, actors[0].state), (Action::Death, HELD_ACTOR | 1));
    assert_eq!(
        (actors[0].vertical_velocity, actors[0].horizontal_magnitude),
        (DEATH_SPEED, DEATH_DRIFT)
    );
    assert_eq!(
        w.calls,
        [
            Call::Y,
            Call::Gravity(PLAYER_GRAVITY),
            Call::X(Horizontal::Positive)
        ]
    );
}
#[test]
fn complete_player_update_retains_terrain_queue_for_b13_tail_instead_of_applying_it_immediately() {
    let mut p = player(Action::Standing);
    let mut actors = pool();
    let mut inputs = Controllers::default();
    let mut r = room();
    let entries = rooms();
    let mut h = health();
    let mut w = World {
        probe: CollisionFacts {
            bits: SOLID_CONTACT,
            ..Default::default()
        },
        queued: Some(Action::TerrainPose),
        ..Default::default()
    };
    let mut c = StepContext {
        room: &mut r,
        rooms: &entries,
        health: &mut h,
        title_rule: TitleRule::OrdinaryDamage,
        stage: 0,
        life_loss_variant: 0,
        current_stage_audio: 0,
    };
    p.step(&mut actors, &mut inputs, &mut c, &data(), &mut w);
    assert_eq!(
        (p.action, p.queued),
        (Action::Standing, Some(Action::TerrainPose))
    );
    assert_eq!(actors[0].animation, IDLE_ANIMATION);
}

#[test]
fn full_swing_uses_the_actual_contact_slot_instead_of_an_older_hit_marker() {
    let mut p = player(Action::Swing);
    let mut actors = pool();
    actors[0].animation = SWING_ANIMATION;
    actors[0].animation_frame = 2;
    actors[0].animation_tick = 0;
    actors[1].hit_marker = 1;
    actors[1].handler = 1;
    actors[2].handler = 8;
    let mut inputs = Controllers::default();
    let mut r = room();
    let entries = rooms();
    let mut h = health();
    let mut w = World {
        horizontal_actor: Some(2),
        ..World::default()
    };
    let mut c = StepContext {
        room: &mut r,
        rooms: &entries,
        health: &mut h,
        title_rule: TitleRule::OrdinaryDamage,
        stage: 0,
        life_loss_variant: 0,
        current_stage_audio: 0,
    };
    let effects = p.step(&mut actors, &mut inputs, &mut c, &data(), &mut w);
    assert_eq!(actors[0].animation, SWING_ANIMATION);
    assert_eq!(effects.sounds, [SWING_SOUND]);
    assert_eq!(actors[1].hit_marker, 1);
}

#[test]
fn direct_swing_horizontal_entry_preserves_momentum_control_and_prior_comparison() {
    let mut p = player(Action::Swing);
    p.momentum_control = 1;
    let mut actors = [ActorContact::default(); ACTOR_SLOTS];
    actors[0] = actor();
    actors[0].animation = SWING_ANIMATION;
    let mut w = World::default();
    p.horizontal_entry(&mut actors, 0, &mut w, false);
    assert_eq!(
        p.momentum_control, 1,
        "E495 bypasses E4A6 momentum-control clear"
    );
    assert_eq!(actors[0].horizontal_magnitude, NORMAL_SPEED);
    assert!(w.calls.is_empty());
    assert!(
        w.returns.is_empty(),
        "directionless direct entry preserves the prior CMP7"
    );
}

#[test]
fn air_horizontal_mode_clears_remembered_momentum_without_returning_floor_comparison() {
    for animation in [AIR_ANIMATION, POGO_ANIMATION] {
        let mut p = player(Action::Airborne);
        p.momentum_direction = 1;
        p.momentum_control = 1;
        let mut actors = [ActorContact::default(); ACTOR_SLOTS];
        actors[0] = actor();
        actors[0].animation = animation;
        let mut w = World::default();
        p.horizontal(&mut actors, 0, &mut w);
        assert_eq!(w.returns, [ReturnOperation::AirModeSelected]);
        assert!(w.returns[0].comparison());
        assert_eq!((p.momentum_direction, p.momentum_control), (0, 1));
    }
}

#[test]
fn entered_standing_walk_finishes_after_land_pose_and_horizontal_state_change() {
    for horizontal_immediate in [None, Some(Action::Damage)] {
        let mut p = player(Action::Standing);
        let mut actors = pool();
        actors[0].animation = LAND_ANIMATION;
        let mut inputs = Controllers::default();
        inputs.ports[0] = input(Buttons::RIGHT, Buttons::default());
        let mut r = room();
        let entries = rooms();
        let mut h = health();
        let mut c = StepContext {
            room: &mut r,
            rooms: &entries,
            health: &mut h,
            title_rule: TitleRule::OrdinaryDamage,
            stage: 0,
            life_loss_variant: 0,
            current_stage_audio: 0,
        };
        let mut w = World {
            horizontal_immediate,
            probe: CollisionFacts {
                bits: SOLID_CONTACT,
                ..Default::default()
            },
            ..Default::default()
        };
        p.step(&mut actors, &mut inputs, &mut c, &data(), &mut w);
        assert_eq!(actors[0].animation, WALK_ANIMATION);
        assert_eq!(p.action, horizontal_immediate.unwrap_or(Action::Standing));
        assert_eq!(
            w.returns.last(),
            Some(&ReturnOperation::WalkAnimation(LAND_ANIMATION))
        );
    }
}

#[test]
fn installed_jump_tail_runs_airborne_horizontal_motion_for_all_three_entries() {
    for action in [
        Action::Standing,
        Action::TerrainPose,
        Action::CarriedJumpable,
    ] {
        let mut p = player(action);
        let mut actors = pool();
        let mut inputs = Controllers::default();
        inputs.ports[0] = input(Buttons::A | Buttons::RIGHT, Buttons::A);
        let mut r = room();
        let entries = rooms();
        let mut h = health();
        let mut c = StepContext {
            room: &mut r,
            rooms: &entries,
            health: &mut h,
            title_rule: TitleRule::OrdinaryDamage,
            stage: 0,
            life_loss_variant: 0,
            current_stage_audio: 0,
        };
        let mut w = World::default();
        p.step(&mut actors, &mut inputs, &mut c, &data(), &mut w);
        assert!(
            w.calls.contains(&Call::X(Horizontal::Positive)),
            "jump entry must reach the airborne horizontal tail: {action:?}"
        );
        assert!(w.calls.contains(&Call::Scan(false)));
        assert_eq!(p.action, Action::Airborne);
    }
}

#[test]
fn complete_damage_fall_releases_control_after_vertical_and_restores_hurt_animation() {
    // EB35: falling physics returns before the exact $3C release and animation05.
    // pre_dispatch decrements protection, so enter one tick above the boundary.
    for (ticks, expected) in [
        (DAMAGE_RELEASE, Action::Damage),
        (DAMAGE_RELEASE + 1, Action::Standing),
        (DAMAGE_RELEASE + 2, Action::Damage),
    ] {
        let mut p = player(Action::Damage);
        p.protection = ticks;
        let mut actors = pool();
        actors[0].vertical_velocity = -200;
        actors[0].horizontal_magnitude = 0x0300;
        actors[0].animation = HURT_ANIMATION;
        let mut inputs = Controllers::default();
        let mut r = room();
        let entries = rooms();
        let mut h = health();
        let mut w = World::default(); // unsupported: vertical must observe damage state
        let mut c = StepContext {
            room: &mut r,
            rooms: &entries,
            health: &mut h,
            title_rule: TitleRule::OrdinaryDamage,
            stage: 4,
            life_loss_variant: 0,
            current_stage_audio: 4,
        };
        p.step(&mut actors, &mut inputs, &mut c, &data(), &mut w);
        assert_eq!(p.action, expected);
        assert_eq!(actors[0].animation, HURT_ANIMATION);
        assert_eq!(
            actors[0].horizontal_magnitude,
            if expected == Action::Standing {
                0x0380
            } else {
                0x0300
            }
        );
        assert!(
            w.calls
                .iter()
                .position(|call| matches!(call, Call::Y))
                .unwrap()
                < w.calls
                    .iter()
                    .position(|call| matches!(call, Call::X(_)))
                    .unwrap()
        );
    }
}

#[test]
fn ceiling_tail_fall_replaces_contact_before_forced_jump_landing_gate() {
    // EA07 tail-enters E970 after a ceiling hit. E970 resets Scratch10 before
    // its downward samples, so the ceiling contact is not a later floor hit.
    let mut p = player(Action::ForcedJump);
    let mut actors = pool();
    actors[0].vertical_velocity = 0x0500;
    actors[0].animation = POGO_ANIMATION;
    let mut inputs = Controllers::default();
    inputs.ports[0].held = Buttons(Buttons::DOWN.0 | Buttons::B.0);
    let mut r = room();
    let entries = rooms();
    let mut h = health();
    let mut c = StepContext {
        room: &mut r,
        rooms: &entries,
        health: &mut h,
        title_rule: TitleRule::OrdinaryDamage,
        stage: 4,
        life_loss_variant: 0,
        current_stage_audio: 4,
    };
    let mut w = World {
        velocity: Some(-51),
        probe_sequence: [
            CollisionFacts {
                bits: crate::collision::SOLID_CONTACT,
                ..Default::default()
            },
            CollisionFacts::default(),
            CollisionFacts::default(),
        ]
        .into(),
        ..Default::default()
    };
    let effects = p.step(&mut actors, &mut inputs, &mut c, &data(), &mut w);
    assert_eq!(
        p.action,
        Action::ForcedJump,
        "a ceiling contact cannot become an unsupported falling landing"
    );
    assert_eq!(actors[0].animation, POGO_ANIMATION);
    assert_eq!(actors[0].vertical_velocity, -51);
    assert!(!effects.sounds.contains(&LAND_SOUND));
    assert!(w.calls.contains(&Call::Snap(Snap::NegativeY)));
    assert_eq!(
        w.calls
            .iter()
            .filter(|call| matches!(call, Call::Scan(true)))
            .count(),
        2
    );
}

#[test]
fn protected_recoil_restores_hurt_pose_after_falling_animation_before_horizontal_contact() {
    // EAD9 calls falling physics first, then writes animation05 without
    // resetting the frame/tick installed by the airborne animation helper.
    // Held Down+B can select pogo during that call even in protected state05.
    const FROZEN_POGO_TICK: u8 = 0x80;
    for (held, expected_tick) in [(Buttons::DOWN.0 | Buttons::B.0, FROZEN_POGO_TICK), (0, 0)] {
        for (render, direction) in [
            (TEST_RENDER, Horizontal::Positive),
            (TEST_RENDER | FACE_POSITIVE, Horizontal::Negative),
        ] {
            let mut p = player(Action::Protected);
            p.protection = RECOIL_THRESHOLD;
            let mut actors = pool();
            actors[0].animation = POSE_ANIMATION;
            actors[0].render = render;
            actors[0].vertical_velocity = -51;
            let mut inputs = Controllers::default();
            inputs.ports[0].held = Buttons(held);
            let mut r = room();
            let entries = rooms();
            let mut h = health();
            let mut context = StepContext {
                room: &mut r,
                rooms: &entries,
                health: &mut h,
                title_rule: TitleRule::OrdinaryDamage,
                stage: 0,
                life_loss_variant: 0,
                current_stage_audio: 0,
            };
            let mut world = World::default();
            p.step(&mut actors, &mut inputs, &mut context, &data(), &mut world);
            assert_eq!(p.action, Action::Protected);
            assert_eq!(actors[0].animation, HURT_ANIMATION);
            assert_eq!(actors[0].animation_tick, expected_tick);
            assert_eq!(world.horizontal_entry_animations, [HURT_ANIMATION]);
            assert_eq!(actors[0].render & FACE_POSITIVE, render & FACE_POSITIVE);
            assert!(world.calls.contains(&Call::X(direction)));
        }
    }
}

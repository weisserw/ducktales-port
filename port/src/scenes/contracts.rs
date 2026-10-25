//! Executable B16 expectations independently derived from source; see trace.md.
use super::*;
use crate::{
    actors::{ACTIVE, ActorContact},
    collision::Position,
    config,
    player::Action,
    status::{Health, HealthSegment},
};
use romatlas_port::assets::Rom;
use std::collections::VecDeque;
const SUBJECT: usize = 2;
const WORLD_STATE: u8 = 0xa0;
const SENTINEL_TICK: u8 = 0x39;
fn rom() -> Vec<u8> {
    let b =
        std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../input/game.rom"))
            .expect("local matching source fixture");
    Rom::from_bytes(b.clone(), config::ROM_SHA256).expect("source identity");
    b
}
fn pool() -> ActorPool {
    [ActorContact::default(); crate::actors::ACTOR_SLOTS]
}
fn actor() -> ActorContact {
    ActorContact {
        playback: crate::presentation::AnimationPlayback::from_codes(SENTINEL_TICK, 5, 0),
        control: crate::actors::ActorControl::from_codes(ACTIVE, 9, 8, 0),
        render: ACTIVE,
        animation: 0x55,
        position: Position {
            world_x: 0x01a0,
            screen_y: 0x90,
            x_fraction: 0x77,
            y_fraction: 0x66,
        },
        horizontal_magnitude: 0x0180,
        vertical_velocity: 0x0123,
        ..Default::default()
    }
}
fn status() -> Status {
    Status {
        reward: [0x24; 6].into(),
        total: [0x24; 7].into(),
        lives: 3,
        health: Health::from_overlay(
            [HealthSegment {
                y: 0,
                tile: 0,
                depleted: 0,
                x: 0,
            }; 5],
            0,
            0,
        ),
        countdown: [1, 2, 3].into(),
        divider: 41,
        dirty: 0,
        ui_pending: 0,
    }
}
fn place(a: u8, y: u8, x: u8, flipped: bool) -> Option<Placement> {
    Some(Placement {
        animation: a,
        y,
        x,
        flipped,
    })
}
fn catalog() -> Catalog {
    let actors = [
        ActorList {
            slots: vec![place(0x80, 0xb3, 0xb0, false)],
            handler: SceneHandler::Bounce,
        },
        ActorList {
            slots: vec![place(0x7f, 0xb3, 0xf0, false)],
            handler: SceneHandler::WalkIn,
        },
        ActorList {
            slots: vec![place(0x83, 0x87, 0xa0, false)],
            handler: SceneHandler::Delay,
        },
        ActorList {
            slots: vec![None, place(0x81, 0xaa, 0x48, false)],
            handler: SceneHandler::Settle,
        },
        ActorList {
            slots: vec![
                place(0x7a, 0x35, 0x44, false),
                place(0x7a, 0x33, 0x57, false),
                place(0x7a, 0x33, 0x5d, true),
                place(0x7a, 0x35, 0x6c, false),
                place(0x7b, 0x33, 0x49, false),
                place(0x7b, 0x33, 0x71, false),
                place(0x7c, 0x3b, 0x5c, false),
            ],
            handler: SceneHandler::Settle,
        },
        ActorList {
            slots: vec![place(0x79, 0x67, 0x78, false)],
            handler: SceneHandler::Settle,
        },
    ];
    let special = [
        SpecialDefinition {
            records: vec![
                SpecialPlacement {
                    animation: 4,
                    x: 0xf0,
                    y: 0x80,
                    handler: 0,
                    profile: 0,
                },
                SpecialPlacement {
                    animation: 0x5a,
                    x: 0xf0,
                    y: 0x80,
                    handler: DEPARTURE_HANDLER,
                    profile: 0,
                },
            ],
            player_action: Action::Hold,
        },
        SpecialDefinition {
            records: vec![
                SpecialPlacement {
                    animation: 0,
                    x: 0x3d0,
                    y: 0xa0,
                    handler: 0,
                    profile: 0,
                },
                SpecialPlacement {
                    animation: 0x6c,
                    x: 0x380,
                    y: 0x10,
                    handler: SPAWNER_HANDLER,
                    profile: 0,
                },
            ],
            player_action: Action::Standing,
        },
    ];
    Catalog {
        actors,
        special,
        parameters: std::array::from_fn(|i| SceneParameters {
            terrain: [2, 2, 2, 1, 1, 1][i],
            chr: [9, 9, 9, 9, 10, 10][i],
            background_end: [15, 15, 15, 31, 47, 63][i],
            sprite_end: [79, 79, 79, 223, 79, 79][i],
            script_offset: [7, 7, 7, 0, 4, 6][i],
        }),
        handlers: [
            SceneHandler::Bounce,
            SceneHandler::WalkIn,
            SceneHandler::Delay,
            SceneHandler::MoveToMark,
            SceneHandler::Settled,
            SceneHandler::Settled,
        ],
        scripts: vec![11, 0x83, 0x88, 0, 6, 0, 7, 0],
        split_y: [0x70, 0x90],
        reward_delays: [30, 30, 30, 60, 60, 60, 120, 120],
    }
}
#[derive(Default)]
struct Boundary {
    calls: Vec<Integration>,
    facts: VecDeque<(Option<Position>, Option<i16>, bool)>,
    observations: Vec<ActorContact>,
}
impl Motion for Boundary {
    fn apply(&mut self, a: &mut ActorContact, o: Integration, _: u8) -> bool {
        self.calls.push(o);
        self.observations.push(*a);
        let (p, v, c) = self.facts.pop_front().unwrap_or((None, None, false));
        if let Some(p) = p {
            a.position = p;
        }
        if let Some(v) = v {
            a.vertical_velocity = v;
        }
        c
    }
}
fn update(a: &mut ActorPool, m: &mut Boundary) -> HandlerStep {
    update_scene_actor(
        a,
        &catalog(),
        COMPLETION_GRAVITY,
        m,
        &mut HandlerContinuation::Entry,
        &mut Display::default(),
    )
}
#[test]
fn all_original_scene_lists_dispatch_parameters_special_records_and_delays_decode_exactly() {
    assert_eq!(Catalog::original(&rom()).unwrap(), catalog());
}
#[test]
fn initializer_skip_preserves_entire_slot_and_terminal_only_overwrites_slot_zero_handler() {
    let c = catalog();
    let mut a = [actor(); 16];
    let before = a;
    initialize_actors(&mut a, &c.actors[3]);
    let mut zero = before[0];
    {
        let actor_value = SceneHandler::Settle as u8;
        zero.set_handler(actor_value);
    };
    assert_eq!(a[0], zero);
    assert_eq!(a[1].animation, 0x81);
    assert_eq!(a[2..], before[2..]);
}
#[test]
fn initializer_writes_only_authored_fields_and_keeps_fractional_motion_identity_and_locals() {
    let mut a = [actor(); 16];
    a[0].spawn_identity = Some(17);
    let old = a[0];
    initialize_actors(&mut a, &catalog().actors[0]);
    let mut e = old;
    e.animation = 0x80;
    e.position.world_x = 0xb0;
    e.position.screen_y = 0xb3;
    {
        let actor_value = ACTIVE;
        e.set_state_code(actor_value);
    };
    e.render = RENDER_SCENE;
    e.playback.restart_clock();
    e.playback.reset_frame();
    {
        let actor_value = 0;
        e.set_handler(actor_value);
    };
    assert_eq!(a[0], e);
}
#[test]
fn flip_uses_absolute_list_byte_offset_nineteen_hex_and_not_slot_nineteen() {
    let c = catalog();
    let mut a = pool();
    initialize_actors(&mut a, &c.actors[4]);
    assert_eq!(a[2].render, 0xd0);
    for (index, actor) in a[..7].iter().enumerate() {
        assert_eq!(actor.render, if index == 2 { 0xd0 } else { RENDER_SCENE });
    }
    assert_eq!(a[7].state_code(), 0);
}
#[test]
fn packed_scene_skip_advances_three_bytes_without_reading_coordinates_as_animations() {
    let b = [0xff, 0x01, 0x02, 0x40, 0x70, 0x80, 0x05];
    assert_eq!(
        actor_list(&b, 0).unwrap(),
        ActorList {
            slots: vec![None, place(0x40, 0x70, 0x80, false)],
            handler: SceneHandler::Settled
        }
    );
}
#[test]
fn packed_scene_terminator_is_any_animation_below_sixteen_without_consuming_triple() {
    for end in 0..6 {
        let b = [end];
        let list = actor_list(&b, 0).unwrap();
        assert!(list.slots.is_empty());
        assert_eq!(list.handler as u8, end);
    }
}
#[test]
fn packed_scene_truncation_rejects_partial_real_and_skip_records() {
    for b in [
        vec![],
        vec![0x40],
        vec![0x40, 0x70],
        vec![0xff],
        vec![0xff, 0x70],
        vec![0x40, 0x70, 0x80],
    ] {
        assert!(actor_list(&b, 0).is_err());
    }
}
#[test]
fn special_six_byte_records_allow_zero_animation_and_stop_on_sign_not_ff_only() {
    let b = [0, 0x34, 0x12, 0x80, DEPARTURE_HANDLER, 7, 0x80];
    assert_eq!(
        special_list(&b, 0).unwrap(),
        vec![SpecialPlacement {
            animation: 0,
            x: 0x1234,
            y: 0x80,
            handler: DEPARTURE_HANDLER,
            profile: 7
        }]
    );
}
#[test]
fn special_record_truncation_rejects_each_missing_field_and_missing_negative_end() {
    let b = [0, 0x34, 0x12, 0x80, DEPARTURE_HANDLER, 7];
    for end in 0..=b.len() {
        assert!(special_list(&b[..end], 0).is_err());
    }
}
#[test]
fn both_special_lists_initialize_even_slots_and_preserve_every_unwritten_field() {
    for d in catalog().special {
        let mut a = [actor(); 16];
        let before = a;
        let mut p = Player {
            action: Action::Death,
            ..Default::default()
        };
        initialize_special_actors(&mut a, &d, &mut p);
        assert_eq!(p.action, d.player_action);
        for (i, r) in d.records.iter().enumerate() {
            let mut e = before[i * 2];
            e.animation = r.animation;
            e.position.world_x = r.x;
            e.position.screen_y = r.y;
            {
                let actor_value = r.handler;
                e.set_handler(actor_value);
            };
            e.profile = r.profile;
            {
                let actor_value = WORLD_STATE;
                e.set_state_code(actor_value);
            };
            e.render = WORLD_STATE;
            e.playback.restart_clock();
            e.playback.reset_frame();
            assert_eq!(a[i * 2], e);
        }
        for i in [1, 3, 4, 5, 15] {
            assert_eq!(a[i], before[i]);
        }
    }
}
#[test]
fn bounce_initialization_integrates_before_gravity_in_same_call() {
    let mut a = pool();
    a[0] = actor();
    {
        let actor_value = SceneHandler::Bounce as u8;
        a[0].set_handler(actor_value);
    };
    {
        let actor_value = ACTIVE;
        a[0].set_state_code(actor_value);
    };
    let mut m = Boundary::default();
    assert_eq!(update(&mut a, &mut m), HandlerStep::Complete);
    assert_eq!(m.calls, vec![Integration::SubtractY, Integration::Gravity]);
    assert_eq!(m.observations[0].vertical_velocity, BOUNCE_SPEED);
    assert_eq!(a[0].local_state_code(), BOUNCE_COUNT);
    assert_eq!(a[0].state_code(), ACTIVE | 1);
}
#[test]
fn bounce_negative_postgravity_velocity_clamps_and_counts_apex() {
    let mut a = pool();
    a[0] = actor();
    {
        let actor_value = 0;
        a[0].set_handler(actor_value);
    };
    {
        let actor_value = ACTIVE | 1;
        a[0].set_state_code(actor_value);
    };
    {
        let actor_value = 2;
        a[0].set_local_state_code(actor_value);
    };
    let mut m = Boundary {
        facts: VecDeque::from([(None, None, false), (None, Some(-1), false)]),
        ..Default::default()
    };
    update(&mut a, &mut m);
    assert_eq!(a[0].vertical_velocity, 0);
    assert_eq!(a[0].state_code(), ACTIVE | 2);
    assert_eq!(a[0].local_state_code(), 1);
}
#[test]
fn bounce_zero_postgravity_velocity_does_not_count_apex() {
    let mut a = pool();
    a[0] = actor();
    {
        let actor_value = 0;
        a[0].set_handler(actor_value);
    };
    {
        let actor_value = ACTIVE | 1;
        a[0].set_state_code(actor_value);
    };
    let mut m = Boundary {
        facts: VecDeque::from([(None, None, false), (None, Some(0), false)]),
        ..Default::default()
    };
    update(&mut a, &mut m);
    assert_eq!(a[0].state_code(), ACTIVE | 1);
    assert_eq!(a[0].local_state_code(), 9);
}
#[test]
fn last_apex_enters_move_handler_frozen_tick_without_resetting_frame() {
    let mut a = pool();
    a[0] = actor();
    {
        let actor_value = 0;
        a[0].set_handler(actor_value);
    };
    {
        let actor_value = ACTIVE | 1;
        a[0].set_state_code(actor_value);
    };
    {
        let actor_value = 1;
        a[0].set_local_state_code(actor_value);
    };
    let mut m = Boundary {
        facts: VecDeque::from([(None, None, false), (None, Some(-1), false)]),
        ..Default::default()
    };
    update(&mut a, &mut m);
    assert_eq!(
        (
            a[0].handler,
            a[0].state_code(),
            a[0].playback.tick_code(),
            a[0].local_state_code(),
            a[0].playback.frame_code()
        ),
        (SceneHandler::MoveToMark as u8, ACTIVE, FROZEN_TICK, 0, 5)
    );
}
#[test]
fn bounce_descent_applies_gravity_before_y_and_reloads_at_inclusive_floor() {
    for (y, reset) in [
        (BOUNCE_FLOOR - 1, false),
        (BOUNCE_FLOOR, true),
        (BOUNCE_FLOOR + 1, true),
    ] {
        let mut a = pool();
        a[0] = actor();
        {
            let actor_value = 0;
            a[0].set_handler(actor_value);
        };
        {
            let actor_value = ACTIVE | 2;
            a[0].set_state_code(actor_value);
        };
        let p = Position {
            screen_y: y,
            ..a[0].position
        };
        let mut m = Boundary {
            facts: VecDeque::from([(None, None, false), (Some(p), None, false)]),
            ..Default::default()
        };
        update(&mut a, &mut m);
        assert_eq!(m.calls, vec![Integration::Gravity, Integration::SubtractY]);
        assert_eq!(
            a[0].state_code(),
            if reset { ACTIVE | 1 } else { ACTIVE | 2 }
        );
        if reset {
            assert_eq!(a[0].vertical_velocity, BOUNCE_SPEED);
        }
    }
}
#[test]
fn walk_initialization_at_mark_moves_without_decrementing_zero_counter() {
    let mut a = pool();
    a[0] = actor();
    {
        let actor_value = SceneHandler::WalkIn as u8;
        a[0].set_handler(actor_value);
    };
    {
        let actor_value = ACTIVE;
        a[0].set_state_code(actor_value);
    };
    a[0].position.world_x = u16::from(WALK_MARK_X);
    let mut m = Boundary::default();
    update(&mut a, &mut m);
    assert_eq!(m.calls, vec![Integration::SubtractX]);
    assert_eq!(a[0].horizontal_magnitude, ONE_PIXEL_SPEED);
    assert_eq!(a[0].local_state_code(), 0);
}
#[test]
fn walk_low_byte_below_mark_wraps_zero_count_even_on_nonzero_world_page() {
    let mut a = pool();
    a[0] = actor();
    {
        let actor_value = 1;
        a[0].set_handler(actor_value);
    };
    {
        let actor_value = ACTIVE;
        a[0].set_state_code(actor_value);
    };
    a[0].position.world_x = 0x018f;
    let mut m = Boundary::default();
    update(&mut a, &mut m);
    assert!(m.calls.is_empty());
    assert_eq!(a[0].local_state_code(), 0xff);
}
#[test]
fn walk_last_count_selects_move_and_preserves_animation_frame() {
    let mut a = pool();
    a[0] = actor();
    {
        let actor_value = 1;
        a[0].set_handler(actor_value);
    };
    {
        let actor_value = ACTIVE | 1;
        a[0].set_state_code(actor_value);
    };
    a[0].position.world_x = 0x8f;
    {
        let actor_value = 1;
        a[0].set_local_state_code(actor_value);
    };
    update(&mut a, &mut Boundary::default());
    assert_eq!(
        (
            a[0].handler,
            a[0].playback.tick_code(),
            a[0].playback.frame_code()
        ),
        (3, FROZEN_TICK, 5)
    );
}
#[test]
fn delay_initialization_returns_without_decrementing_new_zero_count() {
    let mut a = pool();
    a[0] = actor();
    {
        let actor_value = 2;
        a[0].set_handler(actor_value);
    };
    {
        let actor_value = ACTIVE;
        a[0].set_state_code(actor_value);
    };
    update(&mut a, &mut Boundary::default());
    assert_eq!(
        (a[0].state_code(), a[0].local_state_code(), a[0].handler),
        (ACTIVE | 1, 0, 2)
    );
}
#[test]
fn delay_zero_wraps_and_one_expires_without_motion() {
    for (count, next, handler) in [(0, 0xff, 2), (2, 1, 2), (1, 0, 3)] {
        let mut a = pool();
        a[0] = actor();
        {
            let actor_value = 2;
            a[0].set_handler(actor_value);
        };
        {
            let actor_value = ACTIVE | 1;
            a[0].set_state_code(actor_value);
        };
        {
            let actor_value = count;
            a[0].set_local_state_code(actor_value);
        };
        let mut m = Boundary::default();
        update(&mut a, &mut m);
        assert!(m.calls.is_empty());
        assert_eq!((a[0].local_state_code(), a[0].handler), (next, handler));
    }
}
#[test]
fn move_initial_zero_delay_wraps_and_defers_speed_initialization() {
    let mut a = pool();
    a[0] = actor();
    {
        let actor_value = 3;
        a[0].set_handler(actor_value);
    };
    {
        let actor_value = ACTIVE;
        a[0].set_state_code(actor_value);
    };
    {
        let actor_value = 0;
        a[0].set_local_state_code(actor_value);
    };
    let before = a[0];
    let mut m = Boundary::default();
    update(&mut a, &mut m);
    assert_eq!(a[0].local_state_code(), 0xff);
    assert_eq!(a[0].horizontal_magnitude, before.horizontal_magnitude);
    assert_eq!(a[0].vertical_velocity, before.vertical_velocity);
    assert!(m.calls.is_empty());
}
#[test]
fn move_delay_expiry_sets_three_pixel_speeds_then_moves_x_same_call() {
    let mut a = pool();
    a[0] = actor();
    {
        let actor_value = 3;
        a[0].set_handler(actor_value);
    };
    {
        let actor_value = ACTIVE;
        a[0].set_state_code(actor_value);
    };
    {
        let actor_value = 1;
        a[0].set_local_state_code(actor_value);
    };
    a[0].position.world_x = u16::from(MOVE_MARK_X);
    let mut m = Boundary::default();
    update(&mut a, &mut m);
    assert_eq!(a[0].horizontal_magnitude, MOVE_SPEED);
    assert_eq!(a[0].vertical_velocity, MOVE_SPEED as i16);
    assert_eq!(m.calls, vec![Integration::SubtractX]);
}
#[test]
fn move_x_threshold_is_low_byte_and_exclusive_before_vertical_motion() {
    for (x, y, op) in [
        (0x0148, 0x77, Integration::SubtractX),
        (0x0147, 0x78, Integration::SubtractY),
    ] {
        let mut a = pool();
        a[0] = actor();
        {
            let actor_value = 3;
            a[0].set_handler(actor_value);
        };
        {
            let actor_value = ACTIVE | 1;
            a[0].set_state_code(actor_value);
        };
        a[0].position.world_x = x;
        a[0].position.screen_y = y;
        let mut m = Boundary::default();
        update(&mut a, &mut m);
        assert_eq!(m.calls, vec![op]);
    }
}
#[test]
fn move_below_both_marks_enters_palette_countdown_in_same_invocation() {
    let mut a = pool();
    a[0] = actor();
    {
        let actor_value = 3;
        a[0].set_handler(actor_value);
    };
    {
        let actor_value = ACTIVE | 1;
        a[0].set_state_code(actor_value);
    };
    a[0].position.world_x = 0x47;
    a[0].position.screen_y = 0x77;
    {
        let actor_value = 2;
        a[0].set_local_state_code(actor_value);
    };
    let mut m = Boundary::default();
    assert_eq!(update(&mut a, &mut m), HandlerStep::Complete);
    assert_eq!(a[0].local_state_code(), 1);
    assert!(m.calls.is_empty());
}
#[test]
fn palette_countdown_expiry_requests_scene_three_rebuild_before_palette_or_control_commit() {
    let mut a = pool();
    a[0] = actor();
    {
        let actor_value = 3;
        a[0].set_handler(actor_value);
    };
    {
        let actor_value = ACTIVE | 1;
        a[0].set_state_code(actor_value);
    };
    a[0].position.world_x = 0x47;
    a[0].position.screen_y = 0x77;
    {
        let actor_value = 1;
        a[0].set_local_state_code(actor_value);
    };
    let mut d = Display {
        control: 0x90,
        ..Default::default()
    };
    let mut k = HandlerContinuation::Entry;
    assert_eq!(
        update_scene_actor(
            &mut a,
            &catalog(),
            COMPLETION_GRAVITY,
            &mut Boundary::default(),
            &mut k,
            &mut d
        ),
        HandlerStep::RebuildInterior
    );
    assert_eq!(d.control, 0x90);
    assert!(!d.palette_pending);
    assert_ne!(a[0].handler, 5);
}
#[test]
fn actual_scene_handlers_four_and_five_settle_without_countdown_palette_or_motion() {
    for h in [4, 5] {
        let mut a = pool();
        a[0] = actor();
        {
            let actor_value = h;
            a[0].set_handler(actor_value);
        };
        {
            let actor_value = 1;
            a[0].set_local_state_code(actor_value);
        };
        let old = a[0];
        let mut m = Boundary::default();
        update(&mut a, &mut m);
        let mut e = old;
        {
            let actor_value = 5;
            e.set_handler(actor_value);
        };
        assert_eq!(a[0], e);
        assert!(m.calls.is_empty());
    }
}
struct DepartureFixture {
    progression: Progression,
    player: Player,
    status: Status,
    random: RandomState,
    signal: u8,
}
impl Default for DepartureFixture {
    fn default() -> Self {
        Self {
            progression: Progression::default(),
            player: Player::default(),
            status: status(),
            random: RandomState::default(),
            signal: 0,
        }
    }
}
impl DepartureFixture {
    fn context(&mut self) -> DepartureContext<'_> {
        DepartureContext {
            progression: &mut self.progression,
            player: &mut self.player,
            status: &mut self.status,
            random: &mut self.random,
            departure_signal: &mut self.signal,
        }
    }
    fn linked(&mut self, a: &mut ActorPool, m: &mut Boundary) {
        linked_departure(a, SUBJECT, &mut self.context(), m, COMPLETION_GRAVITY)
    }
    fn spawner(&mut self, a: &mut ActorPool, m: &mut Boundary) -> DepartureResult {
        timed_spawner(a, SUBJECT, &mut self.context(), m, &catalog())
    }
}
#[test]
fn linked_initializes_and_freezes_player_and_divider_before_leftward_move() {
    let mut a = pool();
    a[0] = actor();
    a[SUBJECT] = actor();
    {
        let actor_value = WORLD_STATE;
        a[SUBJECT].set_state_code(actor_value);
    };
    let mut f = DepartureFixture::default();
    f.linked(&mut a, &mut Boundary::default());
    assert_eq!((a[0].playback.tick_code(), f.status.divider), (0, 0));
    assert_eq!(a[SUBJECT].state_code(), WORLD_STATE | 1);
    assert_eq!(a[SUBJECT].horizontal_magnitude, ONE_PIXEL_SPEED);
    assert_eq!(a[SUBJECT].vertical_velocity, 0);
    assert_eq!(a[SUBJECT].position.world_x, 0x019f);
    assert_eq!(a[0].position.world_x, 0x019f);
}
#[test]
fn linked_fractional_borrow_changes_integer_but_never_stores_fraction() {
    let mut a = pool();
    a[0] = actor();
    a[SUBJECT] = actor();
    {
        let actor_value = WORLD_STATE | 1;
        a[SUBJECT].set_state_code(actor_value);
    };
    a[SUBJECT].position.world_x = 0x0200;
    a[SUBJECT].position.x_fraction = 0x20;
    a[SUBJECT].horizontal_magnitude = 0x0080;
    let mut f = DepartureFixture::default();
    f.linked(&mut a, &mut Boundary::default());
    assert_eq!(a[SUBJECT].position.world_x, 0x01ff);
    assert_eq!(a[SUBJECT].position.x_fraction, 0x20);
    assert_eq!(a[0].position.world_x, 0x01ff);
    assert_eq!(a[0].position.x_fraction, 0x77);
}
#[test]
fn linked_ordinary_digit_gate_compares_postmove_low_byte_ignoring_page() {
    for (x, signal) in [(0x0111, 0), (0x0110, 1), (0x0100, 0)] {
        let mut a = pool();
        a[SUBJECT] = actor();
        {
            let actor_value = WORLD_STATE | 1;
            a[SUBJECT].set_state_code(actor_value);
        };
        a[SUBJECT].position.world_x = x;
        a[SUBJECT].position.x_fraction = 0;
        a[SUBJECT].horizontal_magnitude = ONE_PIXEL_SPEED;
        let mut f = DepartureFixture::default();
        f.linked(&mut a, &mut Boundary::default());
        assert_eq!(f.progression.scene_signal, signal);
    }
}
#[test]
fn linked_completion_signal_wraps_instead_of_saturating() {
    let mut a = pool();
    a[SUBJECT] = actor();
    {
        let actor_value = WORLD_STATE | 1;
        a[SUBJECT].set_state_code(actor_value);
    };
    a[SUBJECT].position.world_x = 0x10;
    a[SUBJECT].horizontal_magnitude = ONE_PIXEL_SPEED;
    let mut f = DepartureFixture::default();
    f.progression.scene_signal = 0xff;
    f.linked(&mut a, &mut Boundary::default());
    assert_eq!(f.progression.scene_signal, 0);
}
#[test]
fn linked_exact_third_reward_code_seven_brakes_below_b0_only() {
    for (digit, x, speed) in [
        (REWARD_CODE_GATE, 0xb1, 0x0100),
        (REWARD_CODE_GATE, 0xb0, 0x00fc),
        (REWARD_CODE_GATE - 1, 0xb0, 0x0100),
        (0x24, 0xb0, 0x0100),
    ] {
        let mut a = pool();
        a[SUBJECT] = actor();
        {
            let actor_value = WORLD_STATE | 1;
            a[SUBJECT].set_state_code(actor_value);
        };
        a[SUBJECT].position.world_x = x;
        a[SUBJECT].horizontal_magnitude = ONE_PIXEL_SPEED;
        let mut f = DepartureFixture::default();
        f.status.reward.edit_display_codes(|codes| codes[2] = digit);
        f.linked(&mut a, &mut Boundary::default());
        assert_eq!(a[SUBJECT].horizontal_magnitude, speed);
    }
}
#[test]
fn linked_final_brake_enters_vertical_phase_but_does_not_integrate_y_yet() {
    let mut a = pool();
    a[SUBJECT] = actor();
    {
        let actor_value = WORLD_STATE | 1;
        a[SUBJECT].set_state_code(actor_value);
    };
    a[SUBJECT].position.world_x = 0x80;
    a[SUBJECT].horizontal_magnitude = BRAKE_STEP;
    let mut f = DepartureFixture::default();
    f.status
        .reward
        .edit_display_codes(|codes| codes[2] = REWARD_CODE_GATE);
    let mut m = Boundary::default();
    f.linked(&mut a, &mut m);
    assert_eq!(a[SUBJECT].horizontal_magnitude, 0);
    assert_eq!(a[SUBJECT].state_code(), WORLD_STATE | 2);
    assert!(m.calls.is_empty());
}
#[test]
fn linked_brake_underflow_is_source_wrapping_and_not_native_clamp() {
    let mut a = pool();
    a[SUBJECT] = actor();
    {
        let actor_value = WORLD_STATE | 1;
        a[SUBJECT].set_state_code(actor_value);
    };
    a[SUBJECT].position.world_x = 0x80;
    a[SUBJECT].horizontal_magnitude = 3;
    let mut f = DepartureFixture::default();
    f.status
        .reward
        .edit_display_codes(|codes| codes[2] = REWARD_CODE_GATE);
    f.linked(&mut a, &mut Boundary::default());
    assert_eq!(a[SUBJECT].horizontal_magnitude, 0xffff);
    assert_eq!(a[SUBJECT].state_code(), WORLD_STATE | 1);
}
#[test]
fn linked_vertical_copies_postintegration_y_and_uses_integrator_carry_not_gravity() {
    for carry in [false, true] {
        let mut a = pool();
        a[SUBJECT] = actor();
        {
            let actor_value = WORLD_STATE | 2;
            a[SUBJECT].set_state_code(actor_value);
        };
        let y = Position {
            screen_y: 0x04,
            ..a[SUBJECT].position
        };
        let mut m = Boundary {
            facts: VecDeque::from([(None, None, !carry), (Some(y), None, carry)]),
            ..Default::default()
        };
        let mut f = DepartureFixture::default();
        f.linked(&mut a, &mut m);
        assert_eq!(m.calls, vec![Integration::Gravity, Integration::SubtractY]);
        assert_eq!(a[0].position.screen_y, 4);
        assert_eq!(f.progression.special_room, if carry { 0x82 } else { 0 });
    }
}
#[test]
fn hidden_player_spawner_tails_to_common_life_loss_before_any_actor_or_delay_change() {
    let mut a = pool();
    a[0].render = 0;
    a[SUBJECT] = actor();
    let old = a;
    let mut f = DepartureFixture::default();
    let mut m = Boundary::default();
    assert_eq!(f.spawner(&mut a, &mut m), DepartureResult::LifeLoss);
    assert_eq!(a, old);
    assert!(m.calls.is_empty());
}
#[test]
fn spawner_initialization_decrements_sixty_to_fifty_nine_and_returns_without_motion() {
    let mut a = pool();
    a[0].render = ACTIVE;
    a[SUBJECT] = actor();
    {
        let actor_value = WORLD_STATE;
        a[SUBJECT].set_state_code(actor_value);
    };
    let mut f = DepartureFixture::default();
    let mut m = Boundary::default();
    assert_eq!(f.spawner(&mut a, &mut m), DepartureResult::Complete);
    assert_eq!(a[SUBJECT].state_code(), WORLD_STATE | 1);
    assert_eq!(a[SUBJECT].local_state_code(), SPAWNER_DELAY - 1);
    assert_eq!(a[SUBJECT].local_counter_code(), 0);
    assert_eq!(a[SUBJECT].horizontal_magnitude, ONE_PIXEL_SPEED);
    assert_eq!(a[SUBJECT].vertical_velocity, SPAWNER_DOWN_SPEED);
    assert!(m.calls.is_empty());
}
#[test]
fn spawner_last_delay_tick_returns_without_downward_motion() {
    let mut a = pool();
    a[0].render = ACTIVE;
    a[SUBJECT] = actor();
    {
        let actor_value = WORLD_STATE | 1;
        a[SUBJECT].set_state_code(actor_value);
    };
    {
        let actor_value = 1;
        a[SUBJECT].set_local_state_code(actor_value);
    };
    let mut m = Boundary::default();
    DepartureFixture::default().spawner(&mut a, &mut m);
    assert_eq!(a[SUBJECT].local_state_code(), 0);
    assert!(m.calls.is_empty());
}
#[test]
fn spawner_downward_mark_is_inclusive_and_sets_departure_signal_without_travel() {
    for (y, phase, signal) in [(SPAWNER_DOWN_MARK - 1, 1, 0), (SPAWNER_DOWN_MARK, 2, 0x80)] {
        let mut a = pool();
        a[0].render = ACTIVE;
        a[SUBJECT] = actor();
        {
            let actor_value = WORLD_STATE | 1;
            a[SUBJECT].set_state_code(actor_value);
        };
        {
            let actor_value = 0;
            a[SUBJECT].set_local_state_code(actor_value);
        };
        let p = Position {
            screen_y: y,
            ..a[SUBJECT].position
        };
        let mut m = Boundary {
            facts: VecDeque::from([(Some(p), None, false)]),
            ..Default::default()
        };
        let mut f = DepartureFixture::default();
        f.spawner(&mut a, &mut m);
        assert_eq!(m.calls, vec![Integration::AddY]);
        assert_eq!(a[SUBJECT].state_code(), WORLD_STATE | phase);
        assert_eq!(f.signal, signal);
    }
}
#[test]
fn reward_allocator_chooses_highest_free_slot_and_never_uses_slot_eight() {
    let mut a = [actor(); 16];
    {
        let actor_value = 0x7f;
        a[14].set_state_code(actor_value);
    };
    {
        let actor_value = 0;
        a[12].set_state_code(actor_value);
    };
    {
        let actor_value = 0;
        a[8].set_state_code(actor_value);
    };
    let mut r = RandomState::default();
    assert_eq!(
        spawn_reward(
            &mut a,
            SUBJECT,
            &mut r,
            RandomIncrement::One,
            &catalog().reward_delays
        ),
        Some(14)
    );
    assert_eq!(a[12].state_code(), 0);
    assert_eq!(a[8].state_code(), 0);
}
#[test]
fn reward_allocator_all_seven_full_leaves_random_cooldown_and_slot_eight_unchanged() {
    let mut a = [actor(); 16];
    {
        let actor_value = 0;
        a[8].set_state_code(actor_value);
    };
    let old = a;
    let mut r = RandomState {
        words: [0x70, 0x90, 3, 4],
    };
    let before = r;
    assert_eq!(
        spawn_reward(
            &mut a,
            SUBJECT,
            &mut r,
            RandomIncrement::One,
            &catalog().reward_delays
        ),
        None
    );
    assert_eq!(a, old);
    assert_eq!(r, before);
}
#[test]
fn reward_allocator_accepts_slot_nine_when_ten_through_fifteen_are_active() {
    let mut a = [actor(); 16];
    {
        let actor_value = 1;
        a[REWARD_FIRST_SLOT].set_state_code(actor_value);
    };
    assert_eq!(
        spawn_reward(
            &mut a,
            SUBJECT,
            &mut RandomState::default(),
            RandomIncrement::One,
            &catalog().reward_delays
        ),
        Some(REWARD_FIRST_SLOT)
    );
}
#[test]
fn reward_child_overwrites_only_source_written_fields_and_keeps_fractional_position() {
    let mut a = [actor(); 16];
    {
        let actor_value = 0;
        a[15].set_state_code(actor_value);
    };
    a[SUBJECT].position.world_x = 0x3456;
    a[SUBJECT].position.screen_y = 0x78;
    let mut e = a[15];
    {
        let actor_value = WORLD_STATE;
        e.set_state_code(actor_value);
    };
    e.render = WORLD_STATE;
    e.animation = REWARD_ANIMATION;
    {
        let actor_value = REWARD_HANDLER;
        e.set_handler(actor_value);
    };
    e.playback.restart_clock();
    e.playback.reset_frame();
    e.profile = 0;
    e.position.world_x = 0x3456;
    e.position.screen_y = 0x78;
    e.vertical_velocity = BOUNCE_SPEED;
    spawn_reward(
        &mut a,
        SUBJECT,
        &mut RandomState::default(),
        RandomIncrement::One,
        &catalog().reward_delays,
    );
    assert_eq!(a[15], e);
}
#[test]
fn reward_random_adc_preserves_other_words_and_wraps_before_eight_choice_lookup() {
    for (left, right, inc, sum, delay) in [
        (0xff, 0, RandomIncrement::One, 0, 30),
        (0xff, 0xff, RandomIncrement::One, 0xff, 120),
        (1, 1, RandomIncrement::Zero, 2, 30),
        (1, 1, RandomIncrement::One, 3, 60),
        (3, 3, RandomIncrement::Zero, 6, 120),
    ] {
        let mut a = [actor(); 16];
        {
            let actor_value = 0;
            a[15].set_state_code(actor_value);
        };
        let mut r = RandomState {
            words: [left, right, 0x55, 0xaa],
        };
        spawn_reward(&mut a, SUBJECT, &mut r, inc, &catalog().reward_delays);
        assert_eq!(r.words, [left, sum, 0x55, 0xaa]);
        assert_eq!(a[SUBJECT].local_counter_code(), delay);
    }
}
#[test]
fn full_pool_travel_wraps_failed_zero_cooldown_then_moves_without_rng_update() {
    let mut a = [actor(); 16];
    a[0].render = ACTIVE;
    {
        let actor_value = WORLD_STATE | 2;
        a[SUBJECT].set_state_code(actor_value);
    };
    {
        let actor_value = 0;
        a[SUBJECT].set_local_state_code(actor_value);
    };
    {
        let actor_value = 0;
        a[SUBJECT].set_local_counter_code(actor_value);
    };
    let mut f = DepartureFixture::default();
    f.random.words = [1, 2, 3, 4];
    let mut m = Boundary::default();
    f.spawner(&mut a, &mut m);
    assert_eq!(a[SUBJECT].local_counter_code(), 0xff);
    assert_eq!(f.random.words, [1, 2, 3, 4]);
    assert_eq!(m.calls, vec![Integration::SubtractX]);
}
#[test]
fn successful_natural_phase_two_reward_uses_compare_equality_increment_one_then_decrements_delay() {
    let mut a = [actor(); 16];
    a[0].render = ACTIVE;
    {
        let actor_value = 0;
        a[15].set_state_code(actor_value);
    };
    {
        let actor_value = WORLD_STATE | 2;
        a[SUBJECT].set_state_code(actor_value);
    };
    {
        let actor_value = 0;
        a[SUBJECT].set_local_state_code(actor_value);
    };
    {
        let actor_value = 0;
        a[SUBJECT].set_local_counter_code(actor_value);
    };
    let mut f = DepartureFixture::default();
    f.random.words = [1, 1, 0x55, 0xaa];
    f.spawner(&mut a, &mut Boundary::default());
    assert_eq!(f.random.words, [1, 3, 0x55, 0xaa]);
    assert_eq!(a[SUBJECT].local_counter_code(), 59);
    assert_eq!(a[15].handler, REWARD_HANDLER);
}
#[test]
fn spawner_nonzero_cooldown_skips_allocation_but_decrements_before_horizontal_motion() {
    let mut a = pool();
    a[0].render = ACTIVE;
    a[SUBJECT] = actor();
    {
        let actor_value = WORLD_STATE | 2;
        a[SUBJECT].set_state_code(actor_value);
    };
    {
        let actor_value = 0;
        a[SUBJECT].set_local_state_code(actor_value);
    };
    {
        let actor_value = 1;
        a[SUBJECT].set_local_counter_code(actor_value);
    };
    let mut m = Boundary::default();
    DepartureFixture::default().spawner(&mut a, &mut m);
    assert_eq!(a[SUBJECT].local_counter_code(), 0);
    assert_eq!(a[15].state_code(), 0);
    assert_eq!(m.observations[0].local_counter_code(), 0);
}
#[test]
fn spawner_exit_threshold_requires_page_zero_and_postmovement_x_below_seventy() {
    for (x, exit) in [(0x0070, false), (0x006f, true), (0x016f, false)] {
        let mut a = pool();
        a[0].render = ACTIVE;
        a[SUBJECT] = actor();
        {
            let actor_value = WORLD_STATE | 2;
            a[SUBJECT].set_state_code(actor_value);
        };
        {
            let actor_value = 0;
            a[SUBJECT].set_local_state_code(actor_value);
        };
        {
            let actor_value = 2;
            a[SUBJECT].set_local_counter_code(actor_value);
        };
        let p = Position {
            world_x: x,
            ..a[SUBJECT].position
        };
        let mut m = Boundary {
            facts: VecDeque::from([(Some(p), None, false)]),
            ..Default::default()
        };
        DepartureFixture::default().spawner(&mut a, &mut m);
        assert_eq!(
            a[SUBJECT].state_code(),
            if exit {
                WORLD_STATE | 3
            } else {
                WORLD_STATE | 2
            }
        );
        assert_eq!(
            a[SPAWNER_EXIT_SLOT].state_code(),
            if exit { WORLD_STATE } else { 0 }
        );
    }
}
#[test]
fn spawner_exit_actor_overwrites_active_slot_three_and_preserves_unwritten_motion_and_locals() {
    let mut a = pool();
    a[0].render = ACTIVE;
    a[3] = actor();
    a[SUBJECT] = actor();
    {
        let actor_value = WORLD_STATE | 2;
        a[SUBJECT].set_state_code(actor_value);
    };
    {
        let actor_value = 0;
        a[SUBJECT].set_local_state_code(actor_value);
    };
    {
        let actor_value = 2;
        a[SUBJECT].set_local_counter_code(actor_value);
    };
    let mut e = a[3];
    {
        let actor_value = WORLD_STATE;
        e.set_state_code(actor_value);
    };
    e.render = WORLD_STATE;
    e.animation = 0x5a;
    e.position.world_x = 0x30;
    e.position.screen_y = 0x80;
    {
        let actor_value = 0x38;
        e.set_handler(actor_value);
    };
    e.profile = 0;
    e.playback.restart_clock();
    e.playback.reset_frame();
    let p = Position {
        world_x: 0x6f,
        ..a[SUBJECT].position
    };
    let mut m = Boundary {
        facts: VecDeque::from([(Some(p), None, false)]),
        ..Default::default()
    };
    DepartureFixture::default().spawner(&mut a, &mut m);
    assert_eq!(a[3], e);
}
#[test]
fn spawner_upward_exit_clears_only_state_below_ten_after_moving() {
    for (y, active) in [(SPAWNER_EXIT_MARK, true), (SPAWNER_EXIT_MARK - 1, false)] {
        let mut a = pool();
        a[0].render = ACTIVE;
        a[SUBJECT] = actor();
        {
            let actor_value = WORLD_STATE | 3;
            a[SUBJECT].set_state_code(actor_value);
        };
        {
            let actor_value = 0;
            a[SUBJECT].set_local_state_code(actor_value);
        };
        let p = Position {
            screen_y: y,
            ..a[SUBJECT].position
        };
        let mut m = Boundary {
            facts: VecDeque::from([(Some(p), None, false)]),
            ..Default::default()
        };
        DepartureFixture::default().spawner(&mut a, &mut m);
        assert_eq!(m.calls, vec![Integration::SubtractY]);
        assert_eq!(
            a[SUBJECT].state_code(),
            if active { WORLD_STATE | 3 } else { 0 }
        );
        assert_eq!(a[SUBJECT].render, ACTIVE);
        assert_eq!(a[SUBJECT].animation, 0x55);
    }
}
#[test]
fn completion_entry_initializes_slot_eight_only_and_moves_odd_animation_left() {
    let mut a = [actor(); 16];
    let old = a;
    let mut c = Completion::default();
    let mut p = Progression::default();
    let mut d = Display::default();
    let mut gravity = 0;
    let mut m = Boundary::default();
    assert_eq!(
        c.resume(&mut a, &mut p, &mut d, &mut gravity, &mut m),
        CompletionStep::Compose
    );
    assert_eq!(gravity, COMPLETION_GRAVITY);
    assert_eq!(
        m.calls,
        vec![
            Integration::SubtractY,
            Integration::Gravity,
            Integration::SubtractX
        ]
    );
    let first = m.observations[0];
    assert_eq!(
        (first.state_code(), first.render, first.animation),
        (WORLD_STATE, WORLD_STATE, COMPLETION_FIRST)
    );
    assert_eq!(first.position.world_x, COMPLETION_START_X);
    assert_eq!(first.position.screen_y, COMPLETION_START_Y);
    assert_eq!(
        first.position.x_fraction,
        old[COMPLETION_SLOT].position.x_fraction
    );
    assert_eq!(first.vertical_velocity, COMPLETION_UPWARD_SPEED);
    assert_eq!(first.horizontal_magnitude, COMPLETION_HORIZONTAL_SPEED);
    for i in 0..a.len() {
        if i != COMPLETION_SLOT {
            assert_eq!(a[i], old[i]);
        }
    }
}
#[test]
fn completion_await_composition_yields_before_testing_new_visibility() {
    let mut a = pool();
    a[COMPLETION_SLOT] = actor();
    a[COMPLETION_SLOT].render = 0;
    a[COMPLETION_SLOT].animation = COMPLETION_FIRST;
    let mut c = Completion {
        phase: CompletionPhase::AwaitComposition,
        flashes_left: FLASH_CYCLES,
    };
    let mut m = Boundary::default();
    assert_eq!(
        c.resume(
            &mut a,
            &mut Progression::default(),
            &mut Display::default(),
            &mut COMPLETION_GRAVITY.clone(),
            &mut m
        ),
        CompletionStep::Yield
    );
    assert_eq!(a[COMPLETION_SLOT].animation, COMPLETION_FIRST);
    assert!(m.calls.is_empty());
}
#[test]
fn completion_hidden_pose_restarts_next_animation_without_waiting_on_state_active_bit() {
    let mut a = pool();
    a[COMPLETION_SLOT] = actor();
    {
        let actor_value = WORLD_STATE;
        a[COMPLETION_SLOT].set_state_code(actor_value);
    };
    a[COMPLETION_SLOT].render = 0;
    a[COMPLETION_SLOT].animation = COMPLETION_FIRST;
    let mut c = Completion {
        phase: CompletionPhase::AwaitYield,
        flashes_left: FLASH_CYCLES,
    };
    let mut m = Boundary::default();
    c.resume(
        &mut a,
        &mut Progression::default(),
        &mut Display::default(),
        &mut COMPLETION_GRAVITY.clone(),
        &mut m,
    );
    assert_eq!(a[COMPLETION_SLOT].animation, COMPLETION_FIRST + 1);
    assert_eq!(
        m.calls,
        vec![
            Integration::SubtractY,
            Integration::Gravity,
            Integration::AddX
        ]
    );
    assert_eq!(m.observations[0].playback.tick_code(), 0);
    assert_eq!(m.observations[0].playback.frame_code(), 0);
    assert_eq!(m.observations[0].local_counter_code(), 0);
}
#[test]
fn completion_visible_animation_continues_without_reinitializing_pose() {
    let mut a = pool();
    a[COMPLETION_SLOT] = actor();
    a[COMPLETION_SLOT].animation = 0x70;
    a[COMPLETION_SLOT].render = ACTIVE;
    {
        let actor_value = 1;
        a[COMPLETION_SLOT].set_local_counter_code(actor_value);
    };
    let mut c = Completion {
        phase: CompletionPhase::AwaitYield,
        flashes_left: FLASH_CYCLES,
    };
    let mut m = Boundary::default();
    c.resume(
        &mut a,
        &mut Progression::default(),
        &mut Display::default(),
        &mut COMPLETION_GRAVITY.clone(),
        &mut m,
    );
    assert_eq!(
        m.calls,
        vec![
            Integration::Gravity,
            Integration::SubtractY,
            Integration::AddX
        ]
    );
    assert_eq!(m.observations[0].position.world_x, 0x01a0);
    assert_eq!(a[COMPLETION_SLOT].animation, 0x70);
}
#[test]
fn completion_apex_clamps_velocity_before_next_descending_iteration() {
    let mut a = pool();
    a[COMPLETION_SLOT] = actor();
    a[COMPLETION_SLOT].animation = 0x71;
    {
        let actor_value = 0;
        a[COMPLETION_SLOT].set_local_counter_code(actor_value);
    };
    let mut c = Completion {
        phase: CompletionPhase::Moving,
        flashes_left: FLASH_CYCLES,
    };
    let mut m = Boundary {
        facts: VecDeque::from([(None, None, false), (None, Some(-1), false)]),
        ..Default::default()
    };
    c.resume(
        &mut a,
        &mut Progression::default(),
        &mut Display::default(),
        &mut COMPLETION_GRAVITY.clone(),
        &mut m,
    );
    assert_eq!(a[COMPLETION_SLOT].local_counter_code(), 1);
    assert_eq!(a[COMPLETION_SLOT].vertical_velocity, 0);
    assert_eq!(
        m.calls,
        vec![
            Integration::SubtractY,
            Integration::Gravity,
            Integration::SubtractX
        ]
    );
}
#[test]
fn completion_fifth_pose_enters_first_flash_without_clearing_render_or_scene_signal() {
    let mut a = pool();
    a[COMPLETION_SLOT] = actor();
    a[COMPLETION_SLOT].animation = COMPLETION_LAST;
    a[COMPLETION_SLOT].render = 0;
    let mut p = Progression {
        scene_signal: 0x41,
        completion_actor: 0x80,
        impact_latch: 0x77,
        ..Default::default()
    };
    let mut d = Display::default();
    let mut c = Completion {
        phase: CompletionPhase::AwaitYield,
        flashes_left: FLASH_CYCLES,
    };
    assert_eq!(
        c.resume(
            &mut a,
            &mut p,
            &mut d,
            &mut COMPLETION_GRAVITY.clone(),
            &mut Boundary::default()
        ),
        CompletionStep::Wait(FLASH_TICKS)
    );
    assert_eq!(a[COMPLETION_SLOT].animation, COMPLETION_LAST + 1);
    assert_eq!(a[COMPLETION_SLOT].state_code(), 0);
    assert_eq!(a[COMPLETION_SLOT].render, 0);
    assert_eq!(d.active[16], FLASH_COLOR);
    assert!(d.palette_pending);
    assert_eq!(
        (p.scene_signal, p.completion_actor, p.impact_latch),
        (0x41, 0x80, 0x77)
    );
}
#[test]
fn completion_sixteen_color_black_pairs_wait_four_each_and_only_then_clear_completion_latch() {
    let mut a = pool();
    let mut p = Progression {
        scene_signal: 3,
        completion_actor: 1,
        impact_latch: 5,
        ..Default::default()
    };
    let mut d = Display::default();
    let mut c = Completion {
        phase: CompletionPhase::FlashColor,
        flashes_left: FLASH_CYCLES,
    };
    for _ in 0..FLASH_CYCLES {
        assert_eq!(
            c.resume(
                &mut a,
                &mut p,
                &mut d,
                &mut COMPLETION_GRAVITY.clone(),
                &mut Boundary::default()
            ),
            CompletionStep::Wait(FLASH_TICKS)
        );
        assert_eq!(d.active[16], FLASH_COLOR);
        assert_eq!(p.completion_actor, 1);
        assert_eq!(
            c.resume(
                &mut a,
                &mut p,
                &mut d,
                &mut COMPLETION_GRAVITY.clone(),
                &mut Boundary::default()
            ),
            CompletionStep::Wait(FLASH_TICKS)
        );
        assert_eq!(d.active[16], FLASH_BLACK);
        assert_eq!(p.completion_actor, 1);
    }
    assert_eq!(
        c.resume(
            &mut a,
            &mut p,
            &mut d,
            &mut COMPLETION_GRAVITY.clone(),
            &mut Boundary::default()
        ),
        CompletionStep::Complete
    );
    assert_eq!(
        (p.completion_actor, p.scene_signal, p.impact_latch),
        (0, 3, 5)
    );
}
#[test]
fn script_ordinary_values_are_direct_and_zero_is_list_termination() {
    for (b, want) in [(0, None), (1, Some(1)), (0x7f, Some(0x7f))] {
        assert_eq!(script_selector(b, 2, 1), want);
    }
}
#[test]
fn script_negative_nibble_eight_uses_all_three_title_options_independent_of_variation() {
    for option in 0..3 {
        for variation in 0..3 {
            assert_eq!(script_selector(0x88, variation, option), Some(8 + option));
        }
    }
}
#[test]
fn script_other_negative_selectors_add_variation_with_byte_wrap() {
    for (encoded, variation, want) in [(0x83, 0, 3), (0x83, 1, 4), (0x83, 2, 5), (0x8f, 0xff, 14)] {
        assert_eq!(script_selector(encoded, variation, 2), Some(want));
    }
}
#[test]
fn script_zero_variation_sum_falls_through_to_title_addition_and_still_calls_script_zero() {
    assert_eq!(script_selector(0x80, 0, 2), Some(2));
    assert_eq!(script_selector(0x80, 0, 0), Some(0));
    assert_eq!(script_selector(0x81, 0xff, 1), Some(1));
}
const SCENES: [Scene; 6] = [
    Scene::OpeningFirst,
    Scene::OpeningSecond,
    Scene::OpeningThird,
    Scene::Interior,
    Scene::Value,
    Scene::TerminalLead,
];
#[test]
fn all_ending_scene_resources_join_layout_nine_with_explicit_terrain_and_no_spawn_config_nine() {
    for scene in SCENES {
        let i = scene as usize;
        assert_eq!(
            resource_context(scene),
            SceneResourceContext {
                layout: ENDING_LAYOUT,
                terrain: [2, 2, 2, 1, 1, 1][i],
                chr: [9, 9, 9, 9, 10, 10][i],
                spawn_configuration: None
            }
        );
    }
}
struct ContentFixture {
    actors: ActorPool,
    display: Display,
    camera: Camera,
    cursor: StreamCursor,
    scene: u8,
    mode: u8,
    split: u8,
    option: u8,
    variation: u8,
}
impl Default for ContentFixture {
    fn default() -> Self {
        Self {
            actors: [actor(); 16],
            display: Display {
                control: 0x91,
                ..Default::default()
            },
            camera: Camera {
                x: 0x0267,
                room: 5,
                previous_x: 0x12,
            },
            cursor: StreamCursor {
                ring: 31,
                room: 5,
                room_in_row: 4,
            },
            scene: 0,
            mode: GAMEPLAY_DISPLAY,
            split: 0x55,
            option: 1,
            variation: 2,
        }
    }
}
impl ContentFixture {
    fn resume(&mut self, c: &mut Content, r: OwnerResult) -> ContentWork {
        c.resume(
            &catalog(),
            &mut ContentContext {
                actors: &mut self.actors,
                display: &mut self.display,
                camera: &mut self.camera,
                cursor: &mut self.cursor,
                scene_cursor: &mut self.scene,
                display_mode: &mut self.mode,
                split_y: &mut self.split,
                title_option: self.option,
                ending_variation: self.variation,
            },
            r,
        )
    }
}
fn content_work(f: &mut ContentFixture, c: &mut Content) -> Vec<ContentWork> {
    let mut w = vec![];
    for _ in 0..64 {
        let n = f.resume(c, OwnerResult::Complete);
        let done = n == ContentWork::Complete;
        w.push(n);
        if done {
            return w;
        }
    }
    panic!("content continuation exceeded source bounded operations");
}
#[test]
fn content_entry_and_running_fade_leave_all_outgoing_state_intact() {
    let mut f = ContentFixture::default();
    let old = f.actors;
    let mut c = Content::new(Scene::OpeningFirst);
    assert_eq!(
        f.resume(&mut c, OwnerResult::Complete),
        ContentWork::FadeOut
    );
    assert_eq!(
        f.resume(&mut c, OwnerResult::Running),
        ContentWork::Suspended
    );
    assert_eq!(f.actors, old);
    assert_eq!(f.mode, GAMEPLAY_DISPLAY);
    assert_eq!(f.display.control, 0x91);
    assert_eq!(f.cursor.ring, 31);
}
#[test]
fn early_content_builds_two_sixty_four_column_spans_using_scene_three_for_second_span() {
    for scene in &SCENES[..3] {
        let mut f = ContentFixture {
            scene: *scene as u8,
            ..Default::default()
        };
        let mut c = Content::new(*scene);
        let work = content_work(&mut f, &mut c);
        let spans: Vec<_> = work
            .iter()
            .filter(|w| matches!(w, ContentWork::TerrainSpan { .. }))
            .cloned()
            .collect();
        assert_eq!(
            spans,
            vec![
                ContentWork::TerrainSpan {
                    scene: *scene,
                    bank: 2,
                    room_in_row: 0,
                    columns: 64
                },
                ContentWork::TerrainSpan {
                    scene: Scene::Interior,
                    bank: 1,
                    room_in_row: 1,
                    columns: 64
                }
            ]
        );
        assert_eq!(f.scene, *scene as u8);
    }
}
#[test]
fn later_content_builds_one_span_and_never_infers_bank_from_stage_config() {
    for scene in &SCENES[3..] {
        let mut f = ContentFixture::default();
        let mut c = Content::new(*scene);
        let work = content_work(&mut f, &mut c);
        assert_eq!(
            work.iter()
                .filter(|w| matches!(w, ContentWork::TerrainSpan { .. }))
                .cloned()
                .collect::<Vec<_>>(),
            vec![ContentWork::TerrainSpan {
                scene: *scene,
                bank: 1,
                room_in_row: 0,
                columns: 64
            }]
        );
    }
}
#[test]
fn original_scenes_zero_through_three_use_effective_interior_script_list_with_variation_and_option()
{
    for scene in &SCENES[..4] {
        for option in 0..3 {
            for variation in 0..3 {
                let mut f = ContentFixture {
                    option,
                    variation,
                    ..Default::default()
                };
                let mut c = Content::new(*scene);
                let work = content_work(&mut f, &mut c);
                assert_eq!(
                    work.iter()
                        .filter_map(|w| if let ContentWork::ScreenScript(v) = w {
                            Some(*v)
                        } else {
                            None
                        })
                        .collect::<Vec<_>>(),
                    vec![11, 3 + variation, 8 + option]
                );
            }
        }
    }
}
#[test]
fn later_scenes_use_direct_script_six_or_seven_once() {
    for (scene, script) in [(Scene::Value, 6), (Scene::TerminalLead, 7)] {
        let mut f = ContentFixture::default();
        let mut c = Content::new(scene);
        let work = content_work(&mut f, &mut c);
        assert_eq!(
            work.iter()
                .filter_map(|w| if let ContentWork::ScreenScript(v) = w {
                    Some(*v)
                } else {
                    None
                })
                .collect::<Vec<_>>(),
            vec![script]
        );
    }
}
#[test]
fn interior_scene_preserves_actor_clear_path_while_other_scenes_clear_and_hide() {
    for scene in SCENES {
        let mut f = ContentFixture::default();
        let mut c = Content::new(scene);
        let work = content_work(&mut f, &mut c);
        assert_eq!(
            work.contains(&ContentWork::ClearNonplayer),
            scene != Scene::Interior
        );
        assert_eq!(
            work.contains(&ContentWork::HideOam),
            scene != Scene::Interior
        );
        let first_yield = work.iter().position(|w| *w == ContentWork::Yield).unwrap();
        let disable = work
            .iter()
            .position(|w| *w == ContentWork::Disable)
            .unwrap();
        assert!(first_yield < disable);
    }
}
#[test]
fn content_final_enable_yield_defers_split_display_and_camera_page_until_resumption() {
    for (scene, y) in [(Scene::Value, 0x70), (Scene::TerminalLead, 0x90)] {
        let mut f = ContentFixture::default();
        let mut c = Content::new(scene);
        let mut reached = false;
        for _ in 0..64 {
            let w = f.resume(&mut c, OwnerResult::Complete);
            if w == ContentWork::Yield && c.phase == ContentPhase::YieldBeforeDisplay {
                reached = true;
                break;
            }
        }
        assert!(reached);
        assert_eq!((f.camera.x, f.mode, f.split), (0x0267, 0, 0x55));
        assert_eq!(
            f.resume(&mut c, OwnerResult::Running),
            ContentWork::Suspended
        );
        assert_eq!(f.camera.x, 0x0267);
        content_work(&mut f, &mut c);
        assert_eq!((f.camera.x, f.mode, f.split), (0x0167, GAMEPLAY_DISPLAY, y));
    }
}
#[test]
fn early_content_retains_split_position_and_camera_low_and_high_after_display_yield() {
    let mut f = ContentFixture::default();
    let mut c = Content::new(Scene::OpeningSecond);
    content_work(&mut f, &mut c);
    assert_eq!((f.camera.x, f.split, f.mode), (0x0267, 0x55, 0));
}
#[test]
fn content_restores_original_scene_before_actor_composition_and_chr_upload() {
    let mut f = ContentFixture::default();
    let mut c = Content::new(Scene::OpeningThird);
    let work = content_work(&mut f, &mut c);
    let composition = work
        .iter()
        .position(|w| *w == ContentWork::Compose)
        .unwrap();
    let chr = work.iter().position(|w| *w == ContentWork::Chr(9)).unwrap();
    assert!(composition < chr);
    assert_eq!(f.actors[0].animation, 0x83);
    assert_eq!(f.scene, Scene::OpeningThird as u8);
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct SpecialValues {
    display: u8,
    spawn: u8,
    split: u8,
    upper: u8,
    lower: u8,
    start: u8,
    direction: u8,
    terrain: u8,
}
impl Default for SpecialValues {
    fn default() -> Self {
        Self {
            display: GAMEPLAY_DISPLAY,
            spawn: 0x55,
            split: 0x66,
            upper: 0x77,
            lower: 0x88,
            start: 0x99,
            direction: 0,
            terrain: 1,
        }
    }
}
impl SpecialValues {
    fn borrowed(&mut self) -> SpecialControls<'_> {
        SpecialControls {
            display: &mut self.display,
            spawn_control: &mut self.spawn,
            split: &mut self.split,
            upper_cursor: &mut self.upper,
            lower_cursor: &mut self.lower,
            room_start: &mut self.start,
            scan_direction: &mut self.direction,
            terrain_bank: &mut self.terrain,
        }
    }
}
struct SpecialFixture {
    a: ActorPool,
    p: Progression,
    player: Player,
    controls: SpecialValues,
}
impl Default for SpecialFixture {
    fn default() -> Self {
        Self {
            a: [actor(); 16],
            p: Progression {
                special_room: 0x81,
                late_mode: 0xff,
                scene_signal: 5,
                impact_latch: 7,
                ..Default::default()
            },
            player: Player {
                momentum_control: 0x12,
                momentum_direction: 0x34,
                ..Default::default()
            },
            controls: SpecialValues::default(),
        }
    }
}
impl SpecialFixture {
    fn resume(&mut self, s: &mut SpecialSession, r: OwnerResult) -> SpecialWork {
        s.resume(
            &catalog(),
            &mut self.a,
            &mut self.p,
            &mut self.player,
            &mut self.controls.borrowed(),
            r,
        )
    }
}
fn special_work(f: &mut SpecialFixture, s: &mut SpecialSession) -> Vec<SpecialWork> {
    let mut work = vec![];
    for _ in 0..40 {
        let w = f.resume(s, OwnerResult::Complete);
        let done = w == SpecialWork::Complete;
        work.push(w);
        if done {
            return work;
        }
    }
    panic!("special continuation exceeded source bounded operations");
}
#[test]
fn both_actual_negative_special_requests_choose_their_authored_modes() {
    for (request, mode) in [
        (0x81, SpecialRoom::Linked),
        (0x82, SpecialRoom::RewardFlight),
    ] {
        let s = SpecialSession::new(request).unwrap();
        assert_eq!(s.room, mode);
        assert_eq!(s.phase, SpecialPhase::Entry);
    }
}
#[test]
fn special_entry_fade_suspension_preserves_outgoing_pool_player_request_and_controls() {
    let mut f = SpecialFixture::default();
    let old = (f.a, f.player.clone(), f.p.clone(), f.controls.clone());
    let mut s = SpecialSession::new(0x81).unwrap();
    assert_eq!(
        f.resume(&mut s, OwnerResult::Complete),
        SpecialWork::FadeOut
    );
    assert_eq!(
        f.resume(&mut s, OwnerResult::Running),
        SpecialWork::Suspended
    );
    assert_eq!((f.a, f.player, f.p, f.controls), old);
}
#[test]
fn special_reset_precedes_disable_and_clear_but_request_normalization_follows_clear() {
    let mut f = SpecialFixture::default();
    let mut s = SpecialSession::new(0x81).unwrap();
    assert_eq!(
        f.resume(&mut s, OwnerResult::Complete),
        SpecialWork::FadeOut
    );
    assert_eq!(
        f.resume(&mut s, OwnerResult::Complete),
        SpecialWork::Disable
    );
    assert_eq!(f.controls.display, 0);
    assert_eq!(f.controls.spawn, 0);
    assert_eq!(f.controls.split, 0);
    assert_eq!(
        (f.controls.upper, f.controls.lower, f.controls.start),
        (0, 0, 0)
    );
    assert_eq!(f.controls.direction, 1);
    assert_eq!(
        (
            f.player.momentum_control,
            f.player.momentum_direction,
            f.p.late_mode
        ),
        (0, 0, 0)
    );
    assert_eq!(f.p.special_room, 0x81);
    assert_eq!(
        f.resume(&mut s, OwnerResult::Complete),
        SpecialWork::ClearNonplayer
    );
    assert_eq!(f.p.special_room, 0x81);
    assert_eq!(
        f.resume(&mut s, OwnerResult::Complete),
        SpecialWork::Layout(7)
    );
    assert_eq!(f.p.special_room, 1);
    assert_eq!(f.controls.terrain, 3);
}
#[test]
fn special_layout_spawn_and_chr_selectors_are_seven_eight_not_current_gameplay_stage() {
    for (request, selector) in [(0x81, 7), (0x82, 8)] {
        let mut f = SpecialFixture::default();
        f.p.special_room = request;
        f.p.stage = crate::identifiers::StageId(3);
        let mut s = SpecialSession::new(request).unwrap();
        let w = special_work(&mut f, &mut s);
        assert!(w.contains(&SpecialWork::Layout(selector)));
        assert!(w.contains(&SpecialWork::SpawnTables(selector)));
        assert!(w.contains(&SpecialWork::Chr(selector)));
        assert_eq!(f.p.stage, crate::identifiers::StageId(3));
    }
}
#[test]
fn special_checkpoint_uses_eleven_twelve_and_palette_seventy_without_stage_presentation() {
    for (request, checkpoint) in [(0x81, 0x11), (0x82, 0x12)] {
        let mut f = SpecialFixture::default();
        f.p.special_room = request;
        let mut s = SpecialSession::new(request).unwrap();
        let w = special_work(&mut f, &mut s);
        assert!(w.contains(&SpecialWork::Checkpoint(checkpoint)));
        assert!(w.contains(&SpecialWork::Palette(0x70)));
        assert_eq!(f.p.scene_signal, 5);
        assert_eq!(f.p.impact_latch, 7);
    }
}
#[test]
fn special_actor_initialization_precedes_audio_enable_and_yield_for_both_modes() {
    for (request, action, handler) in [
        (0x81, Action::Hold, DEPARTURE_HANDLER),
        (0x82, Action::Standing, SPAWNER_HANDLER),
    ] {
        let mut f = SpecialFixture::default();
        f.p.special_room = request;
        let mut s = SpecialSession::new(request).unwrap();
        let mut reached = false;
        for _ in 0..40 {
            let w = f.resume(&mut s, OwnerResult::Complete);
            if w == SpecialWork::StageAudio(SPECIAL_AUDIO) {
                reached = true;
                break;
            }
        }
        assert!(reached);
        assert_eq!(f.player.action, action);
        assert_eq!(f.a[2].handler, handler);
        assert_eq!(f.a[1], actor());
        assert_eq!(f.resume(&mut s, OwnerResult::Complete), SpecialWork::Enable);
        assert_eq!(f.resume(&mut s, OwnerResult::Complete), SpecialWork::Yield);
        assert_eq!(f.controls.display, 0);
    }
}
#[test]
fn special_yield_suspension_does_not_enable_gameplay_display_or_fade_early() {
    let mut f = SpecialFixture::default();
    let mut s = SpecialSession::new(0x81).unwrap();
    let mut reached = false;
    for _ in 0..40 {
        if f.resume(&mut s, OwnerResult::Complete) == SpecialWork::Yield {
            reached = true;
            break;
        }
    }
    assert!(reached);
    assert_eq!(f.controls.display, 0);
    assert_eq!(
        f.resume(&mut s, OwnerResult::Running),
        SpecialWork::Suspended
    );
    assert_eq!(f.controls.display, 0);
    assert_eq!(f.resume(&mut s, OwnerResult::Complete), SpecialWork::FadeIn);
    assert_eq!(f.controls.display, GAMEPLAY_DISPLAY);
    assert_eq!(
        f.resume(&mut s, OwnerResult::Running),
        SpecialWork::Suspended
    );
    assert_eq!(
        f.resume(&mut s, OwnerResult::Complete),
        SpecialWork::Complete
    );
}
#[test]
fn content_waiting_for_composition_preserves_script_cursor_and_does_not_upload_chr_early() {
    let mut f = ContentFixture::default();
    let mut c = Content::new(Scene::Value);
    let mut reached = false;
    for _ in 0..64 {
        if f.resume(&mut c, OwnerResult::Complete) == ContentWork::Compose {
            reached = true;
            break;
        }
    }
    assert!(reached);
    let old = (c.script_cursor, f.scene, f.actors, f.display.clone());
    assert_eq!(
        f.resume(&mut c, OwnerResult::Running),
        ContentWork::Suspended
    );
    assert_eq!((c.script_cursor, f.scene, f.actors, f.display.clone()), old);
    assert_eq!(
        f.resume(&mut c, OwnerResult::Complete),
        ContentWork::Chr(10)
    );
}
#[test]
fn palette_rebuild_resume_composes_before_setting_palette_nametable_and_settled_handler() {
    let mut a = pool();
    a[0] = actor();
    {
        let actor_value = 3;
        a[0].set_handler(actor_value);
    };
    let mut d = Display {
        control: 0x90,
        ..Default::default()
    };
    let mut k = HandlerContinuation::AwaitRebuild;
    assert_eq!(
        update_scene_actor(
            &mut a,
            &catalog(),
            COMPLETION_GRAVITY,
            &mut Boundary::default(),
            &mut k,
            &mut d
        ),
        HandlerStep::Compose
    );
    assert_eq!(a[0].handler, SceneHandler::Settle as u8);
    assert_eq!(a[1].animation, 0x81);
    assert_eq!(d.control, 0x90);
    assert!(!d.palette_pending);
    assert_eq!(
        update_scene_actor(
            &mut a,
            &catalog(),
            COMPLETION_GRAVITY,
            &mut Boundary::default(),
            &mut k,
            &mut d
        ),
        HandlerStep::Complete
    );
    assert_eq!(a[0].handler, SceneHandler::Settled as u8);
    assert_eq!(d.control, 0x91);
    assert!(d.palette_pending);
}
#[test]
fn all_runtime_frame_aliases_and_every_original_animation_consumer_remain_explicit() {
    let c = runtime_frame_consumers(&rom()).unwrap();
    assert_eq!(c.aliases, [0x04c0; 4]);
    assert_eq!(
        c.animations,
        vec![(0x0a, vec![0]), (0x8c, vec![0]), (0x8d, vec![1])]
    );
}
fn synthetic_changed_upload() -> crate::world::TerrainUpdate {
    use crate::world::{PaletteRun, TerrainUpdate, TileOrigin, TilePage, TileRegion};
    TerrainUpdate {
        region: TileRegion::new(
            TileOrigin {
                page: TilePage::First,
                column: 6,
                row: 10,
            },
            2,
            vec![11, 12, 13, 14],
        ),
        palette: Some(PaletteRun {
            page: TilePage::First,
            first_cell: 17,
            values: vec![0x3f],
        }),
        commit: crate::assets::formats::UploadCommit::AtInterrupt,
    }
}

#[test]
fn source_reset_zeroes_all_four_runtime_aliases_before_any_secondary_writer() {
    let q = crate::presentation::PreparedSecondary::source_zeroed();
    for selector in 0..4 {
        let r = q.runtime_composite(selector);
        assert!(r.len() >= 4);
        assert_eq!(&r[..4], &[0, 0, 0, 0]);
        assert_eq!(
            r.as_ptr(),
            q.encoded[crate::presentation::RUNTIME_COMPOSITE_QUEUE_OFFSET..].as_ptr()
        );
    }
}
#[test]
fn tenth_changed_terrain_record_authors_runtime_frame_header_in_same_canonical_secondary_storage() {
    let mut q = crate::presentation::PreparedSecondary::source_zeroed();
    let mut cursor = 0;
    for _ in 0..10 {
        q.append_changed(&mut cursor, &synthetic_changed_upload());
    }
    assert_eq!(cursor, 140);
    let r = q.runtime_composite(0);
    assert_eq!(
        &r[..13],
        &[1, 11, 12, 0x21, 0x66, 1, 13, 14, 0x23, 0xd1, 0, 0x3f, 0xff]
    );
    assert_eq!(
        r.as_ptr(),
        q.encoded[crate::presentation::RUNTIME_COMPOSITE_QUEUE_OFFSET..].as_ptr()
    );
    for selector in 1..4 {
        assert_eq!(q.runtime_composite(selector).as_ptr(), r.as_ptr());
    }
}
#[test]
fn eleventh_changed_record_overwrites_previous_sentinel_without_resetting_runtime_header() {
    let mut q = crate::presentation::PreparedSecondary::source_zeroed();
    let mut cursor = 0;
    for _ in 0..11 {
        q.append_changed(&mut cursor, &synthetic_changed_upload());
    }
    let r = q.runtime_composite(1);
    assert_eq!(
        &r[..15],
        &[
            1, 11, 12, 0x21, 0x66, 1, 13, 14, 0x23, 0xd1, 0, 0x3f, 0x21, 0x46, 1
        ]
    );
    assert_eq!(cursor, 154);
}
#[test]
fn secondary_changed_cursor_wraps_by_fourteen_while_runtime_inspection_remains_read_only() {
    let mut q = crate::presentation::PreparedSecondary::source_zeroed();
    let mut cursor = 252;
    q.append_changed(&mut cursor, &synthetic_changed_upload());
    assert_eq!(cursor, 10);
    // Absolute base+X+field addressing is not truncated to the cursor byte.
    assert!(q.encoded.len() >= 267);
    assert_eq!(
        &q.encoded[252..267],
        &[
            0x21, 0x46, 1, 11, 12, 0x21, 0x66, 1, 13, 14, 0x23, 0xd1, 0, 0x3f, 0xff
        ]
    );
    let before = q.encoded.clone();
    let _ = q.runtime_composite(2);
    let _ = q.runtime_composite(3);
    assert_eq!(q.encoded, before);
}

#[test]
fn hidden_player_spawner_applies_common_life_loss_immediately_without_waiting_for_frame_tail() {
    let mut a = pool();
    a[0] = actor();
    a[0].render = 0x7f;
    a[SUBJECT] = actor();
    let source = a[SUBJECT];
    let mut f = DepartureFixture::default();
    f.progression.stage = crate::identifiers::StageId(2);
    f.progression.life_loss_variant = 5;
    f.player.action = Action::Hold;
    f.player.queued = Some(Action::Damage);
    f.status.health.alternating = 0xff;
    let mut m = Boundary::default();
    assert_eq!(f.spawner(&mut a, &mut m), DepartureResult::LifeLoss);
    assert_eq!(f.progression.reload_request, 0x8e);
    assert_eq!(f.player.action, Action::Standing);
    assert_eq!(f.player.queued, None);
    assert_eq!(a[0].state_code(), 0);
    assert_eq!(a[0].render, 0x7f);
    assert_eq!(a[SUBJECT], source);
    assert_eq!(f.status.health.alternating, 0);
    assert_eq!(f.status.lives, 3);
    assert!(m.calls.is_empty());
}
#[test]
fn hidden_player_common_life_loss_preserves_first_stage_addition_carry_into_variant() {
    let mut a = pool();
    let mut f = DepartureFixture::default();
    f.progression.stage = crate::identifiers::StageId(0xff);
    f.progression.life_loss_variant = 1;
    f.spawner(&mut a, &mut Boundary::default());
    assert_eq!(f.progression.reload_request, 0x88);
}
#[test]
fn composed_secondary_overflow_updates_the_canonical_world_attribute_cache() {
    let mut q = crate::presentation::PreparedSecondary::source_shared();
    let mut cache = crate::world::WorldCache::default();
    let mut cursor = 252;
    q.append_changed_shared(
        &mut cursor,
        &synthetic_changed_upload(),
        &mut cache.attributes,
    );
    assert_eq!(cursor, 10);
    assert_eq!(
        &cache.attributes[60..75],
        &[
            0x21, 0x46, 1, 11, 12, 0x21, 0x66, 1, 13, 14, 0x23, 0xd1, 0, 0x3f, 0xff
        ]
    );
    assert_eq!(q.encoded.len(), crate::presentation::SECONDARY_PREFIX_BYTES);
    cache.attributes[64] = 0x79;
    assert_eq!(q.shared_bytes(&cache.attributes)[256], 0x79);
}

#[test]
fn scene_delay_preserves_every_initialized_progress_code_and_exact_move_entry_reset() {
    // B48F..B4B4: any nonzero progress is initialized. Expiry writes exactly80,
    // handler3, count0 and frozen tick, retaining frame and the other local byte.
    for progress in 0..16u8 {
        for remaining in [0u8, 1, 2, 0xff] {
            let mut actors = pool();
            actors[0] = actor();
            actors[0].set_handler(SceneHandler::Delay as u8);
            actors[0].set_state_code(0xb0 | progress);
            actors[0].set_local_state_code(remaining);
            let mut motion = Boundary::default();
            update(&mut actors, &mut motion);
            assert!(motion.calls.is_empty());
            let expected = if progress == 0 {
                (0xb1, 0, SceneHandler::Delay as u8)
            } else if remaining == 1 {
                (0x80, 0, SceneHandler::MoveToMark as u8)
            } else {
                (
                    0xb0 | progress,
                    remaining.wrapping_sub(1),
                    SceneHandler::Delay as u8,
                )
            };
            assert_eq!(
                (
                    actors[0].state_code(),
                    actors[0].local_state_code(),
                    actors[0].handler
                ),
                expected
            );
            assert_eq!(actors[0].playback.frame_code(), 5);
            assert_eq!(actors[0].local_counter_code(), 8);
        }
    }
}

#[test]
fn native_scene_control_preserves_all_state_codes_and_partial_handler_changes() {
    for handler in 0..=5u8 {
        for code in 0..=u8::MAX {
            for count in [0, 1, 0xff] {
                let mut a = actor();
                a.set_handler(handler);
                a.set_state_code(code);
                a.set_local_state_code(count);
                a.enter_scene();
                assert_eq!(a.control.codes(), (code, count, 8));
                assert_eq!(a.scene_mut().handler_code(), handler);
                a.set_state_code(code.wrapping_add(0x15));
                assert_eq!(a.control.codes(), (code.wrapping_add(0x15), count, 8));
                a.set_handler(0x19);
                a.select_behavior();
                assert_eq!(a.control.codes(), (code.wrapping_add(0x15), count, 8));
                a.set_handler(handler);
                a.enter_scene();
                assert_eq!(a.control.codes(), (code.wrapping_add(0x15), count, 8));
            }
        }
    }
}

#[test]
fn native_bounce_compatibility_progress_retains_source_motion_order() {
    // B42B uses bit1 to select the gravity/Y ordering. Noncanonical source
    // progress remains lossless even though native play uses Rising/Falling.
    for progress in 0..16u8 {
        let mut a = pool();
        a[0] = actor();
        a[0].set_handler(SceneHandler::Bounce as u8);
        a[0].set_state_code(0xb0 | progress);
        a[0].set_local_state_code(2);
        a[0].position.screen_y = BOUNCE_FLOOR;
        let descending = progress & 2 != 0;
        let mut m = Boundary {
            facts: VecDeque::from(if descending {
                [(None, Some(-1), false), (None, None, false)]
            } else {
                [(None, None, false), (None, Some(-1), false)]
            }),
            ..Default::default()
        };
        update(&mut a, &mut m);
        assert_eq!(
            m.calls,
            if descending {
                vec![Integration::Gravity, Integration::SubtractY]
            } else {
                vec![Integration::SubtractY, Integration::Gravity]
            }
        );
        assert_eq!(
            a[0].state_code(),
            if descending {
                (0xb0 | progress) - 1
            } else if progress == 0 {
                0xb2
            } else {
                (0xb0 | progress) + 1
            }
        );
        assert_eq!(
            a[0].local_state_code(),
            if descending {
                2
            } else if progress == 0 {
                BOUNCE_COUNT - 1
            } else {
                1
            }
        );
    }
}

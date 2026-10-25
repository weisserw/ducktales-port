//! Source-derived B13 policies and owner-order contracts; see trace.md.
//! RecordingSystems supplies only boundary results and immediate external writes;
//! branch selection, dispatch, reset/application and sequencing are real B13 code.
use super::*;
use crate::{
    actors::{ACTOR_SLOTS, ActorContact},
    collision::Position,
    player::Action,
    runtime::Buttons,
    status::{Health, HealthSegment, OVERLAY_SEGMENTS},
};

const ORDINARY_STAGES: usize = 5;
const ALL_LANDS: u8 = 0x1f;
const REQUEST_NEGATIVE: u8 = 0x80;
const PLAYER_ACTIVE: u8 = 0xa0;
const PLAYER_START_Y: u8 = 0x40;
const INITIAL_JUMP: i16 = 0x04bb;
const INITIAL_SPEED: u16 = 0x0180;
const CAMERA_ROW: u8 = 3;
const CAMERA_PAGE: u8 = 4;
const SECONDARY_BYTES: u8 = 0x27;
const OLD_PENDING: u8 = 0x91;
const LOW_CAMERA: u8 = 0x43;
const MARKER: u8 = 0x59;
const LIVE_ANIMATION_TICK: u8 = 0x71;
fn input(pressed: Buttons) -> PortInput {
    PortInput {
        pressed,
        held: pressed,
    }
}
fn status() -> Status {
    Status {
        reward: [0; 6],
        total: [0; 7],
        lives: 3,
        health: Health {
            segments: [HealthSegment {
                y: 0,
                tile: 0,
                depleted: 0,
                x: 0,
            }; OVERLAY_SEGMENTS],
            end: 0xff,
            alternating: 0,
        },
        countdown: [5, 0, 0],
        divider: 60,
        dirty: 0,
        ui_pending: 0,
    }
}
struct Fixture {
    p: Progression,
    player: Player,
    actors: ActorPool,
    camera: Camera,
    cache: WorldCache,
    status: Status,
    displacement: u8,
    cursor: u8,
    pending: u8,
    guard: u8,
    row: u8,
    catalog: Catalog,
}
impl Fixture {
    fn new() -> Self {
        Self {
            p: Progression::default(),
            player: Player::default(),
            actors: [ActorContact::default(); ACTOR_SLOTS],
            camera: Camera {
                x: u16::from(LOW_CAMERA),
                room: 0,
                previous_x: MARKER,
            },
            cache: WorldCache::default(),
            status: status(),
            displacement: MARKER,
            cursor: SECONDARY_BYTES,
            pending: OLD_PENDING,
            guard: 0,
            row: 0,
            catalog: catalog(),
        }
    }
    fn frame(&mut self, pressed: Buttons, systems: &mut Recorder) -> FrameExit {
        foreground(
            &mut Frame {
                progression: &mut self.p,
                player: &mut self.player,
                actors: &mut self.actors,
                camera: &mut self.camera,
                cache: &mut self.cache,
                status: &mut self.status,
                camera_displacement: &mut self.displacement,
                secondary_cursor: &mut self.cursor,
                secondary_pending: &mut self.pending,
                oam_guard: &mut self.guard,
                room_row: &mut self.row,
                catalog: &self.catalog,
            },
            input(pressed),
            systems,
        )
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Seen {
    Player,
    Work(Work),
    GuardAtCompose(u8),
    GuardAtSpawn(u8),
}
struct Recorder {
    events: Vec<Seen>,
    increment: FirstActorIncrement,
    player_dialogue: Option<u8>,
    actor_scene: Option<u8>,
    actor_request: Option<u8>,
    player_action: Option<Action>,
    composition_ending: Option<u8>,
    composition_race: Option<u8>,
    reload_dirty: Option<u8>,
    status_ui: Option<u8>,
    ui_special: Option<u8>,
    suspend: Option<Work>,
    suspend_player: bool,
}
impl Recorder {
    fn new() -> Self {
        Self {
            events: vec![],
            increment: FirstActorIncrement::Zero,
            player_dialogue: None,
            actor_scene: None,
            actor_request: None,
            player_action: None,
            composition_ending: None,
            composition_race: None,
            reload_dirty: None,
            status_ui: None,
            ui_special: None,
            suspend: None,
            suspend_player: false,
        }
    }
    fn work(&self) -> Vec<Work> {
        self.events
            .iter()
            .filter_map(|s| {
                if let Seen::Work(w) = s {
                    Some(*w)
                } else {
                    None
                }
            })
            .collect()
    }
}
impl Systems for Recorder {
    fn player(&mut self, f: &mut Frame<'_>) -> OwnerStep<PlayerReturn> {
        self.events.push(Seen::Player);
        if self.suspend_player {
            self.suspend_player = false;
            return OwnerStep::Suspended;
        }
        if let Some(cursor) = self.player_dialogue {
            f.progression.dialogue_cursor = cursor;
        }
        if let Some(action) = self.player_action {
            f.player.action = action;
        }
        OwnerStep::Complete(PlayerReturn {
            first_actor_increment: self.increment,
        })
    }
    fn perform(&mut self, w: Work, f: &mut Frame<'_>) -> OwnerStep<()> {
        self.events.push(Seen::Work(w));
        if self.suspend == Some(w) {
            self.suspend = None;
            return OwnerStep::Suspended;
        }
        match w {
            Work::Actors(_) => {
                if let Some(v) = self.actor_scene {
                    f.progression.scene_signal = v;
                }
                if let Some(v) = self.actor_request {
                    f.progression.boundary_request = v;
                }
            }
            Work::Compose => {
                self.events.push(Seen::GuardAtCompose(*f.oam_guard));
                if let Some(v) = self.composition_ending {
                    f.progression.ending_latch = v;
                }
                if let Some(v) = self.composition_race {
                    f.progression.race_intro = v;
                }
            }
            Work::Spawn => self.events.push(Seen::GuardAtSpawn(*f.oam_guard)),
            Work::Reload => {
                if let Some(v) = self.reload_dirty {
                    f.status.dirty = v;
                }
            }
            Work::RefreshStatus => {
                if let Some(v) = self.status_ui {
                    f.status.ui_pending = v;
                }
            }
            Work::StageUi => {
                if let Some(v) = self.ui_special {
                    f.progression.special_room = v;
                }
            }
            _ => {}
        }
        OwnerStep::Complete(())
    }
}
fn catalog() -> Catalog {
    let stages = (0..STAGE_CONFIGURATION_COUNT)
        .map(|_| StagePreset {
            terrain_bank: 0,
            camera_room: 0,
            row: 0,
            player_x: 0,
            countdown_pair: [5, 0],
            background_palette: [0; 16],
        })
        .collect();
    Catalog {
        stages,
        sprite_palette: [0; 16],
        checkpoints: vec![
            Checkpoint {
                row: 0,
                camera_x: 0,
                camera_page: 0,
                column_count: 0,
                room: 0,
                column: 0,
                player_y: 0,
                player_x: 0,
                packed_page: 0
            };
            CHECKPOINT_COUNT
        ],
        game_over_overlay: vec![],
        transition_rows: [1, 3, 2, 2, 5],
        transition_pages: [4, 5, 5, 4, 4],
        transition_directions: [2, 2, 2, 1, 2],
        late_rows: [2, 4, 3, 4, 1, 0xff, 0xff, 0xff, 0xff],
        late_pages: [1, 5, 5, 7, 2, 0xff, 0xff, 0xff, 0xff],
    }
}
fn rom() -> Vec<u8> {
    std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom"))
        .expect("local matching user ROM")
}

#[test]
fn ordinary_frame_runs_player_actors_countdown_camera_boundary_composition_spawn_rng_yield_in_order()
 {
    let mut f = Fixture::new();
    let mut s = Recorder::new();
    assert_eq!(f.frame(Buttons::NONE, &mut s), FrameExit::NextFrame);
    assert_eq!(
        s.events,
        vec![
            Seen::Player,
            Seen::Work(Work::ActorBank),
            Seen::Work(Work::Actors(FirstActorIncrement::Zero)),
            Seen::Work(Work::Countdown),
            Seen::Work(Work::Camera),
            Seen::Work(Work::StageBoundary),
            Seen::Work(Work::HideSprites),
            Seen::Work(Work::PresentationBank),
            Seen::Work(Work::Compose),
            Seen::GuardAtCompose(1),
            Seen::Work(Work::ActorBank),
            Seen::Work(Work::Spawn),
            Seen::GuardAtSpawn(0),
            Seen::Work(Work::AdvanceRandom),
            Seen::Work(Work::Yield)
        ]
    );
    assert_eq!(
        (f.camera.previous_x, f.displacement, f.cursor, f.pending),
        (LOW_CAMERA, 0, 0, SECONDARY_BYTES)
    );
}
#[test]
fn both_explicit_player_return_increments_reach_first_actor_dispatch_without_false_default() {
    for increment in [FirstActorIncrement::Zero, FirstActorIncrement::One] {
        let mut f = Fixture::new();
        let mut s = Recorder::new();
        s.increment = increment;
        f.frame(Buttons::NONE, &mut s);
        assert!(s.work().contains(&Work::Actors(increment)));
        assert_eq!(s.events.iter().filter(|e| **e == Seen::Player).count(), 1);
    }
}
#[test]
fn pause_start_toggles_before_composition_and_nonzero_skips_simulation_but_submits_secondary() {
    let mut f = Fixture::new();
    let mut s = Recorder::new();
    f.cache.changed_this_frame = MARKER;
    f.player.queued = Some(Action::TerrainPose);
    f.frame(Buttons::START, &mut s);
    assert_eq!(f.p.status_selector, 1);
    assert_eq!(
        s.work(),
        vec![
            Work::Sound(STATUS_TOGGLE_SOUND),
            Work::HideSprites,
            Work::PresentationBank,
            Work::Compose,
            Work::StatusStreams,
            Work::AdvanceRandom,
            Work::Yield
        ]
    );
    assert_eq!(
        (
            f.camera.previous_x,
            f.displacement,
            f.cache.changed_this_frame
        ),
        (MARKER, MARKER, MARKER)
    );
    assert_eq!(f.player.queued, Some(Action::TerrainPose));
    assert_eq!((f.cursor, f.pending), (0, SECONDARY_BYTES));
}
#[test]
fn unpause_zero_continues_simulation_after_status_composition_in_same_invocation() {
    let mut f = Fixture::new();
    f.p.status_selector = 1;
    let mut s = Recorder::new();
    f.frame(Buttons::START, &mut s);
    assert_eq!(f.p.status_selector, 0);
    assert_eq!(
        s.events
            .iter()
            .filter(|e| matches!(e, Seen::Work(Work::Compose)))
            .count(),
        2
    );
    let status = s
        .events
        .iter()
        .position(|e| *e == Seen::Work(Work::StatusStreams))
        .unwrap();
    let player = s.events.iter().position(|e| *e == Seen::Player).unwrap();
    assert!(status < player);
}
#[test]
fn held_start_without_edge_does_not_toggle_and_all_nonzero_status_bytes_gate_player() {
    for selector in 1..=u8::MAX {
        let mut f = Fixture::new();
        f.p.status_selector = selector;
        let mut s = Recorder::new();
        foreground(
            &mut Frame {
                progression: &mut f.p,
                player: &mut f.player,
                actors: &mut f.actors,
                camera: &mut f.camera,
                cache: &mut f.cache,
                status: &mut f.status,
                camera_displacement: &mut f.displacement,
                secondary_cursor: &mut f.cursor,
                secondary_pending: &mut f.pending,
                oam_guard: &mut f.guard,
                room_row: &mut f.row,
                catalog: &f.catalog,
            },
            PortInput {
                pressed: Buttons::NONE,
                held: Buttons::START,
            },
            &mut s,
        );
        assert_eq!(f.p.status_selector, selector);
        assert!(!s.events.contains(&Seen::Player));
        assert_eq!(s.work(), vec![Work::AdvanceRandom, Work::Yield]);
    }
}
#[test]
fn pause_xor_preserves_high_bits_instead_of_normalizing_to_boolean() {
    let mut f = Fixture::new();
    f.p.status_selector = 0x80;
    let mut s = Recorder::new();
    f.frame(Buttons::START, &mut s);
    assert_eq!(f.p.status_selector, 0x81);
    assert!(!s.events.contains(&Seen::Player));
}
#[test]
fn dialogue_action_bypasses_pause_and_global_selector_even_on_start_edge() {
    let mut f = Fixture::new();
    f.p.status_selector = 1;
    f.player.action = Action::Dialogue;
    let mut s = Recorder::new();
    f.frame(Buttons::START, &mut s);
    assert_eq!(f.p.status_selector, 1);
    assert_eq!(s.events.first(), Some(&Seen::Player));
    assert!(!s.work().contains(&Work::Sound(STATUS_TOGGLE_SOUND)));
    assert!(s.work().contains(&Work::Actors(FirstActorIncrement::Zero)));
}
#[test]
fn dialogue_cursor_is_reread_after_player_and_suppresses_actors_timer_and_boundaries_only() {
    let mut f = Fixture::new();
    let mut s = Recorder::new();
    s.player_dialogue = Some(1);
    f.status.dirty = 1;
    f.frame(Buttons::NONE, &mut s);
    assert_eq!(
        s.work(),
        vec![
            Work::RefreshStatus,
            Work::HideSprites,
            Work::PresentationBank,
            Work::Compose,
            Work::ActorBank,
            Work::Spawn,
            Work::AdvanceRandom,
            Work::Yield
        ]
    );
    assert_eq!(s.events.first(), Some(&Seen::Player));
}
#[test]
fn cleared_dialogue_cursor_allows_actor_dispatch_despite_nonzero_entry_snapshot() {
    let mut f = Fixture::new();
    f.p.dialogue_cursor = 1;
    let mut s = Recorder::new();
    s.player_dialogue = Some(0);
    f.frame(Buttons::NONE, &mut s);
    assert!(s.work().contains(&Work::Countdown));
}
#[test]
fn scene_signal_written_by_live_actor_is_observed_after_countdown_before_boundary() {
    let mut f = Fixture::new();
    let mut s = Recorder::new();
    s.actor_scene = Some(1);
    assert_eq!(f.frame(Buttons::NONE, &mut s), FrameExit::LandSelection);
    assert_eq!(
        s.work(),
        vec![
            Work::ActorBank,
            Work::Actors(FirstActorIncrement::Zero),
            Work::Countdown,
            Work::Selection
        ]
    );
}
#[test]
fn negative_late_mode_skips_camera_boundary_but_keeps_simulation_deferred_and_spawn() {
    for late in [0x80, 0xff] {
        let mut f = Fixture::new();
        f.p.late_mode = late;
        let mut s = Recorder::new();
        f.frame(Buttons::NONE, &mut s);
        let w = s.work();
        assert!(w.contains(&Work::Countdown));
        assert!(!w.contains(&Work::Camera));
        assert!(!w.contains(&Work::StageBoundary));
        assert!(w.contains(&Work::Spawn));
    }
}
#[test]
fn positive_late_mode_does_not_freeze_boundary_work() {
    for late in [0, 1, 0x7f] {
        let mut f = Fixture::new();
        f.p.late_mode = late;
        let mut s = Recorder::new();
        f.frame(Buttons::NONE, &mut s);
        assert!(s.work().contains(&Work::Camera));
        assert!(s.work().contains(&Work::StageBoundary));
    }
}
#[test]
fn negative_boundary_request_clears_then_enters_land_select_without_old_frame_rng() {
    for request in [0x80, 0xff] {
        let mut f = Fixture::new();
        let mut s = Recorder::new();
        s.actor_request = Some(request);
        assert_eq!(f.frame(Buttons::NONE, &mut s), FrameExit::LandSelection);
        assert_eq!(f.p.boundary_request, 0);
        assert_eq!(s.work().last(), Some(&Work::Selection));
        assert!(!s.work().contains(&Work::AdvanceRandom));
    }
}
#[test]
fn positive_boundary_request_clears_actors_and_enters_complete_stage_one_confirmation() {
    for request in [1, 0x7f] {
        let mut f = Fixture::new();
        f.p.stage = StageId(4);
        let mut s = Recorder::new();
        s.actor_request = Some(request);
        assert_eq!(
            f.frame(Buttons::NONE, &mut s),
            FrameExit::StageOneConfirmation
        );
        assert_eq!((f.p.stage, f.p.boundary_request), (StageId(1), 0));
        assert!(
            s.work()
                .ends_with(&[Work::ClearActors, Work::ConfirmStageOne])
        );
        assert!(!s.work().contains(&Work::Spawn));
        assert!(!s.work().contains(&Work::AdvanceRandom));
    }
}
#[test]
fn deferred_gates_observe_prior_owner_writes_in_reload_status_ui_special_order() {
    let mut f = Fixture::new();
    f.p.reload_request = 0x80;
    let mut s = Recorder::new();
    s.reload_dirty = Some(1);
    s.status_ui = Some(1);
    s.ui_special = Some(0x81);
    f.frame(Buttons::NONE, &mut s);
    let w = s.work();
    let start = w.iter().position(|x| *x == Work::Reload).unwrap();
    assert_eq!(
        &w[start..start + 4],
        &[
            Work::Reload,
            Work::RefreshStatus,
            Work::StageUi,
            Work::SpecialRoom
        ]
    );
}
#[test]
fn positive_reload_and_special_request_do_not_enter_negative_only_deferred_helpers() {
    for request in [0, 1, 0x7f] {
        let mut f = Fixture::new();
        f.p.reload_request = request;
        f.p.special_room = request;
        let mut s = Recorder::new();
        f.frame(Buttons::NONE, &mut s);
        assert!(!s.work().contains(&Work::Reload));
        assert!(!s.work().contains(&Work::SpecialRoom));
    }
}
#[test]
fn composition_guard_balances_wrapping_and_spawn_observes_released_guard() {
    let mut f = Fixture::new();
    f.guard = 0xff;
    let mut s = Recorder::new();
    f.frame(Buttons::NONE, &mut s);
    assert!(s.events.contains(&Seen::GuardAtCompose(0)));
    assert!(s.events.contains(&Seen::GuardAtSpawn(0xff)));
    assert_eq!(f.guard, 0xff);
}
#[test]
fn ending_entry_bypasses_player_then_composes_and_reaches_only_rng_yield_tail() {
    let mut f = Fixture::new();
    f.p.ending_latch = 1;
    f.player.queued = Some(Action::Damage);
    let mut s = Recorder::new();
    f.frame(Buttons::START, &mut s);
    assert_eq!(
        s.work(),
        vec![
            Work::Ending,
            Work::HideSprites,
            Work::PresentationBank,
            Work::Compose,
            Work::AdvanceRandom,
            Work::Yield
        ]
    );
    assert_eq!(
        (f.cursor, f.pending, f.camera.previous_x, f.displacement),
        (SECONDARY_BYTES, OLD_PENDING, MARKER, MARKER)
    );
    assert_eq!(f.player.queued, Some(Action::Damage));
}
#[test]
fn ending_and_race_are_reread_after_composition_and_bypass_all_ordinary_tail_effects() {
    for ending in [false, true] {
        let mut f = Fixture::new();
        f.cache.changed_this_frame = MARKER;
        f.player.queued = Some(Action::TerrainPose);
        let mut s = Recorder::new();
        if ending {
            s.composition_ending = Some(1);
        } else {
            s.composition_race = Some(1);
        }
        f.frame(Buttons::NONE, &mut s);
        assert!(!s.work().contains(&Work::Spawn));
        assert_eq!(
            (
                f.camera.previous_x,
                f.displacement,
                f.cache.changed_this_frame
            ),
            (MARKER, MARKER, MARKER)
        );
        assert_eq!((f.cursor, f.pending), (SECONDARY_BYTES, OLD_PENDING));
        assert_eq!(f.player.queued, Some(Action::TerrainPose));
        assert!(s.work().ends_with(&[Work::AdvanceRandom, Work::Yield]));
    }
}
#[test]
fn race_intro_at_entry_does_not_suppress_player_actors_or_countdown() {
    let mut f = Fixture::new();
    f.p.race_intro = 1;
    let mut s = Recorder::new();
    f.frame(Buttons::NONE, &mut s);
    assert_eq!(s.events.first(), Some(&Seen::Player));
    assert!(s.work().contains(&Work::Countdown));
    assert!(!s.work().contains(&Work::Spawn));
}
#[test]
fn ordinary_tail_clears_displacement_and_terrain_but_retains_external_motion_latch() {
    let mut f = Fixture::new();
    f.player.external_motion = MARKER;
    f.cache.changed_this_frame = MARKER;
    let mut s = Recorder::new();
    f.frame(Buttons::NONE, &mut s);
    assert_eq!(
        (
            f.displacement,
            f.cache.changed_this_frame,
            f.player.external_motion
        ),
        (0, 0, MARKER)
    );
}
#[test]
fn zero_secondary_cursor_preserves_previous_pending_and_random_advances_exactly_once() {
    let mut f = Fixture::new();
    f.cursor = 0;
    let mut s = Recorder::new();
    f.frame(Buttons::NONE, &mut s);
    assert_eq!(f.pending, OLD_PENDING);
    assert_eq!(
        s.work()
            .iter()
            .filter(|w| **w == Work::AdvanceRandom)
            .count(),
        1
    );
    assert_eq!(s.work().iter().filter(|w| **w == Work::Yield).count(), 1);
}
#[test]
fn all_queued_actions_except_damage_override_even_negative_protection_and_always_consume_queue() {
    for action in [
        Action::Standing,
        Action::Airborne,
        Action::ForcedJump,
        Action::Climbing,
        Action::Swing,
        Action::Protected,
        Action::TerrainPose,
        Action::FallOut,
        Action::WaitA,
        Action::CarriedLocked,
        Action::CarriedJumpable,
        Action::ExternalLaunch,
        Action::Death,
        Action::Hold,
        Action::Dialogue,
        Action::ConvertTimer,
    ] {
        // Source queue0 denotes no request: the native Some(Standing) is normalized
        // to absence, preserving current action. All nonzero requests commit.
        let mut p = Player {
            action: Action::Swing,
            queued: Some(action),
            protection: 0xff,
            ..Player::default()
        };
        apply_queued_reaction(&mut p);
        assert_eq!(p.queued, None);
        assert_eq!(
            p.action,
            if action == Action::Standing {
                Action::Swing
            } else {
                action
            }
        );
    }
}
#[test]
fn damage_queue_is_consumed_before_rejection_for_every_nonzero_protection_byte() {
    for protection in 0..=u8::MAX {
        let mut p = Player {
            action: Action::Swing,
            queued: Some(Action::Damage),
            protection,
            ..Player::default()
        };
        apply_queued_reaction(&mut p);
        assert_eq!(p.queued, None);
        assert_eq!(
            p.action,
            if protection == 0 {
                Action::Damage
            } else {
                Action::Swing
            }
        );
    }
}
#[test]
fn no_queued_reaction_preserves_immediate_collision_action() {
    let mut p = Player {
        action: Action::Damage,
        ..Player::default()
    };
    apply_queued_reaction(&mut p);
    assert_eq!(p.action, Action::Damage);
}

#[test]
fn suspended_player_continuation_does_not_run_actors_or_rng_and_resume_keeps_its_input_result() {
    let mut f = Fixture::new();
    let mut s = Recorder::new();
    s.suspend_player = true;
    s.increment = FirstActorIncrement::One;
    assert_eq!(f.frame(Buttons::NONE, &mut s), FrameExit::Suspended);
    assert_eq!(s.events, vec![Seen::Player]);
    assert_eq!(f.p.continuation, FrameContinuation::Player);
    assert_eq!((f.cursor, f.pending), (SECONDARY_BYTES, OLD_PENDING));
    assert_eq!(f.frame(Buttons::START, &mut s), FrameExit::NextFrame);
    assert_eq!(f.p.status_selector, 0); // START was sampled during owner's yield, not a new DB40 entry.
    assert!(s.work().contains(&Work::Actors(FirstActorIncrement::One)));
    assert_eq!(
        s.work()
            .iter()
            .filter(|w| **w == Work::AdvanceRandom)
            .count(),
        1
    );
}
#[test]
fn suspended_reload_keeps_postreload_status_composition_spawn_and_rng_pending_until_return() {
    let mut f = Fixture::new();
    f.p.reload_request = REQUEST_NEGATIVE;
    let mut s = Recorder::new();
    s.suspend = Some(Work::Reload);
    assert_eq!(f.frame(Buttons::NONE, &mut s), FrameExit::Suspended);
    assert_eq!(s.work().last(), Some(&Work::Reload));
    assert!(!s.work().contains(&Work::Compose));
    let players = s.events.iter().filter(|e| **e == Seen::Player).count();
    assert_eq!(f.frame(Buttons::NONE, &mut s), FrameExit::NextFrame);
    assert_eq!(
        s.events.iter().filter(|e| **e == Seen::Player).count(),
        players
    );
    assert_eq!(
        s.work()
            .iter()
            .filter(|w| **w == Work::AdvanceRandom)
            .count(),
        1
    );
}
#[test]
fn suspended_composition_retains_dma_guard_and_queued_reaction_until_owner_returns() {
    let mut f = Fixture::new();
    f.player.queued = Some(Action::TerrainPose);
    let mut s = Recorder::new();
    s.suspend = Some(Work::Compose);
    assert_eq!(f.frame(Buttons::NONE, &mut s), FrameExit::Suspended);
    assert_eq!(f.guard, 1);
    assert_eq!(f.player.queued, Some(Action::TerrainPose));
    assert!(!s.work().contains(&Work::Spawn));
    assert_eq!(f.frame(Buttons::NONE, &mut s), FrameExit::NextFrame);
    assert_eq!(f.guard, 0);
    assert_eq!(f.player.action, Action::TerrainPose);
    assert_eq!(f.player.queued, None);
}
#[test]
fn yielded_frame_resumption_does_not_advance_rng_twice_or_run_a_new_input_gate() {
    let mut f = Fixture::new();
    let mut s = Recorder::new();
    s.suspend = Some(Work::Yield);
    assert_eq!(f.frame(Buttons::NONE, &mut s), FrameExit::Suspended);
    assert_eq!(f.p.continuation, FrameContinuation::Yield);
    assert_eq!(f.frame(Buttons::START, &mut s), FrameExit::NextFrame);
    assert_eq!(
        s.work()
            .iter()
            .filter(|w| **w == Work::AdvanceRandom)
            .count(),
        1
    );
    assert_eq!(f.p.status_selector, 0);
}
#[test]
fn original_stage_presets_decode_every_plane_and_preserve_code_adjacent_timer_reads() {
    let c = Catalog::original(&rom()).unwrap();
    assert_eq!(c.stages.len(), STAGE_CONFIGURATION_COUNT);
    let banks = [1, 2, 2, 3, 3, 1, 0, 3, 3];
    let rooms = [40, 48, 0, 0, 32, 0, 0, 0, 0];
    let rows = [5, 6, 0, 0, 4, 0, 0, 0, 0];
    let xs = [128, 128, 128, 128, 128, 128, 128, 128, 208];
    let pairs = [
        [5, 0],
        [5, 0],
        [6, 0],
        [5, 0],
        [6, 0],
        [0xa5, 0xac],
        [0xf0, 0x06],
        [0x20, 0xaf],
        [0xe1, 0x4c],
    ];
    for selector in 0..STAGE_CONFIGURATION_COUNT {
        let p = c.stages[selector];
        assert_eq!(
            (
                p.terrain_bank,
                p.camera_room,
                p.row,
                p.player_x,
                p.countdown_pair
            ),
            (
                banks[selector],
                rooms[selector],
                rows[selector],
                xs[selector],
                pairs[selector]
            )
        );
    }
}
#[test]
fn palette_loader_copies_stage_sixteen_bytes_and_shared_sprite_sixteen_without_selector_alias() {
    let mut bytes = rom();
    const BACKGROUND_OFFSET: usize = 0x1da44;
    const SHARED_OFFSET: usize = 0x1dad4;
    // Synthetic palette input remains independent of the original copyrighted data.
    for selector in 0..STAGE_CONFIGURATION_COUNT {
        bytes[BACKGROUND_OFFSET + selector * 16..BACKGROUND_OFFSET + (selector + 1) * 16]
            .fill(selector as u8);
    }
    bytes[SHARED_OFFSET..SHARED_OFFSET + 16].copy_from_slice(&[7; 16]);
    let c = Catalog::original(&bytes).unwrap();
    for (selector, p) in c.stages.iter().enumerate() {
        assert_eq!(p.background_palette, [selector as u8; 16]);
    }
    assert_eq!(c.sprite_palette, [7; 16]);
}
#[test]
fn catalog_rejects_each_required_truncation_before_publishing_partial_runtime_data() {
    let bytes = rom();
    for end in [
        STAGE_LOOKUPS_OFFSET,
        CHECKPOINTS_OFFSET,
        CHECKPOINTS_OFFSET + CHECKPOINT_COUNT * CHECKPOINT_BYTES - 1,
        GAME_OVER_OVERLAY_OFFSET + GAME_OVER_OVERLAY_BYTES - 1,
    ] {
        assert!(
            Catalog::original(&bytes[..end]).is_err(),
            "missing source recipe at {end:x}"
        );
    }
}
#[test]
fn every_ordinary_stage_boundary_requires_row_and_offset_page_but_not_exact_player_low_x() {
    let c = catalog();
    for selector in 0..ORDINARY_STAGES {
        let mut p = Progression {
            stage: StageId(selector as u8),
            ..Progression::default()
        };
        let row = c.transition_rows[selector];
        let page = c.transition_pages[selector];
        let x = if c.transition_directions[selector] & 2 != 0 {
            (u16::from(page) << 8) - 8
        } else {
            (u16::from(page) << 8) + 8
        };
        assert_eq!(boundary(&mut p, row.wrapping_add(1), x, &c), Boundary::None);
        assert_eq!(
            boundary(&mut p, row, x.wrapping_add(0x100), &c),
            Boundary::None
        );
        assert_eq!(
            boundary(&mut p, row, x, &c),
            Boundary::Horizontal {
                direction: c.transition_directions[selector],
                background_row: selector as u8
            }
        );
    }
}
#[test]
fn boundary_positive_offset_carries_and_negative_offset_borrows_across_world_page() {
    let c = catalog();
    let mut positive = Progression::default();
    assert!(matches!(
        boundary(&mut positive, 1, 0x03f8, &c),
        Boundary::Horizontal { .. }
    ));
    assert_eq!(boundary(&mut positive, 1, 0x03f7, &c), Boundary::None);
    let mut negative = Progression {
        stage: StageId(3),
        ..Progression::default()
    };
    assert!(matches!(
        boundary(&mut negative, 2, 0x0408, &c),
        Boundary::Horizontal { .. }
    ));
    assert_eq!(boundary(&mut negative, 2, 0x0407, &c), Boundary::None);
}
#[test]
fn stage_one_completed_bit_returns_land_select_unless_all_five_bits_set() {
    let c = catalog();
    for progress in 0..=u8::MAX {
        let mut p = Progression {
            stage: StageId(1),
            progress_bits: progress,
            ..Progression::default()
        };
        let got = boundary(&mut p, 3, 0x04f8, &c);
        if progress & 2 != 0 && progress & ALL_LANDS != ALL_LANDS {
            assert_eq!(got, Boundary::LandSelectionRequest);
            assert_eq!(p.boundary_request, REQUEST_NEGATIVE);
        } else {
            assert_eq!(
                got,
                Boundary::Horizontal {
                    direction: 2,
                    background_row: if progress & ALL_LANDS == ALL_LANDS {
                        5
                    } else {
                        1
                    }
                }
            );
        }
    }
}
#[test]
fn all_complete_boundary_selects_row_five_and_sets_route_five_ignoring_upper_progress_bits() {
    let c = catalog();
    for upper in [0, 0x20, 0x40, 0x80, 0xe0] {
        let mut p = Progression {
            progress_bits: ALL_LANDS | upper,
            ..Progression::default()
        };
        assert_eq!(
            boundary(&mut p, 1, 0x03f8, &c),
            Boundary::Horizontal {
                direction: 2,
                background_row: 5
            }
        );
        assert_eq!(p.late_route, 5);
    }
}
#[test]
fn late_position_all_nine_literal_row_page_pairs_set_variant_and_mismatches_retain_old_value() {
    let c = catalog();
    for selector in 0..STAGE_CONFIGURATION_COUNT {
        let mut p = Progression {
            stage: StageId(selector as u8),
            life_loss_variant: MARKER,
            ..Progression::default()
        };
        late_position(
            &mut p,
            c.late_rows[selector].wrapping_add(1),
            c.late_pages[selector],
            &c,
        );
        assert_eq!(p.life_loss_variant, MARKER);
        late_position(
            &mut p,
            c.late_rows[selector],
            c.late_pages[selector].wrapping_add(1),
            &c,
        );
        assert_eq!(p.life_loss_variant, MARKER);
        late_position(&mut p, c.late_rows[selector], c.late_pages[selector], &c);
        assert_eq!(p.life_loss_variant, 5);
    }
}
#[test]
fn horizontal_transition_direction_zero_adds_camera_one_and_player_quarter_pixel() {
    assert_eq!(
        horizontal_step(0x12fe, 0x34f0, 0),
        HorizontalStep {
            camera: 0x12ff,
            player_low_fixed: 0x3530,
            done: false,
            previous_x: Some(0xff),
            clear_displacement: true
        }
    );
    assert_eq!(
        horizontal_step(0x12ff, 0xfff0, 0),
        HorizontalStep {
            camera: 0x1300,
            player_low_fixed: 0x0030,
            done: true,
            previous_x: None,
            clear_displacement: false
        }
    );
}
#[test]
fn horizontal_transition_direction_bit_one_subtracts_camera_one_and_player_quarter_pixel() {
    assert_eq!(
        horizontal_step(0x1202, 0x3400, 2),
        HorizontalStep {
            camera: 0x1201,
            player_low_fixed: 0x33c0,
            done: false,
            previous_x: Some(1),
            clear_displacement: true
        }
    );
    assert_eq!(
        horizontal_step(0x0001, 0x0000, 0xff),
        HorizontalStep {
            camera: 0,
            player_low_fixed: 0xffc0,
            done: true,
            previous_x: None,
            clear_displacement: false
        }
    );
}
#[test]
fn reload_request_threshold_is_unsigned_and_nonzero_special_mode_preempts_every_life_case() {
    for request in 0..=u8::MAX {
        assert_eq!(reload_route(request, 1, 0), ReloadRoute::LandSelection);
        assert_eq!(reload_route(request, 0x80, 9), ReloadRoute::LandSelection);
        if request >= LIFE_LOSS_FIRST {
            assert_eq!(reload_route(request, 0, 0), ReloadRoute::GameOver);
            assert_eq!(
                reload_route(request, 0, 1),
                ReloadRoute::Checkpoint {
                    sound: LIFE_LOSS_SOUND,
                    stage_sound: true
                }
            );
        } else {
            assert_eq!(
                reload_route(request, 0, 0),
                ReloadRoute::Checkpoint {
                    sound: TRANSITION_SOUND,
                    stage_sound: false
                }
            );
        }
    }
}
#[test]
fn reload_wait_uses_sixty_or_one_eighty_for_every_request_without_signed_test() {
    for request in 0..=u8::MAX {
        assert_eq!(
            reload_wait(request),
            if request >= LIFE_LOSS_FIRST {
                LIFE_LOSS_WAIT
            } else {
                TRANSITION_WAIT
            }
        );
    }
}
#[test]
fn only_checkpoint_85_overrides_stage_row_and_life_loss_consumes_one_even_at_one_life() {
    for request in [0x80, 0x84, 0x85, 0x86, 0x87, 0x8b, 0xff] {
        let got = reload_background(request, StageId(4), CAMERA_ROW);
        if request == LATE_CHECKPOINT {
            assert_eq!(
                got,
                ReloadBackground {
                    stage: StageId(5),
                    row: 0,
                    late_mode: 0xff,
                    decrement_life: false,
                    reset_animation: false,
                    restore_health: false
                }
            );
        } else {
            assert_eq!(got.stage, StageId(4));
            assert_eq!(got.row, CAMERA_ROW);
            assert_eq!(got.late_mode, 0);
            assert_eq!(got.decrement_life, request >= LIFE_LOSS_FIRST);
            assert_eq!(got.reset_animation, request >= LIFE_LOSS_FIRST);
            assert_eq!(got.restore_health, request >= LIFE_LOSS_FIRST);
        }
    }
    assert_eq!(
        reload_route(LIFE_LOSS_FIRST, 0, 1),
        ReloadRoute::Checkpoint {
            sound: LIFE_LOSS_SOUND,
            stage_sound: true
        }
    );
    assert_eq!(reload_route(LIFE_LOSS_FIRST, 0, 0), ReloadRoute::GameOver);
}
#[test]
fn late_audio_overrides_death_command_without_resetting_countdown() {
    let c = catalog();
    for late in [0x80, 0xff] {
        assert_eq!(
            restore_audio(StageId(4), late, LIFE_LOSS_SOUND, &c),
            AudioRestore {
                stage_command: Some(LATE_STAGE_SOUND),
                countdown: None,
                divider: None
            }
        );
    }
}
#[test]
fn death_audio_restores_selected_stage_and_upper_timer_pair_units_zero_divider_sixty() {
    let mut c = catalog();
    for selector in 0..ORDINARY_STAGES {
        c.stages[selector].countdown_pair = [selector as u8, 7];
        assert_eq!(
            restore_audio(StageId(selector as u8), 0, LIFE_LOSS_SOUND, &c),
            AudioRestore {
                stage_command: Some(selector as u8),
                countdown: Some([selector as u8, 7, 0]),
                divider: Some(60)
            }
        );
    }
}
#[test]
fn ordinary_current_stage_audio_preserves_countdown_and_does_not_enqueue_again() {
    let c = catalog();
    for command in [0, 1, 6, 9, 11, 0xff] {
        assert_eq!(
            restore_audio(StageId(3), 0, command, &c),
            AudioRestore {
                stage_command: None,
                countdown: None,
                divider: None
            }
        );
    }
}

#[test]
fn original_nineteen_checkpoint_records_decode_all_nine_fields_and_keep_raw_counts() {
    let c = Catalog::original(&rom()).unwrap();
    assert_eq!(c.checkpoints.len(), CHECKPOINT_COUNT);
    // Source parameter expectations, independently read at physical1DDD1 stride9.
    let expected = [
        [7, 0, 1, 63, 57, 0, 180, 192, 57],
        [6, 0, 1, 127, 48, 32, 84, 128, 49],
        [4, 160, 2, 127, 34, 8, 148, 32, 35],
        [3, 0, 7, 127, 30, 32, 116, 192, 31],
        [6, 0, 1, 127, 48, 32, 84, 128, 49],
        [1, 0, 4, 63, 12, 0, 180, 208, 12],
        [2, 224, 3, 127, 19, 24, 112, 96, 20],
        [5, 0, 0, 95, 40, 0, 144, 128, 40],
        [6, 0, 0, 95, 48, 0, 160, 128, 48],
        [0, 0, 0, 95, 0, 0, 80, 96, 0],
        [0, 0, 0, 95, 0, 0, 176, 128, 0],
        [4, 0, 0, 95, 32, 0, 176, 128, 32],
        [2, 32, 1, 127, 16, 40, 176, 160, 17],
        [4, 128, 5, 127, 37, 0, 176, 0, 38],
        [3, 0, 5, 127, 28, 32, 160, 160, 29],
        [4, 0, 7, 127, 38, 32, 112, 128, 39],
        [1, 64, 2, 127, 9, 48, 176, 192, 10],
        [0, 0, 0, 63, 0, 0, 0, 0, 0],
        [0, 0, 3, 127, 2, 32, 64, 208, 3],
    ];
    for (record, raw) in c.checkpoints.iter().zip(expected) {
        assert_eq!(
            *record,
            Checkpoint {
                row: raw[0],
                camera_x: raw[1],
                camera_page: raw[2],
                column_count: raw[3],
                room: raw[4],
                column: raw[5],
                player_y: raw[6],
                player_x: raw[7],
                packed_page: raw[8]
            }
        );
    }
}
#[test]
fn checkpoint_request_low_five_bits_select_record_without_treating_high_bits_as_stage() {
    let mut c = catalog();
    c.checkpoints[7] = Checkpoint {
        row: CAMERA_ROW,
        camera_x: MARKER,
        camera_page: CAMERA_PAGE,
        column_count: 0,
        room: 0x29,
        column: 0x3f,
        player_y: 0x90,
        player_x: 0x80,
        packed_page: 0x3d,
    };
    let rooms = [0; 256];
    for request in [7, 0x27, 0x47, 0x87, 0xa7, 0xc7, 0xe7] {
        let b = checkpoint_build(request, &c, &rooms).unwrap();
        assert_eq!(b.checkpoint, c.checkpoints[7]);
        assert_eq!((b.camera_position, b.camera_room), (0x0459, 0x3c));
        assert_eq!(b.player_page, 5);
        assert_eq!(b.terrain_room_in_row, 1);
        assert_eq!(
            b.visits,
            vec![StreamCursor {
                ring: 0x3f,
                room: 0x29,
                room_in_row: 1
            }]
        );
        assert_eq!(b.final_cursor, b.visits[0]);
    }
}
#[test]
fn checkpoint_column_count_is_inclusive_sixty_four_ninety_six_and_one_twenty_eight() {
    for (count, visits) in [(0, 1), (63, 64), (95, 96), (127, 128)] {
        let mut c = catalog();
        c.checkpoints[0].column_count = count;
        let b = checkpoint_build(0, &c, &[0; 256]).unwrap();
        assert_eq!(b.visits.len(), visits);
        assert_eq!(b.resolved, b.visits);
        assert_eq!(b.final_cursor, *b.visits.last().unwrap());
    }
}
#[test]
fn checkpoint_ring_wrap_advances_both_room_cursors_and_last_visit_does_not_advance() {
    let mut c = catalog();
    c.checkpoints[0].column = 0x3f;
    c.checkpoints[0].room = 7;
    c.checkpoints[0].column_count = 2;
    let b = checkpoint_build(0, &c, &[0; 256]).unwrap();
    assert_eq!(
        b.visits,
        vec![
            StreamCursor {
                ring: 63,
                room: 7,
                room_in_row: 7
            },
            StreamCursor {
                ring: 0,
                room: 8,
                room_in_row: 8
            },
            StreamCursor {
                ring: 1,
                room: 8,
                room_in_row: 8
            }
        ]
    );
    assert_eq!(b.final_cursor, b.visits[2]);
}
#[test]
fn checkpoint_missing_rooms_skip_resolution_but_consume_columns_and_cross_into_next_room() {
    let mut c = catalog();
    c.checkpoints[0].column = 62;
    c.checkpoints[0].column_count = 3;
    let mut rooms = [0; 256];
    rooms[0] = 0xff;
    let b = checkpoint_build(0, &c, &rooms).unwrap();
    assert_eq!(b.visits.len(), 4);
    assert_eq!(
        b.resolved,
        vec![
            StreamCursor {
                ring: 0,
                room: 1,
                room_in_row: 1
            },
            StreamCursor {
                ring: 1,
                room: 1,
                room_in_row: 1
            }
        ]
    );
}
#[test]
fn checkpoint_record_and_room_bounds_fail_explicitly_instead_of_inventing_respawn_data() {
    let c = catalog();
    for request in [19, 31, 0x9f] {
        assert!(checkpoint_build(request, &c, &[0; 256]).is_err());
    }
    let mut c = catalog();
    c.checkpoints[0].room = 2;
    assert!(checkpoint_build(0, &c, &[0; 2]).is_err());
}
#[test]
fn all_complete_no_signal_raw_scan_retains_actual_adjacent_parameter_bytes_without_five_cap() {
    let bytes = rom();
    for (progress, expected) in [
        (
            0x1f,
            RawSelectionEntry {
                selector: 5,
                cursor_y: 0x43,
                name_y: 0x80,
                name_x: 0xa9,
                animation: 0x74,
            },
        ),
        (
            0x3f,
            RawSelectionEntry {
                selector: 6,
                cursor_y: 0x70,
                name_y: 0x60,
                name_x: 0x22,
                animation: 0x75,
            },
        ),
        (
            0x7f,
            RawSelectionEntry {
                selector: 7,
                cursor_y: 0x46,
                name_y: 0x88,
                name_x: 0x8d,
                animation: 0x76,
            },
        ),
        (
            0xff,
            RawSelectionEntry {
                selector: 21,
                cursor_y: 0xd8,
                name_y: 0x22,
                name_x: 0x6b,
                animation: 0x84,
            },
        ),
    ] {
        assert_eq!(raw_selection_entry(&bytes, progress).unwrap(), expected);
    }
}
#[test]
fn incomplete_raw_selection_still_picks_first_missing_low_bit_for_every_upper_progress_pattern() {
    let bytes = rom();
    let ys = [0x70, 0x78, 0x80, 0x88, 0x90];
    let name_y = [0x28, 0x20, 0x30, 0x28, 0x30];
    let name_x = [0x80, 0x60, 0x88, 0x68, 0xa0];
    for progress in 0..=u8::MAX {
        if progress & ALL_LANDS == ALL_LANDS {
            continue;
        }
        let missing = (0..ORDINARY_STAGES)
            .find(|n| progress & (1 << n) == 0)
            .unwrap();
        assert_eq!(
            raw_selection_entry(&bytes, progress).unwrap(),
            RawSelectionEntry {
                selector: missing as u8,
                cursor_y: ys[missing],
                name_y: name_y[missing],
                name_x: name_x[missing],
                animation: 0x6f + missing as u8
            }
        );
    }
}
#[test]
fn raw_selection_rejects_missing_parameter_bytes_without_declaring_ff_equivalent_to_original_land()
{
    let bytes = rom();
    const RAW_PARAMETER_OFFSET: usize = 0x1d8d7;
    assert!(raw_selection_entry(&bytes[..RAW_PARAMETER_OFFSET], 0xff).is_err());
}
#[test]
fn initial_runtime_waits_background_before_loading_spawn_tables_or_mutating_player() {
    assert_eq!(
        lifecycle(Lifecycle::Initial, Phase::Entry, 0, 0),
        Batch {
            operations: vec![Operation::PollBackground],
            next: Phase::BackgroundComplete
        }
    );
    assert_eq!(
        lifecycle(Lifecycle::Initial, Phase::BackgroundComplete, 0, 0),
        Batch {
            operations: vec![
                Operation::LoadSpawnTables,
                Operation::ResetSpawnWindow,
                Operation::InitializePlayer,
                Operation::CopyStageUi,
                Operation::EnableRendering,
                Operation::Yield
            ],
            next: Phase::DisplayYieldComplete
        }
    );
}
#[test]
fn initial_runtime_sets_gameplay_after_enable_yield_and_clears_controls_only_after_fade() {
    assert_eq!(
        lifecycle(Lifecycle::Initial, Phase::DisplayYieldComplete, 0, 0),
        Batch {
            operations: vec![Operation::SetGameplayDisplay, Operation::FadeIn],
            next: Phase::FadeComplete
        }
    );
    assert_eq!(
        lifecycle(Lifecycle::Initial, Phase::FadeComplete, 0, 0),
        Batch {
            operations: vec![
                Operation::ClearInitialControls,
                Operation::CopyStatusAndTimer,
                Operation::RestoreHealth,
                Operation::InitializeCountdown,
                Operation::InitializeOverlaySentinel,
                Operation::RebuildSuppression,
                Operation::EnterGameplay
            ],
            next: Phase::Done
        }
    );
}
#[test]
fn player_runtime_defaults_set_jump_magnitude_and_low_camera_but_retain_world_position_live_velocity_and_locals()
 {
    let mut player = Player {
        jump_velocity: 0,
        action: Action::Swing,
        external_motion: MARKER,
        ..Player::default()
    };
    let mut actors = [ActorContact::default(); ACTOR_SLOTS];
    actors[0] = ActorContact {
        position: Position {
            world_x: 0x0580,
            x_fraction: MARKER,
            screen_y: MARKER,
            y_fraction: MARKER,
        },
        vertical_velocity: -0x123,
        handler: MARKER,
        profile: MARKER,
        local_state: MARKER,
        spawn_identity: Some(MARKER),
        animation_restart: MARKER,
        ..ActorContact::default()
    };
    let old = actors[0];
    let mut camera = Camera {
        x: 0x0345,
        room: 0x2b,
        previous_x: MARKER,
    };
    initialize_player(&mut player, &mut actors, &mut camera);
    assert_eq!(
        (
            actors[0].state,
            actors[0].render,
            actors[0].position.screen_y
        ),
        (PLAYER_ACTIVE, PLAYER_ACTIVE, PLAYER_START_Y)
    );
    assert_eq!(
        (
            actors[0].animation,
            actors[0].animation_tick,
            actors[0].animation_frame,
            actors[0].position.x_fraction
        ),
        (0, 0, 0, 0)
    );
    assert_eq!(actors[0].horizontal_magnitude, INITIAL_SPEED);
    assert_eq!(player.jump_velocity, INITIAL_JUMP);
    assert_eq!(
        (
            actors[0].position.world_x,
            actors[0].position.y_fraction,
            actors[0].vertical_velocity
        ),
        (old.position.world_x, MARKER, -0x123)
    );
    assert_eq!(
        (
            actors[0].handler,
            actors[0].profile,
            actors[0].local_state,
            actors[0].spawn_identity,
            actors[0].animation_restart
        ),
        (MARKER, MARKER, MARKER, Some(MARKER), MARKER)
    );
    assert_eq!(camera.x, 0x0300);
    assert_eq!(
        (player.action, player.external_motion),
        (Action::Swing, MARKER)
    ); // controls clear only after later fade.
}
#[test]
fn reload_entry_clears_impact_and_dispatches_route_before_protection_or_wait() {
    assert_eq!(
        lifecycle(Lifecycle::Reload, Phase::Entry, LIFE_LOSS_FIRST, 0),
        Batch {
            operations: vec![
                Operation::ClearImpact,
                Operation::SelectReloadRoute,
                Operation::GrantReloadProtection,
                Operation::WaitComposed(LIFE_LOSS_WAIT)
            ],
            next: Phase::InitialWaitComplete
        }
    );
}
#[test]
fn reload_special_mode_selection_preempts_protection_wait_fade_and_checkpoint() {
    for special in [1, 0x80, 0xff] {
        assert_eq!(
            lifecycle(Lifecycle::Reload, Phase::Entry, LIFE_LOSS_FIRST, special),
            Batch {
                operations: vec![Operation::ClearImpact, Operation::ReturnToLandSelect],
                next: Phase::Done
            }
        );
    }
}
#[test]
fn reload_initial_wait_then_fade_then_checkpoint_temporary_background_has_distinct_resumption_points()
 {
    assert_eq!(
        lifecycle(Lifecycle::Reload, Phase::InitialWaitComplete, 0x80, 0),
        Batch {
            operations: vec![Operation::FadeOut],
            next: Phase::FadeOutComplete
        }
    );
    assert_eq!(
        lifecycle(Lifecycle::Reload, Phase::FadeOutComplete, 0x85, 0),
        Batch {
            operations: vec![
                Operation::ClearReloadControls,
                Operation::DisableRendering,
                Operation::LoadCheckpoint,
                Operation::ClearActors,
                Operation::InstallTemporaryBackground,
                Operation::PollBackground
            ],
            next: Phase::BackgroundComplete
        }
    );
}
#[test]
fn reload_restores_stage_and_row_before_spawn_window_enable_yield_and_fade() {
    assert_eq!(
        lifecycle(Lifecycle::Reload, Phase::BackgroundComplete, 0x85, 0),
        Batch {
            operations: vec![
                Operation::RestoreBackgroundContext,
                Operation::SetForwardScroll,
                Operation::ResetSpawnWindow,
                Operation::EnableRendering,
                Operation::Yield
            ],
            next: Phase::DisplayYieldComplete
        }
    );
    assert_eq!(
        lifecycle(Lifecycle::Reload, Phase::DisplayYieldComplete, 0x85, 0),
        Batch {
            operations: vec![Operation::SetGameplayDisplay, Operation::FadeIn],
            next: Phase::FadeComplete
        }
    );
}
#[test]
fn reload_spawns_after_fade_then_activates_player_and_runs_second_composed_wait_before_clearing() {
    for request in [0x80, LIFE_LOSS_FIRST] {
        let duration = if request >= LIFE_LOSS_FIRST {
            LIFE_LOSS_WAIT
        } else {
            TRANSITION_WAIT
        };
        assert_eq!(
            lifecycle(Lifecycle::Reload, Phase::FadeComplete, request, 0),
            Batch {
                operations: vec![
                    Operation::RestoreAudioCountdown,
                    Operation::Spawn,
                    Operation::ActivatePlayer,
                    Operation::WaitComposed(duration)
                ],
                next: Phase::FinalWaitComplete
            }
        );
        assert_eq!(
            lifecycle(Lifecycle::Reload, Phase::FinalWaitComplete, request, 0),
            Batch {
                operations: vec![Operation::ClearReloadPlayerAndRequest],
                next: Phase::Done
            }
        );
    }
}
#[test]
fn reload_completion_uses_wait_return_zero_not_duration_as_player_protection() {
    let mut p = Progression {
        reload_request: LIFE_LOSS_FIRST,
        late_mode: 0xff,
        progress_bits: ALL_LANDS,
        ..Progression::default()
    };
    let mut player = Player {
        protection: 1,
        action: Action::Death,
        external_motion: MARKER,
        ..Player::default()
    };
    let mut actors = [ActorContact::default(); ACTOR_SLOTS];
    actors[0].render = MARKER;
    actors[0].state = MARKER;
    finish_reload(&mut p, &mut player, &mut actors);
    assert_eq!(
        (
            p.reload_request,
            player.protection,
            player.action,
            actors[0].state
        ),
        (0, 0, Action::Standing, MARKER)
    );
    assert_eq!(
        (
            p.late_mode,
            p.progress_bits,
            actors[0].render,
            player.external_motion
        ),
        (0xff, ALL_LANDS, MARKER, MARKER)
    );
}
#[test]
fn game_over_queues_nine_copies_eight_records_plus_terminator_then_waits_240_before_reset() {
    assert_eq!(
        lifecycle(Lifecycle::GameOver, Phase::Entry, 0, 0),
        Batch {
            operations: vec![
                Operation::StageSound(GAME_OVER_SOUND),
                Operation::CopyGameOverOverlay,
                Operation::WaitComposed(GAME_OVER_WAIT)
            ],
            next: Phase::FinalWaitComplete
        }
    );
    assert_eq!(
        lifecycle(Lifecycle::GameOver, Phase::FinalWaitComplete, 0, 0),
        Batch {
            operations: vec![Operation::Reset],
            next: Phase::Done
        }
    );
    let c = Catalog::original(&rom()).unwrap();
    assert_eq!(c.game_over_overlay.len(), GAME_OVER_OVERLAY_BYTES);
    assert_eq!(c.game_over_overlay.last(), Some(&0xff));
}
#[test]
fn selection_ordinary_delay_is_180_and_nonzero_special_selector_returns_without_wait() {
    assert_eq!(
        lifecycle(Lifecycle::Selection, Phase::Entry, 0, 0),
        Batch {
            operations: vec![Operation::WaitComposed(LIFE_LOSS_WAIT)],
            next: Phase::FinalWaitComplete
        }
    );
    assert_eq!(
        lifecycle(Lifecycle::Selection, Phase::FinalWaitComplete, 0, 0),
        Batch {
            operations: vec![Operation::ReturnToLandSelect],
            next: Phase::Done
        }
    );
    for special in [1, 0x80, 0xff] {
        assert_eq!(
            lifecycle(Lifecycle::Selection, Phase::Entry, 0, special),
            Batch {
                operations: vec![Operation::ReturnToLandSelect],
                next: Phase::Done
            }
        );
    }
}
#[test]
fn reload_wait_freezes_player_tick_after_each_resume_and_last_zero_finishes() {
    let mut tick = LIVE_ANIMATION_TICK;
    assert_eq!(composed_wait_step(2, &mut tick, true), (1, false));
    assert_eq!(tick, 0);
    tick = LIVE_ANIMATION_TICK;
    assert_eq!(composed_wait_step(1, &mut tick, true), (0, true));
    assert_eq!(tick, 0);
}
#[test]
fn gameover_and_selection_waits_retain_animation_tick_and_zero_count_wraps_to_256_iterations() {
    let mut tick = LIVE_ANIMATION_TICK;
    assert_eq!(composed_wait_step(0, &mut tick, false), (255, false));
    assert_eq!(tick, LIVE_ANIMATION_TICK);
    assert_eq!(composed_wait_step(240, &mut tick, false), (239, false));
    assert_eq!(tick, LIVE_ANIMATION_TICK);
    assert_eq!(composed_wait_step(1, &mut tick, false), (0, true));
    assert_eq!(tick, LIVE_ANIMATION_TICK);
}

#[test]
fn initial_sixteen_control_reset_clears_all_canonical_fields_including_reload_life_variant_and_protection()
 {
    let mut p = Progression {
        late_route: MARKER,
        special_room: MARKER,
        life_loss_variant: MARKER,
        reload_request: MARKER,
        progress_bits: ALL_LANDS,
        status_selector: MARKER,
        ..Progression::default()
    };
    let mut player = Player {
        queued: Some(Action::Damage),
        momentum_control: MARKER,
        momentum_direction: MARKER,
        external_motion: MARKER,
        special_mode: MARKER,
        special_ticks: 0x1234,
        protection: MARKER,
        ..Player::default()
    };
    let mut status = status();
    status.dirty = MARKER;
    status.ui_pending = MARKER;
    let (mut departure, mut dialogue, mut overlay) = (MARKER, MARKER, MARKER);
    clear_control_range(
        ControlReset::Initial,
        &mut p,
        &mut player,
        &mut status,
        &mut TransientOwners {
            departure_signal: &mut departure,
            dialogue_actor_slot: &mut dialogue,
            overlay_gate: &mut overlay,
        },
    );
    assert_eq!(
        (
            p.late_route,
            p.special_room,
            p.life_loss_variant,
            p.reload_request
        ),
        (0, 0, 0, 0)
    );
    assert_eq!(
        (
            player.queued,
            player.momentum_control,
            player.momentum_direction,
            player.external_motion
        ),
        (None, 0, 0, 0)
    );
    assert_eq!(
        (
            player.special_mode,
            player.special_ticks,
            player.protection,
            status.dirty
        ),
        (0, 0, 0, 0)
    );
    assert_eq!((departure, dialogue, overlay), (0, 0, 0));
    assert_eq!(
        (p.progress_bits, p.status_selector, status.ui_pending),
        (ALL_LANDS, MARKER, MARKER)
    );
}
#[test]
fn reload_twelve_control_reset_preserves_life_variant_protection_request_and_overlay_gate() {
    let mut p = Progression {
        late_route: MARKER,
        special_room: MARKER,
        life_loss_variant: MARKER,
        reload_request: LIFE_LOSS_FIRST,
        ..Progression::default()
    };
    let mut player = Player {
        queued: Some(Action::Damage),
        momentum_control: MARKER,
        momentum_direction: MARKER,
        external_motion: MARKER,
        special_mode: MARKER,
        special_ticks: 0x1234,
        protection: 1,
        ..Player::default()
    };
    let mut status = status();
    status.dirty = MARKER;
    let (mut departure, mut dialogue, mut overlay) = (MARKER, MARKER, MARKER);
    clear_control_range(
        ControlReset::Reload,
        &mut p,
        &mut player,
        &mut status,
        &mut TransientOwners {
            departure_signal: &mut departure,
            dialogue_actor_slot: &mut dialogue,
            overlay_gate: &mut overlay,
        },
    );
    assert_eq!(
        (
            p.late_route,
            p.special_room,
            status.dirty,
            departure,
            dialogue
        ),
        (0, 0, 0, 0, 0)
    );
    assert_eq!(
        (
            player.queued,
            player.momentum_control,
            player.momentum_direction,
            player.external_motion
        ),
        (None, 0, 0, 0)
    );
    assert_eq!((player.special_mode, player.special_ticks), (0, 0));
    assert_eq!(
        (
            p.life_loss_variant,
            player.protection,
            p.reload_request,
            overlay
        ),
        (MARKER, 1, LIFE_LOSS_FIRST, MARKER)
    );
}
#[test]
fn stage_presentation_writes_original_nine_preset_owners_and_both_palette_shadows() {
    let mut c = catalog();
    for selector in 0..STAGE_CONFIGURATION_COUNT {
        let source = StagePreset {
            terrain_bank: 3,
            camera_room: 0x29,
            row: 5,
            player_x: 0xa0,
            countdown_pair: [6, 0],
            background_palette: [selector as u8; 16],
        };
        c.stages[selector] = source;
        c.sprite_palette = [7; 16];
        let mut actor = ActorContact {
            position: Position {
                world_x: 0x0780,
                screen_y: MARKER,
                x_fraction: MARKER,
                y_fraction: MARKER,
            },
            ..ActorContact::default()
        };
        let mut camera = Camera {
            x: 0x0759,
            room: 0,
            previous_x: MARKER,
        };
        let mut display = crate::presentation::Display::default();
        assert_eq!(
            load_presentation(
                StageId(selector as u8),
                &mut actor,
                &mut camera,
                &mut display,
                &c
            )
            .unwrap(),
            PresentationSetup {
                terrain_bank: 3,
                row: 5,
                terrain_room_in_row: 1,
                stage_command: selector as u8
            }
        );
        assert_eq!(
            (camera.x, camera.room, actor.position.world_x),
            (0x0159, 0x29, 0x01a0)
        );
        assert_eq!(
            (
                actor.position.screen_y,
                actor.position.x_fraction,
                actor.position.y_fraction
            ),
            (MARKER, MARKER, MARKER)
        );
        assert_eq!(&display.active[..16], &[selector as u8; 16]);
        assert_eq!(&display.base[..16], &[selector as u8; 16]);
        assert_eq!(&display.active[16..], &[7; 16]);
        assert_eq!(&display.base[16..], &[7; 16]);
    }
}
#[test]
fn presentation_missing_named_source_configuration_errors_without_partial_owner_mutation() {
    let c = catalog();
    let mut actor = ActorContact::default();
    let mut camera = Camera::default();
    let mut display = crate::presentation::Display::default();
    let before = (actor, camera, display.clone());
    assert!(load_presentation(StageId(9), &mut actor, &mut camera, &mut display, &c).is_err());
    assert_eq!((actor, camera, display), before);
}
#[test]
fn horizontal_context_saves_original_stage_row_and_restores_after_temporary_stage_five() {
    let mut p = Progression {
        stage: StageId(3),
        late_route: MARKER,
        ..Progression::default()
    };
    let mut row = 2;
    let saved = begin_horizontal(&mut p, &mut row, 3);
    assert_eq!(
        saved,
        SavedHorizontalContext {
            stage: StageId(3),
            row: 2
        }
    );
    assert_eq!((p.stage, row, p.late_route), (StageId(5), 3, MARKER));
    let mut palette = false;
    assert_eq!(
        finish_horizontal(saved, &mut p, &mut row, &mut palette),
        LATE_STAGE_SOUND
    );
    assert_eq!(
        (p.stage, row, p.late_mode, palette),
        (StageId(3), 2, 0xff, true)
    );
}
#[test]
fn horizontal_all_complete_forces_row_five_route_five_before_temporary_background() {
    let mut p = Progression {
        stage: StageId(1),
        progress_bits: 0xff,
        ..Progression::default()
    };
    let mut row = 6;
    let saved = begin_horizontal(&mut p, &mut row, 1);
    assert_eq!((p.stage, row, p.late_route), (StageId(5), 5, 5));
    assert_eq!(
        saved,
        SavedHorizontalContext {
            stage: StageId(1),
            row: 6
        }
    );
}

#[test]
fn reload_activity_is_written_before_final_wait_without_overwriting_render_or_movement() {
    let mut actors = [ActorContact::default(); ACTOR_SLOTS];
    actors[0].render = MARKER;
    actors[0].vertical_velocity = -0x123;
    actors[0].animation_tick = LIVE_ANIMATION_TICK;
    activate_reload_player(&mut actors);
    assert_eq!(
        (
            actors[0].state,
            actors[0].render,
            actors[0].vertical_velocity,
            actors[0].animation_tick
        ),
        (PLAYER_ACTIVE, MARKER, -0x123, LIVE_ANIMATION_TICK)
    );
}

#[test]
fn paused_status_path_still_runs_late_position_pair_check_before_secondary_submission() {
    let mut f = Fixture::new();
    f.p.status_selector = 1;
    f.row = 2;
    f.actors[0].position.world_x = 0x0100;
    let mut s = Recorder::new();
    f.frame(Buttons::NONE, &mut s);
    assert_eq!(f.p.life_loss_variant, 5);
    assert_eq!((f.cursor, f.pending), (0, SECONDARY_BYTES));
}
#[test]
fn ending_and_race_bypass_late_position_even_when_exact_row_page_match() {
    for ending in [false, true] {
        let mut f = Fixture::new();
        f.row = 2;
        f.actors[0].position.world_x = 0x0100;
        f.p.life_loss_variant = MARKER;
        if ending {
            f.p.ending_latch = 1;
        } else {
            f.p.race_intro = 1;
        }
        let mut s = Recorder::new();
        f.frame(Buttons::NONE, &mut s);
        assert_eq!(f.p.life_loss_variant, MARKER);
    }
}

#[test]
fn entered_deferred_owner_resumes_after_consuming_its_gate_before_yield() {
    for work in [
        Work::Reload,
        Work::RefreshStatus,
        Work::StageUi,
        Work::SpecialRoom,
    ] {
        let mut f = Fixture::new();
        let mut systems = Recorder::new();
        systems.suspend = Some(work);
        match work {
            Work::Reload => f.p.reload_request = 0x80,
            Work::RefreshStatus => f.status.dirty = 1,
            Work::StageUi => f.status.ui_pending = 1,
            Work::SpecialRoom => f.p.special_room = 0x80,
            _ => unreachable!(),
        }
        assert_eq!(f.frame(Buttons::NONE, &mut systems), FrameExit::Suspended);
        assert_eq!(f.p.awaiting_owner, Some(work));
        // Models the suspended owner's already-committed consumption of its
        // request, before its final nested scheduler wait has returned.
        match work {
            Work::Reload => f.p.reload_request = 0,
            Work::RefreshStatus => f.status.dirty = 0,
            Work::StageUi => f.status.ui_pending = 0,
            Work::SpecialRoom => f.p.special_room = 1,
            _ => unreachable!(),
        }
        assert_eq!(f.frame(Buttons::NONE, &mut systems), FrameExit::NextFrame);
        assert_eq!(
            systems
                .events
                .iter()
                .filter(|&&event| event == Seen::Work(work))
                .count(),
            2
        );
        assert_eq!(f.p.awaiting_owner, None);
    }
}
#[test]
fn raw_selector_twenty_one_keeps_adjacent_parameters_and_byte_palette_index() {
    let rom = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap();
    let preset = raw_stage_preset(&rom, StageId(21)).unwrap();
    assert_eq!(
        (
            preset.terrain_bank,
            preset.camera_room,
            preset.row,
            preset.player_x
        ),
        (0, 128, 26, 15)
    );
    assert_eq!(preset.background_palette.as_slice(), &rom[0x1da94..0x1daa4]);
}

#[test]
fn committed_vertical_return_enters_after_spawn_without_a_player_random_return() {
    let mut f = Fixture::new();
    f.p.continuation = FrameContinuation::AfterSpawn;
    f.p.first_actor_increment = None;
    f.player.queued = Some(Action::Swing);
    f.cache.changed_this_frame = 1;
    let mut r = Recorder::new();
    assert_eq!(f.frame(Buttons::default(), &mut r), FrameExit::NextFrame);
    assert_eq!(r.work(), [Work::AdvanceRandom, Work::Yield]);
    assert!(!r.events.contains(&Seen::Player));
    assert_eq!(f.player.action, Action::Swing);
    assert_eq!(
        (
            f.camera.previous_x,
            f.displacement,
            f.cache.changed_this_frame
        ),
        (LOW_CAMERA, 0, 0)
    );
    assert_eq!((f.pending, f.cursor), (SECONDARY_BYTES, 0));
    assert_eq!(f.p.first_actor_increment, None);
}

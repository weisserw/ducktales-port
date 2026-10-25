//! Source gameplay orchestration and original respawn recipes (B13).
//!
//! These declarations are pending stage4. See trace.md. Mutable simulation
//! objects are borrowed from their canonical owners. No CPU/RAM image is stored.
use crate::{
    actors::ActorPool,
    identifiers::StageId,
    player::Player,
    runtime::PortInput,
    status::Status,
    world::{Camera, StreamCursor, WorldCache},
};
use anyhow::{Context as _, Result};

pub const LIFE_LOSS_FIRST: u8 = 0x87;
pub const LATE_CHECKPOINT: u8 = 0x85;
pub const TRANSITION_WAIT: u16 = 60;
pub const LIFE_LOSS_WAIT: u16 = 180;
pub const GAME_OVER_WAIT: u16 = 240;
pub const STAGE_CONFIGURATION_COUNT: usize = 9;
pub const CHECKPOINT_COUNT: usize = 19;
pub const CHECKPOINT_BYTES: usize = 9;
pub const STAGE_LOOKUPS_OFFSET: usize = 0x1da20;
pub const CHECKPOINTS_OFFSET: usize = 0x1ddd1;
pub const GAME_OVER_OVERLAY_OFFSET: usize = 0x1def7;
pub const GAME_OVER_OVERLAY_BYTES: usize = 33;
pub const LIFE_LOSS_SOUND: u8 = 10;
pub const GAME_OVER_SOUND: u8 = 9;
pub const TRANSITION_SOUND: u8 = 0x2d;
pub const LATE_STAGE_SOUND: u8 = 6;
pub const STATUS_TOGGLE_SOUND: u8 = 0x15;

/// Source run/progression flags. Player action/protection/queue belong to B06;
/// status values/UI dirtiness to B14; camera/cache to B04; display to B03.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Progression {
    pub stage: StageId,
    pub progress_bits: u8,
    /// B15 contact/escort sourceA1, incremented by message8 and marked negative on escort entry.
    pub escort_prerequisite: u8,
    /// SourceA2 count, incremented by first hit on handler3D.
    pub hit_dialogue_count: u8,
    /// SourceA3 reward progression; bit1 gates message3 boundary increment.
    pub reward_progress: u8,
    /// SourceA6 stage-mask choices accepted by message0F.
    pub accepted_stages: u8,
    /// SourceA8..AA message bits (selector divided by eight); not extra menu flags.
    pub seen_messages: [u8; 3],
    pub boss_hit_points: u8,
    pub impact_latch: u8,
    pub late_route: u8,
    pub late_mode: u8,
    /// SourceAE: stage-one animation2B health extension, used by suppression rules.
    /// Separate from LateStageModeFB and preserved across ordinary stage starts.
    pub stage_one_extension: u8,
    pub boundary_request: u8,
    pub scene_signal: u8,
    pub completion_actor: u8,
    pub ending_latch: u8,
    pub race_intro: u8,
    pub life_loss_variant: u8,
    pub reload_request: u8,
    pub special_room: u8,
    pub dialogue_cursor: u8,
    pub status_selector: u8,
    pub continuation: FrameContinuation,
    /// An already-entered deferred owner resumes even after it consumes its gate.
    pub awaiting_owner: Option<Work>,
    pub first_actor_increment: Option<FirstActorIncrement>,
}
impl Default for Progression {
    fn default() -> Self {
        Self {
            stage: StageId(0),
            progress_bits: 0,
            escort_prerequisite: 0,
            hit_dialogue_count: 0,
            reward_progress: 0,
            accepted_stages: 0,
            seen_messages: [0; 3],
            boss_hit_points: 0,
            impact_latch: 0,
            late_route: 0,
            late_mode: 0,
            stage_one_extension: 0,
            boundary_request: 0,
            scene_signal: 0,
            completion_actor: 0,
            ending_latch: 0,
            race_intro: 0,
            life_loss_variant: 0,
            reload_request: 0,
            special_room: 0,
            dialogue_cursor: 0,
            status_selector: 0,
            continuation: FrameContinuation::Entry,
            awaiting_owner: None,
            first_actor_increment: None,
        }
    }
}
/// Not CPU flags: the semantic addition input required by the first boss slot.
/// There is deliberately no Default. The selected B06 action must provide this
/// result after its final reaching arithmetic/compare; see integration contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FirstActorIncrement {
    Zero,
    One,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerReturn {
    pub first_actor_increment: FirstActorIncrement,
}

/// A temporary owner view for a single foreground invocation. Display queues,
/// audio, bank selection and scheduler live behind Systems, not another buffer.
pub struct Frame<'a> {
    pub progression: &'a mut Progression,
    pub player: &'a mut Player,
    pub actors: &'a mut ActorPool,
    pub camera: &'a mut Camera,
    pub cache: &'a mut WorldCache,
    pub status: &'a mut Status,
    pub camera_displacement: &'a mut u8,
    pub secondary_cursor: &'a mut u8,
    pub secondary_pending: &'a mut u8,
    pub oam_guard: &'a mut u8,
    pub room_row: &'a mut u8,
    pub catalog: &'a Catalog,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Work {
    Ending,
    Sound(u8),
    HideSprites,
    PresentationBank,
    Compose,
    StatusStreams,
    ActorBank,
    Actors(FirstActorIncrement),
    Countdown,
    Camera,
    StageBoundary,
    ClearActors,
    Selection,
    ConfirmStageOne,
    Reload,
    RefreshStatus,
    StageUi,
    SpecialRoom,
    Spawn,
    AdvanceRandom,
    Yield,
}
/// Stage4 adapter must execute each request before returning. A yielding owner
/// suspends the SAME enclosing call and input view; it cannot run the remaining
/// frame early. Native async/continuation storage belongs to that owner. This
/// interface contracts source call ordering without making a second game oracle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnerStep<T> {
    Suspended,
    Complete(T),
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FrameContinuation {
    #[default]
    Entry,
    Ending,
    PauseComposition,
    PauseStatus,
    Player,
    Actors,
    Countdown,
    Camera,
    Boundary,
    Selection,
    Confirmation,
    Reload,
    RefreshStatus,
    StageUi,
    SpecialRoom,
    Composition,
    Spawn,
    /// DBE4: accepted vertical transition discards its enclosing player return
    /// and rejoins after its own spawn update, bypassing actors/countdown.
    AfterSpawn,
    Yield,
}
pub trait Systems {
    /// Poll the canonical B06 continuation. Suspended means its own task yielded;
    /// subsequent calls resume that continuation, never restart its entry.
    fn player(&mut self, frame: &mut Frame<'_>) -> OwnerStep<PlayerReturn>;
    fn perform(&mut self, work: Work, frame: &mut Frame<'_>) -> OwnerStep<()>;
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameExit {
    Suspended,
    NextFrame,
    LandSelection,
    StageOneConfirmation,
}
pub fn foreground(
    _frame: &mut Frame<'_>,
    _input: PortInput,
    _systems: &mut dyn Systems,
) -> FrameExit {
    use FrameContinuation as C;
    loop {
        match _frame.progression.continuation {
            C::Entry => {
                _frame.progression.first_actor_increment = None;
                if _frame.progression.ending_latch != 0 {
                    _frame.progression.continuation = C::Ending;
                    continue;
                }
                if _frame.player.action == crate::player::Action::Dialogue {
                    _frame.progression.continuation = C::Player;
                    continue;
                }
                if _input.pressed.0 & crate::runtime::Buttons::START.0 != 0 {
                    immediate(_systems, Work::Sound(STATUS_TOGGLE_SOUND), _frame);
                    _frame.progression.status_selector ^= 1;
                    immediate(_systems, Work::HideSprites, _frame);
                    immediate(_systems, Work::PresentationBank, _frame);
                    _frame.progression.continuation = C::PauseComposition;
                } else if _frame.progression.status_selector != 0 {
                    finish_frame(_frame, _systems, true);
                } else {
                    _frame.progression.continuation = C::Player;
                }
            }
            C::Ending => {
                if _systems.perform(Work::Ending, _frame) == OwnerStep::Suspended {
                    return FrameExit::Suspended;
                }
                begin_composition(_frame, _systems);
            }
            C::PauseComposition => {
                if _systems.perform(Work::Compose, _frame) == OwnerStep::Suspended {
                    return FrameExit::Suspended;
                }
                _frame.progression.continuation = C::PauseStatus;
            }
            C::PauseStatus => {
                if _systems.perform(Work::StatusStreams, _frame) == OwnerStep::Suspended {
                    return FrameExit::Suspended;
                }
                if _frame.progression.status_selector != 0 {
                    finish_frame(_frame, _systems, true);
                } else {
                    _frame.progression.continuation = C::Player;
                }
            }
            C::Player => {
                let returned = match _systems.player(_frame) {
                    OwnerStep::Suspended => return FrameExit::Suspended,
                    OwnerStep::Complete(value) => value,
                };
                _frame.progression.first_actor_increment = Some(returned.first_actor_increment);
                if _frame.progression.dialogue_cursor != 0 {
                    begin_deferred(_frame);
                } else {
                    immediate(_systems, Work::ActorBank, _frame);
                    _frame.progression.continuation = C::Actors;
                }
            }
            C::Actors => {
                let increment = _frame
                    .progression
                    .first_actor_increment
                    .expect("completed player supplies first-slot addition fact");
                if _systems.perform(Work::Actors(increment), _frame) == OwnerStep::Suspended {
                    return FrameExit::Suspended;
                }
                _frame.progression.first_actor_increment = None;
                _frame.progression.continuation = C::Countdown;
            }
            C::Countdown => {
                if _systems.perform(Work::Countdown, _frame) == OwnerStep::Suspended {
                    return FrameExit::Suspended;
                }
                if _frame.progression.scene_signal != 0 {
                    _frame.progression.continuation = C::Selection;
                } else if _frame.progression.late_mode & 0x80 != 0 {
                    begin_deferred(_frame);
                } else {
                    _frame.progression.continuation = C::Camera;
                }
            }
            C::Camera => {
                if _systems.perform(Work::Camera, _frame) == OwnerStep::Suspended {
                    return FrameExit::Suspended;
                }
                _frame.progression.continuation = C::Boundary;
            }
            C::Boundary => {
                if _systems.perform(Work::StageBoundary, _frame) == OwnerStep::Suspended {
                    return FrameExit::Suspended;
                }
                let request = _frame.progression.boundary_request;
                if request != 0 {
                    _frame.progression.boundary_request = 0;
                    if request & 0x80 != 0 {
                        _frame.progression.continuation = C::Selection;
                    } else {
                        immediate(_systems, Work::ClearActors, _frame);
                        _frame.progression.stage = StageId(1);
                        _frame.progression.continuation = C::Confirmation;
                    }
                } else {
                    begin_deferred(_frame);
                }
            }
            C::Selection => {
                if _systems.perform(Work::Selection, _frame) == OwnerStep::Suspended {
                    return FrameExit::Suspended;
                }
                _frame.progression.continuation = C::Entry;
                return FrameExit::LandSelection;
            }
            C::Confirmation => {
                if _systems.perform(Work::ConfirmStageOne, _frame) == OwnerStep::Suspended {
                    return FrameExit::Suspended;
                }
                _frame.progression.continuation = C::Entry;
                return FrameExit::StageOneConfirmation;
            }
            C::Reload => {
                if deferred_owner(
                    _systems,
                    Work::Reload,
                    _frame.progression.reload_request & 0x80 != 0,
                    _frame,
                ) == OwnerStep::Suspended
                {
                    return FrameExit::Suspended;
                }
                _frame.progression.continuation = C::RefreshStatus;
            }
            C::RefreshStatus => {
                if deferred_owner(
                    _systems,
                    Work::RefreshStatus,
                    _frame.status.dirty != 0,
                    _frame,
                ) == OwnerStep::Suspended
                {
                    return FrameExit::Suspended;
                }
                _frame.progression.continuation = C::StageUi;
            }
            C::StageUi => {
                if deferred_owner(
                    _systems,
                    Work::StageUi,
                    _frame.status.ui_pending != 0,
                    _frame,
                ) == OwnerStep::Suspended
                {
                    return FrameExit::Suspended;
                }
                _frame.progression.continuation = C::SpecialRoom;
            }
            C::SpecialRoom => {
                if deferred_owner(
                    _systems,
                    Work::SpecialRoom,
                    _frame.progression.special_room & 0x80 != 0,
                    _frame,
                ) == OwnerStep::Suspended
                {
                    return FrameExit::Suspended;
                }
                begin_composition(_frame, _systems);
            }
            C::Composition => {
                if _systems.perform(Work::Compose, _frame) == OwnerStep::Suspended {
                    return FrameExit::Suspended;
                }
                *_frame.oam_guard = _frame.oam_guard.wrapping_sub(1);
                if _frame.progression.ending_latch != 0 || _frame.progression.race_intro != 0 {
                    finish_frame(_frame, _systems, false);
                } else {
                    immediate(_systems, Work::ActorBank, _frame);
                    _frame.progression.continuation = C::Spawn;
                }
            }
            C::Spawn => {
                if _systems.perform(Work::Spawn, _frame) == OwnerStep::Suspended {
                    return FrameExit::Suspended;
                }
                _frame.progression.continuation = C::AfterSpawn;
            }
            C::AfterSpawn => {
                _frame.camera.previous_x = _frame.camera.x as u8;
                *_frame.camera_displacement = 0;
                _frame.cache.changed_this_frame = 0;
                apply_queued_reaction(_frame.player);
                finish_frame(_frame, _systems, true);
            }
            C::Yield => {
                if _systems.perform(Work::Yield, _frame) == OwnerStep::Suspended {
                    return FrameExit::Suspended;
                }
                _frame.progression.continuation = C::Entry;
                return FrameExit::NextFrame;
            }
        }
    }
}
pub fn apply_queued_reaction(_player: &mut Player) {
    if let Some(action) = _player.queued.take()
        && action != crate::player::Action::Standing
        && (action != crate::player::Action::Damage || _player.protection == 0)
    {
        _player.action = action;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StagePreset {
    pub terrain_bank: u8,
    pub camera_room: u8,
    pub row: u8,
    pub player_x: u8,
    pub countdown_pair: [u8; 2],
    pub background_palette: [u8; 16],
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Checkpoint {
    pub row: u8,
    pub camera_x: u8,
    pub camera_page: u8,
    pub column_count: u8,
    pub room: u8,
    pub column: u8,
    pub player_y: u8,
    pub player_x: u8,
    pub packed_page: u8,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Catalog {
    pub stages: Vec<StagePreset>,
    pub sprite_palette: [u8; 16],
    pub checkpoints: Vec<Checkpoint>,
    pub game_over_overlay: Vec<u8>,
    pub transition_rows: [u8; 5],
    pub transition_pages: [u8; 5],
    pub transition_directions: [u8; 5],
    pub late_rows: [u8; STAGE_CONFIGURATION_COUNT],
    pub late_pages: [u8; STAGE_CONFIGURATION_COUNT],
}
impl Catalog {
    pub fn original(rom: &[u8]) -> Result<Self> {
        let data = |start: usize, count: usize| {
            rom.get(start..start + count)
                .context("progression recipe bounds")
        };
        let lookups = data(STAGE_LOOKUPS_OFFSET, 36)?;
        let backgrounds = data(0x1da44, 9 * 16)?;
        let timer = data(0x1db3c, 18)?;
        let stages = (0..9)
            .map(|i| StagePreset {
                terrain_bank: lookups[i],
                camera_room: lookups[9 + i],
                row: lookups[18 + i],
                player_x: lookups[27 + i],
                countdown_pair: [timer[i * 2], timer[i * 2 + 1]],
                background_palette: backgrounds[i * 16..i * 16 + 16]
                    .try_into()
                    .expect("sixteen"),
            })
            .collect();
        let records = data(CHECKPOINTS_OFFSET, CHECKPOINT_COUNT * CHECKPOINT_BYTES)?;
        let checkpoints = records
            .chunks_exact(9)
            .map(|r| Checkpoint {
                row: r[0],
                camera_x: r[1],
                camera_page: r[2],
                column_count: r[3],
                room: r[4],
                column: r[5],
                player_y: r[6],
                player_x: r[7],
                packed_page: r[8],
            })
            .collect();
        Ok(Self {
            stages,
            sprite_palette: data(0x1dad4, 16)?.try_into()?,
            checkpoints,
            game_over_overlay: data(GAME_OVER_OVERLAY_OFFSET, GAME_OVER_OVERLAY_BYTES)?.to_vec(),
            transition_rows: data(0x1d05d, 5)?.try_into()?,
            transition_pages: data(0x1d062, 5)?.try_into()?,
            transition_directions: data(0x1d067, 5)?.try_into()?,
            late_rows: data(0x1dc2c, 9)?.try_into()?,
            late_pages: data(0x1dc35, 9)?.try_into()?,
        })
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Boundary {
    None,
    LandSelectionRequest,
    Horizontal { direction: u8, background_row: u8 },
}
pub fn boundary(
    _progression: &mut Progression,
    _row: u8,
    _player_x: u16,
    _catalog: &Catalog,
) -> Boundary {
    let index = usize::from(_progression.stage.0);
    let Some(&row) = _catalog.transition_rows.get(index) else {
        return Boundary::None;
    };
    if row != _row {
        return Boundary::None;
    }
    let direction = _catalog.transition_directions[index];
    let adjusted = _player_x.wrapping_add_signed(if direction & 2 != 0 { 8 } else { -8 });
    if adjusted >> 8 != u16::from(_catalog.transition_pages[index]) {
        return Boundary::None;
    }
    if index == 1
        && _progression.progress_bits & 0x1f != 0x1f
        && _progression.progress_bits & 2 != 0
    {
        _progression.boundary_request = 0x80;
        return Boundary::LandSelectionRequest;
    }
    if _progression.progress_bits & 0x1f == 0x1f {
        _progression.late_route = 5;
    }
    Boundary::Horizontal {
        direction,
        background_row: if _progression.progress_bits & 0x1f == 0x1f {
            5
        } else {
            index as u8
        },
    }
}
pub fn late_position(
    _progression: &mut Progression,
    _row: u8,
    _player_page: u8,
    _catalog: &Catalog,
) {
    let index = usize::from(_progression.stage.0);
    if _catalog.late_rows.get(index) == Some(&_row)
        && _catalog.late_pages.get(index) == Some(&_player_page)
    {
        _progression.life_loss_variant = 5;
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HorizontalStep {
    pub camera: u16,
    pub player_low_fixed: u16,
    pub done: bool,
    pub previous_x: Option<u8>,
    pub clear_displacement: bool,
}
/// Player low-fixed is world-X low/fraction only. Source does not update player
/// world page in this loop; camera is the full page/low-pixel pair.
pub fn horizontal_step(_camera: u16, _player_low_fixed: u16, _direction: u8) -> HorizontalStep {
    let camera = _camera.wrapping_add_signed(if _direction & 2 != 0 { -1 } else { 1 });
    let player_low_fixed =
        _player_low_fixed.wrapping_add_signed(if _direction & 2 != 0 { -0x40 } else { 0x40 });
    let done = camera as u8 == 0;
    HorizontalStep {
        camera,
        player_low_fixed,
        done,
        previous_x: (!done).then_some(camera as u8),
        clear_displacement: !done,
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReloadRoute {
    LandSelection,
    Checkpoint { sound: u8, stage_sound: bool },
    GameOver,
}
pub fn reload_route(_request: u8, _special_room: u8, _lives: u8) -> ReloadRoute {
    if _special_room != 0 {
        ReloadRoute::LandSelection
    } else if _request >= LIFE_LOSS_FIRST {
        if _lives == 0 {
            ReloadRoute::GameOver
        } else {
            ReloadRoute::Checkpoint {
                sound: LIFE_LOSS_SOUND,
                stage_sound: true,
            }
        }
    } else {
        ReloadRoute::Checkpoint {
            sound: TRANSITION_SOUND,
            stage_sound: false,
        }
    }
}
pub fn reload_wait(_request: u8) -> u16 {
    if _request >= LIFE_LOSS_FIRST {
        LIFE_LOSS_WAIT
    } else {
        TRANSITION_WAIT
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReloadBackground {
    pub stage: StageId,
    pub row: u8,
    pub late_mode: u8,
    pub decrement_life: bool,
    pub reset_animation: bool,
    pub restore_health: bool,
}
pub fn reload_background(_request: u8, _stage: StageId, _row: u8) -> ReloadBackground {
    let death = _request >= LIFE_LOSS_FIRST;
    let late = _request == LATE_CHECKPOINT;
    ReloadBackground {
        stage: if late { StageId(5) } else { _stage },
        row: if late { 0 } else { _row },
        late_mode: if late { 0xff } else { 0 },
        decrement_life: death,
        reset_animation: death,
        restore_health: death,
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AudioRestore {
    pub stage_command: Option<u8>,
    pub countdown: Option<[u8; 3]>,
    pub divider: Option<u8>,
}
pub fn restore_audio(
    _stage: StageId,
    _late: u8,
    _current_command: u8,
    _catalog: &Catalog,
) -> AudioRestore {
    if _late & 0x80 != 0 {
        return AudioRestore {
            stage_command: Some(LATE_STAGE_SOUND),
            countdown: None,
            divider: None,
        };
    }
    if _current_command == LIFE_LOSS_SOUND {
        let pair = _catalog.stages[usize::from(_stage.0)].countdown_pair;
        AudioRestore {
            stage_command: Some(_stage.0),
            countdown: Some([pair[0], pair[1], 0]),
            divider: Some(60),
        }
    } else {
        AudioRestore {
            stage_command: None,
            countdown: None,
            divider: None,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckpointBuild {
    pub checkpoint: Checkpoint,
    pub camera_position: u16,
    pub camera_room: u8,
    pub player_page: u8,
    pub terrain_room_in_row: u8,
    pub visits: Vec<StreamCursor>,
    pub resolved: Vec<StreamCursor>,
    pub final_cursor: StreamCursor,
}
pub fn checkpoint_build(
    _request: u8,
    _catalog: &Catalog,
    _rooms: &[u8],
) -> Result<CheckpointBuild> {
    let checkpoint = *_catalog
        .checkpoints
        .get(usize::from(_request & 31))
        .context("bounded checkpoint selector")?;
    let mut cursor = StreamCursor {
        ring: checkpoint.column,
        room: checkpoint.room,
        room_in_row: checkpoint.room & 7,
    };
    let mut visits = Vec::new();
    let mut resolved = Vec::new();
    let mut count = checkpoint.column_count;
    loop {
        visits.push(cursor);
        if *_rooms
            .get(usize::from(cursor.room))
            .context("checkpoint room cursor bounds")?
            != 0xff
        {
            resolved.push(cursor);
        }
        count = count.wrapping_sub(1);
        if count & 0x80 != 0 {
            break;
        }
        cursor.ring = cursor.ring.wrapping_add(1) & 63;
        if cursor.ring == 0 {
            cursor.room = cursor.room.wrapping_add(1);
            cursor.room_in_row = cursor.room_in_row.wrapping_add(1);
        }
    }
    Ok(CheckpointBuild {
        checkpoint,
        camera_position: u16::from(checkpoint.camera_page) << 8 | u16::from(checkpoint.camera_x),
        camera_room: (checkpoint.packed_page & 0xf8) | checkpoint.camera_page,
        player_page: checkpoint.packed_page & 7,
        terrain_room_in_row: checkpoint.room & 7,
        visits,
        resolved,
        final_cursor: cursor,
    })
}
/// Exact raw selection parameter reads. This is not a sixth ordinary menu land
/// or a catalog extension. Natural reachability remains explicitly pending.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RawSelectionEntry {
    pub selector: u8,
    pub cursor_y: u8,
    pub name_y: u8,
    pub name_x: u8,
    pub animation: u8,
}
pub fn raw_selection_entry(_rom: &[u8], _progress: u8) -> Result<RawSelectionEntry> {
    let raw = crate::selection::RawLandCatalog::original(_rom)?;
    let selector = raw.first_available(_progress);
    let mut actors = [crate::actors::ActorContact::default(); crate::actors::ACTOR_SLOTS];
    raw.position(selector, &mut actors, true);
    Ok(RawSelectionEntry {
        selector: selector.0,
        cursor_y: actors[0].position.screen_y,
        name_y: actors[3].position.screen_y,
        name_x: actors[3].position.world_x as u8,
        animation: actors[2].animation,
    })
}

/// Lifecycle continuations resume only after the previously requested operation
/// has completed. Actor/status writes are explicit operations at source points,
/// never eager while building a plan (outgoing fades still see outgoing state).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Lifecycle {
    Initial,
    Reload,
    GameOver,
    Selection,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    Entry,
    BackgroundComplete,
    DisplayYieldComplete,
    FadeComplete,
    InitialWaitComplete,
    FadeOutComplete,
    FinalWaitComplete,
    Done,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Operation {
    PollBackground,
    LoadSpawnTables,
    ResetSpawnWindow,
    InitializePlayer,
    CopyStageUi,
    EnableRendering,
    Yield,
    SetGameplayDisplay,
    FadeIn,
    ClearInitialControls,
    CopyStatusAndTimer,
    RestoreHealth,
    InitializeCountdown,
    InitializeOverlaySentinel,
    RebuildSuppression,
    EnterGameplay,
    ClearImpact,
    SelectReloadRoute,
    GrantReloadProtection,
    WaitComposed(u16),
    FadeOut,
    ClearReloadControls,
    DisableRendering,
    LoadCheckpoint,
    ClearActors,
    InstallTemporaryBackground,
    RestoreBackgroundContext,
    SetForwardScroll,
    RestoreAudioCountdown,
    Spawn,
    ActivatePlayer,
    ClearReloadPlayerAndRequest,
    StageSound(u8),
    CopyGameOverOverlay,
    Reset,
    ReturnToLandSelect,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Batch {
    pub operations: Vec<Operation>,
    pub next: Phase,
}
pub fn lifecycle(_kind: Lifecycle, _phase: Phase, _request: u8, _special_room: u8) -> Batch {
    use Operation::*;
    let (operations, next) = match (_kind, _phase) {
        (Lifecycle::Initial, Phase::Entry) => (vec![PollBackground], Phase::BackgroundComplete),
        (Lifecycle::Initial, Phase::BackgroundComplete) => (
            vec![
                LoadSpawnTables,
                ResetSpawnWindow,
                InitializePlayer,
                CopyStageUi,
                EnableRendering,
                Yield,
            ],
            Phase::DisplayYieldComplete,
        ),
        (Lifecycle::Initial, Phase::DisplayYieldComplete)
        | (Lifecycle::Reload, Phase::DisplayYieldComplete) => {
            (vec![SetGameplayDisplay, FadeIn], Phase::FadeComplete)
        }
        (Lifecycle::Initial, Phase::FadeComplete) => (
            vec![
                ClearInitialControls,
                CopyStatusAndTimer,
                RestoreHealth,
                InitializeCountdown,
                InitializeOverlaySentinel,
                RebuildSuppression,
                EnterGameplay,
            ],
            Phase::Done,
        ),
        (Lifecycle::Reload, Phase::Entry) => {
            if _special_room != 0 {
                (vec![ClearImpact, ReturnToLandSelect], Phase::Done)
            } else {
                (
                    vec![
                        ClearImpact,
                        SelectReloadRoute,
                        GrantReloadProtection,
                        WaitComposed(reload_wait(_request)),
                    ],
                    Phase::InitialWaitComplete,
                )
            }
        }
        (Lifecycle::Reload, Phase::InitialWaitComplete) => (vec![FadeOut], Phase::FadeOutComplete),
        (Lifecycle::Reload, Phase::FadeOutComplete) => (
            vec![
                ClearReloadControls,
                DisableRendering,
                LoadCheckpoint,
                ClearActors,
                InstallTemporaryBackground,
                PollBackground,
            ],
            Phase::BackgroundComplete,
        ),
        (Lifecycle::Reload, Phase::BackgroundComplete) => (
            vec![
                RestoreBackgroundContext,
                SetForwardScroll,
                ResetSpawnWindow,
                EnableRendering,
                Yield,
            ],
            Phase::DisplayYieldComplete,
        ),
        (Lifecycle::Reload, Phase::FadeComplete) => (
            vec![
                RestoreAudioCountdown,
                Spawn,
                ActivatePlayer,
                WaitComposed(reload_wait(_request)),
            ],
            Phase::FinalWaitComplete,
        ),
        (Lifecycle::Reload, Phase::FinalWaitComplete) => {
            (vec![ClearReloadPlayerAndRequest], Phase::Done)
        }
        (Lifecycle::GameOver, Phase::Entry) => (
            vec![
                StageSound(GAME_OVER_SOUND),
                CopyGameOverOverlay,
                WaitComposed(GAME_OVER_WAIT),
            ],
            Phase::FinalWaitComplete,
        ),
        (Lifecycle::GameOver, Phase::FinalWaitComplete) => (vec![Reset], Phase::Done),
        (Lifecycle::Selection, Phase::Entry) => {
            if _special_room != 0 {
                (vec![ReturnToLandSelect], Phase::Done)
            } else {
                (vec![WaitComposed(LIFE_LOSS_WAIT)], Phase::FinalWaitComplete)
            }
        }
        (Lifecycle::Selection, Phase::FinalWaitComplete) => (vec![ReturnToLandSelect], Phase::Done),
        (_, Phase::Done) => (vec![], Phase::Done),
        _ => (vec![], _phase),
    };
    Batch { operations, next }
}
/// Commit only the source's runtime player defaults; player position page/X from
/// presentation stays intact. Call after background and spawn window readiness.
pub fn initialize_player(_player: &mut Player, _actors: &mut ActorPool, _camera: &mut Camera) {
    _camera.x &= 0xff00;
    let actor = &mut _actors[0];
    {
        let actor_value = 0xa0;
        actor.set_state_code(actor_value);
    };
    actor.render = 0xa0;
    actor.position.screen_y = 0x40;
    actor.animation = 0;
    actor.playback.reset_frame();
    actor.playback.restart_clock();
    actor.position.x_fraction = 0;
    _player.jump_velocity = 0x4bb;
    actor.horizontal_magnitude = 0x180;
}
/// Reload's first wait is protected; the last returns A=0, which clears action,
/// protection and request. It does not grant B4 protection after return.
pub fn finish_reload(
    _progression: &mut Progression,
    _player: &mut Player,
    _actors: &mut ActorPool,
) {
    _player.protection = 0;
    _player.action = crate::player::Action::Standing;
    _progression.reload_request = 0;
}
pub fn composed_wait_step(
    _remaining: u16,
    _playback: &mut crate::presentation::AnimationPlayback,
    _freeze_tick: bool,
) -> (u16, bool) {
    if _freeze_tick {
        _playback.restart_clock();
    }
    let count = if _remaining == 0 { 256 } else { _remaining };
    let next = count - 1;
    (next, next == 0)
}
#[cfg(test)]
mod contracts;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlReset {
    Initial,
    Reload,
}
/// These late owners have not had their own contract workers yet. Borrow the
/// canonical future B15/B16/B03 fields instead of adding copies to Progression.
pub struct TransientOwners<'a> {
    pub departure_signal: &'a mut u8,
    pub dialogue_actor_slot: &'a mut u8,
    pub overlay_gate: &'a mut u8,
}
pub fn clear_control_range(
    _kind: ControlReset,
    _progression: &mut Progression,
    _player: &mut Player,
    _status: &mut Status,
    _late_owners: &mut TransientOwners<'_>,
) {
    _player.queued = None;
    _player.momentum_control = 0;
    _player.momentum_direction = 0;
    _player.external_motion = 0;
    _status.dirty = 0;
    _player.special_mode = 0;
    _player.special_ticks = 0;
    _progression.special_room = 0;
    *_late_owners.departure_signal = 0;
    *_late_owners.dialogue_actor_slot = 0;
    _progression.late_route = 0;
    if _kind == ControlReset::Initial {
        _progression.life_loss_variant = 0;
        _player.protection = 0;
        _progression.reload_request = 0;
        *_late_owners.overlay_gate = 0;
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PresentationSetup {
    pub terrain_bank: u8,
    pub row: u8,
    pub terrain_room_in_row: u8,
    pub stage_command: u8,
}
pub fn load_presentation(
    _stage: StageId,
    _actor: &mut crate::actors::ActorContact,
    _camera: &mut Camera,
    _display: &mut crate::presentation::Display,
    _catalog: &Catalog,
) -> Result<PresentationSetup> {
    let preset = _catalog
        .stages
        .get(usize::from(_stage.0))
        .context("source stage configuration")?;
    _camera.room = preset.camera_room;
    let page = preset.camera_room & 7;
    _camera.x = (_camera.x & 0xff) | (u16::from(page) << 8);
    _actor.position.world_x = (u16::from(page) << 8) | u16::from(preset.player_x);
    _display.load_palette(&preset.background_palette, &_catalog.sprite_palette);
    Ok(PresentationSetup {
        terrain_bank: preset.terrain_bank,
        row: preset.row,
        terrain_room_in_row: page,
        stage_command: _stage.0,
    })
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SavedHorizontalContext {
    pub stage: StageId,
    pub row: u8,
}
pub fn begin_horizontal(
    _progression: &mut Progression,
    _row: &mut u8,
    _caller_stage: u8,
) -> SavedHorizontalContext {
    let saved = SavedHorizontalContext {
        stage: _progression.stage,
        row: *_row,
    };
    if _progression.progress_bits & 0x1f == 0x1f {
        *_row = 5;
        _progression.late_route = 5;
    } else {
        *_row = _caller_stage;
    }
    _progression.stage = StageId(5);
    saved
}
pub fn finish_horizontal(
    _saved: SavedHorizontalContext,
    _progression: &mut Progression,
    _row: &mut u8,
    _palette_pending: &mut bool,
) -> u8 {
    _progression.stage = _saved.stage;
    *_row = _saved.row;
    _progression.late_mode = 0xff;
    *_palette_pending = true;
    LATE_STAGE_SOUND
}

/// Executed before the final composed wait, not again after its return.
pub fn activate_reload_player(_actors: &mut ActorPool) {
    {
        let actor_value = 0xa0;
        _actors[0].set_state_code(actor_value);
    };
}

// These leaf operations cannot yield in source; resumable owners use explicit states above.
fn immediate(systems: &mut dyn Systems, work: Work, frame: &mut Frame<'_>) {
    assert_eq!(
        systems.perform(work, frame),
        OwnerStep::Complete(()),
        "non-yielding source leaf suspended"
    );
}
fn begin_deferred(frame: &mut Frame<'_>) {
    frame.progression.continuation = FrameContinuation::Reload;
}
fn begin_composition(frame: &mut Frame<'_>, systems: &mut dyn Systems) {
    *frame.oam_guard = frame.oam_guard.wrapping_add(1);
    immediate(systems, Work::HideSprites, frame);
    immediate(systems, Work::PresentationBank, frame);
    frame.progression.continuation = FrameContinuation::Composition;
}
fn finish_frame(frame: &mut Frame<'_>, systems: &mut dyn Systems, ordinary_tail: bool) {
    if ordinary_tail {
        late_position(
            frame.progression,
            *frame.room_row,
            (frame.actors[0].position.world_x >> 8) as u8,
            frame.catalog,
        );
        if *frame.secondary_cursor != 0 {
            *frame.secondary_pending = *frame.secondary_cursor;
            *frame.secondary_cursor = 0;
        }
    }
    immediate(systems, Work::AdvanceRandom, frame);
    frame.progression.continuation = FrameContinuation::Yield;
}

fn deferred_owner(
    systems: &mut dyn Systems,
    work: Work,
    enabled: bool,
    frame: &mut Frame<'_>,
) -> OwnerStep<()> {
    if !enabled && frame.progression.awaiting_owner != Some(work) {
        return OwnerStep::Complete(());
    }
    frame.progression.awaiting_owner = Some(work);
    let outcome = systems.perform(work, frame);
    if outcome == OwnerStep::Complete(()) {
        frame.progression.awaiting_owner = None;
    }
    outcome
}

/// Raw land selection can enter adjacent stage parameters. Preserve byte-width
/// palette/countdown indexing and contiguous source reads without inventing a
/// five-stage cap. This decoder is a loader boundary, never a runtime ROM read.
pub fn raw_stage_preset(rom: &[u8], stage: StageId) -> Result<StagePreset> {
    const INITIAL_ROOM_PLANE: usize = STAGE_LOOKUPS_OFFSET + STAGE_CONFIGURATION_COUNT;
    const INITIAL_ROW_PLANE: usize = INITIAL_ROOM_PLANE + STAGE_CONFIGURATION_COUNT;
    const INITIAL_X_PLANE: usize = INITIAL_ROW_PLANE + STAGE_CONFIGURATION_COUNT;
    const BACKGROUND_PALETTE_PLANE: usize = 0x1da44;
    const COUNTDOWN_PAIRS: usize = 0x1db3c;
    let index = usize::from(stage.0);
    let palette = BACKGROUND_PALETTE_PLANE + usize::from(stage.0.wrapping_mul(16));
    let timer = COUNTDOWN_PAIRS + usize::from(stage.0.wrapping_mul(2));
    let byte = |offset| {
        rom.get(offset)
            .copied()
            .context("raw stage parameter bounds")
    };
    Ok(StagePreset {
        terrain_bank: byte(STAGE_LOOKUPS_OFFSET + index)?,
        camera_room: byte(INITIAL_ROOM_PLANE + index)?,
        row: byte(INITIAL_ROW_PLANE + index)?,
        player_x: byte(INITIAL_X_PLANE + index)?,
        countdown_pair: [byte(timer)?, byte(timer + 1)?],
        background_palette: rom
            .get(palette..palette + 16)
            .context("raw stage palette")?
            .try_into()?,
    })
}

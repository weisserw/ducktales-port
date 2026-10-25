//! Final encounter, multirow race and permanent ending continuation (B17).
//! Source-derived contracts are adjacent; all game behavior awaits stage4.
use crate::{
    actors::ActorPool,
    dialogue::Dialogue,
    player::Player,
    presentation::{Composition, Display},
    progression::Progression,
    runtime::RandomState,
    scenes::Scene,
    status::Status,
    world::{Camera, StreamCursor, WorldCache},
};
use anyhow::{Context as _, Result};
pub const FINAL_BOSS_HANDLER: u8 = 0x3a;
pub const DEFEATED_HANDLER: u8 = 0x3b;
pub const CLIMBER_HANDLER: u8 = 0x3c;
pub const GOAL_HANDLER: u8 = 0x40;
pub const ATTACK_SLOT: usize = 14;
pub const LEADER_SLOT: usize = 15;
pub const OPENING_Y: u8 = 0x74;
pub const INITIAL_DELAY: u8 = 120;
pub const TELEPORT_FLASH: u8 = 60;
pub const ATTACK_DURATION: u8 = 240;
pub const DEFEAT_PAUSE: u8 = 180;
pub const DEFEAT_FRAME_TICK: u8 = 5;
pub const EXPOSED_PROFILE: u8 = 0x9b;
pub const ATTACK_Y: u8 = 0x58;
pub const ATTACK_ANIMATION: u8 = 0x85;
pub const ATTACK_HANDLER: u8 = 0x11;
pub const ATTACK_PROFILE: u8 = 0x90;
pub const LEADER_FINISH_Y: u8 = 0x90;
pub const GOAL_LOSS_X: u8 = 0xb0;
pub const GOAL_SOUND: u8 = 0x37;
pub const ENDING_AUDIO: u8 = 5;
pub const SCROLL_AUDIO: u8 = 7;
pub const ENDING_ENTRY_WAIT: u16 = 120;
pub const ENDING_PAUSE_INTERVALS: u8 = 10;
pub const ENDING_PAUSE_TICKS: u16 = 60;
pub const SPLIT_BOTTOM: u8 = 0xe8;
pub const SPLIT_VALUE_BOTTOM: u8 = 0xb0;
pub const SPLIT_AUDIO_MARK: u8 = 0xb7;
pub const SPLIT_DELAY_INTERVALS: u8 = 64;
pub const RACE_COLUMNS: u8 = 64;
pub const RACE_ROOM: u8 = 0x17;
pub const RACE_BACKGROUND_STAGE: u8 = 5;
pub const RACE_STAGE: u8 = 1;
pub const RACE_BACKGROUND_ROW: u8 = 6;
pub const RACE_ROW: u8 = 2;
pub const BOSS_TERRAIN_CELL: usize = 0x6f;
pub const BOSS_TERRAIN_CODE: u8 = 0x44;
pub const TELEPORT_TABLE_OFFSET: usize = 0x1fb64;
pub const TELEPORT_POSITIONS: usize = 16;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Catalog {
    pub positions: [u8; TELEPORT_POSITIONS],
    pub attack_offsets: [u8; 2],
    pub facing: [u8; 2],
}
impl Catalog {
    pub fn original(rom: &[u8]) -> Result<Self> {
        let data = rom
            .get(TELEPORT_TABLE_OFFSET..TELEPORT_TABLE_OFFSET + 20)
            .context("final encounter parameters")?;
        Ok(Self {
            positions: data[..16].try_into()?,
            attack_offsets: data[16..18].try_into()?,
            facing: data[18..20].try_into()?,
        })
    }
}
/// Caller arithmetic residue only; no CPU flag/register state is retained.
/// Marked B11 hits overwrite this fact before any final-boss RNG use.
pub struct ActorContext<'a> {
    pub player: &'a mut Player,
    pub progression: &'a mut Progression,
    pub dialogue: &'a mut Dialogue,
    pub status: &'a mut Status,
    pub random: &'a mut RandomState,
    pub terrain: &'a mut WorldCache,
    pub current_row: u8,
    pub camera_page: u8,
    pub random_increment: bool,
    pub catalog: &'a Catalog,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AudioRequest {
    Stage(u8),
    Effect(u8),
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ActorEffects {
    pub audio: Vec<AudioRequest>,
}
/// Overlap is B05's canonical contact operation. It may mutate owners before
/// the caller rereads Player.action; true means source overlap carry is clear.
pub trait Contact {
    fn overlaps(&mut self, pool: &mut ActorPool, player: &mut Player, slot: usize) -> bool;
}
pub fn actor_step(
    _pool: &mut ActorPool,
    _slot: usize,
    _context: &mut ActorContext<'_>,
    _contact: &mut impl Contact,
) -> ActorEffects {
    use crate::player::Action;
    let slot = _slot;
    let mut effects = ActorEffects::default();
    match _pool[slot].handler {
        0x3a => {
            let mut increment = _context.random_increment;
            if _pool[slot].hit_marker != 0 {
                let result = crate::actors::bosses::hit(
                    _pool,
                    slot,
                    &mut _context.progression.boss_hit_points,
                    _context.camera_page,
                );
                if result.defeated {
                    let actor = &mut _pool[slot];
                    actor.animation = 0x2a;
                    actor.animation_tick = 0;
                    actor.animation_frame = 0;
                    actor.profile = 0;
                    actor.state = 0xa0;
                    actor.handler = DEFEATED_HANDLER;
                    return effects;
                }
                increment = false;
            }
            _pool[slot].animation_tick = 0;
            if _pool[slot].state & 15 == 0 {
                _pool[slot].position.screen_y = OPENING_Y;
                _pool[slot].local_state = INITIAL_DELAY;
                _context.terrain.collision[BOSS_TERRAIN_CELL] = BOSS_TERRAIN_CODE;
                _pool[slot].state = _pool[slot].state.wrapping_add(1);
            }
            if _pool[slot].local_state != 0 {
                _pool[slot].local_state -= 1;
                if _pool[slot].local_state != 0 {
                    return effects;
                }
                let words = &mut _context.random.words;
                words[0] = words[0]
                    .wrapping_add(words[1])
                    .wrapping_add(u8::from(increment));
                _pool[slot].position.world_x = (_pool[slot].position.world_x & 0xff00)
                    | u16::from(_context.catalog.positions[usize::from(words[0] & 15)]);
                _pool[slot].local_counter = TELEPORT_FLASH;
                _pool[slot].render |= 2;
                _pool[slot].state &= !0x10;
                _pool[slot].direction = if _pool[slot].position.world_x < _pool[0].position.world_x
                {
                    1
                } else {
                    2
                };
                _pool[slot].render = (_pool[slot].render & !0x40)
                    | _context.catalog.facing[usize::from(_pool[slot].direction & 1)];
                return effects;
            }
            if _pool[slot].local_counter != 0 {
                _pool[slot].local_counter -= 1;
                if _pool[slot].local_counter != 0 {
                    return effects;
                }
                _pool[slot].render &= 0xf0;
                _pool[slot].animation_frame = 1;
                _pool[slot].profile = EXPOSED_PROFILE;
                let parent = _pool[slot];
                let actor = &mut _pool[ATTACK_SLOT];
                actor.position.screen_y = ATTACK_Y;
                actor.animation_tick = 0;
                actor.animation_frame = 0;
                actor.hit_marker = 0;
                actor.state = 0xa0;
                actor.render = 0xa0;
                actor.animation = ATTACK_ANIMATION;
                actor.handler = ATTACK_HANDLER;
                actor.profile = ATTACK_PROFILE;
                actor.position.world_x = (parent.position.world_x & 0xff00)
                    | u16::from((parent.position.world_x as u8).wrapping_add(
                        _context.catalog.attack_offsets[usize::from(parent.direction & 1)],
                    ));
                actor.direction = parent.direction;
                _pool[slot].local_aux2 = ATTACK_DURATION;
                return effects;
            }
            if _pool[slot].local_aux2 == TELEPORT_FLASH {
                _pool[slot].render |= 2;
                _pool[slot].animation_frame = 0;
            }
            _pool[slot].local_aux2 = _pool[slot].local_aux2.wrapping_sub(1);
            if _pool[slot].local_aux2 == 0 {
                _pool[slot].profile = 0;
                _pool[slot].state |= 0x10;
                _pool[slot].local_state = TELEPORT_FLASH;
            }
        }
        DEFEATED_HANDLER => {
            if _pool[slot].state & 15 == 0 {
                if _pool[slot].animation_frame == 0
                    || _pool[slot].animation_tick != DEFEAT_FRAME_TICK
                {
                    return effects;
                }
                _pool[slot].state = 0xb1;
                _pool[slot].local_state = DEFEAT_PAUSE;
            }
            if _pool[slot].local_state != 0 {
                _pool[slot].local_state -= 1;
            } else {
                _pool[slot].state = 0;
                _context.player.action = Action::Dialogue;
                _context.progression.race_intro = _context.progression.race_intro.wrapping_add(1);
                _context.dialogue.message = 0x0b;
            }
        }
        CLIMBER_HANDLER => {
            if _pool[slot].state & 15 == 0 {
                _pool[slot].local_aux2 = 0;
                _pool[slot].local_state = _pool[slot].position.screen_y;
                _pool[slot].local_counter = 2;
                _pool[slot].state = _pool[slot].state.wrapping_add(1);
            }
            if _pool[LEADER_SLOT].local_counter == 0
                && _pool[LEADER_SLOT].local_state < LEADER_FINISH_Y
            {
                _pool[LEADER_SLOT].local_aux2 = _pool[LEADER_SLOT].local_state;
                if _pool[LEADER_SLOT].local_state != 0 {
                    return effects;
                }
            }
            let actor = &mut _pool[slot];
            let (low, borrow) = actor.local_state.overflowing_sub(1);
            actor.local_state = if borrow { low.wrapping_sub(16) } else { low };
            if borrow {
                actor.local_counter = actor.local_counter.wrapping_sub(1);
            }
            if actor.local_counter == _context.current_row {
                actor.state &= !0x10;
                actor.position.screen_y = actor.local_state;
            } else {
                actor.state |= 0x10;
            }
        }
        GOAL_HANDLER => {
            if _pool[slot].state & 15 == 0 {
                if _pool[LEADER_SLOT].local_aux2 != 0 {
                    let next = (_pool[slot].position.world_x as u8).wrapping_add(1);
                    _pool[slot].position.world_x =
                        (_pool[slot].position.world_x & 0xff00) | u16::from(next);
                    if next >= GOAL_LOSS_X {
                        _pool[slot].state = 0;
                        if _context.player.action != Action::Death {
                            _context.dialogue.message = 0x0c;
                            _context.player.action = Action::Dialogue;
                        }
                        return effects;
                    }
                }
                if !_contact.overlaps(_pool, _context.player, slot)
                    || _context.player.action == Action::Death
                {
                    return effects;
                }
                _pool[slot].state = _pool[slot].state.wrapping_add(1);
                effects
                    .audio
                    .extend([AudioRequest::Stage(0xff), AudioRequest::Effect(GOAL_SOUND)]);
                _pool[slot].local_state = 0xf0;
                _context.player.action = Action::Hold;
            }
            _context.status.divider = 0;
            _pool[slot].local_state = _pool[slot].local_state.wrapping_sub(1);
            if _pool[slot].local_state == 0 {
                _context.progression.ending_latch =
                    _context.progression.ending_latch.wrapping_add(1);
            }
        }
        _ => {}
    }
    effects
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RacePhase {
    #[default]
    Entry,
    Clear,
    InitialUpload,
    Column,
    Resolve,
    Yield,
    FinalUpload,
    FinalYield,
    SpawnStart,
    Spawn,
    Done,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RaceTerrain {
    pub phase: RacePhase,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnerResult {
    Running,
    Complete,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RaceWork {
    ClearNonplayer,
    PollBackground,
    ResolveForwardColumn,
    Yield,
    FindSpawnWindow,
    SpawnWindow,
    Suspended,
    Complete,
}
/// All fields borrow the existing B04/B13/B03 owners. Background completion is
/// the exact signed source upload state; callbacks mutate it before returning.
pub struct RaceContext<'a> {
    pub pool: &'a mut ActorPool,
    pub progression: &'a mut Progression,
    pub camera: &'a mut Camera,
    pub cursor: &'a mut StreamCursor,
    pub row: &'a mut u8,
    pub background_state: &'a mut u8,
    pub display: &'a mut Display,
}
impl RaceTerrain {
    pub fn resume(&mut self, _context: &mut RaceContext<'_>, _owner: OwnerResult) -> RaceWork {
        if self.phase != RacePhase::Entry && _owner == OwnerResult::Running {
            return RaceWork::Suspended;
        }
        loop {
            match self.phase {
                RacePhase::Entry => {
                    self.phase = RacePhase::Clear;
                    return RaceWork::ClearNonplayer;
                }
                RacePhase::Clear => {
                    _context.cursor.ring = 0;
                    *_context.background_state = 0;
                    _context.cursor.room = RACE_ROOM;
                    _context.camera.room = RACE_ROOM;
                    _context.cursor.room_in_row = RACE_ROOM & 7;
                    _context.pool[0].position.world_x =
                        (_context.pool[0].position.world_x & 0xff) | 0x700;
                    _context.camera.x = (_context.camera.x & 0xff) | 0x700;
                    _context.progression.stage = crate::identifiers::StageId(RACE_BACKGROUND_STAGE);
                    *_context.row = RACE_BACKGROUND_ROW;
                    self.phase = RacePhase::InitialUpload;
                    return RaceWork::PollBackground;
                }
                RacePhase::InitialUpload => {
                    _context.progression.stage = crate::identifiers::StageId(RACE_STAGE);
                    self.phase = RacePhase::Column;
                }
                RacePhase::Column => {
                    if _context.cursor.ring == RACE_COLUMNS {
                        _context.progression.stage =
                            crate::identifiers::StageId(RACE_BACKGROUND_STAGE);
                        self.phase = RacePhase::FinalUpload;
                        return RaceWork::PollBackground;
                    }
                    self.phase = RacePhase::Resolve;
                    return RaceWork::ResolveForwardColumn;
                }
                RacePhase::Resolve => {
                    _context.cursor.ring = _context.cursor.ring.wrapping_add(1);
                    self.phase = RacePhase::Yield;
                    return RaceWork::Yield;
                }
                RacePhase::Yield => self.phase = RacePhase::Column,
                RacePhase::FinalUpload => {
                    _context.progression.stage = crate::identifiers::StageId(RACE_STAGE);
                    if *_context.background_state & 0x80 == 0 {
                        self.phase = RacePhase::FinalYield;
                        return RaceWork::Yield;
                    }
                    *_context.row = RACE_ROW;
                    self.phase = RacePhase::SpawnStart;
                    return RaceWork::FindSpawnWindow;
                }
                RacePhase::FinalYield => {
                    _context.progression.stage = crate::identifiers::StageId(RACE_BACKGROUND_STAGE);
                    self.phase = RacePhase::FinalUpload;
                    return RaceWork::PollBackground;
                }
                RacePhase::SpawnStart => {
                    _context.progression.late_mode = 0;
                    self.phase = RacePhase::Spawn;
                    return RaceWork::SpawnWindow;
                }
                RacePhase::Spawn => {
                    _context.display.palette_pending = true;
                    self.phase = RacePhase::Done;
                    return RaceWork::Complete;
                }
                RacePhase::Done => return RaceWork::Complete,
            }
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum EndingPhase {
    #[default]
    Entry,
    Transfer,
    EntryWait,
    Audio,
    Layout,
    Build,
    SceneActors,
    SplitPixel,
    SplitYield,
    SplitRaster,
    SplitAudio,
    Pause,
    Terminal,
}
/// Sequence-owned cursors, not duplicate scene content, progression or display.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Ending {
    pub phase: EndingPhase,
    pub variation: u8,
    pub pauses_left: u8,
    pub split_delay: u8,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndingWork {
    TransferValues,
    Wait(u16),
    StageAudio(u8),
    Layout(u8),
    Build(Scene),
    UpdateSceneActors,
    Yield,
    SplitRaster(u8),
    Suspended,
    Complete,
}
pub struct EndingContext<'a> {
    /// Shared B16 ContentContext cursor/display fields; no copies across fades.
    pub scene_cursor: &'a mut u8,
    pub display_mode: &'a mut u8,
    pub progression: &'a mut Progression,
    pub status: &'a mut Status,
    pub pool: &'a mut ActorPool,
    pub composition: &'a mut Composition,
    pub display: &'a mut Display,
    pub current_stage_audio: &'a mut u8,
}
impl Ending {
    pub fn resume(&mut self, _context: &mut EndingContext<'_>, _owner: OwnerResult) -> EndingWork {
        if self.phase != EndingPhase::Entry && _owner == OwnerResult::Running {
            return EndingWork::Suspended;
        }
        loop {
            match self.phase {
                EndingPhase::Entry => {
                    if _context.progression.ending_latch & 0x80 == 0 {
                        _context.progression.ending_latch |= 0x80;
                        self.phase = EndingPhase::Transfer;
                        return EndingWork::TransferValues;
                    }
                    if _context.pool[0].handler != crate::scenes::SceneHandler::Settled as u8 {
                        self.phase = EndingPhase::SceneActors;
                        return EndingWork::UpdateSceneActors;
                    }
                    if *_context.display_mode != 0 {
                        *_context.display_mode = 0;
                        self.split_delay = 0;
                        self.phase = EndingPhase::SplitPixel;
                    } else {
                        self.pauses_left = ENDING_PAUSE_INTERVALS;
                        self.phase = EndingPhase::Pause;
                        return EndingWork::Wait(ENDING_PAUSE_TICKS);
                    }
                }
                EndingPhase::Transfer => {
                    self.phase = EndingPhase::EntryWait;
                    return EndingWork::Wait(ENDING_ENTRY_WAIT);
                }
                EndingPhase::EntryWait => {
                    self.phase = EndingPhase::Audio;
                    return EndingWork::StageAudio(ENDING_AUDIO);
                }
                EndingPhase::Audio => {
                    _context.progression.stage =
                        crate::identifiers::StageId(crate::scenes::ENDING_LAYOUT);
                    self.phase = EndingPhase::Layout;
                    return EndingWork::Layout(crate::scenes::ENDING_LAYOUT);
                }
                EndingPhase::Layout => {
                    self.variation = if _context.progression.progress_bits & 0x7f == 0x7f
                        && _context.status.total[0] != crate::status::BLANK
                    {
                        2
                    } else if _context.status.total[6] == crate::status::BLANK {
                        1
                    } else {
                        0
                    };
                    *_context.scene_cursor = self.variation;
                    self.phase = EndingPhase::Build;
                    return EndingWork::Build(scene_from_cursor(self.variation));
                }
                EndingPhase::Build => {
                    *_context.scene_cursor = _context.scene_cursor.wrapping_add(1);
                    if *_context.scene_cursor == 6 {
                        *_context.scene_cursor = 0;
                    }
                    self.phase = EndingPhase::Entry;
                    return EndingWork::Complete;
                }
                EndingPhase::SceneActors => {
                    self.phase = EndingPhase::Entry;
                    return EndingWork::Complete;
                }
                EndingPhase::SplitPixel => {
                    _context.composition.oam[0].y = _context.composition.oam[0].y.wrapping_add(1);
                    let y = _context.composition.oam[0].y;
                    if y == SPLIT_BOTTOM || (*_context.scene_cursor == 5 && y == SPLIT_VALUE_BOTTOM)
                    {
                        self.phase = EndingPhase::Entry;
                        return EndingWork::Complete;
                    }
                    self.phase = EndingPhase::SplitYield;
                    return EndingWork::Yield;
                }
                EndingPhase::SplitYield => {
                    self.phase = EndingPhase::SplitRaster;
                    return EndingWork::SplitRaster(_context.display.control | 1);
                }
                EndingPhase::SplitRaster => {
                    let y = _context.composition.oam[0].y;
                    if y & 7 != 7 {
                        self.phase = EndingPhase::SplitPixel;
                        continue;
                    }
                    if *_context.scene_cursor == 0 && y >= SPLIT_AUDIO_MARK {
                        if *_context.current_stage_audio != SCROLL_AUDIO {
                            self.phase = EndingPhase::SplitAudio;
                            return EndingWork::StageAudio(SCROLL_AUDIO);
                        }
                        self.phase = EndingPhase::SplitPixel;
                        continue;
                    }
                    self.split_delay = self.split_delay.wrapping_add(1) & 0x3f;
                    if self.split_delay != 0 {
                        self.phase = EndingPhase::SplitYield;
                        return EndingWork::Yield;
                    }
                    self.phase = EndingPhase::SplitPixel;
                }
                EndingPhase::SplitAudio => self.phase = EndingPhase::SplitPixel,
                EndingPhase::Pause => {
                    self.pauses_left = self.pauses_left.wrapping_sub(1);
                    if self.pauses_left != 0 {
                        return EndingWork::Wait(ENDING_PAUSE_TICKS);
                    }
                    if *_context.scene_cursor == 0 {
                        self.phase = EndingPhase::Terminal;
                        return EndingWork::Yield;
                    }
                    if *_context.scene_cursor < 4 {
                        *_context.scene_cursor = 4;
                    }
                    self.phase = EndingPhase::Build;
                    return EndingWork::Build(scene_from_cursor(*_context.scene_cursor));
                }
                EndingPhase::Terminal => return EndingWork::Yield,
            }
        }
    }
}
#[cfg(test)]
mod contracts;

fn scene_from_cursor(cursor: u8) -> Scene {
    match cursor {
        0 => Scene::OpeningFirst,
        1 => Scene::OpeningSecond,
        2 => Scene::OpeningThird,
        3 => Scene::Interior,
        4 => Scene::Value,
        5 => Scene::TerminalLead,
        _ => unreachable!("source ending scene cursor"),
    }
}

//! Special-room and ending-screen content, scene actors and departures (B16).
//! Source trace: trace.md. Declarations only; implementation awaits stage4.
mod actor_state;
use crate::{
    actors::{ActorContact, ActorPool},
    player::Player,
    presentation::Display,
    progression::Progression,
    runtime::RandomState,
    status::Status,
    world::{Camera, StreamCursor},
};
pub use actor_state::{
    BounceProgress, BounceState, DelayState, EntryProgress, MoveState, SceneActorState, WalkState,
};
use anyhow::{Context as _, Result, ensure};
pub const SCENE_COUNT: usize = 6;
pub const SCENE_LISTS_OFFSET: usize = 0x173dc;
pub const SCENE_LISTS_BYTES: usize = 52;
pub const SCENE_HANDLERS_OFFSET: usize = 0x1751d;
pub const SCENE_PARAMETERS_OFFSET: usize = 0x1e12b;
pub const SPECIAL_RECORDS_OFFSET: usize = 0x1dfdf;
pub const REWARD_DELAYS_OFFSET: usize = 0x1fa00;
pub const COMPLETION_SLOT: usize = 8;
pub const COMPLETION_FIRST: u8 = 0x6f;
pub const COMPLETION_LAST: u8 = 0x73;
pub const COMPLETION_GRAVITY: u8 = 0x40;
pub const COMPLETION_UPWARD_SPEED: i16 = 0x077d;
pub const COMPLETION_HORIZONTAL_SPEED: u16 = 0x0200;
pub const COMPLETION_START_Y: u8 = 0xc0;
pub const COMPLETION_START_X: u16 = 0x0080;
pub const FLASH_COLOR: u8 = 0x16;
pub const FLASH_BLACK: u8 = 0x0f;
pub const FLASH_TICKS: u16 = 4;
pub const FLASH_CYCLES: u8 = 16;
pub const BOUNCE_COUNT: u8 = 11;
pub const BOUNCE_SPEED: i16 = 0x0400;
pub const BOUNCE_FLOOR: u8 = 0xb4;
pub const WALK_MARK_X: u8 = 0x90;
pub const MOVE_MARK_X: u8 = 0x48;
pub const MOVE_MARK_Y: u8 = 0x78;
pub const MOVE_SPEED: u16 = 0x0300;
pub const ONE_PIXEL_SPEED: u16 = 0x0100;
pub const RENDER_SCENE: u8 = 0x90;
pub const FROZEN_TICK: u8 = 0x80;
pub const DEPARTURE_HANDLER: u8 = 0x36;
pub const SPAWNER_HANDLER: u8 = 0x37;
pub const REWARD_CODE_GATE: u8 = 7;
pub const BRAKE_START_X: u8 = 0xb0;
pub const BRAKE_STEP: u16 = 4;
pub const DEPARTURE_LEFT_EDGE: u8 = 0x10;
pub const SPAWNER_DELAY: u8 = 60;
pub const SPAWNER_DOWN_SPEED: i16 = 0x0200;
pub const SPAWNER_DOWN_MARK: u8 = 0x50;
pub const SPAWNER_LEFT_MARK: u8 = 0x70;
pub const SPAWNER_EXIT_MARK: u8 = 0x10;
pub const SPAWNER_EXIT_SLOT: usize = 3;
pub const REWARD_FIRST_SLOT: usize = 9;
pub const REWARD_ANIMATION: u8 = 0x28;
pub const REWARD_HANDLER: u8 = 0x13;
pub const SPECIAL_AUDIO: u8 = 0x0b;
pub const GAMEPLAY_DISPLAY: u8 = 0x40;
pub const ENDING_LAYOUT: u8 = 9;
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum Scene {
    OpeningFirst,
    OpeningSecond,
    OpeningThird,
    Interior,
    Value,
    TerminalLead,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum SceneHandler {
    Bounce,
    WalkIn,
    Delay,
    MoveToMark,
    Settle,
    Settled,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Placement {
    pub animation: u8,
    pub y: u8,
    pub x: u8,
    pub flipped: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActorList {
    pub slots: Vec<Option<Placement>>,
    pub handler: SceneHandler,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpecialPlacement {
    pub animation: u8,
    pub x: u16,
    pub y: u8,
    pub handler: u8,
    pub profile: u8,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SpecialRoom {
    Linked,
    RewardFlight,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpecialDefinition {
    pub records: Vec<SpecialPlacement>,
    pub player_action: crate::player::Action,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneParameters {
    pub terrain: u8,
    pub chr: u8,
    pub background_end: u8,
    pub sprite_end: u8,
    pub script_offset: u8,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Catalog {
    pub actors: [ActorList; SCENE_COUNT],
    pub parameters: [SceneParameters; SCENE_COUNT],
    pub special: [SpecialDefinition; 2],
    pub reward_delays: [u8; 8],
    pub handlers: [SceneHandler; SCENE_COUNT],
    pub scripts: Vec<u8>,
    pub split_y: [u8; 2],
}
impl Catalog {
    pub fn original(rom: &[u8]) -> Result<Self> {
        let data = |start: usize, count: usize| {
            rom.get(start..start + count).context("scene recipe bounds")
        };
        let lists = data(SCENE_LISTS_OFFSET, SCENE_LISTS_BYTES)?;
        let mut decoded = Vec::new();
        for i in 0..SCENE_COUNT {
            decoded.push(actor_list(&lists[6..], lists[i])?);
        }
        let parameters = data(SCENE_PARAMETERS_OFFSET, 40)?;
        let special = data(SPECIAL_RECORDS_OFFSET, 30)?;
        let mut definitions = Vec::new();
        for mode in 0..2 {
            definitions.push(SpecialDefinition {
                records: special_list(&special[2..28], special[mode])?,
                player_action: if special[28 + mode] == 0x0e {
                    crate::player::Action::Hold
                } else {
                    crate::player::Action::Standing
                },
            });
        }
        let pointers = data(SCENE_HANDLERS_OFFSET, 12)?;
        let mut handlers = [SceneHandler::Settled; SCENE_COUNT];
        for i in 0..SCENE_COUNT {
            let pointer = u16::from(pointers[i]) | (u16::from(pointers[6 + i]) << 8);
            handlers[i] = match pointer {
                0xb412 => SceneHandler::Bounce,
                0xb46a => SceneHandler::WalkIn,
                0xb48f => SceneHandler::Delay,
                0xb4b5 => SceneHandler::MoveToMark,
                0xb507 => SceneHandler::Settled,
                _ => anyhow::bail!("unknown scene dispatch identity"),
            };
        }
        Ok(Self {
            actors: decoded.try_into().expect("six scenes"),
            parameters: std::array::from_fn(|i| SceneParameters {
                terrain: parameters[i],
                chr: parameters[6 + i],
                background_end: parameters[12 + i],
                sprite_end: parameters[18 + i],
                script_offset: parameters[24 + i],
            }),
            special: definitions.try_into().expect("two special modes"),
            reward_delays: data(REWARD_DELAYS_OFFSET, 8)?.try_into()?,
            handlers,
            scripts: parameters[30..38].to_vec(),
            split_y: parameters[38..40].try_into()?,
        })
    }
}
/// Decode authored packed records; a skipped slot remains None. The flip marker
/// uses the absolute byte cursor, not the resulting actor index.
pub fn actor_list(_bytes: &[u8], _offset: u8) -> Result<ActorList> {
    let mut cursor = _offset;
    let mut slots = Vec::new();
    loop {
        let animation = *_bytes
            .get(usize::from(cursor))
            .context("scene list terminator bounds")?;
        if animation < 0x10 {
            let handler = match animation {
                0 => SceneHandler::Bounce,
                1 => SceneHandler::WalkIn,
                2 => SceneHandler::Delay,
                3 => SceneHandler::MoveToMark,
                4 => SceneHandler::Settle,
                5 => SceneHandler::Settled,
                _ => anyhow::bail!("scene terminal handler"),
            };
            return Ok(ActorList { slots, handler });
        }
        let triple = _bytes
            .get(usize::from(cursor)..usize::from(cursor) + 3)
            .context("scene actor triple bounds")?;
        slots.push(if animation == 0xff {
            None
        } else {
            Some(Placement {
                animation,
                y: triple[1],
                x: triple[2],
                flipped: cursor == 0x19,
            })
        });
        ensure!(slots.len() <= 16, "scene actor slot bounds");
        cursor = cursor.wrapping_add(3);
        ensure!(cursor != 0, "scene list cursor wrap");
    }
}
pub fn special_list(_bytes: &[u8], _offset: u8) -> Result<Vec<SpecialPlacement>> {
    let mut at = usize::from(_offset);
    let mut records = Vec::new();
    loop {
        let animation = *_bytes.get(at).context("special list terminator bounds")?;
        if animation & 0x80 != 0 {
            return Ok(records);
        }
        let r = _bytes.get(at..at + 6).context("special placement bounds")?;
        records.push(SpecialPlacement {
            animation,
            x: u16::from_le_bytes([r[1], r[2]]),
            y: r[3],
            handler: r[4],
            profile: r[5],
        });
        at += 6;
        ensure!(records.len() <= 8, "special even slot bounds");
    }
}
pub fn initialize_actors(_pool: &mut ActorPool, _list: &ActorList) {
    for (slot, placement) in _list.slots.iter().enumerate() {
        if let Some(p) = placement {
            let actor = &mut _pool[slot];
            actor.animation = p.animation;
            actor.position.screen_y = p.y;
            actor.position.world_x = u16::from(p.x);
            actor.playback.restart_clock();
            actor.playback.reset_frame();
            {
                let actor_value = 0x80;
                actor.set_state_code(actor_value);
            };
            actor.render = RENDER_SCENE | if p.flipped { 0x40 } else { 0 };
        }
    }
    {
        let actor_value = _list.handler as u8;
        _pool[0].set_handler(actor_value);
    };
    _pool[0].enter_scene();
}
pub fn initialize_special_actors(
    _pool: &mut ActorPool,
    _definition: &SpecialDefinition,
    _player: &mut Player,
) {
    for (index, record) in _definition.records.iter().enumerate() {
        let actor = &mut _pool[index * 2];
        actor.animation = record.animation;
        actor.position.world_x = record.x;
        actor.position.screen_y = record.y;
        {
            let actor_value = record.handler;
            actor.set_handler(actor_value);
        };
        actor.profile = record.profile;
        {
            let actor_value = 0xa0;
            actor.set_state_code(actor_value);
        };
        actor.render = 0xa0;
        actor.playback.restart_clock();
        actor.playback.reset_frame();
    }
    _player.action = _definition.player_action;
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Integration {
    Gravity,
    SubtractY,
    AddY,
    SubtractX,
    AddX,
}
/// Actual motion/calculation owner B05 mutates the canonical actor. Returned bool
/// is vertical no-borrow/wrap fact where a caller consumes it, not global flags.
pub trait Motion {
    fn apply(&mut self, actor: &mut ActorContact, operation: Integration, gravity: u8) -> bool;
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum HandlerContinuation {
    #[default]
    Entry,
    AwaitRebuild,
    AwaitComposition,
    Done,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HandlerStep {
    Complete,
    RebuildInterior,
    Compose,
    Suspended,
}
pub fn update_scene_actor(
    _pool: &mut ActorPool,
    catalog: &Catalog,
    _gravity: u8,
    _motion: &mut impl Motion,
    _continuation: &mut HandlerContinuation,
    _display: &mut Display,
) -> HandlerStep {
    if *_continuation == HandlerContinuation::AwaitRebuild {
        initialize_actors(_pool, &catalog.actors[Scene::Interior as usize]);
        *_continuation = HandlerContinuation::AwaitComposition;
        return HandlerStep::Compose;
    }
    if *_continuation == HandlerContinuation::AwaitComposition {
        _display.palette_pending = true;
        _display.control |= 1;
        {
            let actor_value = SceneHandler::Settled as u8;
            _pool[0].set_handler(actor_value);
        };
        *_continuation = HandlerContinuation::Done;
        return HandlerStep::Complete;
    }
    if *_continuation == HandlerContinuation::Done {
        return HandlerStep::Complete;
    }
    let actor = &mut _pool[0];
    actor.enter_scene();
    let result = step_scene_motion(actor, _gravity, _motion, _continuation);
    // A callback may replace the source handler before this invocation finishes.
    // Reconcile only at that real ownership boundary, preserving partial fields.
    actor.enter_scene();
    result
}
fn step_scene_motion(
    actor: &mut ActorContact,
    gravity: u8,
    motion: &mut impl Motion,
    continuation: &mut HandlerContinuation,
) -> HandlerStep {
    match actor.scene_mut().kind() {
        Some(SceneHandler::Bounce) => {
            let bounce = actor.scene_for(SceneHandler::Bounce).bounce_mut();
            if bounce.progress.is_entering() {
                bounce.bounces_remaining = BOUNCE_COUNT;
                bounce.progress = BounceProgress::Rising;
                actor.vertical_velocity = BOUNCE_SPEED;
            }
            if !actor
                .scene_for(SceneHandler::Bounce)
                .bounce_mut()
                .progress
                .descending()
            {
                motion.apply(actor, Integration::SubtractY, gravity);
                motion.apply(actor, Integration::Gravity, gravity);
                if actor.vertical_velocity < 0 {
                    actor.vertical_velocity = 0;
                    actor.advance_scene_bounce(true);
                    let bounce = actor.scene_for(SceneHandler::Bounce).bounce_mut();
                    bounce.bounces_remaining = bounce.bounces_remaining.wrapping_sub(1);
                    if bounce.bounces_remaining == 0 {
                        scene_move_entry(actor);
                    }
                }
            } else {
                motion.apply(actor, Integration::Gravity, gravity);
                motion.apply(actor, Integration::SubtractY, gravity);
                if actor.position.screen_y >= BOUNCE_FLOOR {
                    actor.vertical_velocity = BOUNCE_SPEED;
                    actor.advance_scene_bounce(false);
                }
            }
        }
        Some(SceneHandler::WalkIn) => {
            let walk = actor.scene_for(SceneHandler::WalkIn).walk_mut();
            if walk.progress.is_entering() {
                walk.pause_remaining = 0;
                walk.progress.activate();
                actor.horizontal_magnitude = ONE_PIXEL_SPEED;
            }
            if actor.position.world_x as u8 >= WALK_MARK_X {
                motion.apply(actor, Integration::SubtractX, gravity);
            } else {
                let walk = actor.scene_for(SceneHandler::WalkIn).walk_mut();
                walk.pause_remaining = walk.pause_remaining.wrapping_sub(1);
                if walk.pause_remaining == 0 {
                    scene_move_entry(actor);
                }
            }
        }
        Some(SceneHandler::Delay) => {
            let delay = actor.scene_for(SceneHandler::Delay).delay_mut();
            if delay.progress.is_entering() {
                delay.ticks_remaining = 0;
                delay.progress.activate();
            } else {
                delay.ticks_remaining = delay.ticks_remaining.wrapping_sub(1);
                if delay.ticks_remaining == 0 {
                    scene_move_entry(actor);
                }
            }
        }
        Some(SceneHandler::MoveToMark) => {
            let movement = actor.scene_for(SceneHandler::MoveToMark).movement_mut();
            if movement.progress.is_entering() {
                movement.wait_remaining = movement.wait_remaining.wrapping_sub(1);
                if movement.wait_remaining != 0 {
                    return HandlerStep::Complete;
                }
                movement.progress.activate();
                actor.horizontal_magnitude = MOVE_SPEED;
                actor.vertical_velocity = MOVE_SPEED as i16;
            }
            if actor.position.world_x as u8 >= MOVE_MARK_X {
                motion.apply(actor, Integration::SubtractX, gravity);
                return HandlerStep::Complete;
            }
            if actor.position.screen_y >= MOVE_MARK_Y {
                motion.apply(actor, Integration::SubtractY, gravity);
                return HandlerStep::Complete;
            }
            let movement = actor.scene_for(SceneHandler::MoveToMark).movement_mut();
            movement.wait_remaining = movement.wait_remaining.wrapping_sub(1);
            if movement.wait_remaining == 0 {
                *continuation = HandlerContinuation::AwaitRebuild;
                return HandlerStep::RebuildInterior;
            }
        }
        Some(SceneHandler::Settle | SceneHandler::Settled) => {
            actor.set_handler(SceneHandler::Settled as u8)
        }
        None => {}
    }
    HandlerStep::Complete
}
/// Caller state borrowed from B13; physical/animation state remains in ActorPool.
pub struct DepartureContext<'a> {
    pub progression: &'a mut Progression,
    pub player: &'a mut Player,
    pub status: &'a mut Status,
    pub random: &'a mut RandomState,
    pub departure_signal: &'a mut u8,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DepartureResult {
    Complete,
    LifeLoss,
}
pub fn linked_departure(
    _pool: &mut ActorPool,
    _slot: usize,
    _context: &mut DepartureContext<'_>,
    _motion: &mut impl Motion,
    _gravity: u8,
) {
    _pool[0].playback.restart_clock();
    _context.status.divider = 0;
    let actor = &mut _pool[_slot];
    if actor.state_code() & 15 == 0 {
        actor.vertical_velocity = 0;
        actor.horizontal_magnitude = ONE_PIXEL_SPEED;
        {
            let actor_value = actor.state_code().wrapping_add(1);
            actor.set_state_code(actor_value);
        };
    }
    if actor.state_code() & 2 != 0 {
        _motion.apply(actor, Integration::Gravity, _gravity);
        let no_borrow = _motion.apply(actor, Integration::SubtractY, _gravity);
        _pool[0].position.screen_y = _pool[_slot].position.screen_y;
        if no_borrow {
            _context.progression.special_room = 0x82;
        }
        return;
    }
    // The fraction subtraction supplies borrow without storing that fraction.
    let fraction_borrow = actor.position.x_fraction < (actor.horizontal_magnitude as u8);
    actor.position.world_x = actor
        .position
        .world_x
        .wrapping_sub(actor.horizontal_magnitude >> 8)
        .wrapping_sub(u16::from(fraction_borrow));
    _pool[0].position.world_x = _pool[_slot].position.world_x;
    if _context.status.reward.departure_brakes() {
        let actor = &mut _pool[_slot];
        if actor.position.world_x as u8 >= BRAKE_START_X {
            return;
        }
        actor.horizontal_magnitude = actor.horizontal_magnitude.wrapping_sub(BRAKE_STEP);
        if actor.horizontal_magnitude == 0 {
            {
                let actor_value = actor.state_code().wrapping_add(1);
                actor.set_state_code(actor_value);
            };
        }
    } else if (_pool[0].position.world_x as u8) < DEPARTURE_LEFT_EDGE {
        _context.progression.scene_signal = _context.progression.scene_signal.wrapping_add(1);
    }
}
pub fn timed_spawner(
    _pool: &mut ActorPool,
    _slot: usize,
    _context: &mut DepartureContext<'_>,
    _motion: &mut impl Motion,
    _catalog: &Catalog,
) -> DepartureResult {
    if _pool[0].render & 0x80 == 0 {
        let effects = _context.player.commit_life_loss(
            &mut _pool[0],
            _context.progression.stage.0,
            _context.progression.life_loss_variant,
            &mut _context.status.health,
        );
        _context.progression.reload_request = effects.reload.expect("common life-loss request");
        return DepartureResult::LifeLoss;
    }
    let actor = &mut _pool[_slot];
    if actor.state_code() & 15 == 0 {
        actor.horizontal_magnitude = ONE_PIXEL_SPEED;
        actor.vertical_velocity = SPAWNER_DOWN_SPEED;
        {
            let actor_value = 0;
            actor.set_local_counter_code(actor_value);
        };
        {
            let actor_value = SPAWNER_DELAY;
            actor.set_local_state_code(actor_value);
        };
        {
            let actor_value = actor.state_code().wrapping_add(1);
            actor.set_state_code(actor_value);
        };
    }
    if actor.local_state_code() != 0 {
        {
            let actor_value = actor.local_state_code() - (1);
            actor.set_local_state_code(actor_value);
        };
        return DepartureResult::Complete;
    }
    match actor.state_code() & 15 {
        2 => {
            if actor.local_counter_code() == 0 {
                spawn_reward(
                    _pool,
                    _slot,
                    _context.random,
                    RandomIncrement::One,
                    &_catalog.reward_delays,
                );
            }
            {
                let actor_value = _pool[_slot].local_counter_code().wrapping_sub(1);
                _pool[_slot].set_local_counter_code(actor_value);
            };
            _motion.apply(&mut _pool[_slot], Integration::SubtractX, 0);
            if _pool[_slot].position.world_x >> 8 == 0
                && (_pool[_slot].position.world_x as u8) < SPAWNER_LEFT_MARK
            {
                {
                    let actor_value = _pool[_slot].state_code().wrapping_add(1);
                    _pool[_slot].set_state_code(actor_value);
                };
                let actor = &mut _pool[SPAWNER_EXIT_SLOT];
                {
                    let actor_value = 0xa0;
                    actor.set_state_code(actor_value);
                };
                actor.render = 0xa0;
                actor.animation = 0x5a;
                actor.position.world_x = 0x30;
                actor.position.screen_y = 0x80;
                {
                    let actor_value = 0x38;
                    actor.set_handler(actor_value);
                };
                actor.profile = 0;
                actor.playback.restart_clock();
                actor.playback.reset_frame();
            }
        }
        3 => {
            _motion.apply(actor, Integration::SubtractY, 0);
            if actor.position.screen_y < SPAWNER_EXIT_MARK {
                {
                    let actor_value = 0;
                    actor.set_state_code(actor_value);
                };
            }
        }
        _ => {
            _motion.apply(actor, Integration::AddY, 0);
            if actor.position.screen_y >= SPAWNER_DOWN_MARK {
                {
                    let actor_value = actor.state_code().wrapping_add(1);
                    actor.set_state_code(actor_value);
                };
                *_context.departure_signal = 0x80;
            }
        }
    }
    DepartureResult::Complete
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RandomIncrement {
    Zero,
    One,
}
pub fn spawn_reward(
    _pool: &mut ActorPool,
    _slot: usize,
    _random: &mut RandomState,
    _increment: RandomIncrement,
    _delays: &[u8; 8],
) -> Option<usize> {
    let selected = (REWARD_FIRST_SLOT.._pool.len())
        .rev()
        .find(|&index| _pool[index].state_code() & 0x80 == 0)?;
    let parent = _pool[_slot];
    let actor = &mut _pool[selected];
    {
        let actor_value = 0xa0;
        actor.set_state_code(actor_value);
    };
    actor.render = 0xa0;
    actor.animation = REWARD_ANIMATION;
    {
        let actor_value = REWARD_HANDLER;
        actor.set_handler(actor_value);
    };
    actor.playback.restart_clock();
    actor.playback.reset_frame();
    actor.profile = 0;
    actor.position.world_x = parent.position.world_x;
    actor.position.screen_y = parent.position.screen_y;
    actor.vertical_velocity = 0x400;
    _random.words[1] = _random.words[0]
        .wrapping_add(_random.words[1])
        .wrapping_add(u8::from(_increment == RandomIncrement::One));
    {
        let actor_value = _delays[usize::from(_random.words[1] & 7)];
        _pool[_slot].set_local_counter_code(actor_value);
    };
    Some(selected)
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CompletionPhase {
    Entry,
    Moving,
    AwaitComposition,
    AwaitYield,
    FlashColor,
    AwaitColorWait,
    FlashBlack,
    AwaitBlackWait,
    Done,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Completion {
    pub phase: CompletionPhase,
    pub flashes_left: u8,
}
impl Default for Completion {
    fn default() -> Self {
        Self {
            phase: CompletionPhase::Entry,
            flashes_left: FLASH_CYCLES,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompletionStep {
    Compose,
    Yield,
    Wait(u16),
    Complete,
}
impl Completion {
    pub fn resume(
        &mut self,
        _pool: &mut ActorPool,
        _progression: &mut Progression,
        _display: &mut Display,
        _gravity: &mut u8,
        _motion: &mut impl Motion,
    ) -> CompletionStep {
        loop {
            match self.phase {
                CompletionPhase::Entry => {
                    *_gravity = COMPLETION_GRAVITY;
                    _pool[COMPLETION_SLOT].animation = COMPLETION_FIRST;
                    completion_pose(&mut _pool[COMPLETION_SLOT]);
                    self.phase = CompletionPhase::Moving;
                }
                CompletionPhase::Moving => {
                    let actor = &mut _pool[COMPLETION_SLOT];
                    if actor.local_counter_code() == 0 {
                        _motion.apply(actor, Integration::SubtractY, *_gravity);
                        _motion.apply(actor, Integration::Gravity, *_gravity);
                        if actor.vertical_velocity < 0 {
                            actor.vertical_velocity = 0;
                            {
                                let actor_value = actor.local_counter_code().wrapping_add(1);
                                actor.set_local_counter_code(actor_value);
                            };
                        }
                    } else {
                        _motion.apply(actor, Integration::Gravity, *_gravity);
                        _motion.apply(actor, Integration::SubtractY, *_gravity);
                    }
                    _motion.apply(
                        actor,
                        if actor.animation & 1 == 0 {
                            Integration::AddX
                        } else {
                            Integration::SubtractX
                        },
                        *_gravity,
                    );
                    self.phase = CompletionPhase::AwaitComposition;
                    return CompletionStep::Compose;
                }
                CompletionPhase::AwaitComposition => {
                    self.phase = CompletionPhase::AwaitYield;
                    return CompletionStep::Yield;
                }
                CompletionPhase::AwaitYield => {
                    if _pool[COMPLETION_SLOT].render & 0x80 != 0 {
                        self.phase = CompletionPhase::Moving;
                        continue;
                    }
                    _pool[COMPLETION_SLOT].animation =
                        _pool[COMPLETION_SLOT].animation.wrapping_add(1);
                    if _pool[COMPLETION_SLOT].animation <= COMPLETION_LAST {
                        completion_pose(&mut _pool[COMPLETION_SLOT]);
                        self.phase = CompletionPhase::Moving;
                    } else {
                        {
                            let actor_value = 0;
                            _pool[COMPLETION_SLOT].set_state_code(actor_value);
                        };
                        self.phase = CompletionPhase::FlashColor;
                    }
                }
                CompletionPhase::FlashColor => {
                    _display.active[16] = FLASH_COLOR;
                    _display.palette_pending = true;
                    self.phase = CompletionPhase::AwaitColorWait;
                    return CompletionStep::Wait(FLASH_TICKS);
                }
                CompletionPhase::AwaitColorWait => self.phase = CompletionPhase::FlashBlack,
                CompletionPhase::FlashBlack => {
                    _display.active[16] = FLASH_BLACK;
                    _display.palette_pending = true;
                    self.phase = CompletionPhase::AwaitBlackWait;
                    return CompletionStep::Wait(FLASH_TICKS);
                }
                CompletionPhase::AwaitBlackWait => {
                    self.flashes_left = self.flashes_left.wrapping_sub(1);
                    if self.flashes_left == 0 {
                        _progression.completion_actor = 0;
                        self.phase = CompletionPhase::Done;
                    } else {
                        self.phase = CompletionPhase::FlashColor;
                    }
                }
                CompletionPhase::Done => return CompletionStep::Complete,
            }
        }
    }
}
/// A caller explicitly acknowledges completion of a requested owner operation.
/// No later writes happen while a fade, composition or yield is outstanding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnerResult {
    Running,
    Complete,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ContentPhase {
    Entry,
    FadeOut,
    AfterFadeOut,
    YieldBeforeTerrain,
    Terrain,
    Scripts,
    Actors,
    Chr,
    Enable,
    YieldBeforeDisplay,
    FadeIn,
    Done,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Content {
    pub scene: Scene,
    pub phase: ContentPhase,
    pub script_cursor: u8,
    pub operation_cursor: u8,
    pub saved_scene: Scene,
}
impl Content {
    pub fn new(scene: Scene) -> Self {
        Self {
            scene,
            saved_scene: scene,
            phase: ContentPhase::Entry,
            script_cursor: 0,
            operation_cursor: 0,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ContentWork {
    FadeOut,
    ClearNonplayer,
    HideOam,
    Yield,
    Disable,
    FillNametable,
    Palette(Scene),
    TerrainSpan {
        scene: Scene,
        bank: u8,
        room_in_row: u8,
        columns: u8,
    },
    ScreenScript(u8),
    Compose,
    Chr(u8),
    Enable,
    FadeIn,
    Suspended,
    Complete,
}
pub struct ContentContext<'a> {
    pub actors: &'a mut ActorPool,
    pub display: &'a mut Display,
    pub camera: &'a mut Camera,
    pub cursor: &'a mut StreamCursor,
    pub scene_cursor: &'a mut u8,
    pub display_mode: &'a mut u8,
    pub split_y: &'a mut u8,
    pub title_option: u8,
    pub ending_variation: u8,
}
impl Content {
    pub fn resume(
        &mut self,
        _catalog: &Catalog,
        _context: &mut ContentContext<'_>,
        _owner: OwnerResult,
    ) -> ContentWork {
        if self.phase != ContentPhase::Entry && _owner == OwnerResult::Running {
            return ContentWork::Suspended;
        }
        loop {
            match self.phase {
                ContentPhase::Entry => {
                    self.saved_scene = self.scene;
                    self.phase = ContentPhase::FadeOut;
                    return ContentWork::FadeOut;
                }
                ContentPhase::FadeOut => {
                    _context.display.control &= !1;
                    *_context.display_mode = 0;
                    _context.cursor.ring = 0;
                    _context.cursor.room_in_row = 0;
                    self.operation_cursor = 0;
                    self.phase = ContentPhase::AfterFadeOut;
                }
                ContentPhase::AfterFadeOut => {
                    if self.saved_scene != Scene::Interior {
                        if self.operation_cursor == 0 {
                            self.operation_cursor = 1;
                            return ContentWork::ClearNonplayer;
                        }
                        if self.operation_cursor == 1 {
                            {
                                let actor_value = 0;
                                _context.actors[0].set_state_code(actor_value);
                            };
                            {
                                let actor_value = 0;
                                _context.actors[1].set_state_code(actor_value);
                            };
                            self.operation_cursor = 2;
                            return ContentWork::HideOam;
                        }
                    }
                    self.phase = ContentPhase::YieldBeforeTerrain;
                    return ContentWork::Yield;
                }
                ContentPhase::YieldBeforeTerrain => {
                    self.phase = ContentPhase::Terrain;
                    self.operation_cursor = 0;
                    return ContentWork::Disable;
                }
                ContentPhase::Terrain => match self.operation_cursor {
                    0 => {
                        self.operation_cursor = 1;
                        return ContentWork::FillNametable;
                    }
                    1 => {
                        self.operation_cursor = 2;
                        return ContentWork::Palette(self.saved_scene);
                    }
                    2 => {
                        self.operation_cursor = 3;
                        _context.cursor.room = self.saved_scene as u8;
                        _context.cursor.ring = 64;
                        _context.cursor.room_in_row = 0;
                        *_context.scene_cursor = self.saved_scene as u8;
                        return ContentWork::TerrainSpan {
                            scene: self.saved_scene,
                            bank: _catalog.parameters[self.saved_scene as usize].terrain,
                            room_in_row: 0,
                            columns: 64,
                        };
                    }
                    3 if (self.saved_scene as u8) < 3 => {
                        self.operation_cursor = 4;
                        self.scene = Scene::Interior;
                        *_context.scene_cursor = Scene::Interior as u8;
                        _context.cursor.room = 3;
                        _context.cursor.ring = 64;
                        _context.cursor.room_in_row = 1;
                        return ContentWork::TerrainSpan {
                            scene: Scene::Interior,
                            bank: 1,
                            room_in_row: 1,
                            columns: 64,
                        };
                    }
                    _ => {
                        self.script_cursor = _catalog.parameters[self.scene as usize].script_offset;
                        self.phase = ContentPhase::Scripts;
                    }
                },
                ContentPhase::Scripts => {
                    let encoded = _catalog.scripts[usize::from(self.script_cursor)];
                    if encoded != 0 {
                        self.script_cursor = self.script_cursor.wrapping_add(1);
                        if let Some(selector) = script_selector(
                            encoded,
                            _context.ending_variation,
                            _context.title_option,
                        ) {
                            return ContentWork::ScreenScript(selector);
                        }
                    }
                    self.scene = self.saved_scene;
                    *_context.scene_cursor = self.saved_scene as u8;
                    initialize_actors(_context.actors, &_catalog.actors[self.saved_scene as usize]);
                    self.phase = ContentPhase::Actors;
                    return ContentWork::Compose;
                }
                ContentPhase::Actors => {
                    self.phase = ContentPhase::Chr;
                    return ContentWork::Chr(_catalog.parameters[self.saved_scene as usize].chr);
                }
                ContentPhase::Chr => {
                    self.phase = ContentPhase::Enable;
                    return ContentWork::Enable;
                }
                ContentPhase::Enable => {
                    self.phase = ContentPhase::YieldBeforeDisplay;
                    return ContentWork::Yield;
                }
                ContentPhase::YieldBeforeDisplay => {
                    if self.saved_scene as u8 >= 4 {
                        *_context.split_y = _catalog.split_y[self.saved_scene as usize - 4];
                        *_context.display_mode = GAMEPLAY_DISPLAY;
                        _context.camera.x = (_context.camera.x & 0xff) | 0x100;
                    }
                    self.phase = ContentPhase::FadeIn;
                    return ContentWork::FadeIn;
                }
                ContentPhase::FadeIn => {
                    self.phase = ContentPhase::Done;
                    return ContentWork::Complete;
                }
                ContentPhase::Done => return ContentWork::Complete,
            }
        }
    }
}
pub fn script_selector(_encoded: u8, _variation: u8, _option: u8) -> Option<u8> {
    if _encoded == 0 {
        return None;
    }
    if _encoded & 0x80 == 0 {
        return Some(_encoded);
    }
    let nibble = _encoded & 15;
    Some(if nibble == 8 {
        nibble.wrapping_add(_option)
    } else {
        let value = nibble.wrapping_add(_variation);
        if value == 0 {
            value.wrapping_add(_option)
        } else {
            value
        }
    })
}
pub use crate::assets::catalog::SceneResourceContext;
pub fn resource_context(_scene: Scene) -> SceneResourceContext {
    SceneResourceContext {
        layout: ENDING_LAYOUT,
        spawn_configuration: None,
        terrain: if (_scene as u8) < 3 { 2 } else { 1 },
        chr: if (_scene as u8) < 4 { 9 } else { 10 },
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SpecialPhase {
    Entry,
    FadeOut,
    Reset,
    Disable,
    Clear,
    Layout,
    SpawnTables,
    Chr,
    Checkpoint,
    Palette,
    Actors,
    Audio,
    Enable,
    Yield,
    FadeIn,
    Done,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SpecialWork {
    FadeOut,
    ClearNonplayer,
    Disable,
    Layout(u8),
    SpawnTables(u8),
    Chr(u8),
    Checkpoint(u8),
    Palette(u8),
    StageAudio(u8),
    Enable,
    Yield,
    FadeIn,
    Suspended,
    Complete,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SpecialSession {
    pub room: SpecialRoom,
    pub phase: SpecialPhase,
}
impl SpecialSession {
    pub fn new(_request: u8) -> Result<Self> {
        let room = match _request {
            0x81 => SpecialRoom::Linked,
            0x82 => SpecialRoom::RewardFlight,
            _ => anyhow::bail!("unproven special room producer"),
        };
        Ok(Self {
            room,
            phase: SpecialPhase::Entry,
        })
    }
    pub fn resume(
        &mut self,
        _catalog: &Catalog,
        _pool: &mut ActorPool,
        _progression: &mut Progression,
        _player: &mut Player,
        _controls: &mut SpecialControls<'_>,
        _owner: OwnerResult,
    ) -> SpecialWork {
        if self.phase != SpecialPhase::Entry && _owner == OwnerResult::Running {
            return SpecialWork::Suspended;
        }
        let index = usize::from(self.room == SpecialRoom::RewardFlight);
        let mode = index as u8 + 1;
        match self.phase {
            SpecialPhase::Entry => {
                self.phase = SpecialPhase::FadeOut;
                SpecialWork::FadeOut
            }
            SpecialPhase::FadeOut => {
                *_controls.display = 0;
                *_controls.spawn_control = 0;
                _progression.late_mode = 0;
                *_controls.split = 0;
                *_controls.upper_cursor = 0;
                *_controls.lower_cursor = 0;
                *_controls.room_start = 0;
                *_controls.scan_direction = 1;
                _player.momentum_control = 0;
                _player.momentum_direction = 0;
                self.phase = SpecialPhase::Disable;
                SpecialWork::Disable
            }
            SpecialPhase::Disable => {
                self.phase = SpecialPhase::Clear;
                SpecialWork::ClearNonplayer
            }
            SpecialPhase::Clear => {
                _progression.special_room &= 15;
                *_controls.terrain_bank = 3;
                self.phase = SpecialPhase::Layout;
                SpecialWork::Layout(mode + 6)
            }
            SpecialPhase::Layout => {
                self.phase = SpecialPhase::SpawnTables;
                SpecialWork::SpawnTables(mode + 6)
            }
            SpecialPhase::SpawnTables => {
                self.phase = SpecialPhase::Chr;
                SpecialWork::Chr(mode + 6)
            }
            SpecialPhase::Chr => {
                self.phase = SpecialPhase::Checkpoint;
                SpecialWork::Checkpoint(mode + 0x10)
            }
            SpecialPhase::Checkpoint => {
                self.phase = SpecialPhase::Palette;
                SpecialWork::Palette(0x70)
            }
            SpecialPhase::Palette => {
                initialize_special_actors(_pool, &_catalog.special[index], _player);
                self.phase = SpecialPhase::Audio;
                SpecialWork::StageAudio(SPECIAL_AUDIO)
            }
            SpecialPhase::Audio => {
                self.phase = SpecialPhase::Enable;
                SpecialWork::Enable
            }
            SpecialPhase::Enable => {
                self.phase = SpecialPhase::Yield;
                SpecialWork::Yield
            }
            SpecialPhase::Yield => {
                *_controls.display = GAMEPLAY_DISPLAY;
                self.phase = SpecialPhase::FadeIn;
                SpecialWork::FadeIn
            }
            SpecialPhase::FadeIn => {
                self.phase = SpecialPhase::Done;
                SpecialWork::Complete
            }
            SpecialPhase::Done => SpecialWork::Complete,
            SpecialPhase::Reset | SpecialPhase::Actors => unreachable!(
                "internal special phase is performed atomically before its owner request"
            ),
        }
    }
}
/// Temporary borrowed values that belong to runtime/world/presentation, not a
/// retained duplicate state store. Tests can observe each actually written field.
pub struct SpecialControls<'a> {
    pub display: &'a mut u8,
    pub spawn_control: &'a mut u8,
    pub split: &'a mut u8,
    pub upper_cursor: &'a mut u8,
    pub lower_cursor: &'a mut u8,
    pub room_start: &'a mut u8,
    pub scan_direction: &'a mut u8,
    pub terrain_bank: &'a mut u8,
}
/// Independent resource analysis, retaining all aliases including unreachable-
/// as-yet-unproven ones. No fake author for runtime composite data is supplied.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeFrameConsumers {
    pub animations: Vec<(u8, Vec<u8>)>,
    pub aliases: [u16; 4],
}
pub fn runtime_frame_consumers(_rom: &[u8]) -> Result<RuntimeFrameConsumers> {
    let data = |start: usize, count: usize| {
        _rom.get(start..start + count)
            .context("runtime frame consumer bounds")
    };
    let low = data(0x15691, 4)?;
    let high = data(0x15789, 4)?;
    let aliases = std::array::from_fn(|i| u16::from(low[i]) | (u16::from(high[i]) << 8));
    let low = data(0x171d7, 144)?;
    let high = data(0x17267, 144)?;
    let mut animations = Vec::new();
    for index in 0..144 {
        let pointer = usize::from(low[index]) | (usize::from(high[index]) << 8);
        let at = 0x14010
            + pointer
                .checked_sub(0x8000)
                .context("animation source bank")?;
        let header = data(at, 2)?;
        let frames = data(at + 2, usize::from(header[0]) + 1)?;
        let consumers: Vec<u8> = frames.iter().copied().filter(|&frame| frame < 4).collect();
        if !consumers.is_empty() {
            animations.push((index as u8, consumers));
        }
    }
    Ok(RuntimeFrameConsumers {
        animations,
        aliases,
    })
}
#[cfg(test)]
mod contracts;

fn scene_move_entry(actor: &mut ActorContact) {
    actor.start_scene_move();
    actor.playback.freeze_at_start();
}

fn completion_pose(actor: &mut ActorContact) {
    {
        let actor_value = 0xa0;
        actor.set_state_code(actor_value);
    };
    actor.render = 0xa0;
    actor.position.screen_y = COMPLETION_START_Y;
    actor.position.world_x = COMPLETION_START_X;
    actor.playback.restart_clock();
    actor.playback.reset_frame();
    {
        let actor_value = 0;
        actor.set_local_counter_code(actor_value);
    };
    actor.horizontal_magnitude = COMPLETION_HORIZONTAL_SPEED;
    actor.vertical_velocity = COMPLETION_UPWARD_SPEED;
}

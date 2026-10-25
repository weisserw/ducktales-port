//! Interactive terrain objects and item trigger policies (B08).
//! Canonical actors/player/status remain borrowed; see trace.md.
use crate::{
    actors::{ActorPool, SUPPRESSION_BYTES, SpawnCatalog},
    collision::Movement,
    player::Player,
    runtime::RandomState,
    status::{Status, TitleRule},
};
use anyhow::{Context as _, Result};
pub const TERRAIN_HANDLER: u8 = 0x08;
pub const MOVING_HANDLER: u8 = 0x09;
pub const FALLING_HANDLER: u8 = 0x0a;
pub const LAUNCHED_HANDLER: u8 = 0x0b;
pub const RECOVERY_HANDLER: u8 = 0x13;
pub const CONTACT_RECOVERY_HANDLER: u8 = 0x14;
pub const TRANSFORM_HANDLER: u8 = 0x15;
pub const REVEAL_HANDLER: u8 = 0x16;
pub const SLIDING_HANDLER: u8 = 0x17;
pub const DISPENSER_HANDLER: u8 = 0x2c;
pub const DISPENSED_HANDLER: u8 = 0x2d;
pub const RELEASED_HANDLER: u8 = 0x42;
pub const TERRAIN_REPRESENTATION: u8 = 0x10;
pub const REACTION_ANIMATION: u8 = 0x2a;
pub const REAPPEARING_ANIMATION: u8 = 0x29;
pub const SPECIAL_TERRAIN_ANIMATION: u8 = 0x1b;
pub const IGNORED_HIT_ANIMATION: u8 = 0x22;
pub const STAGE_REWARD_ANIMATION: u8 = 0x3f;
pub const REVEAL_TICK: u8 = 5;
pub const SWING_FRAME: u8 = 3;
pub const CONTACT_DELAY: u8 = 0x1e;
pub const EXPIRING_COUNTDOWN: u8 = 0xff;
pub const BLINK_COUNTDOWN: u8 = 0x78;
pub const DISPENSE_PERIOD: u8 = 0x3c;
pub const DISPENSE_COUNT: usize = 4;
pub const SECONDARY_HANDLERS: usize = 64;
pub const MOTION_PRESETS: usize = 9;
///869D pushing can launch animation3F (selector25hex), beyond nine ordinary
/// motion programs. This named data recipe includes adjacent instruction bytes.
pub const TERRAIN_LAUNCH_ENTRIES: usize = 38;
pub const TERRAIN_LAUNCH_PHYSICAL: usize = 0x1891e;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MotionPhase {
    Arc,
    FallThenSlide,
    ContactTransfer,
    SlidePose,
    TouchPose,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Catalog {
    pub motion: [MotionPhase; MOTION_PRESETS],
    pub launch_vertical: [i16; TERRAIN_LAUNCH_ENTRIES],
    pub launch_horizontal: [u16; TERRAIN_LAUNCH_ENTRIES],
    pub recovered: [u8; 14],
    pub stage_rewards: [u8; 15],
    pub transform_animation: [u8; 20],
    pub transform_handler: [u8; 20],
    pub transform_profile: [u8; 20],
    pub transform_y_offset: [u8; 20],
    pub secondary: [u8; SECONDARY_HANDLERS],
    pub slide_nudge: [i16; 4],
    pub dispenser_direction: [u8; DISPENSE_COUNT],
    pub dispenser_speed: [u16; DISPENSE_COUNT],
    pub reward_sound: [u8; 7],
}
impl Catalog {
    pub fn original(rom: &[u8]) -> Result<Self> {
        fn bytes<const N: usize>(rom: &[u8], cpu: usize) -> Result<[u8; N]> {
            Ok(rom
                .get(0x10010 + cpu..0x10010 + cpu + N)
                .context("object table bounds")?
                .try_into()?)
        }
        let launch: [u8; TERRAIN_LAUNCH_ENTRIES * 4] = bytes(rom, 0x890e)?;
        let low: [u8; 4] = bytes(rom, 0x8ede)?;
        let high: [u8; 4] = bytes(rom, 0x8ee2)?;
        let speed_low: [u8; 4] = bytes(rom, 0xa2b2)?;
        let speed_high: [u8; 4] = bytes(rom, 0xa2b6)?;
        use MotionPhase::*;
        Ok(Self {
            motion: [
                Arc,
                FallThenSlide,
                ContactTransfer,
                Arc,
                Arc,
                Arc,
                SlidePose,
                Arc,
                TouchPose,
            ],
            launch_vertical: std::array::from_fn(|i| {
                i16::from_le_bytes([launch[i * 4], launch[i * 4 + 1]])
            }),
            launch_horizontal: std::array::from_fn(|i| {
                u16::from_le_bytes([launch[i * 4 + 2], launch[i * 4 + 3]])
            }),
            recovered: bytes(rom, 0x8fb8)?,
            stage_rewards: bytes(rom, 0x9016)?,
            transform_animation: bytes(rom, 0x8fc6)?,
            transform_handler: bytes(rom, 0x8fda)?,
            transform_profile: bytes(rom, 0x8fee)?,
            transform_y_offset: bytes(rom, 0x9002)?,
            secondary: bytes(rom, 0x9025)?,
            slide_nudge: std::array::from_fn(|i| i16::from_le_bytes([low[i], high[i]])),
            dispenser_direction: bytes(rom, 0xa2ae)?,
            dispenser_speed: std::array::from_fn(|i| {
                u16::from_le_bytes([speed_low[i], speed_high[i]])
            }),
            reward_sound: bytes(rom, 0x8c0c)?,
        })
    }
}
/// Immediate calls to B05/B10/B04: returned bits/slot refer to committed work.
/// Integrators return their source-specific boundary result, not generic hit.
pub trait World {
    fn motion(
        &mut self,
        actors: &mut ActorPool,
        player: &mut Player,
        slot: usize,
        movement: Movement,
        start: u8,
        remaining: u8,
    ) -> u8;
    fn integrate(&mut self, actors: &mut ActorPool, slot: usize, operation: Integration) -> bool;
    fn player_contact(&mut self, actors: &mut ActorPool, player: &mut Player, slot: usize) -> bool;
    fn actor_contact(&mut self, actors: &mut ActorPool, slot: usize) -> Option<usize>;
    fn replace_terrain(&mut self, actors: &mut ActorPool, slot: usize, changed: &mut u8);
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Integration {
    FallingY,
    Gravity,
    PositiveX,
    NegativeX,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Work {
    Motion(Movement, u8, u8),
    Integrate(Integration),
    PlayerContact,
    ActorContact,
    ReplaceTerrain(usize),
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Effects {
    pub sounds: Vec<u8>,
    /// Indices in sounds already submitted to the canonical queue for RNG provenance.
    pub submitted_sounds: Vec<usize>,
    pub spawned: Vec<usize>,
    pub palette: Option<[u8; 2]>,
    pub reward_animation: Option<u8>,
}
/// Progression fields are a temporary borrowed view for the future B13 owner,
/// written back immediately; no independent progression state is retained.
pub struct Context<'a> {
    pub player: &'a mut Player,
    pub random: &'a mut RandomState,
    pub status: &'a mut Status,
    pub suppression: &'a mut [u8; SUPPRESSION_BYTES],
    pub changed: &'a mut u8,
    pub stage_bits: &'a mut u8,
    pub reward_flags: &'a mut u8,
    pub stage_one_extension: &'a mut bool,
    pub completion_latch: &'a mut u8,
    pub dialogue: &'a mut Option<u8>,
    pub stage: u8,
    pub late_mode: bool,
    pub composition_tick: u8,
    pub rule: TitleRule,
    /// QueueAudioCommand's actual producer slot comparison determines the
    /// secondary RNG increment. Borrow the canonical queue and use THAT submit's
    /// result immediately; never cache its producer across calls.
    pub audio_queue: &'a mut crate::audio::CommandQueue,
    pub catalog: &'a Catalog,
    pub spawn: &'a SpawnCatalog,
}
/// Only the real immutable/caller values used by recovery/reaction. Other
/// actor families borrow this entry without dummy status/progression owners.
#[derive(Clone, Copy)]
pub struct RecoveryParameters<'a> {
    pub stage: u8,
    pub late_mode: bool,
    pub rule: TitleRule,
    pub catalog: &'a Catalog,
}
impl<'a> Context<'a> {
    fn recovery_parameters(&self) -> RecoveryParameters<'a> {
        RecoveryParameters {
            stage: self.stage,
            late_mode: self.late_mode,
            rule: self.rule,
            catalog: self.catalog,
        }
    }
}
pub fn step(
    actors: &mut ActorPool,
    slot: usize,
    context: &mut Context<'_>,
    world: &mut dyn World,
) -> Effects {
    let mut call = ObjectCall {
        actors,
        slot,
        context,
        world,
        effects: Effects::default(),
    };
    call.step();
    call.effects
}
pub fn reaction(
    actors: &mut ActorPool,
    slot: usize,
    context: &mut Context<'_>,
    world: &mut dyn World,
) -> Effects {
    let mut call = ObjectCall {
        actors,
        slot,
        context,
        world,
        effects: Effects::default(),
    };
    call.reaction();
    call.effects
}
pub fn secondary(
    actors: &mut ActorPool,
    slot: usize,
    context: &mut Context<'_>,
    increment: bool,
) -> Effects {
    let parameters = context.recovery_parameters();
    secondary_shared(actors, slot, context.random, parameters, increment)
}
pub fn secondary_shared(
    actors: &mut ActorPool,
    slot: usize,
    random: &mut RandomState,
    parameters: RecoveryParameters<'_>,
    increment: bool,
) -> Effects {
    let mut effects = Effects::default();
    let sum = random.words[0]
        .wrapping_add(random.words[1])
        .wrapping_add(u8::from(increment));
    random.words[1] = sum;
    if sum & 3 != 0 {
        return effects;
    }
    let Some(&animation) = parameters
        .catalog
        .secondary
        .get(usize::from(actors[slot].handler))
    else {
        return effects;
    };
    if animation == 0 {
        return effects;
    }
    if let Some(child) = free(actors) {
        initialize_recovery(&mut actors[child], animation, &parameters, &mut effects);
        actors[child].position.world_x = actors[slot].position.world_x;
        actors[child].position.screen_y = actors[slot].position.screen_y;
        effects.spawned.push(child);
    }
    effects
}
pub fn collect(actors: &mut ActorPool, slot: usize, context: &mut Context<'_>) -> Effects {
    let animation = actors[slot].animation;
    actors[slot].state = 0;
    let mut effects = Effects {
        reward_animation: Some(animation),
        ..Effects::default()
    };
    match animation {
        0x23..=0x29 => {
            effects
                .sounds
                .push(context.catalog.reward_sound[usize::from(animation - 0x23)]);
            match animation {
                0x23 => context.status.add_reward(3, 2),
                0x24 => context.status.health.recover_one(),
                0x25 => context.status.health.restore(),
                0x26 => {
                    context.player.protection = 0;
                    context.player.special_mode = 1;
                    context.player.special_ticks = 0x1e0;
                }
                0x27 => context.status.gain_life(),
                0x28 => context.status.add_reward(2, 1),
                _ => {
                    *context.reward_flags |= if context.stage == 4 { 4 } else { 2 };
                    *context.dialogue = Some(if context.stage == 4 { 0x10 } else { 0x0e });
                    context.player.action = crate::player::Action::Dialogue;
                }
            }
        }
        0x61 => {
            effects.sounds.push(0x15);
            context.status.add_reward(2, 5);
        }
        0x2b => {
            if context.stage == 1 {
                *context.stage_one_extension = true;
            }
            context.status.health.extend();
        }
        0x6f..=0x76 => {
            effects.sounds.push(0x37);
            *context.completion_latch = 0;
            context.status.add_reward(0, 1);
            *context.stage_bits |= 1u8 << (animation - 0x6f);
            if animation < 0x74 {
                context.player.action = crate::player::Action::ConvertTimer;
            } else {
                context.player.action = crate::player::Action::Dialogue;
                *context.dialogue = Some(0x0d);
            }
        }
        _ => {}
    }
    effects
}
pub fn launch_terrain(actors: &mut ActorPool, slot: usize, context: &mut Context<'_>) {
    launch_terrain_shared(actors, slot, context.player.pose_direction, context.catalog);
}
pub fn launch_terrain_shared(
    actors: &mut ActorPool,
    slot: usize,
    pose_direction: u8,
    catalog: &Catalog,
) {
    let actor = &mut actors[slot];
    let selector = actor.animation.wrapping_sub(0x1a);
    actor.direction = pose_direction;
    actor.state = (actor.state & 0xf0) | selector;
    actor.vertical_velocity = catalog.launch_vertical[usize::from(selector)];
    actor.horizontal_magnitude = catalog.launch_horizontal[usize::from(selector)];
    actor.motion_flags = 0;
    actor.handler = actor.handler.wrapping_add(1);
}
/// Exact869D entry skips the preceding swing and animation3F tests.
pub fn activate_terrain(
    actors: &mut ActorPool,
    slot: usize,
    context: &mut Context<'_>,
    world: &mut dyn World,
) -> Effects {
    if *context.changed != 0 {
        return Effects::default();
    }
    *context.changed = context.changed.wrapping_add(1);
    if actors[slot].state & TERRAIN_REPRESENTATION != 0 {
        actors[slot].state &= !TERRAIN_REPRESENTATION;
        world.replace_terrain(actors, slot, context.changed);
    }
    launch_terrain(actors, slot, context);
    Effects::default()
}
fn free(actors: &ActorPool) -> Option<usize> {
    (1..actors.len())
        .rev()
        .find(|&i| actors[i].state & 0x80 == 0)
}
fn merge(into: &mut Effects, other: Effects) {
    let offset = into.sounds.len();
    into.submitted_sounds.extend(
        other
            .submitted_sounds
            .into_iter()
            .map(|index| index + offset),
    );
    into.sounds.extend(other.sounds);
    into.spawned.extend(other.spawned);
    if other.palette.is_some() {
        into.palette = other.palette;
    }
    if other.reward_animation.is_some() {
        into.reward_animation = other.reward_animation;
    }
}
fn initialize_recovery(
    actor: &mut crate::actors::ActorContact,
    animation: u8,
    context: &RecoveryParameters<'_>,
    effects: &mut Effects,
) {
    actor.animation = crate::status::option_reward(context.rule, animation);
    actor.animation_frame = 0;
    actor.animation_tick = 0;
    actor.profile = 0;
    if matches!(actor.animation, 0x1b | 0x22) {
        actor.handler = 8;
        actor.profile = 0x80;
        actor.local_state = 0;
        actor.local_counter = 0;
        actor.motion_flags = 0;
        if context.late_mode {
            actor.local_state = context.stage.wrapping_add(6);
            if actor.local_state == 0x0a {
                effects.palette = Some([0x0f, 0x29]);
            }
        }
    } else {
        actor.handler = 0x13;
        actor.state = 0xa1;
        actor.vertical_velocity = 0;
        actor.local_state = 0;
        actor.local_counter = 0;
    }
}
struct ObjectCall<'a, 'b, 'c> {
    actors: &'a mut ActorPool,
    slot: usize,
    context: &'a mut Context<'b>,
    world: &'c mut dyn World,
    effects: Effects,
}
impl ObjectCall<'_, '_, '_> {
    fn motion(&mut self, movement: Movement, start: u8, remaining: u8) -> u8 {
        self.world.motion(
            self.actors,
            self.context.player,
            self.slot,
            movement,
            start,
            remaining,
        )
    }
    fn integrate(&mut self, operation: Integration) -> bool {
        self.world.integrate(self.actors, self.slot, operation)
    }
    fn contact(&mut self) -> bool {
        self.world
            .player_contact(self.actors, self.context.player, self.slot)
    }
    fn actor_contact(&mut self) -> Option<usize> {
        self.world.actor_contact(self.actors, self.slot)
    }
    fn replace(&mut self, slot: usize) {
        self.world
            .replace_terrain(self.actors, slot, self.context.changed);
    }
    fn clocks(&mut self) {
        self.actors[self.slot].animation_frame = 0;
        self.actors[self.slot].animation_tick = 0;
    }
    fn suppress(&mut self) {
        if let Some(id) = self.actors[self.slot].spawn_identity {
            self.context.suppression[usize::from(id >> 3)] |= 1 << (id & 7);
        }
    }
    fn removed_effect(&mut self, select_animation: bool) {
        let s = self.slot;
        if select_animation {
            self.actors[s].animation = REACTION_ANIMATION;
        }
        self.actors[s].handler = 0;
        self.actors[s].profile = 0;
        self.actors[s].render = (self.actors[s].render & 0xf0) | 8;
        self.effects.sounds.push(0x16);
    }
    fn special_reward(&mut self) {
        let s = self.slot;
        self.suppress();
        let local = usize::from(self.actors[s].local_state);
        let stage = if self.actors[s].animation == 0x3f {
            self.context.catalog.stage_rewards[usize::from(self.context.stage) * 3 + local]
        } else {
            0
        };
        self.actors[s].local_state = if stage != 0 {
            stage
        } else {
            self.context.catalog.recovered[local]
        };
        self.reward_arc();
    }
    fn reward_arc(&mut self) {
        let s = self.slot;
        self.actors[s].state &= 0xa0;
        self.actors[s].profile = 0;
        self.clocks();
        self.actors[s].vertical_velocity = 0x400;
        self.actors[s].handler = 0x13;
        self.actors[s].animation = 0x2a;
    }
    fn reaction(&mut self) {
        let parameters = self.context.recovery_parameters();
        let effects = reaction_shared(
            self.actors,
            self.slot,
            self.context.random,
            self.context.audio_queue,
            parameters,
            &mut |actors, slot, operation| self.world.integrate(actors, slot, operation),
        );
        merge(&mut self.effects, effects);
    }
    fn launched(&mut self) {
        launched_shared(self.actors, self.slot, &mut |actors, slot, operation| {
            self.world.integrate(actors, slot, operation)
        });
    }
    fn step(&mut self) {
        match self.actors[self.slot].handler {
            TERRAIN_HANDLER => self.terrain(),
            MOVING_HANDLER => self.moving(),
            FALLING_HANDLER => self.falling(),
            LAUNCHED_HANDLER => self.launched(),
            RECOVERY_HANDLER => self.recovery(),
            CONTACT_RECOVERY_HANDLER => self.contact_recovery(),
            TRANSFORM_HANDLER => self.transform(),
            REVEAL_HANDLER => self.reveal(),
            SLIDING_HANDLER => self.sliding(),
            DISPENSER_HANDLER => self.dispenser(),
            DISPENSED_HANDLER => self.dispensed(),
            RELEASED_HANDLER => self.released(),
            _ => {}
        }
    }
    fn face(&mut self) {
        let s = self.slot;
        self.actors[s].direction =
            if self.actors[s].position.world_x >= self.actors[0].position.world_x {
                2
            } else {
                1
            };
    }
    fn transform(&mut self) {
        let s = self.slot;
        if self.actors[s].render & 0x80 == 0 {
            self.actors[s].state = 0;
            return;
        }
        self.actors[s].profile = 0x81;
        if self.actors[0].animation_tick == 0
            && self.actors[0].animation == 9
            && self.actors[0].animation_frame == 3
            && self.contact()
        {
            self.suppress();
            let index =
                usize::from(self.context.stage) * 4 + usize::from(self.actors[s].local_state);
            self.actors[s].position.screen_y = self.actors[s]
                .position
                .screen_y
                .wrapping_sub(self.context.catalog.transform_y_offset[index]);
            if self.context.catalog.transform_handler[index] == 0 {
                self.actors[s].local_state = 0x28;
                self.reward_arc();
                return;
            }
            self.actors[s].profile = 0;
            self.clocks();
            self.actors[s].hit_marker = 0;
            self.actors[s].state &= 0xe0;
            if index == 6 {
                self.actors[s].handler = 0x17;
                self.actors[s].profile = 0x10;
                self.actors[s].animation = 0x21;
            } else {
                self.actors[s].animation = 0x2a;
                self.actors[s].handler = self.actors[s].handler.wrapping_add(1);
            }
            return;
        }
        if self.context.stage == 1 && self.actors[s].local_state >= 2 {
            self.actors[s].state &= 0xe0;
            if self.actors[0].animation != 9 {
                self.actors[s].animation = 0x5b;
                self.actors[s].profile = if self.actors[s]
                    .position
                    .screen_y
                    .abs_diff(self.actors[0].position.screen_y)
                    >= 0x18
                {
                    0x83
                } else {
                    0x82
                };
            }
        }
    }
    fn reveal(&mut self) {
        let s = self.slot;
        if self.actors[s].render & 0x80 == 0 {
            self.actors[s].state = 0;
            return;
        }
        if self.actors[s].animation_frame == 0 || self.actors[s].animation_tick != 5 {
            return;
        }
        let index = usize::from(self.context.stage) * 4 + usize::from(self.actors[s].local_state);
        if index == 7 {
            self.actors[s].position.screen_y = self.actors[s].position.screen_y.wrapping_add(0x18);
        }
        self.actors[s].animation = self.context.catalog.transform_animation[index];
        self.actors[s].handler = self.context.catalog.transform_handler[index];
        self.actors[s].profile = self.context.catalog.transform_profile[index];
        self.clocks();
        self.actors[s].hit_marker = 0;
        if index == 4 {
            self.effects.sounds.push(0x30);
        }
    }
    fn sliding(&mut self) {
        let s = self.slot;
        if self.actors[s].render & 0x80 == 0 {
            self.actors[s].state = 0;
            return;
        }
        if self.actors[s].state & 15 == 0 {
            self.actors[s].vertical_velocity = 0;
            self.actors[s].local_state = 0;
            self.actors[s].horizontal_magnitude = 0x200;
            self.face();
            self.actors[s].state = self.actors[s].state.wrapping_add(1);
        }
        if self.actors[s].local_state != 8 {
            if self.context.composition_tick & 3 == 0 {
                let index = usize::from(self.actors[s].local_state & 3);
                self.actors[s].position.world_x = self.actors[s]
                    .position
                    .world_x
                    .wrapping_add_signed(self.context.catalog.slide_nudge[index]);
                self.actors[s].local_state = self.actors[s].local_state.wrapping_add(1);
            }
            return;
        }
        let floor = self.motion(Movement::Fall, 0x2d, 1);
        self.actors[s].local_counter = floor;
        if floor & 1 != 0 && self.actors[s].horizontal_magnitude != 0 {
            self.actors[s].horizontal_magnitude =
                self.actors[s].horizontal_magnitude.wrapping_sub(0x10);
        }
        let mut stopped = self.actors[s].state & 2 != 0;
        if !stopped {
            let wall = if self.actors[s].direction & 2 == 0 {
                self.motion(Movement::PositiveX, 0x29, 1)
            } else {
                self.motion(Movement::NegativeX, 0x2b, 1)
            };
            self.contact();
            stopped = self.actors[s].horizontal_magnitude == 0 || wall & 1 != 0;
        }
        if stopped && floor & 1 != 0 {
            self.actors[s].handler = 8;
            self.actors[s].profile = 0x80;
        }
    }
    fn dispenser(&mut self) {
        let s = self.slot;
        if self.actors[s].state & 15 == 0 {
            let a = self.actors[s].position.world_x;
            let p = self.actors[0].position.world_x;
            let distance = if a >= p { (a - p) as u8 } else { (p - a) as u8 };
            if distance >= 4 {
                return;
            }
            self.actors[s].state = self.actors[s].state.wrapping_add(1);
            self.actors[s].local_state = 0;
            self.actors[s].local_counter = 0;
        }
        let counter = self.actors[s].local_counter;
        if counter == 0 {
            let Some(child) = free(self.actors) else {
                return;
            };
            self.effects.sounds.push(0x25);
            self.actors[s].animation = self.actors[s].animation.wrapping_add(1);
            self.clocks();
            let parent = self.actors[s];
            let index = usize::from(parent.local_state);
            let actor = &mut self.actors[child];
            actor.state = 0xa0;
            actor.render = 0xa0;
            actor.animation = 0x24;
            actor.position.world_x = parent.position.world_x;
            actor.position.screen_y = parent.position.screen_y.wrapping_sub(0x10);
            actor.animation_tick = 0;
            actor.animation_frame = 0;
            actor.profile = 0;
            actor.handler = 0x2d;
            actor.vertical_velocity = 0x2d4;
            actor.direction = self.context.catalog.dispenser_direction[index];
            actor.horizontal_magnitude = self.context.catalog.dispenser_speed[index];
            self.effects.spawned.push(child);
            self.actors[s].local_state = self.actors[s].local_state.wrapping_add(1);
            self.actors[s].local_counter = if self.actors[s].local_state == 4 {
                0xff
            } else {
                0x3c
            };
            return;
        }
        if counter & 0x80 == 0 {
            self.actors[s].local_counter -= 1;
        }
        if self.actors[s].animation != 0x48 && self.actors[s].animation_tick == 7 {
            self.actors[s].animation = self.actors[s].animation.wrapping_sub(1);
            self.clocks();
        }
    }
    fn dispensed(&mut self) {
        let s = self.slot;
        if self.actors[s].state & 15 == 0 {
            self.integrate(Integration::FallingY);
            self.integrate(Integration::Gravity);
            if self.actors[s].vertical_velocity < 0 {
                self.actors[s].vertical_velocity = 0;
                self.actors[s].state = self.actors[s].state.wrapping_add(1);
            }
        } else {
            if (self.actors[s].vertical_velocity as u16 >> 8) != 0xfd {
                self.integrate(Integration::Gravity);
            }
            self.integrate(Integration::FallingY);
        }
        if self.actors[s].direction != 0 {
            self.integrate(if self.actors[s].direction & 1 != 0 {
                Integration::PositiveX
            } else {
                Integration::NegativeX
            });
            if self.actors[s].state & 15 != 0 {
                self.actors[s].horizontal_magnitude =
                    self.actors[s].horizontal_magnitude.wrapping_sub(0x40);
                if self.actors[s].horizontal_magnitude == 0 {
                    self.actors[s].direction = 0;
                }
            }
        }
        if self.contact() {
            self.actors[s].state = 0;
            self.context.status.health.recover_one();
            return;
        }
        if self.actors[s].position.screen_y >= 0xf0 {
            self.actors[s].state = 0;
            self.actors[s].render = 0;
        }
    }
    fn released(&mut self) {
        let s = self.slot;
        if self.actors[s].state & 15 == 0 {
            if self.actors[0].animation == 9 && self.actors[0].animation_frame == 3 {
                self.actors[s].profile = 0x81;
                if self.contact() {
                    self.actors[s].state = self.actors[s].state.wrapping_add(1);
                    self.actors[s].animation = 0x2a;
                    self.clocks();
                    self.actors[s].profile = 0;
                    self.actors[s].position.screen_y =
                        self.actors[s].position.screen_y.wrapping_sub(0x18);
                    return;
                }
            }
            self.actors[s].profile = 0x83;
            if self.actors[0].animation == 9
                && self.actors[s]
                    .position
                    .screen_y
                    .wrapping_sub(self.actors[0].position.screen_y)
                    >= 0x18
            {
                self.actors[s].profile = 0x84;
            }
            return;
        }
        if !matches!(self.actors[s].animation, 0x25 | 0x28) {
            if self.actors[s].animation_frame == 0 || self.actors[s].animation_tick != 5 {
                return;
            }
            self.actors[s].animation = crate::status::option_reward(self.context.rule, 0x25);
            self.clocks();
            self.actors[s].local_state = 0;
            self.actors[s].vertical_velocity = 0;
            self.actors[s].horizontal_magnitude = 0x120;
            self.face();
            return;
        }
        let floor = self.motion(Movement::Fall, 0x2d, 1);
        if self.contact() {
            merge(&mut self.effects, collect(self.actors, s, self.context));
            return;
        }
        if floor & 1 == 0 {
            self.actors[s].local_state = self.actors[s].local_state.wrapping_add(1);
        } else if self.actors[s].local_state != 0 {
            self.actors[s].state = 0xa2;
            self.actors[s].local_state = 0xff;
            self.collect_contact();
            return;
        }
        if self.actors[s].direction & 1 != 0 {
            self.motion(Movement::PositiveX, 0x29, 0);
        } else {
            self.motion(Movement::NegativeX, 0x2b, 0);
        }
    }
    fn hit_fallback(&mut self, skip_hit_gate: bool) {
        let s = self.slot;
        if !skip_hit_gate && self.actors[s].hit_marker == 0 {
            return;
        }
        if self.actors[s].state & TERRAIN_REPRESENTATION != 0 {
            if *self.context.changed != 0 {
                return;
            }
            self.replace(s);
            self.actors[s].state &= !TERRAIN_REPRESENTATION;
        }
        match self.actors[s].animation {
            0x1b | 0x3f => self.special_reward(),
            0x22 => {}
            _ => self.removed_effect(true),
        }
    }
    fn terrain(&mut self) {
        let s = self.slot;
        if self.actors[s].render & 0x80 == 0 && !matches!(self.actors[s].animation, 0x1d | 0x1e) {
            self.actors[s].state = 0;
            return;
        }
        if self.actors[0].animation_tick == 0
            && self.actors[0].animation == 9
            && self.contact()
            && self.actors[0].animation_frame == SWING_FRAME
        {
            if self.actors[s].animation == 0x3f {
                self.special_reward();
            } else {
                merge(
                    &mut self.effects,
                    activate_terrain(self.actors, s, self.context, self.world),
                );
            }
        } else {
            self.hit_fallback(false);
        }
    }
    fn horizontal(&mut self) -> u8 {
        if self.actors[self.slot].direction & 1 != 0 {
            self.motion(Movement::PositiveX, 0x29, 1)
        } else {
            self.motion(Movement::NegativeX, 0x2b, 1)
        }
    }
    fn moving(&mut self) {
        let s = self.slot;
        let phase = self.context.catalog.motion[usize::from(self.actors[s].state & 15)];
        match phase {
            MotionPhase::Arc => {
                let mut floor = 0;
                if self.actors[s].motion_flags == 0 {
                    self.motion(Movement::Rise, 0x2f, 1);
                    if self.actors[s].vertical_velocity < 0 {
                        self.actors[s].vertical_velocity = 0;
                        self.actors[s].motion_flags = self.actors[s].motion_flags.wrapping_add(1);
                    }
                } else {
                    floor = self.motion(Movement::Fall, 0x2d, 1);
                }
                let wall = self.horizontal();
                if (floor | wall) & 1 != 0 {
                    self.removed_effect(true);
                } else if self.actor_contact().is_some() {
                    self.removed_effect(false);
                }
            }
            MotionPhase::FallThenSlide | MotionPhase::SlidePose => {
                let special = phase == MotionPhase::FallThenSlide;
                if !special {
                    self.actors[s].animation = 0x20;
                }
                let floor = self.motion(Movement::Fall, 0x2d, 1);
                if floor & 1 == 0 {
                    self.actors[s].motion_flags = 1;
                } else if self.actors[s].motion_flags != 0 || self.horizontal() & 1 != 0 {
                    if special {
                        self.special_reward();
                    } else {
                        self.removed_effect(true);
                    }
                    return;
                }
                if self.actor_contact().is_some() && !special {
                    self.removed_effect(false);
                }
            }
            MotionPhase::ContactTransfer => {
                if self.motion(Movement::NegativeY, 0x2f, 1) & 1 != 0 || self.horizontal() & 1 != 0
                {
                    self.removed_effect(true);
                    return;
                }
                if let Some(target) = self.actor_contact() {
                    if self.actors[target].handler == 8 && self.actors[target].animation == 0x1b {
                        launch_terrain(self.actors, target, self.context);
                        self.removed_effect(true);
                    } else {
                        self.removed_effect(false);
                    }
                }
            }
            MotionPhase::TouchPose => {
                let mut bits = self.motion(Movement::Fall, 0x2d, 1);
                if bits & 1 == 0 {
                    self.actors[s].motion_flags = 1;
                } else if self.actors[s].motion_flags == 0 {
                    bits = self.horizontal();
                }
                if self.actors[s].motion_flags == 0
                    && let Some(target) = self.actor_contact()
                {
                    self.actors[s].animation = 0x22;
                    if self.actors[target].handler == 8 {
                        let source = self.slot;
                        self.slot = target;
                        self.hit_fallback(true);
                        self.slot = source;
                        return;
                    }
                }
                if bits & 1 != 0 {
                    self.actors[s].handler = 8;
                }
            }
        }
    }
    fn falling(&mut self) {
        let s = self.slot;
        if self.actors[s].render & 0x80 == 0 {
            self.actors[s].state = 0;
            return;
        }
        if self.actors[s].state & 15 == 0 {
            if *self.context.changed != 0 {
                return;
            }
            let a = self.actors[s].position.world_x;
            let p = self.actors[0].position.world_x;
            let distance = if a >= p {
                a.wrapping_sub(p) as u8
            } else {
                p.wrapping_sub(a) as u8
            };
            if distance
                >= if self.actors[s].animation == 0x87 {
                    0x20
                } else {
                    4
                }
            {
                return;
            }
            self.actors[s].vertical_velocity = 0;
            self.actors[s].state = self.actors[s].state.wrapping_add(1);
            if self.actors[s].animation != 0x87 {
                self.actors[s].state &= !TERRAIN_REPRESENTATION;
                self.actors[s].render = self.actors[s].state & 0xf0;
                self.replace(s);
                if self.actors[s].position.screen_y < self.actors[0].position.screen_y {
                    self.actors[s].profile = 0x10;
                }
            }
        }
        if self.contact() || self.motion(Movement::Fall, 0x2d, 0) & 1 != 0 {
            self.removed_effect(true);
        }
    }
    fn collect_contact(&mut self) {
        use crate::player::Action;
        if self.context.player.action == Action::Damage {
            return;
        }
        if self.context.player.action == Action::Death {
            self.actors[self.slot].state = 0;
            return;
        }
        if self.contact() {
            merge(
                &mut self.effects,
                collect(self.actors, self.slot, self.context),
            );
        }
    }
    fn recovery(&mut self) {
        let s = self.slot;
        if self.actors[s].render & 0x80 == 0 {
            if self.actors[s].animation == 0x29
                && let Some(id) = self.actors[s].spawn_identity
            {
                self.context.suppression[usize::from(id >> 3)] &= !(1 << (id & 7));
            }
            self.actors[s].state = 0;
            return;
        }
        if self.actors[s].animation == 0x2a {
            if self.actors[s].animation_frame == 0 || self.actors[s].animation_tick != 5 {
                return;
            }
            self.actors[s].animation =
                crate::status::option_reward(self.context.rule, self.actors[s].local_state);
            self.clocks();
        }
        if self.actors[s].state & 15 == 0 {
            self.integrate(Integration::FallingY);
            self.integrate(Integration::Gravity);
            if self.actors[s].vertical_velocity >= 0 {
                return;
            }
            self.actors[s].state = self.actors[s].state.wrapping_add(1);
            self.actors[s].vertical_velocity = 0;
        }
        if self.actors[s].state & 2 == 0 {
            if self.motion(Movement::Fall, 0x2d, 1) & 1 != 0 {
                self.actors[s].state = self.actors[s].state.wrapping_add(1);
                self.actors[s].local_state = 0xff;
            }
        } else if self.actors[s].animation < 0x29 || self.actors[s].animation == 0x61 {
            let count = self.actors[s].local_state;
            if count == 0 {
                self.actors[s].state = 0;
                return;
            }
            if count == 0x78 {
                self.actors[s].render |= 2;
            }
            self.actors[s].local_state = count - 1;
        }
        self.collect_contact();
    }
    fn contact_recovery(&mut self) {
        let s = self.slot;
        if self.actors[s].render & 0x80 == 0 {
            self.actors[s].state = 0;
            return;
        }
        if self.actors[s].state & 15 == 0 {
            if self.context.player.protection != 0 || !self.contact() {
                return;
            }
            self.suppress();
            self.actors[s].state = self.actors[s].state.wrapping_add(1);
            self.actors[s].local_counter = CONTACT_DELAY;
        }
        if self.actors[s].state & 2 == 0 {
            self.actors[s].local_counter = self.actors[s].local_counter.wrapping_sub(1);
            if self.actors[s].local_counter != 0 {
                return;
            }
            self.actors[s].state &= !TERRAIN_REPRESENTATION;
            self.actors[s].vertical_velocity = 0;
            self.actors[s].local_state = self.actors[s].animation;
            self.actors[s].animation = 0x2a;
            self.actors[s].state = self.actors[s].state.wrapping_add(1);
            return;
        }
        if self.actors[s].animation != 0x2a {
            // 8C9B branches directly to 8CE1, bypassing the ordinary reset.
            self.actors[s].handler = 8;
            self.actors[s].profile = 0x80;
            self.actors[s].local_state = 0;
            self.actors[s].local_counter = 0;
            self.actors[s].motion_flags = 0;
            if self.context.late_mode {
                self.actors[s].local_state = self.context.stage.wrapping_add(6);
                if self.actors[s].local_state == 0x0a {
                    self.effects.palette = Some([0x0f, 0x29]);
                }
            }
            return;
        }
        if self.actors[s].animation == 0x2a {
            if self.actors[s].animation_frame == 0 || self.actors[s].animation_tick != 5 {
                return;
            }
            self.actors[s].animation = self.actors[s].local_state;
        }
        let animation = self.actors[s].animation;
        initialize_recovery(
            &mut self.actors[s],
            animation,
            &self.context.recovery_parameters(),
            &mut self.effects,
        );
    }
}
#[cfg(test)]
mod contracts;

/// Shared launched trajectory; callbacks mutate the supplied live pool before
/// each following source condition. Negative-X returns its no-borrow fact.
pub fn launched_shared(
    actors: &mut ActorPool,
    s: usize,
    integrate: &mut impl FnMut(&mut ActorPool, usize, Integration) -> bool,
) {
    if actors[s].state & 15 == 0 {
        actors[s].motion_flags = 0;
        actors[s].state = actors[s].state.wrapping_add(1);
        actors[s].vertical_velocity = 0x2d4;
        actors[s].horizontal_magnitude = 0x80;
    }
    if actors[s].motion_flags == 0 {
        integrate(actors, s, Integration::FallingY);
        integrate(actors, s, Integration::Gravity);
        if actors[s].vertical_velocity < 0 {
            actors[s].motion_flags = actors[s].motion_flags.wrapping_add(1);
            actors[s].vertical_velocity = 0;
        }
    } else {
        integrate(actors, s, Integration::Gravity);
        if integrate(actors, s, Integration::FallingY) {
            actors[s].state = 0;
            return;
        }
    }
    let positive = actors[s].direction & 1 != 0;
    let boundary = integrate(
        actors,
        s,
        if positive {
            Integration::PositiveX
        } else {
            Integration::NegativeX
        },
    );
    if boundary == positive {
        actors[s].state = 0;
    }
}

/// Immediate hit recovery used by ordinary families and bosses. Audio submit
/// provenance is sampled from this one canonical queue before secondary RNG.
pub fn reaction_shared(
    actors: &mut ActorPool,
    slot: usize,
    random: &mut RandomState,
    queue: &mut crate::audio::CommandQueue,
    parameters: RecoveryParameters<'_>,
    integrate: &mut impl FnMut(&mut ActorPool, usize, Integration) -> bool,
) -> Effects {
    let mut effects = Effects::default();
    if actors[slot].render & 0x80 == 0 {
        actors[slot].state = 0;
        actors[slot].render = 0;
        return effects;
    }
    let submission = queue.submit(0x35);
    effects.submitted_sounds.push(0);
    effects.sounds.push(0x35);
    merge(
        &mut effects,
        secondary_shared(
            actors,
            slot,
            random,
            parameters,
            submission.random_increment(),
        ),
    );
    actors[slot].animation_tick = 0x80;
    actors[slot].animation_frame = 0;
    actors[slot].profile = 0;
    actors[slot].state &= 0xf0;
    actors[slot].handler = 0x0b;
    actors[slot].direction = if actors[slot].position.world_x >= actors[0].position.world_x {
        1
    } else {
        2
    };
    launched_shared(actors, slot, integrate);
    effects
}

//! Land encounter policies and shared boss hit/treasure rules (B11).
//! Source-derived contracts and address corrections are
//! adjacent in trace.md; canonical actor/player/world/RNG owners are borrowed.
mod programs;
use crate::{
    actors::ActorPool,
    collision::Movement,
    player::Player,
    runtime::RandomState,
    world::{Camera, WorldCache},
};
use anyhow::{Context as _, Result};
pub use programs::{
    ContactGate, HorizontalHeading, PerimeterStep, VerticalCorrection, VerticalHeading, WaveStep,
    perimeter_program, wave_program,
};
pub const ATTACK: u8 = 0x26;
pub const PROJECTILE: u8 = 0x27;
pub const IMPACT: u8 = 0x2e;
pub const PERIMETER: u8 = 0x28;
pub const WAVE: u8 = 0x2a;
pub const WAVE_CHILD: u8 = 0x2b;
pub const CENTER: u8 = 0x29;
pub const HIT_FLASH: u8 = 0x78;
pub const FLASH_RENDER: u8 = 2;
pub const REVEAL: u8 = 0x2a;
pub const ATTACK_IDLE: u8 = 0x5e;
pub const ATTACK_FIRE: u8 = 0x5d;
pub const ATTACK_CHILD_ANIMATION: u8 = 0x5f;
pub const PERIMETER_IDLE: u8 = 0x62;
pub const PERIMETER_MOVING: u8 = 0x63;
pub const WAVE_MOVING: u8 = 0x64;
pub const WAVE_TURN: u8 = 0x65;
pub const CENTER_MOVING: u8 = 0x66;
pub const CENTER_IDLE: u8 = 0x67;
pub const CENTER_JUMP: u8 = 0x68;
pub const OPENING_TICKS: u8 = 60;
pub const SHORT_DELAY: u8 = 15;
pub const IMPACT_TICKS: u8 = 30;
pub const PATH_STEPS: usize = 20;
pub const MOTION_STEPS: usize = 14;
pub const WAVE_ENTRIES: usize = 14;
pub const FIRST_CHILD: usize = 8;
pub const LAST_TRIPLET: usize = 10;
pub const FINAL_DEFEAT_ANIMATION: u8 = 0x84;
pub const TREASURE_ANIMATION: u8 = 0x1b;
pub const TREASURE_HANDLER: u8 = 0x14;
pub const TREASURE_WAIT: u8 = 0x3c;
pub const TREASURE_Y: u8 = 0xb8;
pub const CENTER_X: u8 = 0x80;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttackPhase {
    Opening,
    Height,
    Preset,
    Land,
    Fire,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Catalog {
    pub attack_entries: [AttackPhase; 5],
    pub heights: [u8; 4],
    pub delays: [u8; 2],
    pub directions: [u8; 2],
    pub targets: [u8; 8],
    pub motion: [u8; MOTION_STEPS],
    pub child_offsets: [u8; 2],
    pub attack_facing: [u8; 2],
    pub child_vertical: [i16; 3],
    pub child_horizontal: [u16; 3],
    pub jump_delays: [u8; 8],
    pub split_offsets: [u8; 2],
    pub perimeter: [PerimeterStep; PATH_STEPS],
    pub corrections: [i8; 2],
    pub wave: [WaveStep; WAVE_ENTRIES],
    pub sequences: [u8; 2],
    pub shake_x: [u8; 2],
    pub center_facing: [u8; 2],
    pub center_speeds: [u16; 2],
}
impl Catalog {
    pub fn original(rom: &[u8]) -> Result<Self> {
        fn bytes<const N: usize>(rom: &[u8], cpu: usize) -> Result<[u8; N]> {
            Ok(rom
                .get(0x10010 + cpu..0x10010 + cpu + N)
                .context("encounter table bounds")?
                .try_into()?)
        }
        let _: [u8; 10] = bytes(rom, 0xa585)?;
        let yl: [u8; 3] = bytes(rom, 0xa5b2)?;
        let yh: [u8; 3] = bytes(rom, 0xa5b5)?;
        let xl: [u8; 3] = bytes(rom, 0xa5b8)?;
        let xh: [u8; 3] = bytes(rom, 0xa5bb)?;
        let sl: [u8; 2] = bytes(rom, 0xac39)?;
        let sh: [u8; 2] = bytes(rom, 0xac3b)?;
        let corrections: [u8; 2] = bytes(rom, 0xa8d4)?;
        use AttackPhase::*;
        Ok(Self {
            attack_entries: [Opening, Height, Preset, Land, Fire],
            heights: bytes(rom, 0xa58f)?,
            delays: bytes(rom, 0xa593)?,
            directions: bytes(rom, 0xa595)?,
            targets: bytes(rom, 0xa597)?,
            motion: bytes(rom, 0xa5a0)?,
            child_offsets: bytes(rom, 0xa5ae)?,
            attack_facing: bytes(rom, 0xa5b0)?,
            child_vertical: std::array::from_fn(|i| i16::from_le_bytes([yl[i], yh[i]])),
            child_horizontal: std::array::from_fn(|i| u16::from_le_bytes([xl[i], xh[i]])),
            jump_delays: bytes(rom, 0xa6c9)?,
            split_offsets: bytes(rom, 0xa6d1)?,
            perimeter: perimeter_program(bytes(rom, 0xa8ac)?, bytes(rom, 0xa8c0)?),
            corrections: corrections.map(|v| v as i8),
            wave: wave_program(bytes(rom, 0xaaa8)?, bytes(rom, 0xaab6)?),
            sequences: bytes(rom, 0xaac4)?,
            shake_x: bytes(rom, 0xaac6)?,
            center_facing: bytes(rom, 0xac37)?,
            center_speeds: std::array::from_fn(|i| u16::from_le_bytes([sl[i], sh[i]])),
        })
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Integration {
    FallingY,
    PositiveX,
    NegativeX,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Work {
    Motion(Movement, u8, u8),
    Integrate(Integration),
    PlayerContact,
    Preset(u8),
    Reaction,
}
/// Boundary facts preserve caller-specific ADC increment, rather than carrying
/// CPU registers/flags into native state. Mutations are immediate owner writes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MotionFacts {
    pub bits: u8,
    pub random_increment: bool,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Effects {
    pub sounds: Vec<u8>,
    /// Command observations already committed by the immediate sound boundary.
    pub submitted_sounds: Vec<usize>,
    pub children: Vec<usize>,
    pub treasure: Option<usize>,
    pub defeated: bool,
}
pub trait World {
    /// Commit at the source command point; true marks the retained observation as submitted.
    fn submit_sound(&mut self, _command: u8) -> bool {
        false
    }
    // Separate borrows keep canonical owners visible across immediate callbacks.
    #[allow(clippy::too_many_arguments)]
    fn motion(
        &mut self,
        actors: &mut ActorPool,
        player: &mut Player,
        terrain: &WorldCache,
        slot: usize,
        movement: Movement,
        start: u8,
        remaining: u8,
    ) -> MotionFacts;
    /// True translates source vertical continuation carry; it is not a contact.
    fn integrate(&mut self, actors: &mut ActorPool, slot: usize, operation: Integration) -> bool;
    fn player_contact(&mut self, actors: &mut ActorPool, player: &mut Player, slot: usize);
    fn preset(&mut self, actors: &mut ActorPool, slot: usize, selector: u8);
    /// B08 owns shared reaction/secondary allocation and samples fresh B18 queue
    /// producer at the sound call; no stale queue snapshot lives in Context.
    fn reaction(
        &mut self,
        actors: &mut ActorPool,
        player: &mut Player,
        random: &mut RandomState,
        slot: usize,
    );
}
/// HP/impact are immediate borrowed B13 run state; split controls are immediate
/// B03 writeback fields, not an independently retained display model. B04 owns
/// Camera/WorldCache. Presentation advances animation after this call.
pub struct Context<'a> {
    pub player: &'a mut Player,
    pub random: &'a mut RandomState,
    pub hit_points: &'a mut u8,
    pub impact: &'a mut u8,
    pub camera: &'a mut Camera,
    pub terrain: &'a mut WorldCache,
    pub split_low: &'a mut u8,
    pub split_control: &'a mut u8,
    pub composition_tick: u8,
    /// Source caller's arithmetic residue needed only by unseeded local ADC.
    /// B10 slots2..15 establish false via CPX16; slot1 inherits its caller.
    pub random_increment: bool,
    pub catalog: &'a Catalog,
}
pub fn step(
    actors: &mut ActorPool,
    slot: usize,
    context: &mut Context<'_>,
    world: &mut dyn World,
) -> Effects {
    let mut call = BossCall {
        actors,
        slot,
        context,
        world,
        effects: Effects::default(),
    };
    call.run();
    call.effects
}
pub fn hit(actors: &mut ActorPool, slot: usize, hit_points: &mut u8, camera_page: u8) -> Effects {
    if actors[slot].boss_protection == 0 {
        *hit_points = hit_points.wrapping_sub(1);
        if *hit_points == 0 {
            return treasure(actors, slot, camera_page);
        }
        actors[slot].boss_protection = HIT_FLASH;
        actors[slot].render |= FLASH_RENDER;
    } else {
        actors[slot].boss_protection -= 1;
        if actors[slot].boss_protection == 0 {
            actors[slot].hit_marker = 0;
            actors[slot].render &= !FLASH_RENDER;
        }
    }
    Effects::default()
}
pub fn treasure(actors: &mut ActorPool, slot: usize, camera_page: u8) -> Effects {
    let mut effects = Effects {
        defeated: true,
        ..Effects::default()
    };
    if actors[slot].animation == FINAL_DEFEAT_ANIMATION {
        return effects;
    }
    let selected = (1..actors.len())
        .rev()
        .find(|&i| actors[i].state_code() & 0x80 == 0)
        .unwrap_or(1);
    let actor = &mut actors[selected];
    actor.position.screen_y = TREASURE_Y;
    actor.position.world_x = u16::from(camera_page) << 8 | u16::from(CENTER_X);
    actor.hit_marker = 0;
    actor.playback.restart_clock();
    actor.playback.reset_frame();
    actor.profile = 0;
    {
        let actor_value = TREASURE_WAIT;
        actor.set_local_counter_code(actor_value);
    };
    actor.boss_protection = 0;
    {
        let actor_value = 0xb1;
        actor.set_state_code(actor_value);
    };
    actor.render = 0xb0;
    actor.animation = TREASURE_ANIMATION;
    {
        let actor_value = TREASURE_HANDLER;
        actor.set_handler(actor_value);
    };
    effects.treasure = Some(selected);
    effects
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
    into.children.extend(other.children);
    if other.treasure.is_some() {
        into.treasure = other.treasure;
    }
    into.defeated |= other.defeated;
}
struct BossCall<'a, 'b, 'c> {
    actors: &'a mut ActorPool,
    slot: usize,
    context: &'a mut Context<'b>,
    world: &'c mut dyn World,
    effects: Effects,
}
impl BossCall<'_, '_, '_> {
    fn sound(&mut self, command: u8) {
        if self.world.submit_sound(command) {
            self.effects
                .submitted_sounds
                .push(self.effects.sounds.len());
        }
        self.effects.sounds.push(command);
    }
    // Separate borrows keep canonical owners visible across immediate callbacks.
    #[allow(clippy::too_many_arguments)]
    fn motion(&mut self, movement: Movement, start: u8, remaining: u8) -> MotionFacts {
        self.world.motion(
            self.actors,
            self.context.player,
            self.context.terrain,
            self.slot,
            movement,
            start,
            remaining,
        )
    }
    fn integrate(&mut self, operation: Integration) -> bool {
        self.world.integrate(self.actors, self.slot, operation)
    }
    fn contact(&mut self) {
        self.world
            .player_contact(self.actors, self.context.player, self.slot);
    }
    fn clocks(&mut self) {
        let actor = &mut self.actors[self.slot];
        actor.playback.restart_clock();
        actor.playback.reset_frame();
    }
    fn increment(&mut self) {
        let actor = &mut self.actors[self.slot];
        {
            let actor_value = actor.state_code().wrapping_add(1);
            actor.set_state_code(actor_value);
        };
    }
    fn decrement(&mut self) {
        let actor = &mut self.actors[self.slot];
        {
            let actor_value = actor.state_code().wrapping_sub(1);
            actor.set_state_code(actor_value);
        };
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
    fn signed_y(&mut self) {
        let a = &mut self.actors[self.slot];
        let before = u16::from(a.position.screen_y) << 8 | u16::from(a.position.y_fraction);
        let next = before.wrapping_add(a.vertical_velocity as u16);
        a.position.screen_y = (next >> 8) as u8;
        a.position.y_fraction = next as u8;
    }
    fn walk(&mut self, positive: u8, negative: u8, remaining: u8) -> MotionFacts {
        if self.actors[self.slot].direction & 1 != 0 {
            self.motion(Movement::PositiveX, positive, remaining)
        } else {
            self.motion(Movement::NegativeX, negative, remaining)
        }
    }
    fn run(&mut self) {
        let s = self.slot;
        let handler = self.actors[s].handler;
        if matches!(handler, ATTACK | IMPACT | PERIMETER | WAVE | CENTER)
            && self.actors[s].hit_marker != 0
        {
            let result = hit(
                self.actors,
                s,
                self.context.hit_points,
                (self.context.camera.x >> 8) as u8,
            );
            let defeated = result.defeated;
            merge(&mut self.effects, result);
            if defeated {
                if handler == IMPACT {
                    *self.context.split_control = 0;
                    *self.context.split_low = 0xc0;
                }
                self.world
                    .reaction(self.actors, self.context.player, self.context.random, s);
                return;
            }
            // Shared F6D7 begins CLC, overriding the incoming first-slot fact.
            self.context.random_increment = false;
        }
        match handler {
            ATTACK => self.attack(),
            IMPACT => self.impact(),
            PERIMETER => self.perimeter(),
            WAVE => self.wave(),
            CENTER => self.center(),
            PROJECTILE => {
                self.contact();
                if !self.integrate(Integration::FallingY) {
                    {
                        let actor_value = 0;
                        self.actors[s].set_state_code(actor_value);
                    };
                    return;
                }
                self.integrate(if self.actors[s].direction & 1 != 0 {
                    Integration::PositiveX
                } else {
                    Integration::NegativeX
                });
            }
            WAVE_CHILD => {
                if self.motion(Movement::PositiveY, 0x2d, 0).bits & 1 != 0 {
                    {
                        let actor_value = 0;
                        self.actors[s].set_state_code(actor_value);
                    };
                    self.actors[s].render = 0;
                }
                self.contact();
            }
            _ => {}
        }
    }
    fn opening_tail(&mut self) {
        self.increment();
        self.actors[self.slot].animation = REVEAL;
        self.clocks();
        self.actors[self.slot].vertical_velocity = 0x380;
    }
    fn attack(&mut self) {
        let s = self.slot;
        match self.context.catalog.attack_entries[usize::from(self.actors[s].state_code() & 15)] {
            AttackPhase::Opening => {
                self.context.terrain.collision[0x6f] = 0x44;
                self.actors[s].playback.restart_clock();
                {
                    let actor_value = self.actors[s].local_state_code().wrapping_add(1);
                    self.actors[s].set_local_state_code(actor_value);
                };
                if self.actors[s].local_state_code() == OPENING_TICKS {
                    self.opening_tail();
                }
            }
            AttackPhase::Height => {
                if self.actors[s].animation == REVEAL {
                    if self.actors[s].playback.at_initial_frame(0)
                        || !self.actors[s].playback.at_running_tick(5)
                    {
                        return;
                    }
                    self.actors[s].animation = ATTACK_IDLE;
                    self.clocks();
                    let words = &mut self.context.random.words;
                    words[1] = words[0].wrapping_add(words[2]).wrapping_add(1);
                    {
                        let actor_value = self.context.catalog.heights[usize::from(words[1] & 3)];
                        self.actors[s].set_local_state_code(actor_value);
                    };
                    return;
                }
                self.integrate(Integration::FallingY);
                if self.actors[s].position.screen_y >= self.actors[s].local_state_code() {
                    return;
                }
                let words = &mut self.context.random.words;
                words[2] = words[1].wrapping_add(words[3]);
                let index = usize::from(words[2] & 7);
                {
                    let actor_value = self.context.catalog.targets[index];
                    self.actors[s].set_local_counter_code(actor_value);
                };
                {
                    let actor_value = self.context.catalog.delays[index & 1];
                    self.actors[s].set_local_state_code(actor_value);
                };
                self.actors[s].local_aux2 = 0;
                self.actors[s].sequence_cursor = 0;
                self.actors[s].direction = self.context.catalog.directions[index & 1];
                self.increment();
            }
            AttackPhase::Preset => {
                if self.actors[s].local_state_code() != 0 {
                    {
                        let actor_value = self.actors[s].local_state_code() - (1);
                        self.actors[s].set_local_state_code(actor_value);
                    };
                }
                if self.actors[s].local_aux2 == 0 {
                    let selector =
                        self.context.catalog.motion[usize::from(self.actors[s].sequence_cursor)];
                    self.world.preset(self.actors, s, selector);
                    if self.actors[s].horizontal_magnitude & 0x8000 != 0 {
                        self.actors[s].horizontal_magnitude =
                            self.actors[s].horizontal_magnitude.wrapping_neg();
                    }
                    self.actors[s].sequence_cursor = self.actors[s].sequence_cursor.wrapping_add(1);
                    if self.actors[s].sequence_cursor == 14 {
                        self.actors[s].sequence_cursor = 0;
                    }
                    self.actors[s].local_aux2 = 3;
                }
                self.actors[s].local_aux2 = self.actors[s].local_aux2.wrapping_sub(1);
                self.signed_y();
                let facts = if self.actors[s].direction & 2 == 0 {
                    self.motion(Movement::PositiveX, 0x18, 2)
                } else {
                    self.motion(Movement::NegativeX, 0x1b, 2)
                };
                if self.actors[s].local_state_code() == 0
                    && (self.actors[s].position.world_x as u8)
                        .abs_diff(self.actors[s].local_counter_code())
                        < 4
                {
                    self.increment();
                    self.actors[s].vertical_velocity = 0x380;
                }
                if self.actors[s].local_aux2 == 0 && facts.bits & 1 != 0 {
                    self.actors[s].direction ^= 3;
                }
            }
            AttackPhase::Land => {
                if self.motion(Movement::PositiveY, 0x1e, 2).bits & 1 == 0 {
                    return;
                }
                self.actors[s].animation = REVEAL;
                self.clocks();
                self.increment();
                self.face();
                let index = usize::from(self.actors[s].direction & 1);
                self.actors[s].render =
                    (self.actors[s].render & !0x40) | self.context.catalog.attack_facing[index];
            }
            AttackPhase::Fire => {
                if self.actors[s].animation == REVEAL {
                    if self.actors[s].playback.at_initial_frame(0)
                        || !self.actors[s].playback.at_running_tick(5)
                    {
                        return;
                    }
                    self.actors[s].animation = ATTACK_FIRE;
                    self.clocks();
                    {
                        let actor_value = 0;
                        self.actors[s].set_local_counter_code(actor_value);
                    };
                    {
                        let actor_value = SHORT_DELAY;
                        self.actors[s].set_local_state_code(actor_value);
                    };
                    return;
                }
                if self.actors[s].local_state_code() != 0 {
                    {
                        let actor_value = self.actors[s].local_state_code() - (1);
                        self.actors[s].set_local_state_code(actor_value);
                    };
                    self.actors[s].playback.restart_clock();
                    return;
                }
                if self.actors[s].local_counter_code() != 0 {
                    {
                        let actor_value = self.actors[s].state_code() & (0xf0);
                        self.actors[s].set_state_code(actor_value);
                    };
                    self.opening_tail();
                    return;
                }
                if !self.actors[s].playback.at_initial_frame(2) {
                    return;
                }
                let parent = self.actors[s];
                let offset = self.context.catalog.child_offsets[usize::from(parent.direction & 1)];
                for index in (0..3).rev() {
                    let child = &mut self.actors[FIRST_CHILD + index];
                    {
                        let actor_value = 0xa0;
                        child.set_state_code(actor_value);
                    };
                    child.render = 0xa0;
                    child.animation = ATTACK_CHILD_ANIMATION;
                    child.playback.restart_clock();
                    child.playback.reset_frame();
                    child.position.screen_y = parent.position.screen_y;
                    child.position.world_x = (parent.position.world_x & 0xff00)
                        | u16::from((parent.position.world_x as u8).wrapping_add(offset));
                    child.direction = parent.direction;
                    child.vertical_velocity = self.context.catalog.child_vertical[index];
                    child.horizontal_magnitude = self.context.catalog.child_horizontal[index];
                    {
                        let actor_value = PROJECTILE;
                        child.set_handler(actor_value);
                    };
                    child.profile = 0x1d;
                    self.effects.children.push(FIRST_CHILD + index);
                }
                self.sound(0x1c);
                {
                    let actor_value = IMPACT_TICKS;
                    self.actors[s].set_local_state_code(actor_value);
                };
                {
                    let actor_value = self.actors[s].local_counter_code().wrapping_add(1);
                    self.actors[s].set_local_counter_code(actor_value);
                };
            }
        }
    }
    fn impact(&mut self) {
        let s = self.slot;
        if self.actors[s].state_code() & 15 == 0 {
            {
                let actor_value = 0;
                self.actors[s].set_local_counter_code(actor_value);
            };
            self.actors[s].position.screen_y = 0xac;
            {
                let actor_value = 0x1e;
                self.actors[s].set_local_state_code(actor_value);
            };
            self.actors[s].horizontal_magnitude = 0x100;
            self.actors[s].direction = 1;
            self.context.terrain.collision[0x2f] = 0x44;
            self.increment();
        }
        if self.actors[s].local_state_code() == 0 {
            if self.actors[s].state_code() & 2 == 0 {
                self.motion(Movement::Rise, 0x64, 0);
                if self.actors[s].vertical_velocity < 0 {
                    self.increment();
                    self.actors[s].vertical_velocity = 0;
                }
            } else {
                if self.actors[s].local_counter_code() == 0 {
                    if self.motion(Movement::Fall, 0x63, 0).bits & 1 == 0 {
                        self.impact_horizontal();
                        return;
                    }
                    self.sound(0x1e);
                    *self.context.impact = self.context.impact.wrapping_add(1);
                    {
                        let actor_value = 0x1e;
                        self.actors[s].set_local_counter_code(actor_value);
                    };
                }
                {
                    let actor_value = self.actors[s].local_counter_code() - (1);
                    self.actors[s].set_local_counter_code(actor_value);
                };
                if self.actors[s].local_counter_code() == 0 {
                    self.decrement();
                    let words = &mut self.context.random.words;
                    words[0] = words[0]
                        .wrapping_add(words[3])
                        .wrapping_add(u8::from(self.context.random_increment));
                    {
                        let actor_value =
                            self.context.catalog.jump_delays[usize::from(words[0] & 7)];
                        self.actors[s].set_local_state_code(actor_value);
                    };
                    *self.context.split_low = 0xc0;
                    *self.context.split_control = 0;
                    *self.context.impact = 0;
                    return;
                }
                let index = usize::from((self.context.composition_tick >> 2) & 1);
                *self.context.split_control = index as u8;
                *self.context.split_low = self.context.catalog.split_offsets[index];
                return;
            }
        }
        self.impact_horizontal();
    }
    fn impact_horizontal(&mut self) {
        let s = self.slot;
        let facts = self.walk(0x61, 0x62, 0);
        if self.actors[s].local_state_code() == 0 {
            return;
        }
        {
            let actor_value = self.actors[s].local_state_code() - (1);
            self.actors[s].set_local_state_code(actor_value);
        };
        if self.actors[s].local_state_code() == 0 {
            self.actors[s].vertical_velocity = 0x6b7;
            self.face();
        }
        if facts.bits & 1 != 0 {
            self.actors[s].direction ^= 3;
        }
    }
    fn perimeter_node(&mut self) {
        let s = self.slot;
        let step = self.context.catalog.perimeter[usize::from(self.actors[s].local_counter_code())];
        let heading = if self.actors[s].local_state_code() == 1 {
            step.horizontal
        } else {
            step.horizontal.mirrored()
        };
        self.actors[s].direction = heading.direction_code();
        self.actors[s].motion_flags = step.motion_code();
    }
    fn perimeter(&mut self) {
        let s = self.slot;
        if self.actors[s].state_code() & 15 == 0 {
            if self.actors[s].animation == REVEAL {
                if self.actors[s].playback.at_initial_frame(0)
                    || !self.actors[s].playback.at_running_tick(5)
                {
                    return;
                }
                self.actors[s].animation = PERIMETER_IDLE;
                self.clocks();
                self.actors[s].profile |= 0x80;
                return;
            }
            self.actors[s].vertical_velocity = 0;
            {
                let actor_value = 0;
                self.actors[s].set_local_state_code(actor_value);
            };
            self.actors[s].motion_flags = 0;
            self.actors[s].position.screen_y = 0xb0;
            self.actors[s].position.world_x = (self.actors[s].position.world_x & 0xff00) | 0x80;
            self.context.terrain.collision[0x67] = 0x55;
            self.increment();
        }
        if self.actors[s].state_code() & 2 == 0 {
            if self.actors[s].motion_flags == 0 {
                if self.motion(Movement::Fall, 0x31, 2).bits & 1 == 0 {
                    return;
                }
                self.sound(0x22);
                self.actors[s].vertical_velocity = 0x2d4;
                self.actors[s].motion_flags = self.actors[s].motion_flags.wrapping_add(1);
                {
                    let actor_value = self.actors[s].local_state_code().wrapping_add(1);
                    self.actors[s].set_local_state_code(actor_value);
                };
                if self.actors[s].local_state_code() == 3 {
                    self.increment();
                    let words = &mut self.context.random.words;
                    words[1] = words[0].wrapping_add(words[1]).wrapping_add(1);
                    self.actors[s].direction = (words[1] & 1) + 1;
                    {
                        let actor_value = self.actors[s].direction;
                        self.actors[s].set_local_state_code(actor_value);
                    };
                    self.actors[s].animation = REVEAL;
                    self.clocks();
                    {
                        let actor_value = 0;
                        self.actors[s].set_local_counter_code(actor_value);
                    };
                    self.actors[s].local_aux2 = 0;
                    self.actors[s].horizontal_magnitude = 0x398;
                    self.actors[s].vertical_velocity = 0x398;
                    self.perimeter_node();
                }
            } else {
                self.motion(Movement::Rise, 0x52, 2);
                if self.actors[s].vertical_velocity < 0 {
                    self.actors[s].vertical_velocity = 0;
                    self.actors[s].motion_flags = self.actors[s].motion_flags.wrapping_sub(1);
                }
            }
            return;
        }
        if self.actors[s].animation == REVEAL {
            if self.actors[s].playback.at_initial_frame(0)
                || !self.actors[s].playback.at_running_tick(5)
            {
                return;
            }
            self.actors[s].animation = PERIMETER_MOVING;
            self.clocks();
            self.actors[s].profile &= 0x7f;
        }
        self.contact();
        let vertical = if self.actors[s].motion_flags & 4 != 0 {
            self.motion(Movement::PositiveY, 0x31, 2)
        } else {
            self.motion(Movement::NegativeY, 0x52, 2)
        };
        let horizontal = self.walk(0x34, 0x37, 2);
        let step = self.context.catalog.perimeter[usize::from(self.actors[s].local_counter_code())];
        let horizontal_contact = horizontal.bits & crate::collision::SOLID_CONTACT != 0;
        let vertical_contact = vertical.bits & crate::collision::SOLID_CONTACT != 0;
        if step.gate.accepts(horizontal_contact, vertical_contact) {
            if step.gate.needs_y_correction() {
                let index = usize::from((self.actors[s].motion_flags >> 3) & 1);
                self.actors[s].position.screen_y = self.actors[s]
                    .position
                    .screen_y
                    .wrapping_add_signed(self.context.catalog.corrections[index]);
            }
            {
                let actor_value = self.actors[s].local_counter_code().wrapping_add(1);
                self.actors[s].set_local_counter_code(actor_value);
            };
            if usize::from(self.actors[s].local_counter_code())
                == self.context.catalog.perimeter.len()
            {
                self.actors[s].local_aux2 = self.actors[s].local_aux2.wrapping_add(1);
                {
                    let actor_value = 0;
                    self.actors[s].set_local_counter_code(actor_value);
                };
            }
            self.perimeter_node();
        } else if self.actors[s].local_aux2 != 0
            && (self.actors[s].position.world_x as u8).abs_diff(0x80) < 3
        {
            self.actors[s].animation = REVEAL;
            self.clocks();
            {
                let actor_value = self.actors[s].state_code() & (0xf0);
                self.actors[s].set_state_code(actor_value);
            };
        }
    }
    fn wave(&mut self) {
        let s = self.slot;
        if self.actors[s].state_code() & 15 == 0 {
            {
                let actor_value = 0;
                self.actors[s].set_local_state_code(actor_value);
            };
            {
                let actor_value = 0;
                self.actors[s].set_local_counter_code(actor_value);
            };
            self.clocks();
            self.actors[s].horizontal_magnitude = 0x1cc;
            self.actors[s].direction = 2;
            self.context.terrain.collision[0x20] = 0x11;
            self.actors[s].vertical_velocity = 0x6ee;
            self.increment();
        }
        if self.actors[s].state_code() & 2 != 0 {
            self.wave_children();
            return;
        }
        self.wave_bounce();
    }
    fn wave_bounce(&mut self) {
        let s = self.slot;
        if self.actors[s].local_counter_code() != 0 {
            self.actors[s].playback.restart_clock();
            {
                let actor_value = self.actors[s].local_counter_code() - (1);
                self.actors[s].set_local_counter_code(actor_value);
            };
            if self.actors[s].local_counter_code() == 0 {
                self.actors[s].playback.advance_frame();
            }
            return;
        }
        if self.actors[s].local_state_code() == 0 {
            if !self.actors[s].playback.at_initial_frame(2) {
                return;
            }
            self.actors[s].playback.restart_clock();
            self.motion(Movement::Rise, 0x52, 2);
            if self.actors[s].vertical_velocity < 0 {
                self.actors[s].vertical_velocity = 0;
                {
                    let actor_value = self.actors[s].local_state_code().wrapping_add(1);
                    self.actors[s].set_local_state_code(actor_value);
                };
                return;
            }
        } else {
            if self.actors[s].playback.at_initial_frame(2) {
                self.actors[s].playback.restart_clock();
            }
            if self.motion(Movement::Fall, 0x31, 2).bits & 1 != 0 {
                {
                    let animation_value = 1;
                    self.actors[s].playback.set_frame_code(animation_value);
                };
                if self.actors[s].playback.at_running_tick(0) {
                    self.sound(0x1e);
                }
                if !self.actors[s].playback.at_running_tick(7) {
                    return;
                }
                {
                    let actor_value = 0x1e;
                    self.actors[s].set_local_counter_code(actor_value);
                };
                self.actors[s].vertical_velocity = 0x6ee;
                {
                    let actor_value = 0;
                    self.actors[s].set_local_state_code(actor_value);
                };
                self.actors[s].playback.reset_frame();
            }
        }
        if self.walk(0x34, 0x37, 2).bits & 1 != 0 && self.actors[s].local_counter_code() != 0 {
            self.actors[s].direction ^= 3;
            self.increment();
            self.actors[s].animation = WAVE_TURN;
            self.clocks();
            self.sound(0x23);
        }
    }
    fn wave_children(&mut self) {
        let s = self.slot;
        if self.actors[s].animation != WAVE_MOVING {
            if self.actors[s].playback.at_initial_frame(0)
                || !self.actors[s].playback.at_running_tick(7)
            {
                return;
            }
            self.actors[s].animation = WAVE_MOVING;
            self.actors[s].playback.reset_frame();
            {
                let actor_value = 0;
                self.actors[s].set_local_state_code(actor_value);
            };
            let words = &mut self.context.random.words;
            words[1] = words[0].wrapping_add(words[1]).wrapping_add(1);
            {
                let actor_value = self.context.catalog.sequences[usize::from(words[1] & 1)];
                self.actors[s].set_local_counter_code(actor_value);
            };
            self.actors[s].local_aux2 = 8;
        }
        loop {
            self.actors[s].playback.restart_clock();
            let shake =
                self.context.catalog.shake_x[usize::from((self.context.composition_tick >> 2) & 1)];
            self.context.camera.x = (self.context.camera.x & 0xff00) | u16::from(shake);
            let index = usize::from(self.actors[s].local_counter_code());
            let (x, at_tick) = match self.context.catalog.wave[index] {
                WaveStep::Spawn { x, at_tick } => (x, at_tick),
                WaveStep::AwaitLastChild => {
                    // The original06F0+15+nextslot expression aliases the last child's state.
                    let last = usize::from(self.actors[s].local_aux2.wrapping_sub(1));
                    if self.actors[last].state_code() != 0 {
                        return;
                    }
                    self.decrement();
                    {
                        let actor_value = 0;
                        self.actors[s].set_local_state_code(actor_value);
                    };
                    {
                        let actor_value = 0;
                        self.actors[s].set_local_counter_code(actor_value);
                    };
                    self.actors[s].local_aux2 = 0;
                    self.context.camera.x &= 0xff00;
                    self.actors[s].vertical_velocity = 0x6ee;
                    self.wave_bounce();
                    return;
                }
            };
            if self.actors[s].local_state_code() != at_tick {
                {
                    let actor_value = self.actors[s].local_state_code().wrapping_add(1);
                    self.actors[s].set_local_state_code(actor_value);
                };
                return;
            }
            let slot = usize::from(self.actors[s].local_aux2);
            let child = &mut self.actors[slot];
            child.position.world_x = 0x400 | u16::from(x);
            child.position.screen_y = 0x38;
            {
                let actor_value = 0xa0;
                child.set_state_code(actor_value);
            };
            child.render = 0xa0;
            child.animation = 0x47;
            child.playback.restart_clock();
            child.playback.reset_frame();
            child.profile = 0x10;
            {
                let actor_value = WAVE_CHILD;
                child.set_handler(actor_value);
            };
            child.vertical_velocity = 0x180;
            self.effects.children.push(slot);
            {
                let actor_value = self.actors[s].local_counter_code().wrapping_add(1);
                self.actors[s].set_local_counter_code(actor_value);
            };
            self.actors[s].local_aux2 = self.actors[s].local_aux2.wrapping_add(1);
            if self.actors[s].local_counter_code() == 0 {
                {
                    let actor_value = self.actors[s].local_state_code().wrapping_add(1);
                    self.actors[s].set_local_state_code(actor_value);
                };
                return;
            }
        }
    }
    fn center_idle(&mut self) {
        let s = self.slot;
        self.actors[s].position.world_x = (self.actors[s].position.world_x & 0xff00) | 0x80;
        {
            let actor_value = self.actors[s].local_counter_code().wrapping_sub(1);
            self.actors[s].set_local_counter_code(actor_value);
        };
        self.decrement();
        self.actors[s].animation = CENTER_IDLE;
        self.clocks();
        self.actors[s].local_aux2 = 0x1e;
    }
    fn center(&mut self) {
        let s = self.slot;
        if self.actors[s].state_code() & 15 == 0 {
            self.actors[s].vertical_velocity = 0;
            self.actors[s].position.world_x = (self.actors[s].position.world_x & 0xff00) | 0x80;
            self.context.terrain.collision[0x2f] = 0x44;
            self.increment();
            self.actors[s].local_aux2 = 0x0f;
            self.actors[s].animation = CENTER_IDLE;
        }
        if self.actors[s].local_aux2 != 0 {
            self.actors[s].local_aux2 -= 1;
            let index = usize::from(
                (self.actors[s].position.world_x as u8) < (self.actors[0].position.world_x as u8),
            );
            self.actors[s].render =
                (self.actors[s].render & !0x40) | self.context.catalog.center_facing[index];
            return;
        }
        if self.actors[s].state_code() & 2 == 0 {
            self.increment();
            {
                let actor_value = 0;
                self.actors[s].set_local_state_code(actor_value);
            };
            {
                let actor_value = 0;
                self.actors[s].set_local_counter_code(actor_value);
            };
            self.clocks();
            self.actors[s].animation = CENTER_MOVING;
            let words = &mut self.context.random.words;
            let first = u16::from(words[0])
                + u16::from(words[1])
                + u16::from(self.context.random_increment);
            words[3] = first as u8;
            self.actors[s].direction = (words[3] & 1) + 1;
            words[1] = words[2]
                .wrapping_add(words[3])
                .wrapping_add(u8::from(first > 255));
            let index = usize::from(words[1] & 1);
            {
                let actor_value = index as u8;
                self.actors[s].set_local_state_code(actor_value);
            };
            self.actors[s].horizontal_magnitude = self.context.catalog.center_speeds[index];
            if index != 0 {
                {
                    let actor_value = self.actors[s].local_state_code() + (1);
                    self.actors[s].set_local_state_code(actor_value);
                };
                self.actors[s].vertical_velocity = 0x6ee;
                self.actors[s].animation = CENTER_JUMP;
            }
        }
        if self.actors[s].local_state_code() != 0 {
            if self.actors[s].local_state_code() & 1 == 0 {
                self.motion(Movement::Rise, 0x5e, 2);
                if self.actors[s].vertical_velocity < 0 {
                    {
                        let actor_value = self.actors[s].local_state_code() - (1);
                        self.actors[s].set_local_state_code(actor_value);
                    };
                    self.actors[s].vertical_velocity = 0;
                }
            } else if self.motion(Movement::Fall, 0x5b, 2).bits & 1 != 0 {
                if self.actors[s].local_counter_code() != 0 {
                    self.center_idle();
                    return;
                }
                self.actors[s].vertical_velocity = 0x548;
                {
                    let actor_value = self.actors[s].local_state_code().wrapping_add(1);
                    self.actors[s].set_local_state_code(actor_value);
                };
                {
                    let actor_value = self.actors[s].local_counter_code().wrapping_add(1);
                    self.actors[s].set_local_counter_code(actor_value);
                };
                self.actors[s].direction ^= 3;
                return;
            }
        }
        if self.walk(0x55, 0x58, 2).bits & 1 != 0 {
            self.actors[s].direction ^= 3;
            {
                let actor_value = self.actors[s].local_counter_code().wrapping_add(1);
                self.actors[s].set_local_counter_code(actor_value);
            };
            return;
        }
        if self.actors[s].local_state_code() == 0
            && self.actors[s].local_counter_code() != 0
            && (self.actors[s].position.world_x as u8).abs_diff(0x80) < 4
        {
            self.center_idle();
        }
    }
}
#[cfg(test)]
mod contracts;

//! Platforms, rider attachment and external motion policies (B09).
//! Canonical ActorPool/Player are borrowed; see trace.md.
use crate::{
    actors::ActorPool,
    collision::{HorizontalDirection, Movement, VerticalOperation},
    player::Player,
};
use anyhow::{Context as _, Result};
pub const WALKING: u8 = 0x0c;
pub const FLIGHT: u8 = 0x0d;
pub const RIDEABLE: u8 = 0x0e;
pub const VERTICAL: u8 = 0x0f;
pub const PUSHING: u8 = 0x1d;
pub const CARRYING: u8 = 0x21;
pub const EXTERNAL: u8 = 0x24;
pub const LIFT: u8 = 0x38;
pub const FOUR_PHASE: u8 = 0x39;
pub const FLIGHT_ENTRIES: usize = 14;
pub const RIDE_ENTRIES: usize = 36;
pub const PHASE_TICKS: u8 = 0x30;
pub const CARRY_CONTACT_PROFILE: u8 = 0x8a;
pub const PLATFORM_PROFILE: u8 = 0x84;
pub const WALK_SPEED: u16 = 0x0180;
pub const BRAKE_STEP: u16 = 8;
pub const BRAKE_RESTART: u16 = 0x02ff;
pub const FLIGHT_HOLD: u8 = 5;
pub const CARRY_ANIMATION: u8 = 0x3e;
pub const CARRIED_ANIMATION: u8 = 0x8b;
pub const PUSH_ANIMATION: u8 = 0x59;
pub const WALK_ANIMATION: u8 = 0x58;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Catalog {
    pub flight: [u8; FLIGHT_ENTRIES],
    pub ride: [u8; RIDE_ENTRIES],
    pub ride_duration: [u8; 2],
    pub phases: [u8; 4],
    pub player_steps: [i16; 2],
    pub push_steps: [i16; 2],
}
impl Catalog {
    pub fn original(rom: &[u8]) -> Result<Self> {
        fn bytes<const N: usize>(rom: &[u8], at: usize) -> Result<[u8; N]> {
            Ok(rom
                .get(at..at + N)
                .context("platform table bounds")?
                .try_into()?)
        }
        let push: [u8; 4] = bytes(rom, 0x19d96)?;
        let player: [u8; 4] = bytes(rom, 0x1feea)?;
        Ok(Self {
            flight: bytes(rom, 0x19225)?,
            ride: bytes(rom, 0x192fd)?,
            ride_duration: bytes(rom, 0x19321)?,
            phases: bytes(rom, 0x1fee6)?,
            player_steps: std::array::from_fn(|i| {
                i16::from_le_bytes([player[i * 2], player[i * 2 + 1]])
            }),
            push_steps: std::array::from_fn(|i| i16::from_le_bytes([push[i * 2], push[i * 2 + 1]])),
        })
    }
}
/// Return committed B05 facts. The residual count is needed by A04C's second
/// call (A=1, Y from the first wrapper); it is NOT its initial count reloaded.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MotionFacts {
    pub bits: u8,
    pub continuation_remaining: u8,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Integration {
    Horizontal(HorizontalDirection),
    Vertical(VerticalOperation),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Work {
    Motion(Movement, u8, u8),
    Integrate(Integration),
    PlayerContact(u8),
    ActorContact,
    Preset(u8),
    Reaction,
    ActivateTerrain(usize),
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Effects {
    pub sounds: Vec<u8>,
    /// Shared reaction observations already submitted to the canonical ring.
    pub submitted_sounds: Vec<usize>,
}
/// Shared helpers are immediate committed boundaries, not delayed effect queues.
/// Production borrows B05/B08/B10 owners and immutable SpawnCatalog. Reaction
/// handles fresh B18 producer comparison/RNG/secondary allocation in B08.
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
        special: &mut crate::actors::SpecialControl,
        slot: usize,
        movement: Movement,
        start: u8,
        remaining: u8,
    ) -> MotionFacts;
    fn integrate(&mut self, actors: &mut ActorPool, slot: usize, operation: Integration) -> bool;
    fn player_contact(&mut self, actors: &mut ActorPool, player: &mut Player, slot: usize) -> bool;
    fn actor_contact(&mut self, actors: &mut ActorPool, slot: usize) -> Option<usize>;
    fn preset(&mut self, actors: &mut ActorPool, slot: usize, selector: u8);
    fn reaction(
        &mut self,
        actors: &mut ActorPool,
        player: &mut Player,
        slot: usize,
        special: &mut crate::actors::SpecialControl,
    ) -> Effects;
    /// Actual869D entry: changed-counter gate, then replacement and86B5. The
    /// swing/frame/animation3F gates before869D are intentionally bypassed.
    fn activate_terrain(
        &mut self,
        actors: &mut ActorPool,
        player: &mut Player,
        target: usize,
        changed: &mut u8,
    ) -> Effects;
}
pub struct Context<'a> {
    pub player: &'a mut Player,
    pub special: &'a mut crate::actors::SpecialControl,
    pub changed: &'a mut u8,
    pub completion: &'a mut u8,
    pub held: u8,
    pub catalog: &'a Catalog,
}
pub fn step(
    actors: &mut ActorPool,
    slot: usize,
    context: &mut Context<'_>,
    world: &mut dyn World,
) -> Effects {
    let mut call = PlatformCall {
        actors,
        slot,
        context,
        world,
        effects: Effects::default(),
    };
    call.run();
    call.effects
}
struct PlatformCall<'a, 'b, 'c> {
    actors: &'a mut ActorPool,
    slot: usize,
    context: &'a mut Context<'b>,
    world: &'c mut dyn World,
    effects: Effects,
}
impl PlatformCall<'_, '_, '_> {
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
            self.context.special,
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
    fn face(&mut self) {
        let s = self.slot;
        self.actors[s].direction =
            if self.actors[s].position.world_x >= self.actors[0].position.world_x {
                2
            } else {
                1
            };
    }
    fn increment(&mut self) {
        let s = self.slot;
        self.actors[s].state = self.actors[s].state.wrapping_add(1);
    }
    fn decrement(&mut self) {
        let s = self.slot;
        self.actors[s].state = self.actors[s].state.wrapping_sub(1);
    }
    fn clocks(&mut self) {
        let s = self.slot;
        self.actors[s].animation_tick = 0;
        self.actors[s].animation_frame = 0;
    }
    fn distance(&self) -> u8 {
        let a = self.actors[self.slot].position.world_x;
        let p = self.actors[0].position.world_x;
        if a >= p { (a - p) as u8 } else { (p - a) as u8 }
    }
    fn y_distance(&self) -> u8 {
        self.actors[self.slot]
            .position
            .screen_y
            .abs_diff(self.actors[0].position.screen_y)
    }
    fn reaction(&mut self) {
        let result = self.world.reaction(
            self.actors,
            self.context.player,
            self.slot,
            self.context.special,
        );
        merge_effects(&mut self.effects, result);
    }
    fn signed_y(&mut self) {
        let actor = &mut self.actors[self.slot];
        let before = u16::from(actor.position.screen_y) << 8 | u16::from(actor.position.y_fraction);
        let after = before.wrapping_add(actor.vertical_velocity as u16);
        actor.position.screen_y = (after >> 8) as u8;
        actor.position.y_fraction = after as u8;
    }
    fn run(&mut self) {
        use crate::player::Action;
        let s = self.slot;
        match self.actors[s].handler {
            VERTICAL => {
                self.actors[s].vertical_velocity = WALK_SPEED as i16;
                if self.actors[s].state & 15 == 0 {
                    self.integrate(Integration::Vertical(
                        VerticalOperation::SubtractEncodedVelocity,
                    ));
                    if self.actors[s].position.screen_y >= 0x68 {
                        return;
                    }
                    self.increment();
                } else {
                    self.integrate(Integration::Vertical(VerticalOperation::AddEncodedVelocity));
                }
                if self.distance() < 0x1d && self.y_distance() < 0x18 {
                    self.actors[0].position.screen_y =
                        self.actors[0].position.screen_y.wrapping_add(2);
                }
                if self.actors[s].position.screen_y >= 0xa8 {
                    self.decrement();
                }
            }
            WALKING => {
                if self.actors[s].state & 15 == 0 {
                    let a = self.actors[s].position.world_x;
                    let p = self.actors[0].position.world_x;
                    if p < a || (p.wrapping_sub(a) as u8) < 0x10 {
                        return;
                    }
                    self.actors[s].state &= !0x10;
                    self.actors[s].vertical_velocity = 0;
                    self.actors[s].horizontal_magnitude = WALK_SPEED;
                    self.actors[s].direction = 1;
                    self.increment();
                }
                self.contact();
                if self.motion(Movement::Fall, 0x31, 2).bits & 1 == 0 {
                    return;
                }
                let bits = if self.actors[s].direction & 1 != 0 {
                    self.motion(Movement::PositiveX, 0x34, 2).bits
                } else {
                    self.motion(Movement::NegativeX, 0x37, 2).bits
                };
                if bits & 1 != 0 {
                    self.actors[s].direction ^= 3;
                }
                if self.actors[s].animation_tick == 0 {
                    self.sound(0x27);
                }
            }
            FLIGHT => self.flight(),
            RIDEABLE => self.ride(),
            PUSHING => self.pushing(),
            CARRYING => {
                if self.actors[s].state & 15 == 0 {
                    if !self.contact() {
                        return;
                    }
                    self.context.player.action = Action::CarriedLocked;
                    self.actors[s].animation = CARRY_ANIMATION;
                    self.actors[0].animation = CARRIED_ANIMATION;
                    self.clocks();
                    self.actors[0].animation_frame = 0;
                    self.actors[0].animation_tick = 0;
                    self.increment();
                    self.actors[s].vertical_velocity = 0;
                    self.actors[s].horizontal_magnitude = WALK_SPEED;
                }
                self.actors[s].local_state = Action::CarriedJumpable as u8;
                if self.motion(Movement::Fall, 0x31, 2).bits & 1 != 0 {
                    self.motion(Movement::NegativeX, 0x38, 0);
                    self.actors[s].local_state = Action::CarriedLocked as u8;
                }
                if matches!(
                    self.context.player.action,
                    Action::CarriedLocked | Action::CarriedJumpable
                ) {
                    self.context.player.action = if self.actors[s].local_state == 10 {
                        Action::CarriedLocked
                    } else {
                        Action::CarriedJumpable
                    };
                    self.actors[0].position.screen_y =
                        self.actors[s].position.screen_y.wrapping_sub(0x0b);
                    self.actors[0].position.world_x = self.actors[s].position.world_x;
                    if self.actors[s].animation_tick == 0 {
                        self.sound(0x33);
                    }
                }
            }
            EXTERNAL => {
                if self.actors[s].state & 15 == 0 {
                    if self.distance() >= 4 {
                        return;
                    }
                    self.actors[s].horizontal_magnitude = WALK_SPEED;
                    self.increment();
                }
                let facts = self.motion(Movement::PositiveX, 0x35, 0);
                if facts.bits & 1 != 0 {
                    self.motion(Movement::PositiveX, 1, facts.continuation_remaining);
                    self.actors[s].handler = 0;
                }
                if self.distance() < 0x15 && self.y_distance() < 0x15 {
                    self.context.player.external_motion =
                        self.context.player.external_motion.wrapping_add(1);
                }
            }
            LIFT => {
                if self.actors[s].state & 15 == 0 {
                    if !self.contact() || self.context.player.action == Action::Death {
                        return;
                    }
                    self.increment();
                    self.actors[0].animation = 4;
                    self.actors[0].animation_frame = 0;
                    self.actors[0].position.world_x = self.actors[s].position.world_x;
                    self.context.player.action = Action::Hold;
                }
                self.actors[0].animation_tick = 0;
                self.actors[s].position.screen_y = self.actors[s].position.screen_y.wrapping_sub(2);
                self.actors[0].position.screen_y = self.actors[s].position.screen_y;
                if self.actors[s].position.screen_y < 0x10 {
                    *self.context.completion = self.context.completion.wrapping_add(1);
                }
            }
            FOUR_PHASE => {
                if self.actors[s].state & 15 == 0 {
                    self.actors[s].local_state = 0;
                    self.actors[s].local_counter = 0;
                    self.actors[s].horizontal_magnitude = 0x100;
                    self.increment();
                }
                if self.actors[s].local_counter == 0 {
                    let index = usize::from(self.actors[s].local_state & 3);
                    self.actors[s].direction = self.context.catalog.phases[index];
                    self.actors[s].local_counter = PHASE_TICKS;
                    self.actors[s].local_state = self.actors[s].local_state.wrapping_add(1);
                }
                self.integrate(Integration::Horizontal(
                    if self.actors[s].direction & 1 != 0 {
                        HorizontalDirection::Positive
                    } else {
                        HorizontalDirection::Negative
                    },
                ));
                self.actors[s].local_counter = self.actors[s].local_counter.wrapping_sub(1);
                self.actors[s].profile = CARRY_CONTACT_PROFILE;
                if self.contact() {
                    let index = usize::from((self.actors[s].direction & 2) >> 1);
                    self.actors[0].position.world_x = self.actors[0]
                        .position
                        .world_x
                        .wrapping_add_signed(self.context.catalog.player_steps[index]);
                }
                self.actors[s].profile = PLATFORM_PROFILE;
            }
            _ => {}
        }
    }
    fn flight(&mut self) {
        let s = self.slot;
        if self.actors[s].render & 0x80 == 0 || self.actors[s].hit_marker != 0 {
            *self.context.special = crate::actors::SpecialControl::Cooldown(0x3c);
            self.reaction();
            return;
        }
        if self.actors[s].state & 15 == 0 {
            self.increment();
            self.actors[s].local_state = 0;
            self.actors[s].local_counter = 0;
            self.face();
            if self.actors[s].direction & 2 == 0 {
                self.actors[s].render |= 0x40;
            }
        }
        if self.actors[s].local_state == 0 {
            let selector = self.context.catalog.flight[usize::from(self.actors[s].local_counter)];
            self.world.preset(self.actors, s, selector);
            if self.actors[s].horizontal_magnitude & 0x8000 != 0 {
                self.actors[s].horizontal_magnitude =
                    self.actors[s].horizontal_magnitude.wrapping_neg();
            }
            self.actors[s].local_counter = self.actors[s].local_counter.wrapping_add(1);
            if self.actors[s].local_counter == 14 {
                self.actors[s].local_counter = 0;
            }
            self.actors[s].local_state = FLIGHT_HOLD;
        }
        self.actors[s].local_state = self.actors[s].local_state.wrapping_sub(1);
        self.signed_y();
        let positive = self.actors[s].direction & 2 == 0;
        if self.integrate(Integration::Horizontal(if positive {
            HorizontalDirection::Positive
        } else {
            HorizontalDirection::Negative
        })) == positive
        {
            crate::actors::remove(self.actors, s, self.context.special, true, 0);
            return;
        }
        if self.actors[s].animation_tick == 0 {
            self.sound(0x2b);
        }
        self.contact();
    }
    fn ride(&mut self) {
        use crate::player::Action;
        let s = self.slot;
        if self.actors[s].state & 15 == 0 {
            self.actors[s].local_state = 0;
            self.actors[s].local_counter = 0;
            self.actors[s].position.x_fraction = 0;
            self.actors[s].position.y_fraction = 0;
            self.increment();
        }
        if self.actors[s].animation_tick == 0 {
            self.sound(0x26);
        }
        if self.actors[s].local_state == 0 {
            let packed = self.context.catalog.ride[usize::from(self.actors[s].local_counter)];
            self.world.preset(self.actors, s, packed & 31);
            self.actors[s].local_state =
                self.context.catalog.ride_duration[usize::from(packed >> 7)];
            self.actors[s].local_counter = self.actors[s].local_counter.wrapping_add(1);
            if self.actors[s].local_counter == 36 {
                self.actors[s].local_counter = 0;
            }
        }
        self.actors[s].local_state = self.actors[s].local_state.wrapping_sub(1);
        self.signed_y();
        let actor = &mut self.actors[s];
        let fixed = u32::from(actor.position.world_x) << 8 | u32::from(actor.position.x_fraction);
        let next = fixed.wrapping_add_signed(i32::from(actor.horizontal_magnitude as i16));
        actor.position.world_x = (next >> 8) as u16;
        actor.position.x_fraction = next as u8;
        if matches!(self.context.player.action, Action::Damage | Action::Death) {
            return;
        }
        if self.context.player.action != Action::WaitA {
            if !self.contact() || self.context.held & crate::runtime::Buttons::UP.0 == 0 {
                return;
            }
            self.context.player.action = Action::WaitA;
            self.actors[0].animation = 4;
            self.actors[0].animation_tick = 0;
            self.actors[0].animation_frame = 0;
        }
        self.actors[0].position.world_x = self.actors[s].position.world_x;
        self.actors[0].position.screen_y = self.actors[s].position.screen_y;
    }
    fn pushing(&mut self) {
        let s = self.slot;
        if self.actors[s].render & 0x80 == 0 || self.actors[s].hit_marker != 0 {
            if self.actors[s].render & 0x80 != 0 {
                self.actors[s].animation = WALK_ANIMATION;
            }
            *self.context.special = crate::actors::SpecialControl::Cooldown(0x3c);
            self.reaction();
            return;
        }
        if self.motion(Movement::Fall, 0x4a, 2).bits & 1 == 0 {
            return;
        }
        if self.actors[s].state & 15 == 0 {
            self.actors[s].animation_tick = 0;
            if self.distance() < 0x50 {
                self.actors[s].vertical_velocity = 0;
                self.actors[s].local_state = 0;
                self.actors[s].local_counter = 0;
                self.actors[s].horizontal_magnitude = 0x100;
                self.face();
                self.increment();
            }
            return;
        }
        if self.actors[s].state & 2 != 0 {
            if self.actors[s].animation_frame == 5 {
                self.decrement();
                self.actors[s].animation = self.actors[s].animation.wrapping_sub(1);
                self.clocks();
                self.actors[s].horizontal_magnitude =
                    (self.actors[s].horizontal_magnitude & 0xff) | 0x200;
                self.actors[s].local_counter = self.actors[s].local_counter.wrapping_add(1);
                return;
            }
            if self.actors[s].local_state != 0 && self.actors[s].animation_frame == 3 {
                self.context.player.pose_direction = self.actors[s].direction;
                let target = usize::from(self.actors[s].local_state);
                let effects = self.world.activate_terrain(
                    self.actors,
                    self.context.player,
                    target,
                    self.context.changed,
                );
                merge_effects(&mut self.effects, effects);
                self.actors[s].local_state = 0;
            }
            return;
        }
        let facts = if self.actors[s].direction & 1 != 0 {
            self.motion(Movement::PositiveX, 0x44, 2)
        } else {
            self.motion(Movement::NegativeX, 0x47, 2)
        };
        if self.actors[s].local_counter == 0 {
            if facts.bits & 1 == 0 {
                return;
            }
            if let Some(target) = self.world.actor_contact(self.actors, s) {
                self.actors[s].animation = WALK_ANIMATION;
                if self.actors[target].handler == 8 {
                    self.actors[s].local_state = target as u8;
                    self.increment();
                    self.actors[s].animation = self.actors[s].animation.wrapping_add(1);
                    self.actors[s].local_counter = self.actors[s].local_counter.wrapping_add(1);
                    self.clocks();
                    let index = usize::from((self.actors[s].direction & 2) >> 1);
                    self.actors[s].position.world_x = self.actors[s]
                        .position
                        .world_x
                        .wrapping_add_signed(self.context.catalog.push_steps[index]);
                    return;
                }
            }
            self.actors[s].horizontal_magnitude =
                self.actors[s].horizontal_magnitude.wrapping_add(0x100);
            self.actors[s].local_counter = self.actors[s].local_counter.wrapping_add(1);
            return;
        }
        if self.actors[s].local_state != 0 {
            return;
        }
        self.actors[s].state ^= 0x40;
        self.actors[s].horizontal_magnitude =
            self.actors[s].horizontal_magnitude.wrapping_sub(BRAKE_STEP);
        if self.actors[s].horizontal_magnitude & 0x8000 != 0 {
            self.actors[s].local_state = 0xff;
            self.actors[s].horizontal_magnitude = BRAKE_RESTART;
            self.actors[s].direction ^= 3;
        }
    }
}
#[cfg(test)]
mod contracts;

fn merge_effects(into: &mut Effects, other: Effects) {
    let offset = into.sounds.len();
    into.submitted_sounds.extend(
        other
            .submitted_sounds
            .into_iter()
            .map(|index| index + offset),
    );
    into.sounds.extend(other.sounds);
}

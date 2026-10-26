//! Ordinary actor families, projectiles and theft triggers (B07).
//! Source-derived contracts and immediate callback boundaries are adjacent.
use crate::{
    actors::{ActorPool, AmbushPhase, SpawnCatalog},
    collision::{Geometry, Movement},
    player::Player,
    runtime::RandomState,
    status::Status,
};
use anyhow::{Context as _, Result};
/// Native identity for each distinct B07 gameplay target. Encodings belong to
/// the adapter; state and position remain in the canonical ActorPool.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Family {
    JumpWalker,
    BounceWalker,
    Thrower,
    Projectile,
    RandomPose,
    ProximityRise,
    ContactPose,
    WaitingJumper,
    PresetFlight,
    PairedController,
    LedgeWalker,
    Ambush,
    Eruption,
    RepeatingJump,
    CompanionPatrol,
    FastLedgeWalker,
    TimedPatrol,
    VerticalPatrol,
    TriggeredJump,
    LeftWalker,
    Approach,
    TimedStealer,
    JumpingStealer,
    HitOnly,
    PlayerLauncher,
}
pub const LOCAL_PHASE: u8 = 0x0f;
pub const PERSISTENT_STATE: u8 = 0xf0;
pub const FLIP_X: u8 = 0x40;
pub const PAIR_COMPANION_HANDLER: u8 = 8;
pub const THROW_PROJECTILE_HANDLER: u8 = 4;
pub const THROW_PROJECTILE_ANIMATION: u8 = 0x17;
pub const THROW_PROJECTILE_PROFILE: u8 = 0x17;
pub const THROW_SOUND: u8 = 0x1b;
pub const THROW_DURATION: u8 = 0x3c;
pub const PAIR_WALK_DURATION: u8 = 0x20;
pub const PAIR_WAIT_DURATION: u8 = 0x3c;
pub const THIEF_PAUSE: u8 = 0x78;
pub const SHARED_REACTION_RETRY: u8 = 0x3c;
pub const RESTART_FRAME: u8 = 0x80;
pub const FROZEN_TICK: u8 = 0x80;
pub const BOUNCE_HEIGHT: i16 = 0x0400;
pub const REPEAT_JUMP_SPEED: i16 = 0x04e5;
pub const PATROL_JUMP_SPEED: i16 = 0x02d4;
pub const CONTACT_PROTECTION: u8 = 0x44;
pub const LAUNCH_SPEED: i16 = 0x0728;
pub const LAUNCH_HORIZONTAL: u16 = 0x0400;
pub const HELD_LAUNCH_TICKS: u8 = 7;
pub const FLIGHT_PRESETS: usize = 14;
pub const PAIR_PHASES: usize = 8;
/// Named immutable ROM consumer definitions. No original payload is embedded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Catalog {
    pub attach_y: [u8; 2],
    pub attach_poses: [u8; 2],
    pub contact_profiles: [u8; 3],
    pub flight_sequence: [u8; FLIGHT_PRESETS],
    pub companion_directions: [u8; 4],
    pub companion_flips: [u8; 4],
    pub companion_random_phases: [u8; 2],
    pub companion_animations: [u8; PAIR_PHASES],
    pub companion_animation_flips: [u8; PAIR_PHASES],
    pub ambush_sequence: [u8; FLIGHT_PRESETS],
    pub ambush_flips: [u8; 2],
    pub eruption_velocity: [i16; 3],
    pub eruption_rise_event: [u8; 3],
    pub eruption_fall_event: [u8; 3],
    pub fast_support: [u8; 2],
    pub stealer_intervals: [u8; 2],
}
impl Catalog {
    pub fn original(rom: &[u8]) -> Result<Self> {
        fn bytes<const N: usize>(rom: &[u8], cpu: usize) -> Result<[u8; N]> {
            Ok(rom
                .get(0x10010 + cpu..0x10010 + cpu + N)
                .context("enemy table bounds")?
                .try_into()?)
        }
        let low: [u8; 3] = bytes(rom, 0x9a75)?;
        let high: [u8; 3] = bytes(rom, 0x9a78)?;
        Ok(Self {
            attach_y: bytes(rom, 0x8661)?,
            attach_poses: bytes(rom, 0x8663)?,
            contact_profiles: bytes(rom, 0x8665)?,
            flight_sequence: bytes(rom, 0x9544)?,
            companion_directions: bytes(rom, 0x96bb)?,
            companion_flips: bytes(rom, 0x96bf)?,
            companion_random_phases: bytes(rom, 0x96c3)?,
            companion_animations: bytes(rom, 0x96c5)?,
            companion_animation_flips: bytes(rom, 0x96cd)?,
            ambush_sequence: bytes(rom, 0x9946)?,
            ambush_flips: bytes(rom, 0x9954)?,
            eruption_velocity: std::array::from_fn(|i| i16::from_le_bytes([low[i], high[i]])),
            eruption_rise_event: bytes(rom, 0x9a7b)?,
            eruption_fall_event: bytes(rom, 0x9a7e)?,
            fast_support: bytes(rom, 0x9df5)?,
            stealer_intervals: bytes(rom, 0xacbc)?,
        })
    }
}
/// Each result is the B05 boundary's already committed observation. Full bits,
/// not a boolean: CompanionPatrol deliberately checks all bits after falling.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ContactFacts {
    pub bits: u8,
    pub actor: Option<usize>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Integration {
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
    FallingY,
    Gravity,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Work {
    Motion {
        movement: Movement,
        start: u8,
        remaining: u8,
    },
    Integrate(Integration),
    PlayerContact,
    Reaction,
    LaunchTerrain(usize),
}
/// B05/B08 integration boundary, analogous to player::ContactWorld. Each call
/// borrows canonical state and commits mutations before returning. This is not
/// a deferred request queue or a second implementation of shared collision.
pub trait World {
    /// Commit at the source command point; true marks the retained observation as submitted.
    fn submit_sound(&mut self, _command: u8) -> bool {
        false
    }
    // Explicit owner borrows and source probe/motion inputs form this boundary.
    #[allow(clippy::too_many_arguments)]
    fn motion(
        &mut self,
        actors: &mut ActorPool,
        player: &mut Player,
        special_spawn: &mut u8,
        slot: usize,
        movement: Movement,
        start: u8,
        remaining: u8,
        gravity: u8,
    ) -> ContactFacts;
    fn integrate(
        &mut self,
        actors: &mut ActorPool,
        slot: usize,
        operation: Integration,
        gravity: u8,
    );
    fn player_contact(&mut self, actors: &mut ActorPool, player: &mut Player, slot: usize) -> bool;
    fn reaction(
        &mut self,
        actors: &mut ActorPool,
        player: &mut Player,
        random: &mut RandomState,
        slot: usize,
    );
    fn launch_terrain(&mut self, actors: &mut ActorPool, slot: usize, pose_direction: u8);
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Effects {
    pub work: Vec<Work>,
    pub sounds: Vec<u8>,
    /// Command observations already committed by the immediate sound boundary.
    pub submitted_sounds: Vec<usize>,
    pub palette_pair: Option<[u8; 2]>,
    pub palette_upload: bool,
    pub spawned: Vec<usize>,
    pub thefts: u8,
}
/// Mutable references are the authoritative owners, never copied local stores.
/// Effects are ordered observations/prepared palette/audio requests, while actor,
/// player, status and RNG changes occur inside the source call.
pub struct Context<'a> {
    pub player: &'a mut Player,
    pub random: &'a mut RandomState,
    pub status: &'a mut Status,
    pub special_spawn: &'a mut u8,
    pub gravity: u8,
    pub geometry: &'a Geometry,
    pub spawn: &'a SpawnCatalog,
    pub catalog: &'a Catalog,
}
pub fn family(selector: u8) -> Option<Family> {
    use Family::*;
    Some(match selector {
        1 => JumpWalker,
        2 => BounceWalker,
        3 => Thrower,
        4 => Projectile,
        5 => RandomPose,
        6 => ProximityRise,
        7 => ContactPose,
        0x10 => WaitingJumper,
        0x11 => PresetFlight,
        0x12 => PairedController,
        0x18 => LedgeWalker,
        0x19 => Ambush,
        0x1a => Eruption,
        0x1b => RepeatingJump,
        0x1c => CompanionPatrol,
        0x1e => FastLedgeWalker,
        0x1f => TimedPatrol,
        0x20 => VerticalPatrol,
        0x22 => TriggeredJump,
        0x23 => LeftWalker,
        0x25 => Approach,
        0x30 | 0x31 => TimedStealer,
        0x32 => JumpingStealer,
        0x33 => HitOnly,
        0x34 => PlayerLauncher,
        _ => return None,
    })
}
pub fn step(
    actors: &mut ActorPool,
    slot: usize,
    context: &mut Context<'_>,
    world: &mut dyn World,
) -> Effects {
    let Some(family) = family(actors[slot].handler) else {
        return Effects::default();
    };
    actors[slot].select_behavior();
    let mut call = FamilyCall {
        actors,
        slot,
        context,
        world,
        effects: Effects::default(),
    };
    call.run(family);
    call.actors[slot].select_behavior();
    call.effects
}
/// One source handler invocation borrows the live actor pool and all shared owners.
struct FamilyCall<'a, 'b, 'c> {
    actors: &'a mut ActorPool,
    slot: usize,
    context: &'a mut Context<'b>,
    world: &'c mut dyn World,
    effects: Effects,
}
impl FamilyCall<'_, '_, '_> {
    fn sound(&mut self, command: u8) {
        if self.world.submit_sound(command) {
            self.effects
                .submitted_sounds
                .push(self.effects.sounds.len());
        }
        self.effects.sounds.push(command);
    }
    fn motion(&mut self, movement: Movement, start: u8, remaining: u8) -> ContactFacts {
        self.effects.work.push(Work::Motion {
            movement,
            start,
            remaining,
        });
        self.world.motion(
            self.actors,
            self.context.player,
            self.context.special_spawn,
            self.slot,
            movement,
            start,
            remaining,
            self.context.gravity,
        )
    }
    fn integrate(&mut self, operation: Integration) {
        self.effects.work.push(Work::Integrate(operation));
        self.world
            .integrate(self.actors, self.slot, operation, self.context.gravity);
    }
    fn contact(&mut self) -> bool {
        self.effects.work.push(Work::PlayerContact);
        self.world
            .player_contact(self.actors, self.context.player, self.slot)
    }
    fn reaction(&mut self) {
        self.effects.work.push(Work::Reaction);
        self.world.reaction(
            self.actors,
            self.context.player,
            self.context.random,
            self.slot,
        );
    }
    fn reset_clocks(&mut self) {
        self.actors[self.slot].playback.reset_frame();
        self.actors[self.slot].playback.restart_clock();
    }
    fn face(&mut self) {
        self.actors[self.slot].direction =
            if self.actors[self.slot].position.world_x >= self.actors[0].position.world_x {
                2
            } else {
                1
            };
    }
    fn distance(&self) -> u8 {
        let a = self.actors[self.slot].position.world_x;
        let p = self.actors[0].position.world_x;
        a.abs_diff(p).min(255) as u8
    }
    fn vertical_distance(&self) -> u8 {
        self.actors[self.slot]
            .position
            .screen_y
            .abs_diff(self.actors[0].position.screen_y)
    }
    fn walk(&mut self, positive: u8, negative: u8, remaining: u8) -> ContactFacts {
        if self.actors[self.slot].direction & 1 != 0 {
            self.motion(Movement::PositiveX, positive, remaining)
        } else {
            self.motion(Movement::NegativeX, negative, remaining)
        }
    }
    fn reverse(&mut self) {
        self.actors[self.slot].direction ^= 3;
    }
    fn active_init(&mut self) -> bool {
        self.actors[self.slot].state_code() & 15 == 0
    }
    fn increment(&mut self) {
        {
            let actor_value = self.actors[self.slot].state_code().wrapping_add(1);
            self.actors[self.slot].set_state_code(actor_value);
        };
    }
    fn decrement(&mut self) {
        {
            let actor_value = self.actors[self.slot].state_code().wrapping_sub(1);
            self.actors[self.slot].set_state_code(actor_value);
        };
    }
    fn free(&self) -> Option<usize> {
        (1..self.actors.len())
            .rev()
            .find(|&i| self.actors[i].state_code() & 0x80 == 0)
    }
    fn run(&mut self, family: Family) {
        use Family::*;
        let s = self.slot;
        // Only these source entries share the visibility/retry prefix.
        if matches!(
            family,
            JumpWalker | Thrower | RepeatingJump | Approach | JumpingStealer
        ) && (self.actors[s].render & 0x80 == 0 || self.actors[s].hit_marker != 0)
        {
            if family == JumpWalker && self.actors[s].render & 0x80 != 0 {
                self.actors[s].animation = 0x0c;
            }
            *self.context.special_spawn = SHARED_REACTION_RETRY;
            self.reaction();
            return;
        }
        if self.actors[s].hit_marker != 0
            && matches!(
                family,
                BounceWalker
                    | RandomPose
                    | HitOnly
                    | LedgeWalker
                    | FastLedgeWalker
                    | TimedPatrol
                    | VerticalPatrol
                    | TimedStealer
            )
        {
            match family {
                BounceWalker => self.actors[s].animation = 0x0c,
                RandomPose => self.actors[s].animation = 0x0f,
                _ => {}
            }
            self.reaction();
            return;
        }
        match family {
            JumpWalker | BounceWalker | JumpingStealer => self.jump_walker(family),
            Thrower => self.thrower(),
            Projectile => {
                let facts = if self.actors[s].render & FLIP_X != 0 {
                    self.motion(Movement::PositiveX, 0x24, 0)
                } else {
                    self.motion(Movement::NegativeX, 0x25, 0)
                };
                self.contact();
                if facts.bits & 1 != 0 {
                    {
                        let actor_value = 0;
                        self.actors[s].set_state_code(actor_value);
                    };
                    self.actors[s].render = 0;
                    self.actors[s].spawn_identity = None;
                }
            }
            RandomPose => {
                if self.actors[s].playback.at_initial_frame(0x0c) {
                    let words = &mut self.context.random.words;
                    let sum = words[0].wrapping_add(words[1]).wrapping_add(1);
                    words[2] = sum;
                    self.actors[s].animation = (sum & 1) + 0x0d;
                    if self.actors[s].animation == 0x0e {
                        self.sound(0x2a);
                    }
                    self.reset_clocks();
                }
                self.actors[s].render = (self.actors[s].render & !FLIP_X)
                    | if self.actors[s].position.world_x < self.actors[0].position.world_x {
                        FLIP_X
                    } else {
                        0
                    };
            }
            ProximityRise => {
                if self.actors[s].render & 0x80 == 0 {
                    {
                        let actor_value = 0;
                        self.actors[s].set_state_code(actor_value);
                    };
                    return;
                }
                if self.actors[s].hit_marker != 0 {
                    self.actors[s].animation = 0x11;
                    self.reaction();
                    return;
                }
                if self.active_init() {
                    {
                        let actor_value = 0;
                        self.actors[s].set_local_state_code(actor_value);
                    };
                    self.increment();
                }
                if self.actors[s].state_code() & 2 == 0 {
                    self.actors[s].vertical_velocity = 0x0200;
                    if self.motion(Movement::NegativeY, 0x27, 0).bits & 1 == 0 {
                        return;
                    }
                    if self.distance() >= 0x32 {
                        {
                            let actor_value = 0;
                            self.actors[s].set_local_state_code(actor_value);
                        };
                    } else if self.actors[s].local_state_code() != 0 {
                        {
                            let actor_value = self.actors[s].local_state_code() - (1);
                            self.actors[s].set_local_state_code(actor_value);
                        };
                    } else {
                        self.increment();
                    }
                } else {
                    self.actors[s].vertical_velocity = 0x0300;
                    self.integrate(Integration::PositiveY);
                    if self.actors[s].position.screen_y >= 0xb0 || self.vertical_distance() < 0x18 {
                        self.decrement();
                        {
                            let actor_value = 0x96;
                            self.actors[s].set_local_state_code(actor_value);
                        };
                    }
                }
            }
            LedgeWalker | FastLedgeWalker => {
                let fast = family == FastLedgeWalker;
                if self.active_init() {
                    self.actors[s].vertical_velocity = 0;
                    self.actors[s].horizontal_magnitude = if fast { 0x133 } else { 0x80 };
                    self.actors[s].direction = 1;
                    self.increment();
                }
                if fast {
                    self.contact();
                }
                let index = usize::from(self.actors[s].direction & 1);
                let start = if fast {
                    self.context.catalog.fast_support[index]
                } else {
                    0x2d + index as u8
                };
                if self.motion(Movement::Fall, start, 0).bits & 1 == 0 {
                    self.reverse();
                }
                let facts = if fast {
                    self.walk(0x19, 0x1c, 0)
                } else {
                    self.walk(0x29, 0x2b, 0)
                };
                if facts.bits & 1 != 0 {
                    self.reverse();
                }
            }
            TimedPatrol | VerticalPatrol => self.timed_patrol(family),
            TimedStealer => self.timed_stealer(),
            LeftWalker => {
                if self.active_init() {
                    self.actors[s].vertical_velocity = 0;
                    self.actors[s].horizontal_magnitude = 0x180;
                    self.increment();
                }
                self.contact();
                if self.motion(Movement::Fall, 0x31, 2).bits & 1 != 0 {
                    self.integrate(Integration::NegativeX);
                    if self.actors[s].playback.at_running_tick(0) {
                        self.sound(0x27);
                    }
                }
            }
            HitOnly => {}
            PlayerLauncher => {
                if matches!(
                    self.context.player.action,
                    crate::player::Action::Damage | crate::player::Action::Death
                ) {
                    return;
                }
                if self.active_init() {
                    self.actors[s].playback.restart_clock();
                    if !self.contact() {
                        return;
                    }
                    self.context.player.action = crate::player::Action::ExternalLaunch;
                    self.increment();
                    {
                        let animation_value = HELD_LAUNCH_TICKS;
                        self.actors[s].playback.set_tick_code(animation_value);
                    };
                    self.actors[0].horizontal_magnitude = LAUNCH_HORIZONTAL;
                    self.actors[0].vertical_velocity = LAUNCH_SPEED;
                    self.actors[0].motion_flags = 0;
                    self.sound(0x28);
                }
                if self.actors[s].playback.at_start() {
                    self.actors[s].playback.freeze_at_start();
                }
            }
            ContactPose => self.contact_pose(),
            WaitingJumper => self.waiting_jumper(),
            PresetFlight => self.preset_flight(),
            RepeatingJump => {
                if self.active_init() {
                    self.actors[s].horizontal_magnitude = 0x100;
                    self.face();
                    {
                        let actor_value = self.actors[s].state_code() | (2);
                        self.actors[s].set_state_code(actor_value);
                    };
                }
                self.contact();
                if self.actors[s].state_code() & 2 == 0 {
                    if !self.actors[s].playback.at_initial_frame(0) {
                        self.actors[s].playback.freeze_at_start();
                    }
                    self.motion(Movement::Rise, 0x21, 2);
                    if self.actors[s].vertical_velocity < 0 {
                        self.actors[s].vertical_velocity = 0;
                        self.increment();
                    }
                } else if self.motion(Movement::Fall, 0x1e, 2).bits & 1 != 0 {
                    self.decrement();
                    self.reset_clocks();
                    self.actors[s].vertical_velocity = REPEAT_JUMP_SPEED;
                }
                self.walk(0x18, 0x1b, 2);
            }
            TriggeredJump => self.triggered_jump(),
            PairedController => self.paired_controller(),
            CompanionPatrol => self.companion_patrol(),
            Eruption => self.eruption(),
            Ambush => self.ambush(),
            Approach => self.approach(),
        }
    }
    fn ambush(&mut self) {
        const REST_ANIMATION: u8 = 0x39;
        const MOVE_ANIMATION: u8 = 0x3b;
        const ATTACK_ANIMATION: u8 = 0x3c;
        const FALL_ANIMATION: u8 = 0x5c;
        const CLOSE_DISTANCE: u8 = 12;
        const OPENING_DELAY: u8 = 30;
        const REST_DELAY: u8 = 120;
        const RISE_SPEED: i16 = 0x0280;
        const OPENING_PLAYER_GAP: u8 = 0x28;
        const TRACKING_PLAYER_GAP: u8 = 0x50;
        const POSE_SWITCH_TICK: u8 = 8;
        const PRESET_DELAY: u8 = 3;
        const TRACK_SOUND: u8 = 0x32;
        const ATTACK_SOUND: u8 = 0x1f;
        const LAND_SOUND: u8 = 0x20;
        const FALL_SAMPLE: u8 = 0x40;
        const RISE_SAMPLE: u8 = 0x42;
        const RIGHT_SAMPLE: u8 = 0x3a;
        const LEFT_SAMPLE: u8 = 0x3d;
        const POSITIVE_FACING: u8 = 1;
        const NEGATIVE_DIRECTION: u8 = 2;
        let s = self.slot;
        if self.actors[s].render & crate::actors::VISIBLE == 0 || self.actors[s].hit_marker != 0 {
            self.reaction();
            return;
        }
        if self.actors[s].animation != REST_ANIMATION {
            self.contact();
        }
        // The native phase is canonical across contact/movement callbacks. Shared
        // legacy callers see projections of this owner, never a shadow state byte.
        match self.actors[s].ambush_mut().phase {
            AmbushPhase::WaitingForPlayer => {
                if self.distance() < CLOSE_DISTANCE {
                    self.actors[s].advance_ambush();
                    let behavior = self.actors[s].ambush_mut();
                    behavior.delay_remaining = OPENING_DELAY;
                    behavior.preset_step = 0;
                    self.actors[s].vertical_velocity = RISE_SPEED;
                }
            }
            AmbushPhase::Opening => {
                if self.actors[s].ambush_mut().delay_remaining != 0 {
                    self.actors[s].ambush_mut().delay_remaining -= 1;
                    if self.actors[s].ambush_mut().delay_remaining == 0 {
                        self.actors[s].animation = ATTACK_ANIMATION;
                        self.reset_clocks();
                    }
                    return;
                }
                if self.actors[s].animation == ATTACK_ANIMATION
                    && self.actors[s].playback.at_running_tick(POSE_SWITCH_TICK)
                {
                    self.actors[s].animation = MOVE_ANIMATION;
                    self.reset_clocks();
                }
                let facts = self.motion(Movement::PositiveY, FALL_SAMPLE, 1);
                if facts.bits & crate::collision::SOLID_CONTACT != 0
                    || self.actors[0]
                        .position
                        .screen_y
                        .wrapping_sub(self.actors[s].position.screen_y)
                        < OPENING_PLAYER_GAP
                {
                    self.actors[s].advance_ambush();
                    self.face();
                    let index = usize::from(self.actors[s].direction & POSITIVE_FACING);
                    self.actors[s].render |= self.context.catalog.ambush_flips[index];
                    self.actors[s].animation = MOVE_ANIMATION;
                    self.reset_clocks();
                }
            }
            AmbushPhase::Tracking | AmbushPhase::Closing => {
                if self.actors[s].ambush_mut().delay_remaining == 0 {
                    let step = self.actors[s].ambush_mut().preset_step;
                    self.load_preset(
                        self.context.catalog.ambush_sequence[usize::from(step)],
                        true,
                    );
                    let behavior = self.actors[s].ambush_mut();
                    behavior.preset_step = step.wrapping_add(1);
                    if usize::from(behavior.preset_step)
                        == self.context.catalog.ambush_sequence.len()
                    {
                        behavior.preset_step = 0;
                    }
                    behavior.delay_remaining = PRESET_DELAY;
                }
                if self.actors[s].motion_flags == 0 {
                    self.motion(Movement::PositiveY, FALL_SAMPLE, 1);
                } else {
                    self.motion(Movement::NegativeY, RISE_SAMPLE, 1);
                }
                if self.actors[s].direction & NEGATIVE_DIRECTION == 0 {
                    self.motion(Movement::PositiveX, RIGHT_SAMPLE, 2);
                } else {
                    self.motion(Movement::NegativeX, LEFT_SAMPLE, 2);
                }
                if self.actors[s].playback.at_running_tick(0) {
                    self.sound(TRACK_SOUND);
                }
                let actor_y = self.actors[s].position.screen_y;
                let player_y = self.actors[0].position.screen_y;
                if !self.actors[s].ambush_mut().phase.counts_motion_delay()
                    && player_y >= actor_y
                    && player_y - actor_y >= TRACKING_PLAYER_GAP
                {
                    self.actors[s].advance_ambush();
                }
                if self.actors[s].ambush_mut().phase.counts_motion_delay() {
                    let behavior = self.actors[s].ambush_mut();
                    behavior.delay_remaining = behavior.delay_remaining.wrapping_sub(1);
                }
                // Both gates can advance in one invocation. Do not defer the
                // close-distance transition after entering Closing above.
                if self.distance() < CLOSE_DISTANCE {
                    self.actors[s].animation = ATTACK_ANIMATION;
                    self.reset_clocks();
                    self.actors[s].advance_ambush();
                    self.actors[s].vertical_velocity = RISE_SPEED;
                    self.sound(ATTACK_SOUND);
                }
            }
            AmbushPhase::Attacking => {
                if self.actors[s].animation == REST_ANIMATION {
                    let behavior = self.actors[s].ambush_mut();
                    behavior.delay_remaining = behavior.delay_remaining.wrapping_sub(1);
                    if behavior.delay_remaining == 0 {
                        behavior.preset_step = 0;
                        behavior.phase = AmbushPhase::Tracking;
                        self.actors[s].animation = MOVE_ANIMATION;
                        self.reset_clocks();
                        self.actors[s].profile &= !crate::collision::PROFILE_ENABLED;
                    }
                    return;
                }
                if self.actors[s].animation == ATTACK_ANIMATION
                    && self.actors[s].playback.at_running_tick(POSE_SWITCH_TICK)
                {
                    self.actors[s].animation = FALL_ANIMATION;
                    self.reset_clocks();
                }
                if self.motion(Movement::PositiveY, FALL_SAMPLE, 1).bits
                    & crate::collision::SOLID_CONTACT
                    != 0
                {
                    self.actors[s].animation = REST_ANIMATION;
                    self.reset_clocks();
                    self.actors[s].ambush_mut().delay_remaining = REST_DELAY;
                    self.actors[s].profile |= crate::collision::PROFILE_ENABLED;
                    self.sound(LAND_SOUND);
                }
            }
            AmbushPhase::Unrecognized(_) => {}
        }
    }
    fn reset_approach(&mut self) {
        let s = self.slot;
        self.increment();
        self.actors[s].motion_flags = 0;
        self.reset_clocks();
        self.actors[s].animation = 0x4c;
        self.actors[s].vertical_velocity = 0x1cc;
        self.actors[s].horizontal_magnitude = 0x1cc;
    }
    fn setup_approach_jump(&mut self) {
        let s = self.slot;
        self.actors[s].motion_flags = 8;
        self.actors[s].vertical_velocity = PATROL_JUMP_SPEED;
        self.actors[s].horizontal_magnitude &= 0xff00;
    }
    fn approach_walk(&mut self) -> ContactFacts {
        if self.actors[self.slot].direction & 2 == 0 {
            self.motion(Movement::PositiveX, 0x18, 2)
        } else {
            self.motion(Movement::NegativeX, 0x1b, 2)
        }
    }
    fn approach(&mut self) {
        let s = self.slot;
        if self.active_init() {
            self.actors[s].motion_flags = 0;
            self.actors[s].horizontal_magnitude = 0x1cc;
            self.face();
            self.increment();
        }
        if self.actors[s].state_code() & 15 == 3 {
            if self.actors[s].motion_flags == 0 {
                let facts = self.motion(Movement::NegativeY, 0x21, 2);
                if facts.bits & 1 != 0 || self.actors[s].position.screen_y < 0x88 {
                    self.actors[s].motion_flags = self.actors[s].motion_flags.wrapping_add(1);
                }
            } else if self.approach_walk().bits & 1 != 0 {
                self.motion(Movement::NegativeY, 0x21, 2);
            }
            return;
        }
        if self.actors[s].state_code() & 2 == 0 {
            if self.distance() >= 0x40 {
                self.approach_walk();
                return;
            }
            self.increment();
            self.actors[s].vertical_velocity = 0x1cc;
        }
        let motion = self.actors[s].motion_flags;
        if motion == 0 {
            if self.motion(Movement::PositiveY, 0x1e, 2).bits & 1 != 0 {
                self.reset_clocks();
                self.actors[s].animation = 0x4d;
                self.sound(0x24);
                self.setup_approach_jump();
            }
            return;
        }
        if motion & 8 != 0 {
            if self.actors[s].animation == 0x4d {
                if !self.actors[s].playback.at_initial_frame(2) {
                    return;
                }
                self.actors[s].playback.freeze_at_start();
            }
            let horizontal = self.approach_walk();
            self.motion(Movement::Rise, 0x21, 2);
            if self.actors[s].vertical_velocity < 0 {
                if horizontal.bits & 1 != 0 {
                    self.reset_approach();
                } else {
                    self.actors[s].motion_flags = 4;
                    self.actors[s].vertical_velocity = 0;
                }
            }
            return;
        }
        if self.motion(Movement::Fall, 0x1e, 2).bits & 1 == 0 {
            self.approach_walk();
            return;
        }
        if self.actors[s].animation != 0x4e {
            self.sound(0x24);
            self.actors[s].animation = 0x4e;
            self.actors[s].playback.reset_frame();
            {
                let animation_value = 1;
                self.actors[s].playback.set_tick_code(animation_value);
            };
        }
        if !self.actors[s].playback.at_start() {
            return;
        }
        let old = self.actors[s].direction;
        self.face();
        let changed = old != self.actors[s].direction;
        self.actors[s].direction = old;
        if changed {
            self.reset_approach();
        } else {
            self.actors[s].animation = 0x4d;
            {
                let animation_value = 2;
                self.actors[s].playback.set_frame_code(animation_value);
            };
            self.actors[s].playback.restart_clock();
            self.setup_approach_jump();
        }
    }
    fn paired_controller(&mut self) {
        let s = self.slot;
        if self.actors[s].animation == 0x31 {
            if self.actors[s].local_state_code() != 0 {
                {
                    let actor_value = self.actors[s].local_state_code() - (1);
                    self.actors[s].set_local_state_code(actor_value);
                };
                return;
            }
            self.actors[s].animation = 0x32;
            self.reaction();
            return;
        }
        if self.active_init() {
            let existing = (1..self.actors.len()).rev().find(|&i| {
                self.actors[i].state_code() & 0x80 != 0
                    && matches!(self.actors[i].animation, 0x1d | 0x1e)
                    && self.actors[i].position.world_x == self.actors[s].position.world_x
            });
            let link = if let Some(index) = existing {
                index
            } else if let Some(index) = self.free() {
                let parent = self.actors[s];
                let actor = &mut self.actors[index];
                actor.position.world_x = parent.position.world_x;
                actor.position.screen_y = parent.position.screen_y.wrapping_add(8);
                actor.profile = 0x80;
                {
                    let actor_value = 8;
                    actor.set_handler(actor_value);
                };
                {
                    let actor_value = 0xa0;
                    actor.set_state_code(actor_value);
                };
                actor.playback.restart_clock();
                actor.playback.reset_frame();
                actor.hit_marker = 0;
                actor.animation = 0x1e;
                self.effects.spawned.push(index);
                index
            } else {
                0
            };
            self.actors[s].local_aux2 = link as u8;
            {
                let actor_value = 0;
                self.actors[s].set_local_state_code(actor_value);
            };
            {
                let actor_value = 0;
                self.actors[s].set_local_counter_code(actor_value);
            };
            self.actors[s].horizontal_magnitude = 0x100;
            self.increment();
            return;
        }
        let link = usize::from(self.actors[s].local_aux2);
        if self.actors[s].hit_marker != 0 || self.actors[link].handler == 9 {
            self.sound(0x2c);
            self.actors[s].animation = 0x31;
            self.reset_clocks();
            self.actors[s].profile = 0;
            {
                let actor_value = 0x3c;
                self.actors[s].set_local_state_code(actor_value);
            };
            return;
        }
        if self.actors[s].local_state_code() == 0 {
            let phase = self.actors[s].local_counter_code();
            let index = usize::from(phase);
            if self.actors[link].state_code() & 0x80 != 0
                && matches!(self.actors[link].animation, 0x1d | 0x1e)
            {
                self.actors[link].animation = self.context.catalog.companion_animations[index];
                self.actors[link].render = (self.actors[link].render & !FLIP_X)
                    | self.context.catalog.companion_animation_flips[index];
            }
            if phase & 1 == 0 {
                self.actors[s].direction = self.context.catalog.companion_directions[index / 2];
                self.actors[s].render = (self.actors[s].render & !FLIP_X)
                    | self.context.catalog.companion_flips[index / 2];
                {
                    let actor_value = PAIR_WALK_DURATION;
                    self.actors[s].set_local_state_code(actor_value);
                };
                self.actors[s].animation = 0x2f;
            } else {
                {
                    let actor_value = PAIR_WAIT_DURATION;
                    self.actors[s].set_local_state_code(actor_value);
                };
                self.actors[s].direction = 0;
                self.actors[s].animation = 0x30;
            }
            self.reset_clocks();
            {
                let actor_value = phase.wrapping_add(1);
                self.actors[s].set_local_counter_code(actor_value);
            };
            if self.actors[s].local_counter_code() & 3 == 0 {
                let sum = self.context.random.words[0]
                    .wrapping_add(self.context.random.words[2])
                    .wrapping_add(phase & 1);
                {
                    let actor_value =
                        self.context.catalog.companion_random_phases[usize::from(sum & 1)];
                    self.actors[s].set_local_counter_code(actor_value);
                };
            }
        }
        {
            let actor_value = self.actors[s].local_state_code().wrapping_sub(1);
            self.actors[s].set_local_state_code(actor_value);
        };
        if self.actors[s].direction != 0 {
            self.integrate(if self.actors[s].direction & 2 == 0 {
                Integration::PositiveX
            } else {
                Integration::NegativeX
            });
        }
        self.contact();
    }
    fn companion_patrol(&mut self) {
        let s = self.slot;
        if self.actors[s].hit_marker != 0 {
            self.actors[s].animation = 0x56;
            self.reaction();
            return;
        }
        if self.active_init() {
            if self.distance() >= 0x50 {
                return;
            }
            self.actors[s].vertical_velocity = 0;
            self.actors[s].horizontal_magnitude = 0x133;
            {
                let actor_value = 0x50;
                self.actors[s].set_local_state_code(actor_value);
            };
            {
                let actor_value = (self.actors[s].state_code() & 0xef) | 1;
                self.actors[s].set_state_code(actor_value);
            };
            self.actors[s].direction = 1;
        }
        self.contact();
        if self.actors[s].state_code() & 2 != 0 {
            if self.actors[s].state_code() & 1 == 0 {
                self.motion(Movement::Rise, 0x2f, 1);
                if self.actors[s].vertical_velocity < 0 {
                    self.actors[s].playback.advance_frame();
                    self.actors[s].vertical_velocity = 0;
                    self.increment();
                }
            } else if self.motion(Movement::Fall, 0x2d, 1).bits & 1 != 0 {
                self.reset_clocks();
                self.actors[s].animation = 0x55;
                self.actors[s].position.screen_y = self.actors[s].position.screen_y.wrapping_add(1);
                self.decrement();
                self.decrement();
            }
            return;
        }
        let start = 0x2d + (self.actors[s].direction & 1);
        let mut launch = self.motion(Movement::Fall, start, 0).bits == 0;
        if !launch {
            self.actors[s].position.screen_y = self.actors[s].position.screen_y.wrapping_add(2);
            launch = self.walk(0x29, 0x2b, 0).bits & 1 != 0;
            if !launch {
                {
                    let actor_value = self.actors[s].local_state_code().wrapping_sub(1);
                    self.actors[s].set_local_state_code(actor_value);
                };
                launch = self.actors[s].local_state_code() == 0;
            }
        }
        if !launch {
            return;
        }
        self.reverse();
        {
            let actor_value = 0x50;
            self.actors[s].set_local_state_code(actor_value);
        };
        self.increment();
        self.actors[s].vertical_velocity = 0x400;
        self.actors[s].animation = 0x56;
        self.actors[s].playback.reset_frame();
        self.actors[s].playback.freeze_at_start();
        self.sound(0x34);
        if let Some(index) = self.free() {
            let parent = self.actors[s];
            let actor = &mut self.actors[index];
            {
                let actor_value = 0xa8;
                actor.set_state_code(actor_value);
            };
            actor.render = 0xa8;
            {
                let actor_value = 0;
                actor.set_handler(actor_value);
            };
            actor.profile = 0;
            actor.animation = 0x57;
            actor.playback.restart_clock();
            actor.playback.reset_frame();
            actor.position.screen_y = parent.position.screen_y.wrapping_add(4);
            actor.position.world_x = parent.position.world_x;
            self.effects.spawned.push(index);
        }
    }
    fn eruption(&mut self) {
        let s = self.slot;
        if self.actors[s].hit_marker != 0 {
            self.reaction();
            return;
        }
        let old_y = self.actors[s].position.screen_y;
        self.actors[s].profile = if old_y != 0 && old_y < 0xd0 { 0x91 } else { 0 };
        let selector = usize::from(self.actors[s].local_state_code());
        if self.active_init() {
            if self.distance() >= 0x20 {
                return;
            }
            self.actors[s].motion_flags = 0;
            {
                let actor_value = 0;
                self.actors[s].set_local_counter_code(actor_value);
            };
            self.actors[s].local_aux2 = 0;
            self.actors[s].position.screen_y = 0xef;
            self.actors[s].vertical_velocity = self.context.catalog.eruption_velocity[selector];
            self.actors[s].playback.freeze_at_start();
            self.actors[s].playback.reset_frame();
            self.increment();
        }
        self.actors[s].local_aux2 = self.actors[s].local_aux2.wrapping_add(1);
        let rising = self.actors[s].motion_flags == 0;
        if rising {
            self.integrate(Integration::FallingY);
            self.integrate(Integration::Gravity);
            if self.actors[s].vertical_velocity < 0 {
                self.actors[s].motion_flags = self.actors[s].motion_flags.wrapping_add(1);
                self.actors[s].vertical_velocity = 0;
                {
                    let animation_value = 1;
                    self.actors[s].playback.set_frame_code(animation_value);
                };
            }
        } else if self.actors[s].local_counter_code() == 0 {
            self.integrate(Integration::Gravity);
            self.integrate(Integration::FallingY);
            if self.actors[s].position.screen_y >= 0xf0 {
                self.actors[s].position.y_fraction = 0;
                self.actors[s].position.screen_y = 0;
                {
                    let actor_value = 0x78;
                    self.actors[s].set_local_counter_code(actor_value);
                };
            }
        } else {
            {
                let actor_value = self.actors[s].local_counter_code() - (1);
                self.actors[s].set_local_counter_code(actor_value);
            };
            if self.actors[s].local_counter_code() == 0 {
                {
                    let actor_value = self.actors[s].state_code() & (0xf0);
                    self.actors[s].set_state_code(actor_value);
                };
                return;
            }
        }
        let event = if rising {
            self.context.catalog.eruption_rise_event[selector]
        } else {
            self.context.catalog.eruption_fall_event[selector]
        };
        if event == 0 || event != self.actors[s].local_aux2 {
            return;
        }
        self.effects.palette_pair = Some(if rising { [0x19, 0x27] } else { [0x21, 0x17] });
        self.effects.palette_upload = true;
        self.sound(0x21);
        if let Some(index) = self.free() {
            let x = self.actors[s].position.world_x;
            let actor = &mut self.actors[index];
            {
                let actor_value = 0xa8;
                actor.set_state_code(actor_value);
            };
            actor.render = 0xa8;
            actor.position.screen_y = 0xbc;
            actor.position.world_x = x;
            actor.animation = 0x37;
            actor.playback.restart_clock();
            actor.playback.reset_frame();
            actor.profile = 0;
            {
                let actor_value = 0;
                actor.set_handler(actor_value);
            };
            self.effects.spawned.push(index);
        }
    }
    fn contact_pose(&mut self) {
        use crate::player::Action;
        let s = self.slot;
        if self.context.player.action == Action::Death {
            return;
        }
        if self.actors[s].render & 0x80 == 0 {
            {
                let actor_value = 0;
                self.actors[s].set_state_code(actor_value);
            };
            return;
        }
        if self.actors[s].hit_marker != 0 {
            self.reaction();
            return;
        }
        if !self.active_init() {
            let protection = self.context.player.protection;
            if protection != 0 && protection != 8 {
                return;
            }
            if protection == 8 {
                self.actors[s].render = (self.actors[0].render & 0xfc) ^ FLIP_X;
                self.sound(0x1a);
                self.actors[s].animation = self.actors[s].animation.wrapping_add(1);
                if self.actors[s].animation != 0 {
                    self.reset_clocks();
                    return;
                }
            }
            self.actors[s].render &= !FLIP_X;
            self.actors[s].animation = self.actors[s].animation.wrapping_sub(1);
            self.decrement();
            self.reset_clocks();
            return;
        }
        if self.actors[s].playback.at_initial_frame(0) {
            self.actors[s].playback.restart_clock();
            if self.distance() < 0x0c {
                if self.actors[s].animation == 0x12
                    && self.actors[s].position.screen_y < self.actors[0].position.screen_y
                {
                    self.reset_clocks();
                    return;
                }
                {
                    let animation_value = 1;
                    self.actors[s].playback.set_frame_code(animation_value);
                };
            }
        }
        if self.actors[s].playback.at_initial_frame(2) && self.actors[s].playback.at_running_tick(0)
        {
            self.sound(0x19);
        }
        if let Some(&profile) = self
            .context
            .catalog
            .contact_profiles
            .get(usize::from(self.actors[s].playback.frame_code()))
        {
            self.actors[s].profile = profile;
        }
        if !self.contact() {
            return;
        }
        if self.distance() >= 8 {
            self.context.player.action = Action::Damage;
            return;
        }
        self.increment();
        {
            let animation_value = 2;
            self.actors[s].playback.set_frame_code(animation_value);
        };
        self.actors[s].playback.freeze_at_start();
        self.context.player.protection = CONTACT_PROTECTION;
        self.context.player.action = Action::Protected;
        let index = usize::from(self.actors[s].animation != 0x12);
        self.actors[0].position.screen_y = self.actors[s]
            .position
            .screen_y
            .wrapping_add(self.context.catalog.attach_y[index]);
        self.actors[0].position.world_x = self.actors[s].position.world_x;
        self.actors[0].horizontal_magnitude = 0x300;
        self.actors[0].vertical_velocity = 0;
        self.actors[0].playback.restart_clock();
        self.actors[0].playback.reset_frame();
        self.actors[0].animation = self.context.catalog.attach_poses[index];
    }
    fn waiting_jumper(&mut self) {
        let s = self.slot;
        if self.active_init() {
            self.increment();
            self.actors[s].vertical_velocity = 0;
            {
                let actor_value = 0;
                self.actors[s].set_local_counter_code(actor_value);
            };
            self.actors[s].horizontal_magnitude = 0x200;
            {
                let actor_value = 0x78;
                self.actors[s].set_local_state_code(actor_value);
            };
            self.face();
        }
        if self.actors[s].state_code() & 2 == 0 {
            if self.actors[s].local_state_code() != 0 {
                {
                    let actor_value = self.actors[s].local_state_code() - (1);
                    self.actors[s].set_local_state_code(actor_value);
                };
                self.actors[s].playback.restart_clock();
                return;
            }
            if !self.actors[s].playback.at_initial_frame(3) {
                return;
            }
            self.reset_clocks();
            self.actors[s].animation = 0x2d;
            self.actors[s].profile |= 0x80;
            self.increment();
        }
        let counter = self.actors[s].local_counter_code();
        if counter != 0 && self.actors[s].playback.at_initial_frame(3) {
            self.actors[s].playback.restart_clock();
        }
        if counter != 0 && counter & 0x80 == 0 {
            self.motion(Movement::Rise, 9, 2);
            if self.actors[s].vertical_velocity < 0 {
                {
                    let actor_value = (self.actors[s].vertical_velocity as u16 >> 8) as u8;
                    self.actors[s].set_local_counter_code(actor_value);
                };
                self.actors[s].vertical_velocity = 0;
            }
            return;
        }
        if self.motion(Movement::Fall, 6, 2).bits & 1 == 0 {
            return;
        }
        if counter & 0x80 != 0 {
            self.reset_clocks();
            self.actors[s].animation = 0x42;
            {
                let actor_value = 0;
                self.actors[s].set_local_counter_code(actor_value);
            };
            {
                let actor_value = 0x1e;
                self.actors[s].set_local_state_code(actor_value);
            };
            self.decrement();
            return;
        }
        let facts = self.walk(0, 3, 2);
        if self.actors[s].playback.at_running_tick(0) {
            self.sound(0x31);
        }
        if self.actors[s].hit_marker != 0 {
            self.reaction();
            return;
        }
        if facts.bits & 1 != 0 {
            self.reverse();
            self.actors[s].animation = 0x2e;
            self.reset_clocks();
            self.actors[s].profile &= 0x7f;
            if let Some(target) = facts.actor
                && self.actors[target].handler == 8
            {
                self.effects.work.push(Work::LaunchTerrain(target));
                self.world
                    .launch_terrain(self.actors, target, self.context.player.pose_direction);
            }
            self.sound(0x1d);
            {
                let actor_value = self.actors[s].local_counter_code().wrapping_add(1);
                self.actors[s].set_local_counter_code(actor_value);
            };
            self.actors[s].vertical_velocity = PATROL_JUMP_SPEED;
        }
    }
    fn load_preset(&mut self, preset: u8, absolute_vertical: bool) {
        let s = self.slot;
        let index = usize::from(preset);
        self.actors[s].horizontal_magnitude = self.context.spawn.horizontal_presets[index];
        if self.actors[s].horizontal_magnitude & 0x8000 != 0 {
            self.actors[s].horizontal_magnitude =
                self.actors[s].horizontal_magnitude.wrapping_neg();
        }
        self.actors[s].vertical_velocity = self.context.spawn.vertical_presets[index];
        if absolute_vertical {
            self.actors[s].motion_flags = 0;
            if self.actors[s].vertical_velocity < 0 {
                self.actors[s].motion_flags = 8;
                self.actors[s].vertical_velocity = self.actors[s].vertical_velocity.wrapping_neg();
            }
        }
    }
    fn preset_flight(&mut self) {
        let s = self.slot;
        if self.actors[s].animation == 0x85 && self.actors[s].hit_marker != 0 {
            self.reaction();
            return;
        }
        if self.actors[s].render & 0x80 == 0 {
            {
                let actor_value = 0;
                self.actors[s].set_state_code(actor_value);
            };
            self.actors[s].render = 0;
            *self.context.special_spawn = 0x3c;
            return;
        }
        if self.active_init() {
            if self.actors[s].animation != 0x85 {
                if self.distance() >= 0x60 {
                    return;
                }
                self.sound(0x30);
            }
            self.increment();
            {
                let actor_value = 0;
                self.actors[s].set_local_state_code(actor_value);
            };
            {
                let actor_value = 0;
                self.actors[s].set_local_counter_code(actor_value);
            };
            self.actors[s].local_aux2 = 0;
            self.actors[s].vertical_velocity = 0x300;
            self.face();
            if self.actors[s].direction & 2 == 0 {
                self.actors[s].render |= FLIP_X;
            }
        }
        self.contact();
        if self.actors[s].state_code() & 2 == 0 {
            let a = self.actors[s].position.screen_y;
            let p = self.actors[0].position.screen_y;
            if a >= p && a - p >= 0x10 {
                self.increment();
            }
            self.integrate(Integration::PositiveY);
            return;
        }
        if self.actors[s].local_state_code() == 0 {
            let counter = self.actors[s].local_counter_code();
            self.load_preset(
                self.context.catalog.flight_sequence[usize::from(counter)],
                false,
            );
            {
                let actor_value = if counter >= 7 && self.actors[s].local_aux2 & 1 != 0 {
                    8
                } else {
                    3
                };
                self.actors[s].set_local_state_code(actor_value);
            };
            {
                let actor_value = counter.wrapping_add(1);
                self.actors[s].set_local_counter_code(actor_value);
            };
            if self.actors[s].local_counter_code() == 14 {
                {
                    let actor_value = 0;
                    self.actors[s].set_local_counter_code(actor_value);
                };
                self.actors[s].local_aux2 = self.actors[s].local_aux2.wrapping_add(1);
            }
        }
        let a = self.actors[s].position.screen_y;
        let p = self.actors[0].position.screen_y;
        if self.actors[s].state_code() & 1 == 0 && p >= a && p - a >= 0x20 {
            self.increment();
        }
        if self.actors[s].state_code() & 1 != 0 {
            {
                let actor_value = self.actors[s].local_state_code().wrapping_sub(1);
                self.actors[s].set_local_state_code(actor_value);
            };
        }
        let actor = &mut self.actors[s];
        let fixed =
            (u16::from(actor.position.screen_y) << 8) | u16::from(actor.position.y_fraction);
        let next = fixed.wrapping_add(actor.vertical_velocity as u16);
        actor.position.screen_y = (next >> 8) as u8;
        actor.position.y_fraction = next as u8;
        self.integrate(if self.actors[s].direction & 2 == 0 {
            Integration::PositiveX
        } else {
            Integration::NegativeX
        });
    }
    fn triggered_jump(&mut self) {
        let s = self.slot;
        if self.active_init() {
            if self.distance() >= 0x32 {
                self.actors[s].playback.restart_clock();
                self.actors[s].render &= !FLIP_X;
                return;
            }
            self.actors[s].vertical_velocity = PATROL_JUMP_SPEED;
            self.actors[s].horizontal_magnitude = 0x180;
            self.actors[s].direction = 1;
            self.increment();
        }
        self.contact();
        let phase = self.actors[s].state_code() & 15;
        if phase == 2 {
            self.motion(Movement::Rise, 0x53, 0);
            if self.actors[s].vertical_velocity < 0 {
                self.actors[s].vertical_velocity = 0;
                self.increment();
            }
            return;
        }
        if phase == 3 {
            if self.actors[s].playback.at_running_tick(0) {
                self.sound(0x27);
            }
            if self.motion(Movement::Fall, 0x31, 2).bits & 1 == 0 {
                return;
            }
        }
        let old = self.actors[s].direction;
        let facts = if old & 1 == 0 {
            self.motion(Movement::PositiveX, 0x36, 0)
        } else {
            self.motion(Movement::NegativeX, 0x39, 0)
        };
        if facts.bits & 1 != 0 {
            self.reverse();
        }
        if old != self.actors[s].direction {
            self.actors[s].playback.restart_clock();
            self.actors[s].render &= !FLIP_X;
        } else if phase != 3 {
            self.increment();
        }
    }
    fn jump_walker(&mut self, family: Family) {
        let s = self.slot;
        if self.active_init() {
            self.actors[s].vertical_velocity = 0;
            self.increment();
            self.face();
            match family {
                Family::JumpWalker => self.actors[s].horizontal_magnitude = 0x100,
                Family::BounceWalker => {
                    self.actors[s].horizontal_magnitude = self.context.geometry.velocity_presets[0]
                }
                _ => {
                    self.actors[s].horizontal_magnitude = self.context.geometry.velocity_presets[1];
                    {
                        let actor_value = 0;
                        self.actors[s].set_local_state_code(actor_value);
                    };
                }
            }
        }
        let mut fall = true;
        if family != Family::BounceWalker && self.actors[s].state_code() & 2 != 0 {
            self.motion(Movement::Rise, 0x21, 2);
            if self.actors[s].vertical_velocity < 0 {
                self.decrement();
                self.actors[s].vertical_velocity = 0;
            } else {
                fall = false;
                self.reset_clocks();
            }
        }
        if fall && self.motion(Movement::Fall, 0x1e, 2).bits & 1 == 0 {
            self.reset_clocks();
        }
        let facts = self.walk(0x18, 0x1b, 2);
        if family == Family::BounceWalker {
            if facts.bits & 1 != 0 {
                self.reverse();
            }
            return;
        }
        if family == Family::JumpingStealer
            && self.actors[s].local_state_code() == 0
            && self.contact()
        {
            self.context.status.steal(&mut self.actors[s]);
            self.effects.thefts += 1;
        }
        if self.actors[s].state_code() & 2 == 0
            && facts.bits & 1 != 0
            && self.actors[s].vertical_velocity == 0
        {
            self.increment();
            self.actors[s].vertical_velocity = BOUNCE_HEIGHT;
        }
    }
    fn timed_patrol(&mut self, family: Family) {
        let s = self.slot;
        let vertical = family == Family::VerticalPatrol;
        if self.active_init() {
            if vertical {
                self.actors[s].vertical_velocity = 0x100;
                self.face();
            } else {
                self.actors[s].horizontal_magnitude = 0x100;
                self.actors[s].direction = 1;
            }
            {
                let actor_value = 0x16;
                self.actors[s].set_local_state_code(actor_value);
            };
            self.increment();
            if vertical {
                self.actors[s].render = (self.actors[s].render & !FLIP_X)
                    | if self.actors[s].direction & 1 != 0 {
                        FLIP_X
                    } else {
                        0
                    };
            }
        }
        self.contact();
        let positive = self.actors[s].direction & 1 != 0;
        if vertical {
            self.integrate(if positive {
                Integration::NegativeY
            } else {
                Integration::PositiveY
            });
        } else {
            self.actors[s].render =
                (self.actors[s].render & !FLIP_X) | if positive { FLIP_X } else { 0 };
            self.integrate(if positive {
                Integration::PositiveX
            } else {
                Integration::NegativeX
            });
        }
        {
            let actor_value = self.actors[s].local_state_code().wrapping_sub(1);
            self.actors[s].set_local_state_code(actor_value);
        };
        if self.actors[s].local_state_code() == 0 {
            self.reverse();
            {
                let actor_value = if vertical { 0x2c } else { 0x20 };
                self.actors[s].set_local_state_code(actor_value);
            };
        }
    }
    fn timed_stealer(&mut self) {
        let s = self.slot;
        if self.active_init() {
            let interval =
                self.context.catalog.stealer_intervals[usize::from(self.actors[s].handler - 0x30)];
            {
                let actor_value = 0;
                self.actors[s].set_local_counter_code(actor_value);
            };
            {
                let actor_value = interval;
                self.actors[s].set_local_state_code(actor_value);
            };
            self.actors[s].local_aux2 = interval;
            self.actors[s].horizontal_magnitude = 0x100;
            self.actors[s].direction = 1;
            self.increment();
        }
        if self.actors[s].local_counter_code() != 0 {
            {
                let actor_value = self.actors[s].local_counter_code() - (1);
                self.actors[s].set_local_counter_code(actor_value);
            };
            return;
        }
        let positive = self.actors[s].direction & 1 != 0;
        self.actors[s].render =
            (self.actors[s].render & !FLIP_X) | if positive { FLIP_X } else { 0 };
        self.integrate(if positive {
            Integration::PositiveX
        } else {
            Integration::NegativeX
        });
        if self.contact() {
            self.context.status.steal(&mut self.actors[s]);
            self.effects.thefts += 1;
        }
        {
            let actor_value = self.actors[s].local_state_code().wrapping_sub(1);
            self.actors[s].set_local_state_code(actor_value);
        };
        if self.actors[s].local_state_code() == 0 {
            self.reverse();
            {
                let actor_value = THIEF_PAUSE;
                self.actors[s].set_local_counter_code(actor_value);
            };
            {
                let actor_value = self.actors[s].local_aux2.wrapping_mul(2);
                self.actors[s].set_local_state_code(actor_value);
            };
        }
    }
    fn thrower(&mut self) {
        let s = self.slot;
        if self.active_init() {
            self.actors[s].vertical_velocity = 0;
            {
                let actor_value = 0;
                self.actors[s].set_local_state_code(actor_value);
            };
            {
                let actor_value = 0;
                self.actors[s].set_local_counter_code(actor_value);
            };
            self.increment();
            self.actors[s].horizontal_magnitude = self.context.geometry.velocity_presets[0];
            self.face();
        }
        if self.actors[s].state_code() & 15 == 3 {
            if self.actors[s].local_counter_code() == 0
                && self.actors[s].playback.frame_code() & 15 != 0
                && let Some(child) = self.free()
            {
                let parent = self.actors[s];
                let actor = &mut self.actors[child];
                actor.playback.restart_clock();
                actor.playback.reset_frame();
                actor.horizontal_magnitude = 0x200;
                {
                    let actor_value = parent.state_code();
                    actor.set_state_code(actor_value);
                };
                actor.render = parent.render;
                actor.animation = 0x17;
                {
                    let actor_value = 4;
                    actor.set_handler(actor_value);
                };
                actor.position.world_x = parent.position.world_x;
                actor.position.screen_y = parent.position.screen_y.wrapping_sub(8);
                actor.profile = 0x17;
                self.effects.spawned.push(child);
                self.sound(THROW_SOUND);
                {
                    let actor_value = self.actors[s].local_counter_code().wrapping_add(1);
                    self.actors[s].set_local_counter_code(actor_value);
                };
            }
            {
                let actor_value = self.actors[s].local_state_code().wrapping_sub(1);
                self.actors[s].set_local_state_code(actor_value);
            };
            if self.actors[s].local_state_code() == 0 {
                self.reset_clocks();
                self.actors[s].animation = 0x14;
                {
                    let actor_value = self.actors[s].state_code() & (0xf1);
                    self.actors[s].set_state_code(actor_value);
                };
            }
            return;
        }
        let mut fall = true;
        if self.actors[s].state_code() & 2 != 0 {
            self.motion(Movement::Rise, 9, 2);
            if self.actors[s].vertical_velocity < 0 {
                self.decrement();
                self.actors[s].vertical_velocity = 0;
            } else {
                fall = false;
            }
        }
        if fall && self.motion(Movement::Fall, 6, 2).bits & 1 != 0 {
            self.actors[s].animation = 0x14;
        } else {
            self.actors[s].animation = 0x15;
            self.reset_clocks();
        }
        let facts = self.walk(0, 3, 2);
        if self.actors[s].state_code() & 2 != 0 || self.actors[s].vertical_velocity != 0 {
            return;
        }
        if facts.bits & 1 != 0 {
            self.increment();
            self.actors[s].vertical_velocity = 0x400;
        } else if self.actors[s].local_counter_code() == 0 && self.distance() < 0x51 {
            self.actors[s].playback.restart_clock();
            self.actors[s].playback.start_looping_at(0);
            {
                let animation_value = 1;
                self.actors[s].playback.set_restart_code(animation_value);
            };
            self.actors[s].animation = 0x16;
            {
                let actor_value = (self.actors[s].state_code() & 0xf0) | 3;
                self.actors[s].set_state_code(actor_value);
            };
            {
                let actor_value = THROW_DURATION;
                self.actors[s].set_local_state_code(actor_value);
            };
        }
    }
}
#[cfg(test)]
mod contracts;

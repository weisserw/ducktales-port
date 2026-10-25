//! Player control policies and wrapper contracts (B06). See trace.md.
//! ActorContact is the sole physical/animation state; input and health are borrowed.
//! Stage3 declarations deliberately contain no gameplay implementation.
use crate::{
    actors::{ActorContact, ActorPool},
    runtime::{Buttons, Controllers, PortInput},
    status::{Health, TitleRule},
};
use anyhow::{Context, Result};
pub const HANDLERS: usize = 17;
pub const PLAYER_GRAVITY: u8 = 0x33;
pub const NORMAL_SPEED: u16 = 0x0180;
pub const MOMENTUM_CAP: u16 = 0x0280;
pub const MOMENTUM_ACCELERATION: u16 = 0x10;
pub const MOMENTUM_RELEASE: u16 = 6;
pub const MOMENTUM_REVERSAL: u16 = 0x10;
pub const ANIMATION_DELAY: u8 = 0x10;
pub const POGO_SPEED: i16 = 0x05a8;
pub const DAMAGE_SPEED: i16 = 0x0169;
pub const DAMAGE_PROTECTION: u8 = 0x46;
pub const DAMAGE_RELEASE: u8 = 0x3c;
pub const RECOIL_THRESHOLD: u8 = 8;
pub const MAX_PUSH: u8 = 12;
pub const BOTTOM_ROOM_Y: u8 = 0xe0;
pub const TOP_CLIMB_Y: u8 = 0x39;
pub const LIFE_LOSS_Y: u8 = 0xd0;
pub const LIFE_LOSS_REQUEST: u8 = 0x87;
pub const POGO_HEIGHT_LIMIT: u8 = 0xa0;
pub const LATE_ROUTE_Y: u8 = 0xc0;
pub const LATE_ROUTE_MESSAGE: u8 = 0x0c;
pub const LAUNCH_PAGE: u8 = 7;
pub const LAUNCH_CEILING: u8 = 0x18;
pub const DEATH_SPEED: i16 = 0x0400;
pub const DEATH_DRIFT: u16 = 0x0080;
pub const SPECIAL_AUDIO: u8 = 8;
pub const IDLE_ANIMATION: u8 = 0;
pub const WALK_ANIMATION: u8 = 1;
pub const AIR_ANIMATION: u8 = 2;
pub const CROUCH_ANIMATION: u8 = 3;
pub const CLIMB_ANIMATION: u8 = 4;
pub const HURT_ANIMATION: u8 = 5;
pub const POGO_ANIMATION: u8 = 6;
pub const STRIKE_READY_ANIMATION: u8 = 7;
pub const LAND_ANIMATION: u8 = 8;
pub const SWING_ANIMATION: u8 = 9;
pub const POSE_ANIMATION: u8 = 0x2c;
pub const BLOCKED_SWING_ANIMATION: u8 = 0x41;
pub const RESTART_FRAME: u8 = 0x80;
pub const POGO_START_FRAME: u8 = 0x81;
pub const FROZEN_TICK: u8 = 0x80;
pub const SWING_SOUND: u8 = 0x13;
pub const BLOCKED_SOUND: u8 = 0x18;
pub const LAND_SOUND: u8 = 0x10;
pub const POGO_SOUND: u8 = 0x11;
pub const CLIMB_SOUND: u8 = 0x12;
pub const DAMAGE_SOUND: u8 = 0x14;
pub const BLOCKED_CYCLES: u8 = 5;
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum Action {
    #[default]
    Standing = 0,
    Airborne = 1,
    ForcedJump = 2,
    Climbing = 3,
    Swing = 4,
    Protected = 5,
    Damage = 6,
    TerrainPose = 7,
    FallOut = 8,
    WaitA = 9,
    CarriedLocked = 10,
    CarriedJumpable = 11,
    ExternalLaunch = 12,
    Death = 13,
    Hold = 14,
    Dialogue = 15,
    ConvertTimer = 16,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Handler {
    Standing,
    Airborne,
    ForcedJump,
    Climbing,
    Swing,
    Protected,
    Damage,
    TerrainPose,
    FallOut,
    WaitA,
    Carried,
    ExternalLaunch,
    Death,
    Hold,
    Dialogue,
    ConvertTimer,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerData {
    pub handlers: [Handler; HANDLERS],
    pub facing: [u8; 2],
    pub column_starts: [u8; 2],
    pub climb_offsets: [u8; 4],
    pub rise_thresholds: [u16; 2],
    pub vertical_samples: [u8; 2],
    pub transition: [u8; 8],
}
impl PlayerData {
    pub fn original(rom: &[u8]) -> Result<Self> {
        const FACING: usize = 0x1e649;
        const COLUMN_STARTS: usize = 0x1e74e;
        const CLIMB_OFFSETS: usize = 0x1e7f6;
        const TRANSITION: usize = 0x1e8f3;
        const VERTICAL: usize = 0x1eac3;
        let data = |start: usize, count: usize| {
            rom.get(start..start + count)
                .context("player parameter bounds")
        };
        use Handler::*;
        const TARGETS: [Handler; HANDLERS] = [
            Standing,
            Airborne,
            ForcedJump,
            Climbing,
            Swing,
            Protected,
            Damage,
            TerrainPose,
            FallOut,
            WaitA,
            Carried,
            Carried,
            ExternalLaunch,
            Death,
            Hold,
            Dialogue,
            ConvertTimer,
        ];
        let vertical = data(VERTICAL, 6)?;
        Ok(Self {
            handlers: TARGETS,
            facing: data(FACING, 2)?.try_into()?,
            column_starts: data(COLUMN_STARTS, 2)?.try_into()?,
            climb_offsets: data(CLIMB_OFFSETS, 4)?.try_into()?,
            rise_thresholds: [
                u16::from_le_bytes([vertical[2], vertical[3]]),
                u16::from_le_bytes([vertical[4], vertical[5]]),
            ],
            vertical_samples: vertical[..2].try_into()?,
            transition: data(TRANSITION, 8)?.try_into()?,
        })
    }
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Player {
    pub action: Action,
    pub queued: Option<Action>,
    pub protection: u8,
    pub special_mode: u8,
    pub special_ticks: u16,
    pub gravity: u8,
    pub animation_delay: u8,
    pub pose_direction: u8,
    pub momentum_control: u8,
    pub momentum_direction: u8,
    pub external_motion: u8,
    pub actor_contacts: u8,
    pub pogo_height_control: u8,
    pub jump_velocity: i16,
}
/// Results returned at B05 boundaries, not a second collision implementation.
/// Immediate actions have already been applied by the scan before player policy.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CollisionFacts {
    /// Exact slot returned by the actor scan, distinct from preexisting hit marks.
    pub actor: Option<usize>,
    /// Probe result slots0..3, including the source alias at slot1.
    pub classes: [u8; 4],
    pub bits: u8,
    pub actor_bits: u8,
    pub penetration: Option<u8>,
    pub render: u8,
}
pub use crate::collision::HorizontalDirection as Horizontal;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Vertical {
    Falling,
    Rising,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Work {
    VerticalFall,
    VerticalRise,
    DamageContinuation,
    ProtectedContinuation,
    Horizontal(Horizontal),
    HorizontalInput,
    Gravity,
    IntegrateY,
    IntegrateX,
    Terrain { start: u8, remaining: u8 },
    ActorScan,
    CostOverlay,
    DamageHealth,
    DeathTrajectory,
    UpwardProbe,
    DescendProbe,
    Dialogue,
    ConvertTimer,
    VerticalTransition,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Effects {
    pub work: Vec<Work>,
    pub sounds: Vec<u8>,
    pub stage_sounds: Vec<u8>,
    pub reload: Option<u8>,
    pub message: Option<u8>,
    pub transition: bool,
    pub scroll: Option<u8>,
    pub spawn: Option<u8>,
    pub collision_bits: u8,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RisePolicy {
    Fall,
    StopThenFall,
    Rise,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HorizontalPlan {
    pub direction: Option<Horizontal>,
    pub external_positive_step: bool,
    pub update_facing_after: bool,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RoomContext {
    pub camera: u16,
    pub camera_room: u8,
    pub terrain_room: u8,
    pub row: u8,
    pub rows: u8,
    pub columns: u8,
    pub late_route: u8,
    pub previous_scroll: u8,
    pub terrain_column: u8,
    pub special_spawn: u8,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoomResult {
    Rejected,
    Message,
    Committed,
}
/// Final comparisons of player policy, distinct from movement helper outcomes.
/// The composed caller retains the latest actual operation only for this call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReturnOperation {
    CrouchAnimation(u8),
    WalkAnimation(u8),
    StrikeAnimation(u8),
    HorizontalModeAnimation(u8),
    AirAnimation(u8),
    PogoAnimation(u8),
    RememberedPoseDirection { held: u8, remembered: u8 },
    MomentumMinimum,
    AirModeSelected,
    ProtectionHold(u8),
    ClimbTerrainClass(u8),
    ClimbScreenBoundary { y: u8, upward: bool },
    ClimbSampleOffset(u8),
    ClimbSamplePair { first: u8, second: u8 },
    ClimbOddScreenY(u8),
    LateRouteScreenY(u8),
    RoomEntry(u8),
    RoomRowBoundary { row: u8, rows: u8 },
    FallingScreenBoundary(u8),
    PoseAdjustmentOverflow(bool),
    PoseAlreadySelected,
    CarriedState(u8),
    LaunchWorldPage(u8),
    LifeLossVariantOverflow(bool),
    SpecialDuration { old_ticks: u16 },
    SwingFrameSix(u8),
    SwingFrameTwo(u8),
    BlockedSwing,
    ExemptSwingContact,
}
impl ReturnOperation {
    pub fn comparison(self) -> bool {
        match self {
            Self::CrouchAnimation(previous) => previous >= CROUCH_ANIMATION,
            Self::WalkAnimation(previous) => previous >= WALK_ANIMATION,
            Self::StrikeAnimation(previous) => previous >= STRIKE_READY_ANIMATION,
            Self::HorizontalModeAnimation(previous) => previous >= POGO_ANIMATION,
            Self::AirAnimation(previous) => previous >= STRIKE_READY_ANIMATION,
            Self::PogoAnimation(previous) => POGO_ANIMATION >= previous,
            Self::RememberedPoseDirection { held, remembered } => held >= remembered,
            Self::MomentumMinimum => false,
            Self::AirModeSelected | Self::PoseAlreadySelected => true,
            Self::ProtectionHold(ticks) => ticks >= RECOIL_THRESHOLD,
            Self::ClimbTerrainClass(class) => class >= 2,
            Self::ClimbScreenBoundary { y, upward } => {
                y >= if upward { TOP_CLIMB_Y } else { BOTTOM_ROOM_Y }
            }
            Self::ClimbSampleOffset(nibble) => nibble >= 8,
            Self::ClimbSamplePair { first, second } => first >= second,
            Self::ClimbOddScreenY(y) => y >= 16,
            Self::LateRouteScreenY(y) => y >= LATE_ROUTE_Y,
            Self::RoomEntry(room) => room == u8::MAX,
            Self::RoomRowBoundary { row, rows } => row >= rows,
            Self::FallingScreenBoundary(y) => y >= BOTTOM_ROOM_Y,
            Self::PoseAdjustmentOverflow(overflow) | Self::LifeLossVariantOverflow(overflow) => {
                overflow
            }
            Self::CarriedState(state) => state >= Action::CarriedLocked as u8,
            Self::LaunchWorldPage(page) => page >= LAUNCH_PAGE,
            Self::SpecialDuration { old_ticks } => old_ticks != 0,
            Self::SwingFrameSix(frame) => frame >= 6,
            Self::SwingFrameTwo(frame) => frame >= 2,
            Self::BlockedSwing | Self::ExemptSwingContact => true,
        }
    }
}
/// Source integration ownership stays in B05. A wrapper must invoke these calls
/// in the ordered sequence contracted in Player::vertical and launch/death.
/// The boundary borrows the canonical pool; callbacks may apply immediate contact
/// state/slot changes. Queued terrain state stays separate until B13 frame tail.
pub trait ContactWorld {
    /// Final aggregate returned by an actual player physics call.
    fn collision_returned(&mut self, _bits: u8) {}
    /// A named policy comparison can supersede the latest collision result.
    /// Fixtures that do not inspect caller return provenance need no observer.
    fn returned(&mut self, _operation: ReturnOperation) {}
    /// The adapter resolves this against that actual source submission before
    /// returning. No source queue consumer runs inside the player body.
    fn returned_audio(&mut self, _command: u8, _stage_command: bool) {}

    fn overlay_cost(&mut self, _health: &mut Health) {}
    /// B05's actor-zero wrapper has release/return semantics beyond raw X
    /// integration. The default preserves boundary-only synthetic fixtures.
    fn positive_with_collision(
        &mut self,
        actors: &mut ActorPool,
        player: &mut Player,
    ) -> CollisionFacts {
        self.integrate_x(&mut actors[0], Horizontal::Positive);
        let facts = self.probe(actors, player, 0, 2);
        if facts.bits & crate::collision::SOLID_CONTACT != 0 {
            self.snap(&mut actors[0], crate::collision::Snap::PositiveX);
        }
        facts
    }
    fn gravity(&mut self, actor: &mut ActorContact, step: u8);
    fn integrate_y(&mut self, actor: &mut ActorContact);
    fn integrate_x(&mut self, actor: &mut ActorContact, direction: Horizontal);
    fn scan(
        &mut self,
        actors: &mut ActorPool,
        player: &mut Player,
        vertical: bool,
    ) -> CollisionFacts;
    fn snap(&mut self, actor: &mut ActorContact, direction: crate::collision::Snap);
    fn probe(
        &mut self,
        actors: &mut ActorPool,
        player: &mut Player,
        start: u8,
        remaining: u8,
    ) -> CollisionFacts;
}
/// Caller-owned gameplay context borrowed for a complete player update. No
/// duplicate world layout/camera/health store belongs to Player.
pub struct StepContext<'a> {
    pub room: &'a mut RoomContext,
    pub rooms: &'a [u8],
    pub health: &'a mut Health,
    pub title_rule: TitleRule,
    pub stage: u8,
    pub life_loss_variant: u8,
    pub current_stage_audio: u8,
}
impl Player {
    /// Complete source call: pre-dispatch timers, action body, immediate
    /// continuations. Work entries are ordered observations, never a delayed
    /// replacement for source same-frame mutations. Dialogue/conversion bodies
    /// are delegated to their owners at this call boundary.
    pub fn step(
        &mut self,
        _actors: &mut ActorPool,
        _inputs: &mut Controllers,
        _context: &mut StepContext<'_>,
        data: &PlayerData,
        _world: &mut dyn ContactWorld,
    ) -> Effects {
        let special_duration =
            (self.special_mode != 0 && self.special_mode & 0x80 == 0).then_some(self.special_ticks);
        let mut effects = self.pre_dispatch(_context.current_stage_audio, _context.stage);
        if let Some(old_ticks) = special_duration {
            _world.returned(ReturnOperation::SpecialDuration { old_ticks });
            if self.special_ticks == 0 {
                _world.returned_audio(_context.stage, true);
            }
        }
        let entry = self.action;
        let input = _inputs.ports[0];
        match entry {
            Action::Standing => {
                merge_effects(
                    &mut effects,
                    self.vertical(_actors, Vertical::Falling, input.held.0, data, _world),
                );
                let standing_animation = _actors[0].animation;
                let walking_continuation = self.action == Action::Standing
                    && input.held.0 & (Buttons::LEFT.0 | Buttons::RIGHT.0) != 0
                    && input.held.0 & Buttons::DOWN.0 == 0
                    && input.pressed.0 & Buttons::A.0 == 0
                    && !(standing_animation == STRIKE_READY_ANIMATION
                        && input.pressed.0 & Buttons::B.0 != 0);
                let policy = self.standing_controls(&mut _actors[0], input);
                if self.action == Action::Swing {
                    _world.returned(ReturnOperation::StrikeAnimation(standing_animation));
                }
                if self.action == Action::Standing
                    && policy
                        .work
                        .iter()
                        .all(|work| matches!(work, Work::CostOverlay))
                {
                    _world.returned(ReturnOperation::CrouchAnimation(standing_animation));
                }
                self.execute_policy(
                    policy,
                    &mut effects,
                    _actors,
                    _inputs,
                    _context,
                    data,
                    _world,
                    walking_continuation,
                );
            }
            Action::Airborne | Action::ForcedJump => {
                merge_effects(
                    &mut effects,
                    self.airborne_call(
                        _actors,
                        _inputs,
                        _context,
                        data,
                        _world,
                        entry == Action::ForcedJump,
                    ),
                );
            }
            Action::Climbing => {
                let direction = input.held.0 & 12;
                let last = if direction != 0 {
                    _actors[0].vertical_velocity = _actors[0].horizontal_magnitude as i16;
                    let start = if direction == Buttons::UP.0 {
                        0x14
                    } else {
                        0x10
                    };
                    // The downward wrapper negates the magnitude only for integration.
                    if direction != Buttons::UP.0 {
                        _actors[0].vertical_velocity = _actors[0].vertical_velocity.wrapping_neg();
                    }
                    _world.integrate_y(&mut _actors[0]);
                    _actors[0].vertical_velocity = _actors[0].horizontal_magnitude as i16;
                    let facts = _world.probe(_actors, self, start, 3);
                    if facts.bits & crate::collision::SOLID_CONTACT != 0 {
                        _world.snap(
                            &mut _actors[0],
                            if start == 0x14 {
                                crate::collision::Snap::NegativeY
                            } else {
                                crate::collision::Snap::PositiveY
                            },
                        );
                    }
                    facts.classes[3]
                } else {
                    0
                };
                if direction != 0 {
                    _world.returned(ReturnOperation::ClimbTerrainClass(last));
                    if last == 2 {
                        _world.returned(ReturnOperation::ClimbScreenBoundary {
                            y: _actors[0].position.screen_y,
                            upward: direction == Buttons::UP.0,
                        });
                    }
                }
                let policy = self.climbing(&mut _actors[0], input, last);
                let transition = policy.work.contains(&Work::VerticalTransition);
                merge_effects(&mut effects, policy);
                if transition {
                    merge_effects(
                        &mut effects,
                        self.room_transition_observed(
                            _actors,
                            input.held.0,
                            _context.room,
                            _context.rooms,
                            data,
                            Some(_world),
                        )
                        .1,
                    );
                }
            }
            Action::Swing => {
                merge_effects(
                    &mut effects,
                    self.vertical(_actors, Vertical::Rising, input.held.0, data, _world),
                );
                let swing_animation = _actors[0].animation;
                let swing_frame = _actors[0].playback.frame_code();
                let swing_tick = _actors[0].playback.tick_code();
                let direct_walk = swing_frame == 6 && input.held.0 & self.pose_direction != 0;
                if swing_animation == BLOCKED_SWING_ANIMATION {
                    _world.returned(ReturnOperation::BlockedSwing);
                } else if direct_walk || swing_tick != 0 {
                    _world.returned(ReturnOperation::SwingFrameSix(swing_frame));
                } else {
                    _world.returned(ReturnOperation::SwingFrameTwo(swing_frame));
                }
                let contact = if _actors[0].animation != BLOCKED_SWING_ANIMATION
                    && _actors[0].playback.at_initial_frame(2)
                    && _actors[0].playback.at_running_tick(0)
                {
                    _world
                        .scan(_actors, self, false)
                        .actor
                        .map(|slot| _actors[slot].handler)
                } else {
                    None
                };
                if swing_animation != BLOCKED_SWING_ANIMATION
                    && !direct_walk
                    && swing_frame == 2
                    && swing_tick == 0
                {
                    if matches!(contact, Some(8 | 0x15 | 0x42)) {
                        _world.returned(ReturnOperation::ExemptSwingContact);
                    } else {
                        _world.returned_audio(BLOCKED_SOUND, false);
                    }
                }
                let policy = self.swing(&mut _actors[0], input.held.0, contact);
                merge_effects(&mut effects, policy);
            }
            Action::Protected | Action::Damage => {
                if entry == Action::Protected && self.protection >= RECOIL_THRESHOLD {
                    _world.returned(ReturnOperation::ProtectionHold(self.protection));
                }
                let policy = if entry == Action::Protected {
                    self.protected(&mut _actors[0], _context.health, _context.title_rule)
                } else {
                    self.damage(&mut _actors[0], _context.health, _context.title_rule)
                };
                self.execute_policy(
                    policy,
                    &mut effects,
                    _actors,
                    _inputs,
                    _context,
                    data,
                    _world,
                    false,
                );
            }
            Action::TerrainPose => {
                _world.returned(if _actors[0].animation == POSE_ANIMATION {
                    ReturnOperation::PoseAlreadySelected
                } else {
                    ReturnOperation::PoseAdjustmentOverflow(
                        _actors[0].position.screen_y.overflowing_add(4).1,
                    )
                });
                let policy = self.pose(&mut _actors[0], input);
                self.execute_policy(
                    policy,
                    &mut effects,
                    _actors,
                    _inputs,
                    _context,
                    data,
                    _world,
                    false,
                );
            }
            Action::FallOut => {
                merge_effects(
                    &mut effects,
                    self.vertical(_actors, Vertical::Falling, input.held.0, data, _world),
                );
                let policy = self.fall_out(
                    &mut _actors[0],
                    _context.stage,
                    _context.life_loss_variant,
                    _context.health,
                );
                let horizontal = policy.work.contains(&Work::HorizontalInput);
                merge_effects(&mut effects, policy);
                if horizontal {
                    merge_effects(&mut effects, self.horizontal(_actors, input.held.0, _world));
                }
            }
            Action::WaitA => self.wait_a(&mut _actors[0], input),
            Action::CarriedLocked | Action::CarriedJumpable => {
                _world.returned(ReturnOperation::CarriedState(self.action as u8));
                let visible = input.held.0 & Buttons::DOWN.0 == 0;
                let jump =
                    entry == Action::CarriedJumpable && input.pressed == Buttons::A && visible;
                let facts = if visible && !jump {
                    _world.probe(_actors, self, 3, 2)
                } else {
                    CollisionFacts::default()
                };
                let policy = self.carried(&mut _actors[0], input, facts);
                self.execute_policy(
                    policy,
                    &mut effects,
                    _actors,
                    _inputs,
                    _context,
                    data,
                    _world,
                    false,
                );
                if self.action == Action::Damage {
                    let policy = self.damage(&mut _actors[0], _context.health, _context.title_rule);
                    self.execute_policy(
                        policy,
                        &mut effects,
                        _actors,
                        _inputs,
                        _context,
                        data,
                        _world,
                        false,
                    );
                }
            }
            Action::ExternalLaunch => merge_effects(&mut effects, self.launch(_actors, _world)),
            Action::Death => merge_effects(
                &mut effects,
                self.death(
                    _actors,
                    _world,
                    _context.stage,
                    _context.life_loss_variant,
                    _context.health,
                ),
            ),
            Action::Hold => self.hold(&mut _actors[0]),
            Action::Dialogue => effects.work.push(Work::Dialogue),
            Action::ConvertTimer => effects.work.push(Work::ConvertTimer),
        }
        if effects.reload.is_some() {
            let (base, overflow) = _context.stage.overflowing_add(LIFE_LOSS_REQUEST);
            _world.returned(ReturnOperation::LifeLossVariantOverflow(
                u16::from(base) + u16::from(_context.life_loss_variant) + u16::from(overflow)
                    > u16::from(u8::MAX),
            ));
        }
        effects
    }
    /// The jump installation entries tail-call this complete body, including
    /// landing, horizontal motion and both room-boundary continuations.
    fn airborne_call(
        &mut self,
        _actors: &mut ActorPool,
        _inputs: &mut Controllers,
        _context: &mut StepContext<'_>,
        data: &PlayerData,
        _world: &mut dyn ContactWorld,
        forced: bool,
    ) -> Effects {
        let mut effects = Effects::default();
        if forced {
            self.animation_delay = ANIMATION_DELAY;
            self.forced_jump(_inputs);
        }
        merge_effects(
            &mut effects,
            self.vertical(
                _actors,
                Vertical::Rising,
                _inputs.ports[0].held.0,
                data,
                _world,
            ),
        );
        let facts = CollisionFacts {
            bits: effects.collision_bits,
            actor_bits: self.actor_contacts,
            ..CollisionFacts::default()
        };
        let policy = self.landing(&mut _actors[0], _inputs.ports[0], facts);
        let strike_return = policy.work.is_empty() && self.action == Action::Swing;
        self.execute_policy(
            policy,
            &mut effects,
            _actors,
            _inputs,
            _context,
            data,
            _world,
            false,
        );
        if strike_return {
            _world.returned(ReturnOperation::StrikeAnimation(STRIKE_READY_ANIMATION));
        }
        if !strike_return {
            _world.returned(ReturnOperation::AirAnimation(_actors[0].animation));
            self.airborne_horizontal_exit(&mut _actors[0], effects.collision_bits);
            merge_effects(
                &mut effects,
                self.upward_probe(_actors, _inputs.ports[0].held.0, data, _world),
            );
            merge_effects(
                &mut effects,
                self.airborne_boundaries_observed(
                    _actors,
                    _inputs,
                    _context.room,
                    _context.rooms,
                    data,
                    Some(_world),
                ),
            );
        }
        effects
    }
    // A source continuation borrows each canonical owner for the duration of this call.
    #[allow(clippy::too_many_arguments)]
    fn execute_policy(
        &mut self,
        policy: Effects,
        effects: &mut Effects,
        actors: &mut ActorPool,
        inputs: &mut Controllers,
        context: &mut StepContext<'_>,
        data: &PlayerData,
        world: &mut dyn ContactWorld,
        walking_continuation: bool,
    ) {
        let work = policy.work.clone();
        merge_effects(effects, policy);
        for request in work {
            match request {
                Work::VerticalFall | Work::VerticalRise => {
                    let direction = if request == Work::VerticalFall {
                        Vertical::Falling
                    } else {
                        Vertical::Rising
                    };
                    if request == Work::VerticalRise && self.action == Action::Airborne {
                        merge_effects(
                            effects,
                            self.airborne_call(actors, inputs, context, data, world, false),
                        );
                    } else {
                        merge_effects(
                            effects,
                            self.vertical(actors, direction, inputs.ports[0].held.0, data, world),
                        );
                    }
                }
                Work::DamageContinuation | Work::ProtectedContinuation => {
                    // EB35 and EAD9 return from vertical physics before selecting
                    // the hurt pose. Falling callbacks can replace animation.
                    // Only damage state06 releases control at its exact boundary.
                    if request == Work::DamageContinuation && self.protection == DAMAGE_RELEASE {
                        actors[0].horizontal_magnitude =
                            (actors[0].horizontal_magnitude & 0xff00) | 0x80;
                        self.action = Action::Standing;
                    }
                    actors[0].animation = HURT_ANIMATION;
                    let direction = if actors[0].render & crate::collision::FACE_POSITIVE != 0 {
                        Horizontal::Negative
                    } else {
                        Horizontal::Positive
                    };
                    let render = actors[0].render;
                    merge_effects(effects, self.horizontal_direction(actors, direction, world));
                    actors[0].render = (actors[0].render & !crate::collision::FACE_POSITIVE)
                        | (render & crate::collision::FACE_POSITIVE);
                }
                Work::HorizontalInput => {
                    let horizontal = self.horizontal_entry(
                        actors,
                        inputs.ports[0].held.0,
                        world,
                        self.action != Action::Swing,
                    );
                    // E448's JSR returns to WalkCollisionGate even if a
                    // contact changed player state or the old pose was LAND.
                    // Other source horizontal tails have no walking sequel.
                    if walking_continuation {
                        let previous_animation = actors[0].animation;
                        let solid =
                            horizontal.collision_bits & crate::collision::SOLID_CONTACT != 0;
                        world.returned(
                            if solid
                                && (previous_animation == STRIKE_READY_ANIMATION
                                    || self.animation_delay.wrapping_sub(1) == 0)
                            {
                                ReturnOperation::StrikeAnimation(previous_animation)
                            } else {
                                ReturnOperation::WalkAnimation(previous_animation)
                            },
                        );
                        self.finish_walk(
                            &mut actors[0],
                            inputs.ports[0],
                            horizontal.collision_bits,
                        );
                    }
                    merge_effects(effects, horizontal);
                }
                Work::Horizontal(direction) => {
                    let render = actors[0].render;
                    merge_effects(effects, self.horizontal_direction(actors, direction, world));
                    actors[0].render = (actors[0].render & !crate::collision::FACE_POSITIVE)
                        | (render & crate::collision::FACE_POSITIVE);
                }
                Work::CostOverlay => world.overlay_cost(context.health),
                Work::DeathTrajectory => merge_effects(
                    effects,
                    self.death(
                        actors,
                        world,
                        context.stage,
                        context.life_loss_variant,
                        context.health,
                    ),
                ),
                _ => {}
            }
        }
    }
    pub fn pre_dispatch(&mut self, current_audio: u8, stage: u8) -> Effects {
        self.gravity = PLAYER_GRAVITY;
        if self.protection != 0 && self.protection & 0x80 == 0 {
            self.protection -= 1;
        }
        let mut effects = Effects::default();
        if self.special_mode != 0 && self.special_mode & 0x80 == 0 {
            if current_audio != SPECIAL_AUDIO {
                effects.stage_sounds.push(SPECIAL_AUDIO);
            }
            self.special_ticks = self.special_ticks.wrapping_sub(1);
            if self.special_ticks == 0 {
                self.special_mode = 0;
                effects.stage_sounds.push(stage);
            }
        }
        effects
    }
    pub fn dispatch(&self, data: &PlayerData) -> Handler {
        data.handlers[self.action as usize]
    }
    /// After shared vertical mechanics, choose immediate control continuation.
    pub fn standing_controls(&mut self, actor: &mut ActorContact, input: PortInput) -> Effects {
        let mut effects = Effects::default();
        if self.action != Action::Standing {
            effects.work.push(Work::HorizontalInput);
            return effects;
        }
        if input.pressed.0 & Buttons::SELECT.0 != 0 {
            effects.work.push(Work::CostOverlay);
        }
        if actor.animation == STRIKE_READY_ANIMATION && input.pressed.0 & Buttons::B.0 != 0 {
            actor.animation = SWING_ANIMATION;
            reset_animation(actor);
            self.action = Action::Swing;
            self.momentum_direction = 0;
            actor.horizontal_magnitude = NORMAL_SPEED;
            effects.work.push(Work::HorizontalInput);
            return effects;
        }
        if input.pressed.0 & Buttons::A.0 != 0 {
            actor.vertical_velocity = self.jump_velocity;
            self.action = Action::Airborne;
            effects.work.push(Work::VerticalRise);
            return effects;
        }
        let movement = input.held.0 & (Buttons::RIGHT.0 | Buttons::LEFT.0 | Buttons::DOWN.0);
        if movement == 0 {
            if actor.animation != LAND_ANIMATION {
                actor.animation = IDLE_ANIMATION;
                reset_animation(actor);
                self.animation_delay = ANIMATION_DELAY;
            }
            effects.work.push(Work::HorizontalInput);
        } else if movement & Buttons::DOWN.0 != 0 {
            if actor.animation != CROUCH_ANIMATION {
                actor.animation = CROUCH_ANIMATION;
                actor.playback.restart_clock();
                actor.playback.start_looping_at(0);
                {
                    let animation_value = 1;
                    actor.playback.set_restart_code(animation_value);
                };
                self.animation_delay = ANIMATION_DELAY;
            }
        } else {
            effects.work.push(Work::HorizontalInput);
        }
        effects
    }
    pub fn finish_walk(&mut self, actor: &mut ActorContact, input: PortInput, bits: u8) {
        if bits & crate::collision::SOLID_CONTACT != 0 {
            if actor.animation == STRIKE_READY_ANIMATION {
                return;
            }
            self.animation_delay = self.animation_delay.wrapping_sub(1);
            if self.animation_delay == 0 {
                self.pose_direction = input.held.0 & 3;
                actor.animation = STRIKE_READY_ANIMATION;
                reset_animation(actor);
                return;
            }
        } else {
            self.animation_delay = ANIMATION_DELAY;
        }
        if actor.animation != WALK_ANIMATION {
            actor.animation = WALK_ANIMATION;
            reset_animation(actor);
        }
    }
    pub fn horizontal_plan(&mut self, actor: &mut ActorContact, held: u8) -> HorizontalPlan {
        let held_direction = held & 3;
        let direction = |bits: u8| {
            if bits & Buttons::LEFT.0 != 0 {
                Some(Horizontal::Negative)
            } else if bits & Buttons::RIGHT.0 != 0 {
                Some(Horizontal::Positive)
            } else {
                None
            }
        };
        let mut plan = HorizontalPlan {
            direction: None,
            external_positive_step: false,
            update_facing_after: false,
        };
        if !matches!(actor.animation, AIR_ANIMATION | POGO_ANIMATION)
            && (self.momentum_control != 0 || self.momentum_direction != 0)
        {
            self.momentum_control = 0;
            if self.momentum_direction == 0 {
                if held_direction == 0 {
                    return plan;
                }
                self.momentum_direction = held_direction;
            }
            if held_direction == self.momentum_direction {
                actor.horizontal_magnitude = actor
                    .horizontal_magnitude
                    .wrapping_add(MOMENTUM_ACCELERATION);
                if actor.horizontal_magnitude >> 8 == MOMENTUM_CAP >> 8
                    && actor.horizontal_magnitude as u8 >= MOMENTUM_CAP as u8
                {
                    actor.horizontal_magnitude = MOMENTUM_CAP;
                }
            } else {
                let reversing = held_direction != 0;
                actor.horizontal_magnitude =
                    actor.horizontal_magnitude.wrapping_sub(if reversing {
                        MOMENTUM_REVERSAL
                    } else {
                        MOMENTUM_RELEASE
                    });
                if actor.horizontal_magnitude >> 8 == NORMAL_SPEED >> 8
                    && (actor.horizontal_magnitude as u8) < NORMAL_SPEED as u8
                {
                    actor.horizontal_magnitude = NORMAL_SPEED;
                    self.momentum_direction = 0;
                }
                plan.update_facing_after = reversing;
            }
            plan.direction = direction(self.momentum_direction);
            return plan;
        }
        self.ordinary_horizontal_plan(actor, held)
    }
    /// E495 is a separate entry used by the standing B action. It bypasses
    /// animation/momentum tests without clearing MomentumControl itself.
    fn ordinary_horizontal_plan(&mut self, actor: &mut ActorContact, held: u8) -> HorizontalPlan {
        let held_direction = held & (Buttons::LEFT.0 | Buttons::RIGHT.0);
        let direction = |bits: u8| {
            if bits & Buttons::LEFT.0 != 0 {
                Some(Horizontal::Negative)
            } else if bits & Buttons::RIGHT.0 != 0 {
                Some(Horizontal::Positive)
            } else {
                None
            }
        };
        let mut plan = HorizontalPlan {
            direction: None,
            external_positive_step: false,
            update_facing_after: false,
        };
        self.momentum_direction = 0;
        actor.horizontal_magnitude = NORMAL_SPEED;
        if self.external_motion != 0 {
            self.external_motion = 0;
            if held_direction == 0 {
                plan.direction = Some(Horizontal::Positive);
                return plan;
            }
            plan.external_positive_step = true;
        }
        plan.direction = direction(held_direction);
        plan
    }
    pub fn horizontal(
        &mut self,
        actors: &mut ActorPool,
        held: u8,
        world: &mut dyn ContactWorld,
    ) -> Effects {
        self.horizontal_entry(actors, held, world, true)
    }
    fn horizontal_entry(
        &mut self,
        actors: &mut ActorPool,
        held: u8,
        world: &mut dyn ContactWorld,
        mode_entry: bool,
    ) -> Effects {
        let animation = actors[0].animation;
        if mode_entry {
            world.returned(if matches!(animation, AIR_ANIMATION | POGO_ANIMATION) {
                ReturnOperation::AirModeSelected
            } else {
                ReturnOperation::HorizontalModeAnimation(animation)
            });
        }
        let remembered = self.momentum_direction;
        let momentum_path = mode_entry
            && !matches!(animation, AIR_ANIMATION | POGO_ANIMATION)
            && (self.momentum_control != 0 || remembered != 0);
        let plan = if mode_entry {
            self.horizontal_plan(&mut actors[0], held)
        } else {
            self.ordinary_horizontal_plan(&mut actors[0], held)
        };
        if momentum_path
            && remembered != 0
            && self.momentum_direction == 0
            && plan.direction.is_none()
        {
            world.returned(ReturnOperation::MomentumMinimum);
        }
        let mut effects = Effects::default();
        if plan.external_positive_step {
            let position = &mut actors[0].position;
            let fixed = (u32::from(position.world_x) << 8 | u32::from(position.x_fraction))
                .wrapping_add(u32::from(NORMAL_SPEED));
            position.world_x = (fixed >> 8) as u16;
            position.x_fraction = fixed as u8;
        }
        if let Some(direction) = plan.direction {
            merge_effects(
                &mut effects,
                self.horizontal_direction(actors, direction, world),
            );
        }
        if plan.update_facing_after {
            actors[0].render = (actors[0].render & !crate::collision::FACE_POSITIVE)
                | if held & Buttons::RIGHT.0 != 0 {
                    crate::collision::FACE_POSITIVE
                } else {
                    0
                };
        }
        effects
    }
    /// Recoil calls the movement wrapper directly, retaining the installed speed.
    fn horizontal_direction(
        &mut self,
        actors: &mut ActorPool,
        direction: Horizontal,
        world: &mut dyn ContactWorld,
    ) -> Effects {
        let mut effects = Effects::default();
        let before = actors[0].position.world_x;
        effects.work.push(Work::IntegrateX);
        world.integrate_x(&mut actors[0], direction);
        if direction == Horizontal::Positive {
            actors[0].render |= crate::collision::FACE_POSITIVE;
        } else {
            actors[0].render &= !crate::collision::FACE_POSITIVE;
        }
        effects.work.push(Work::ActorScan);
        let contacts = world.scan(actors, self, false);
        let mut spawn = if direction == Horizontal::Positive {
            1
        } else {
            2
        };
        if let Some(penetration) = contacts.penetration {
            let correction = u16::from(penetration.min(MAX_PUSH));
            let x = &mut actors[0].position.world_x;
            *x = if direction == Horizontal::Positive {
                x.wrapping_sub(correction)
            } else {
                x.wrapping_add(correction)
            };
            if before != *x {
                spawn = if before >= *x { 1 } else { 2 };
            }
        }
        let start = if spawn & 2 != 0 { 3 } else { 0 };
        effects.work.push(Work::Terrain {
            start,
            remaining: 2,
        });
        let terrain = world.probe(actors, self, start, 2);
        if terrain.bits & crate::collision::SOLID_CONTACT != 0 {
            world.snap(
                &mut actors[0],
                if spawn & 1 != 0 {
                    crate::collision::Snap::PositiveX
                } else {
                    crate::collision::Snap::NegativeX
                },
            );
        }
        effects.collision_bits = contacts.bits | terrain.bits;
        world.collision_returned(effects.collision_bits);
        effects.scroll = Some(match direction {
            Horizontal::Positive => 1,
            Horizontal::Negative => 2,
        });
        effects.spawn = Some(spawn);
        effects
    }
    pub fn rise_policy(&self, actor: &ActorContact, held: u8, data: &PlayerData) -> RisePolicy {
        if actor.vertical_velocity < 0 {
            return RisePolicy::Fall;
        }
        if (self.action as u8) < Action::Protected as u8 {
            let threshold = data.rise_thresholds[usize::from((self.action as u8 & 2) >> 1)];
            if (actor.vertical_velocity as u16) < threshold && held & Buttons::A.0 == 0 {
                return RisePolicy::StopThenFall;
            }
        }
        RisePolicy::Rise
    }
    pub fn vertical(
        &mut self,
        actors: &mut ActorPool,
        direction: Vertical,
        held: u8,
        data: &PlayerData,
        world: &mut dyn ContactWorld,
    ) -> Effects {
        const TERMINAL_FALL_INTEGER: u8 = 0xf4;
        const RISING_MOTION: u8 = 8;
        let falling = if direction == Vertical::Falling {
            true
        } else {
            match self.rise_policy(&actors[0], held, data) {
                RisePolicy::Fall => true,
                RisePolicy::StopThenFall => {
                    actors[0].vertical_velocity = 0;
                    actors[0].position.y_fraction = 0;
                    true
                }
                RisePolicy::Rise => false,
            }
        };
        self.actor_contacts = 0;
        actors[0].motion_flags = if falling {
            crate::collision::DESCENDING_MOTION
        } else {
            RISING_MOTION
        };
        let mut effects = Effects::default();
        if falling && (actors[0].vertical_velocity as u16 >> 8) as u8 != TERMINAL_FALL_INTEGER {
            effects.work.push(Work::Gravity);
            world.gravity(&mut actors[0], self.gravity);
        }
        let old_y = actors[0].position.screen_y;
        effects.work.push(Work::IntegrateY);
        world.integrate_y(&mut actors[0]);
        effects.work.push(Work::ActorScan);
        let scan = world.scan(actors, self, true);
        if let Some(penetration) = scan.penetration {
            let y = &mut actors[0].position.screen_y;
            *y = if falling {
                y.wrapping_sub(penetration)
            } else {
                y.wrapping_add(penetration).wrapping_add(1)
            };
        }
        let motion = if old_y == actors[0].position.screen_y {
            actors[0].motion_flags
        } else if old_y >= actors[0].position.screen_y {
            RISING_MOTION
        } else {
            crate::collision::DESCENDING_MOTION
        };
        let start = data.vertical_samples[usize::from((motion >> 2) & 1)];
        let render = actors[0].render;
        let saved_contacts = self.actor_contacts | scan.actor_bits;
        effects.work.push(Work::Terrain {
            start,
            remaining: 2,
        });
        let terrain = world.probe(actors, self, start, 2);
        if terrain.bits & crate::collision::SOLID_CONTACT != 0 {
            world.snap(
                &mut actors[0],
                if motion & crate::collision::DESCENDING_MOTION != 0 {
                    crate::collision::Snap::PositiveY
                } else {
                    crate::collision::Snap::NegativeY
                },
            );
        }
        self.actor_contacts |= saved_contacts | terrain.actor_bits;
        actors[0].render = render;
        effects.collision_bits = scan.bits | terrain.bits;
        world.collision_returned(effects.collision_bits);
        if falling {
            if effects.collision_bits & crate::collision::SOLID_CONTACT != 0 {
                actors[0].vertical_velocity = 0;
            } else {
                if self.action == Action::Standing {
                    self.action = Action::Airborne;
                }
                self.observe_airborne_return(&actors[0], held, world);
                self.airborne_animation(&mut actors[0], held);
            }
        } else if effects.collision_bits & crate::collision::SOLID_CONTACT != 0 {
            actors[0].vertical_velocity = 0;
            actors[0].position.y_fraction = 0;
            // EA07 tail-enters the falling routine, whose entry clears the
            // caller-visible contact result. Keep the work already performed,
            // but return the falling call's contacts rather than the ceiling.
            let falling = self.vertical(actors, Vertical::Falling, held, data, world);
            let final_contacts = falling.collision_bits;
            merge_effects(&mut effects, falling);
            effects.collision_bits = final_contacts;
        } else {
            effects.work.push(Work::Gravity);
            world.gravity(&mut actors[0], self.gravity);
            self.observe_airborne_return(&actors[0], held, world);
            self.airborne_animation(&mut actors[0], held);
        }
        effects
    }
    fn observe_airborne_return(
        &self,
        actor: &ActorContact,
        held: u8,
        world: &mut dyn ContactWorld,
    ) {
        if self.action == Action::Swing {
            world.returned(ReturnOperation::AirModeSelected);
        } else if self.action == Action::ForcedJump
            || held & (Buttons::DOWN.0 | Buttons::B.0) == Buttons::DOWN.0 | Buttons::B.0
        {
            world.returned(ReturnOperation::PogoAnimation(actor.animation));
        } else if actor.animation != STRIKE_READY_ANIMATION {
            world.returned(ReturnOperation::AirAnimation(actor.animation));
        } else {
            world.returned(ReturnOperation::RememberedPoseDirection {
                held: held & 3,
                remembered: self.pose_direction,
            });
        }
    }
    pub fn airborne_animation(&self, actor: &mut ActorContact, held: u8) {
        if self.action == Action::Swing {
            return;
        }
        let pogo_buttons = Buttons::DOWN.0 | Buttons::B.0;
        if self.action == Action::ForcedJump || held & pogo_buttons == pogo_buttons {
            if actor.animation != POGO_ANIMATION {
                actor.animation = POGO_ANIMATION;
                actor.playback.freeze_at_start();
                actor.playback.reset_frame();
            }
        } else if actor.animation != STRIKE_READY_ANIMATION || held & 3 != self.pose_direction {
            actor.animation = AIR_ANIMATION;
            reset_animation(actor);
        }
    }
    pub fn landing(
        &mut self,
        actor: &mut ActorContact,
        input: PortInput,
        facts: CollisionFacts,
    ) -> Effects {
        let mut effects = Effects::default();
        if actor.animation == STRIKE_READY_ANIMATION && input.pressed.0 & Buttons::B.0 != 0 {
            actor.animation = SWING_ANIMATION;
            reset_animation(actor);
            self.action = Action::Swing;
            return effects;
        }
        effects.work.push(Work::HorizontalInput);
        if actor.motion_flags & crate::collision::DESCENDING_MOTION == 0
            || facts.bits & crate::collision::SOLID_CONTACT == 0
        {
            return effects;
        }
        if actor.animation == STRIKE_READY_ANIMATION || self.action == Action::Swing {
            self.action = Action::Standing;
            return effects;
        }
        if facts.actor_bits & 1 != 0
            && input.held.0 & Buttons::B.0 != 0
            && (self.action == Action::ForcedJump || input.held.0 & Buttons::DOWN.0 != 0)
            && (self.pogo_height_control == 0 || actor.position.screen_y < POGO_HEIGHT_LIMIT)
        {
            self.action = Action::ForcedJump;
            actor.vertical_velocity = POGO_SPEED;
            actor.animation = POGO_ANIMATION;
            {
                let animation_value = 2;
                actor.playback.set_restart_code(animation_value);
            };
            actor.playback.start_looping_at(1);
            effects.sounds.push(POGO_SOUND);
        } else {
            self.action = Action::Standing;
            actor.animation = LAND_ANIMATION;
            {
                let animation_value = 1;
                actor.playback.set_restart_code(animation_value);
            };
            actor.playback.start_looping_at(0);
            effects.sounds.push(LAND_SOUND);
        }
        actor.playback.restart_clock();
        effects
    }
    pub fn airborne_boundaries(
        &mut self,
        _actors: &mut ActorPool,
        input: &mut Controllers,
        _room: &mut RoomContext,
        _rooms: &[u8],
        data: &PlayerData,
    ) -> Effects {
        self.airborne_boundaries_observed(_actors, input, _room, _rooms, data, None)
    }
    fn airborne_boundaries_observed(
        &mut self,
        _actors: &mut ActorPool,
        input: &mut Controllers,
        _room: &mut RoomContext,
        _rooms: &[u8],
        data: &PlayerData,
        mut observer: Option<&mut dyn ContactWorld>,
    ) -> Effects {
        let mut effects = Effects {
            work: vec![Work::UpwardProbe],
            ..Effects::default()
        };
        if _actors[0].motion_flags & crate::collision::DESCENDING_MOTION == 0
            || input.ports[1].held.0 & Buttons::UP.0 != 0
        {
            return effects;
        }
        if let Some(world) = observer.as_mut() {
            world.returned(ReturnOperation::FallingScreenBoundary(
                _actors[0].position.screen_y,
            ));
        }
        if _actors[0].position.screen_y < BOTTOM_ROOM_Y {
            return effects;
        }
        let next_row = _room.row.wrapping_add(1);
        if let Some(world) = observer.as_mut() {
            world.returned(ReturnOperation::RoomRowBoundary {
                row: next_row,
                rows: _room.rows,
            });
        }
        if next_row == _room.rows {
            return effects;
        }
        effects.work.push(Work::DescendProbe);
        if _room.camera as u8 & 0x80 != 0 {
            _room.camera = _room.camera.wrapping_add(0x100);
            _room.camera_room = _room.camera_room.wrapping_add(1);
        }
        _room.camera &= 0xff00;
        _room.terrain_column = data.column_starts[usize::from(_room.previous_scroll & 1)];
        input.ports[0].held = Buttons::DOWN;
        merge_effects(
            &mut effects,
            self.room_transition_observed(_actors, Buttons::DOWN.0, _room, _rooms, data, observer)
                .1,
        );
        effects
    }
    pub fn upward_probe(
        &mut self,
        _actors: &mut ActorPool,
        held: u8,
        data: &PlayerData,
        _world: &mut dyn ContactWorld,
    ) -> Effects {
        if held & Buttons::UP.0 == 0 {
            return Effects::default();
        }
        let mut effects = Effects {
            work: vec![Work::Terrain {
                start: 0x0c,
                remaining: 3,
            }],
            ..Effects::default()
        };
        let facts = _world.probe(_actors, self, 0x0c, 3);
        effects.collision_bits = facts.bits;
        if facts.bits & crate::collision::SOLID_CONTACT == 0 {
            let a = &_actors[0];
            let matched = (0..4).rev().find(|&index| {
                (a.position.world_x as u8).wrapping_add(data.climb_offsets[index]) & 15 < 8
                    && facts.classes[index] == 2
            });
            let operation = if let Some(index) = matched {
                let pair = index & 2;
                if facts.classes[pair] == facts.classes[pair + 1] || index & 1 == 0 {
                    ReturnOperation::ClimbSamplePair {
                        first: facts.classes[pair],
                        second: facts.classes[pair + 1],
                    }
                } else {
                    ReturnOperation::ClimbOddScreenY(a.position.screen_y)
                }
            } else {
                let nibble = (a.position.world_x as u8).wrapping_add(data.climb_offsets[0]) & 15;
                if nibble >= 8 {
                    ReturnOperation::ClimbSampleOffset(nibble)
                } else {
                    ReturnOperation::ClimbTerrainClass(facts.classes[0])
                }
            };
            _world.returned(operation);
        }
        if let Some(direction) =
            self.climb_alignment(&mut _actors[0], facts.classes, facts.bits, data)
        {
            effects.scroll = Some(if direction == Horizontal::Positive {
                1
            } else {
                2
            });
        }
        effects
    }
    pub fn airborne_horizontal_exit(&self, actor: &mut ActorContact, bits: u8) {
        if actor.animation == STRIKE_READY_ANIMATION && bits & crate::collision::SOLID_CONTACT == 0
        {
            actor.animation = AIR_ANIMATION;
            reset_animation(actor);
        }
    }
    pub fn climb_alignment(
        &mut self,
        actor: &mut ActorContact,
        results: [u8; 4],
        bits: u8,
        data: &PlayerData,
    ) -> Option<Horizontal> {
        if bits & crate::collision::SOLID_CONTACT != 0 {
            return None;
        }
        for index in (0..4).rev() {
            if (actor.position.world_x as u8).wrapping_add(data.climb_offsets[index]) & 15 >= 8
                || results[index] != 2
            {
                continue;
            }
            let right = index >= 2;
            actor.position.world_x = if right {
                (actor.position.world_x & !15).wrapping_add(16)
            } else {
                actor.position.world_x & !15
            };
            let pair = index & 2;
            if results[pair] != results[pair + 1] {
                actor.position.screen_y = if index & 1 == 0 {
                    actor.position.screen_y & 0xf0 | 8
                } else {
                    (actor.position.screen_y & 0xf0).wrapping_sub(16) | 8
                };
            }
            actor.animation = CLIMB_ANIMATION;
            reset_animation(actor);
            self.action = Action::Climbing;
            return Some(if right {
                Horizontal::Positive
            } else {
                Horizontal::Negative
            });
        }
        None
    }
    pub fn forced_jump(&mut self, input: &mut Controllers) -> Effects {
        input.ports[0].held.0 |= (input.ports[0].held.0 & Buttons::B.0) << 1;
        if input.ports[0].held.0 & Buttons::A.0 == 0 {
            self.action = Action::Airborne;
        }
        Effects {
            work: vec![Work::VerticalRise],
            ..Effects::default()
        }
    }
    pub fn climbing(
        &mut self,
        actor: &mut ActorContact,
        input: PortInput,
        last_class: u8,
    ) -> Effects {
        self.animation_delay = ANIMATION_DELAY;
        let mut effects = Effects::default();
        let direction = input.held.0 & (Buttons::UP.0 | Buttons::DOWN.0);
        if direction == 0 {
            if input.pressed.0 & Buttons::A.0 != 0 {
                self.action = Action::Standing;
                actor.vertical_velocity = 0;
            } else {
                actor.playback.freeze();
            }
            return effects;
        }
        actor.playback.resume();
        if actor.playback.at_running_tick(0) {
            effects.sounds.push(CLIMB_SOUND);
        }
        actor.vertical_velocity = actor.horizontal_magnitude as i16;
        let up = direction == Buttons::UP.0;
        effects.work.push(Work::Terrain {
            start: if up { 0x14 } else { 0x10 },
            remaining: 3,
        });
        if last_class != 2 {
            self.action = Action::Standing;
            actor.vertical_velocity = 0;
        } else if (up && actor.position.screen_y < TOP_CLIMB_Y)
            || (!up && actor.position.screen_y >= BOTTOM_ROOM_Y)
        {
            effects.work.push(Work::VerticalTransition);
        }
        effects
    }
    pub fn room_transition(
        &mut self,
        _actors: &mut ActorPool,
        held: u8,
        _room: &mut RoomContext,
        _rooms: &[u8],
        data: &PlayerData,
    ) -> (RoomResult, Effects) {
        self.room_transition_observed(_actors, held, _room, _rooms, data, None)
    }
    fn room_transition_observed(
        &mut self,
        _actors: &mut ActorPool,
        held: u8,
        _room: &mut RoomContext,
        _rooms: &[u8],
        data: &PlayerData,
        mut observer: Option<&mut dyn ContactWorld>,
    ) -> (RoomResult, Effects) {
        let mut effects = Effects::default();
        if _room.late_route != 0
            && let Some(world) = observer.as_mut()
        {
            world.returned(ReturnOperation::LateRouteScreenY(
                _actors[0].position.screen_y,
            ));
        }
        if _room.late_route != 0 && _actors[0].position.screen_y >= LATE_ROUTE_Y {
            self.action = Action::Dialogue;
            effects.message = Some(LATE_ROUTE_MESSAGE);
            return (RoomResult::Message, effects);
        }
        if _room.camera & 0xff != 0 || _room.camera >> 8 & 1 == 0 {
            return (RoomResult::Rejected, effects);
        }
        let index = usize::from((held >> 2) & 2);
        let (stride, carry) =
            (_room.columns ^ data.transition[index]).overflowing_add(data.transition[index + 1]);
        let camera = stride
            .wrapping_add(_room.camera_room)
            .wrapping_add(u8::from(carry));
        let terrain = stride.wrapping_add(_room.terrain_room);
        let row = _room.row.wrapping_add(data.transition[index + 5]);
        let room_entry = _rooms.get(usize::from(camera)).copied().unwrap_or(u8::MAX);
        if let Some(world) = observer.as_mut() {
            world.returned(ReturnOperation::RoomEntry(room_entry));
        }
        if room_entry == u8::MAX {
            return (RoomResult::Rejected, effects);
        }
        if let Some(world) = observer.as_mut() {
            world.returned(ReturnOperation::RoomRowBoundary {
                row,
                rows: _room.rows,
            });
        }
        if row >= _room.rows {
            return (RoomResult::Rejected, effects);
        }
        _room.row = row;
        _room.camera_room = camera;
        _room.terrain_room = terrain;
        // X survives from the direction selector: upward writes actor slot two.
        _actors[index].position.screen_y = data.transition[index + 4];
        _room.special_spawn = 0;
        _actors[0].render &= !3;
        effects.transition = true;
        effects.work.push(Work::VerticalTransition);
        (RoomResult::Committed, effects)
    }
    pub fn swing(
        &mut self,
        actor: &mut ActorContact,
        held: u8,
        contact_handler: Option<u8>,
    ) -> Effects {
        let mut effects = Effects {
            work: vec![Work::VerticalRise],
            ..Effects::default()
        };
        if actor.animation == BLOCKED_SWING_ANIMATION {
            if actor.playback.at_running_tick(0) && actor.playback.at_initial_frame(0) {
                {
                    let actor_value = actor.local_state_code().wrapping_sub(1);
                    actor.set_local_state_code(actor_value);
                };
                if actor.local_state_code() == 0 {
                    self.action = Action::Standing;
                    self.animation_delay = ANIMATION_DELAY;
                }
            }
            return effects;
        }
        if actor.playback.at_initial_frame(6) {
            self.action = Action::Standing;
            if held & self.pose_direction != 0 {
                actor.animation = STRIKE_READY_ANIMATION;
                reset_animation(actor);
                return effects;
            }
            self.animation_delay = ANIMATION_DELAY;
        }
        if actor.playback.at_running_tick(0) && actor.playback.at_initial_frame(2) {
            effects.sounds.push(SWING_SOUND);
            effects.work.push(Work::ActorScan);
            if !matches!(contact_handler, Some(8 | 0x15 | 0x42)) {
                reset_animation(actor);
                {
                    let actor_value = BLOCKED_CYCLES;
                    actor.set_local_state_code(actor_value);
                };
                actor.animation = BLOCKED_SWING_ANIMATION;
                effects.sounds.push(BLOCKED_SOUND);
            }
        }
        effects
    }
    pub fn protected(
        &mut self,
        actor: &mut ActorContact,
        health: &mut Health,
        rule: TitleRule,
    ) -> Effects {
        if self.protection >= RECOIL_THRESHOLD {
            return Effects::default();
        }
        if self.protection == 0 {
            self.action = Action::Standing;
            actor.horizontal_magnitude = NORMAL_SPEED;
            health.damage(rule);
            let mut effects = Effects {
                work: vec![Work::DamageHealth],
                ..Effects::default()
            };
            if health.exhausted() {
                self.action = Action::Death;
                {
                    let actor_value = actor.state_code() & (0xf0);
                    actor.set_state_code(actor_value);
                };
                effects.work.push(Work::DeathTrajectory);
            }
            return effects;
        }
        Effects {
            // EAD9: the vertical helper can choose an airborne/pogo pose;
            // the protected continuation overrides it before horizontal recoil.
            work: vec![Work::VerticalFall, Work::ProtectedContinuation],
            ..Effects::default()
        }
    }
    pub fn damage(
        &mut self,
        actor: &mut ActorContact,
        health: &mut Health,
        rule: TitleRule,
    ) -> Effects {
        let mut effects = Effects::default();
        if self.protection == 0 {
            effects.sounds.push(DAMAGE_SOUND);
            actor.vertical_velocity = DAMAGE_SPEED;
            reset_animation(actor);
            actor.horizontal_magnitude &= 0xff00;
            self.protection = DAMAGE_PROTECTION;
            health.damage(rule);
            effects.work.push(Work::DamageHealth);
            if health.exhausted() {
                self.action = Action::Death;
                {
                    let actor_value = actor.state_code() & (0xf0);
                    actor.set_state_code(actor_value);
                };
                effects.work.push(Work::DeathTrajectory);
                return effects;
            }
        }
        effects.work.push(if actor.vertical_velocity < 0 {
            Work::VerticalFall
        } else {
            Work::VerticalRise
        });
        effects.work.push(Work::DamageContinuation);
        effects
    }
    pub fn pose(&mut self, actor: &mut ActorContact, input: PortInput) -> Effects {
        if actor.animation != POSE_ANIMATION {
            actor.animation = POSE_ANIMATION;
            reset_animation(actor);
            actor.position.screen_y = actor.position.screen_y.wrapping_add(4);
        }
        let mut effects = Effects::default();
        if input.pressed.0 & Buttons::A.0 != 0 {
            actor.vertical_velocity = self.jump_velocity;
            self.action = Action::Airborne;
            effects.work.push(Work::VerticalRise);
        }
        effects
    }
    /// Shared EBB4 entry used by falling, dialogue and hidden-player departure.
    /// Applies action/queue/activity/alternation clears immediately; returns the
    /// stage-plus-variant reload request for canonical B13, without vertical motion.
    pub fn commit_life_loss(
        &mut self,
        actor: &mut ActorContact,
        stage: u8,
        variant: u8,
        health: &mut Health,
    ) -> Effects {
        let (base, carry) = stage.overflowing_add(LIFE_LOSS_REQUEST);
        self.action = Action::Standing;
        self.queued = None;
        {
            let actor_value = 0;
            actor.set_state_code(actor_value);
        };
        health.alternating = 0;
        Effects {
            reload: Some(base.wrapping_add(variant).wrapping_add(u8::from(carry))),
            ..Effects::default()
        }
    }
    pub fn fall_out(
        &mut self,
        actor: &mut ActorContact,
        stage: u8,
        variant: u8,
        health: &mut Health,
    ) -> Effects {
        let mut effects = Effects {
            work: vec![Work::VerticalFall],
            ..Effects::default()
        };
        if actor.position.screen_y >= LIFE_LOSS_Y {
            merge_effects(
                &mut effects,
                self.commit_life_loss(actor, stage, variant, health),
            );
        } else {
            effects.work.push(Work::HorizontalInput);
        }
        effects
    }
    pub fn wait_a(&mut self, actor: &mut ActorContact, input: PortInput) {
        if input.pressed.0 & Buttons::A.0 != 0 {
            self.action = Action::Standing;
        }
        actor.playback.restart_clock();
    }
    pub fn carried(
        &mut self,
        actor: &mut ActorContact,
        input: PortInput,
        facts: CollisionFacts,
    ) -> Effects {
        {
            let actor_value = 0xa0;
            actor.set_state_code(actor_value);
        };
        let mut effects = Effects {
            scroll: Some(2),
            spawn: Some(2),
            ..Effects::default()
        };
        if input.held.0 & Buttons::DOWN.0 != 0 {
            {
                let actor_value = 0;
                actor.set_state_code(actor_value);
            };
            return effects;
        }
        if self.action == Action::CarriedJumpable && input.pressed == Buttons::A {
            actor.vertical_velocity = self.jump_velocity;
            self.action = Action::Airborne;
            effects.work.push(Work::VerticalRise);
            return effects;
        }
        effects.work.push(Work::Terrain {
            start: 3,
            remaining: 2,
        });
        if facts.bits & crate::collision::SOLID_CONTACT != 0 {
            actor.position.screen_y = actor.position.screen_y.wrapping_add(8);
            self.action = Action::Damage;
            // The complete call consumes this request immediately through StartPlayerDamageArc.
            effects.work.push(Work::DamageHealth);
        }
        effects
    }
    pub fn launch(&mut self, _actors: &mut ActorPool, _world: &mut dyn ContactWorld) -> Effects {
        let actor = &mut _actors[0];
        let mut effects = Effects {
            scroll: Some(1),
            spawn: Some(1),
            ..Effects::default()
        };
        if actor.motion_flags != 0 {
            {
                let actor_value = 0xe0;
                actor.set_state_code(actor_value);
            };
            effects.work.extend([Work::Gravity, Work::IntegrateY]);
            _world.gravity(actor, self.gravity);
            _world.integrate_y(actor);
        } else if actor.position.screen_y >= LAUNCH_CEILING {
            effects.work.extend([Work::IntegrateY, Work::Gravity]);
            _world.integrate_y(actor);
            _world.gravity(actor, self.gravity);
        }
        effects.work.extend([
            Work::IntegrateX,
            Work::Terrain {
                start: 0,
                remaining: 2,
            },
        ]);
        let facts = _world.positive_with_collision(_actors, self);
        let actor = &mut _actors[0];
        if actor.motion_flags == 0 {
            _world.returned(ReturnOperation::LaunchWorldPage(
                (actor.position.world_x >> 8) as u8,
            ));
        }
        if actor.motion_flags == 0 && actor.position.world_x >> 8 == u16::from(LAUNCH_PAGE) {
            actor.motion_flags = 1;
            actor.vertical_velocity = 0;
        }
        if facts.bits & crate::collision::SOLID_CONTACT != 0 {
            self.action = Action::Standing;
            actor.horizontal_magnitude = NORMAL_SPEED;
        }
        actor.animation = AIR_ANIMATION;
        reset_animation(actor);
        effects.collision_bits = facts.bits;
        effects
    }
    pub fn death(
        &mut self,
        _actors: &mut ActorPool,
        _world: &mut dyn ContactWorld,
        stage: u8,
        variant: u8,
        health: &mut Health,
    ) -> Effects {
        let actor = &mut _actors[0];
        if actor.state_code() & 15 == 0 {
            actor.motion_flags = 0;
            actor.playback.reset_frame();
            actor.animation = HURT_ANIMATION;
            actor.direction = 1;
            actor.vertical_velocity = DEATH_SPEED;
            actor.horizontal_magnitude = DEATH_DRIFT;
            {
                let actor_value = actor.state_code().wrapping_add(1);
                actor.set_state_code(actor_value);
            };
        }
        let mut effects = Effects::default();
        if actor.motion_flags == 0 {
            effects.work.extend([Work::IntegrateY, Work::Gravity]);
            _world.integrate_y(actor);
            _world.gravity(actor, self.gravity);
            if actor.vertical_velocity < 0 {
                actor.motion_flags = actor.motion_flags.wrapping_add(1);
                actor.vertical_velocity = 0;
                return effects;
            }
        } else {
            effects.work.extend([Work::Gravity, Work::IntegrateY]);
            _world.gravity(actor, self.gravity);
            _world.integrate_y(actor);
            if actor.position.screen_y >= BOTTOM_ROOM_Y {
                merge_effects(
                    &mut effects,
                    self.commit_life_loss(actor, stage, variant, health),
                );
                return effects;
            }
        }
        effects.work.push(Work::IntegrateX);
        _world.integrate_x(actor, Horizontal::Positive);
        effects
    }
    pub fn hold(&self, actor: &mut ActorContact) {
        actor.playback.restart_clock();
    }
}
#[cfg(test)]
mod contracts;

fn reset_animation(actor: &mut ActorContact) {
    actor.playback.reset_frame();
    actor.playback.restart_clock();
}

fn merge_effects(into: &mut Effects, other: Effects) {
    into.work.extend(other.work);
    into.sounds.extend(other.sounds);
    into.stage_sounds.extend(other.stage_sounds);
    if other.reload.is_some() {
        into.reload = other.reload;
    }
    if other.message.is_some() {
        into.message = other.message;
    }
    if other.scroll.is_some() {
        into.scroll = other.scroll;
    }
    if other.spawn.is_some() {
        into.spawn = other.spawn;
    }
    into.transition |= other.transition;
    into.collision_bits |= other.collision_bits;
}

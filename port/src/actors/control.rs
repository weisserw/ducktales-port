//! Canonical actor behavior control. Converted families retain native behavior;
//! projection methods support remaining source-contract callers without a shadow
//! byte owner. Handler changes preserve every partial-write-visible value.
use crate::scenes::{SceneActorState, SceneHandler};
pub const AMBUSH_HANDLER: u8 = 0x19;
const PHASE_MASK: u8 = 0x0f;
const FLAGS_MASK: u8 = !PHASE_MASK;
const PHASE_CARRY: u8 = 0x10;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AmbushPhase {
    WaitingForPlayer,
    Opening,
    Tracking,
    Closing,
    Attacking,
    /// Unsupported original dispatcher inputs remain observable but do no work.
    Unrecognized(u8),
}
impl AmbushPhase {
    fn decode(code: u8) -> Self {
        match code {
            0 => Self::WaitingForPlayer,
            1 => Self::Opening,
            2 => Self::Tracking,
            3 => Self::Closing,
            4 => Self::Attacking,
            code => Self::Unrecognized(code),
        }
    }
    fn code(self) -> u8 {
        match self {
            Self::WaitingForPlayer => 0,
            Self::Opening => 1,
            Self::Tracking => 2,
            Self::Closing => 3,
            Self::Attacking => 4,
            Self::Unrecognized(code) => code,
        }
    }
    pub fn counts_motion_delay(self) -> bool {
        match self {
            Self::Opening | Self::Closing => true,
            Self::Unrecognized(code) => code & 1 != 0,
            _ => false,
        }
    }
    fn next(self) -> Self {
        match self {
            Self::WaitingForPlayer => Self::Opening,
            Self::Opening => Self::Tracking,
            Self::Tracking => Self::Closing,
            Self::Closing => Self::Attacking,
            Self::Attacking => Self::Unrecognized(5),
            Self::Unrecognized(code) => Self::decode(code.wrapping_add(1) & PHASE_MASK),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AmbushBehavior {
    pub phase: AmbushPhase,
    pub delay_remaining: u8,
    pub preset_step: u8,
}
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum ActorControl {
    /// Existing families retain their source-contract fields until converted.
    Legacy {
        state: u8,
        local_state: u8,
        local_counter: u8,
    },
    Ambush {
        flags: u8,
        behavior: AmbushBehavior,
    },
    Scene {
        flags: u8,
        behavior: SceneActorState,
        preserved_counter: u8,
    },
}
impl Default for ActorControl {
    fn default() -> Self {
        Self::Legacy {
            state: 0,
            local_state: 0,
            local_counter: 0,
        }
    }
}
// Representation selection itself is not a gameplay change or journal event.
impl PartialEq for ActorControl {
    fn eq(&self, other: &Self) -> bool {
        self.codes() == other.codes()
    }
}
impl Eq for ActorControl {}
impl ActorControl {
    pub fn from_codes(state: u8, local_state: u8, local_counter: u8, handler: u8) -> Self {
        if handler == AMBUSH_HANDLER {
            Self::Ambush {
                flags: state & FLAGS_MASK,
                behavior: AmbushBehavior {
                    phase: AmbushPhase::decode(state & PHASE_MASK),
                    delay_remaining: local_state,
                    preset_step: local_counter,
                },
            }
        } else {
            Self::Legacy {
                state,
                local_state,
                local_counter,
            }
        }
    }
    pub fn codes(self) -> (u8, u8, u8) {
        match self {
            Self::Legacy {
                state,
                local_state,
                local_counter,
            } => (state, local_state, local_counter),
            Self::Ambush { flags, behavior } => (
                flags | behavior.phase.code(),
                behavior.delay_remaining,
                behavior.preset_step,
            ),
            Self::Scene {
                flags,
                behavior,
                preserved_counter,
            } => (
                flags | behavior.progress_code(),
                behavior.counter_code(),
                preserved_counter,
            ),
        }
    }
    pub fn select_handler(&mut self, handler: u8) {
        if matches!(self, Self::Scene { .. }) {
            self.enter_scene(handler);
            return;
        }
        // Retain a converted family's native owner unchanged on normal calls.
        // Only a real legacy/native handler boundary requires a projection.
        if matches!((&*self, handler), (Self::Ambush { .. }, AMBUSH_HANDLER))
            || matches!(&*self, Self::Legacy { .. }) && handler != AMBUSH_HANDLER
        {
            return;
        }
        let (state, local_state, local_counter) = self.codes();
        *self = Self::from_codes(state, local_state, local_counter, handler);
    }
    pub fn set_state_code(&mut self, value: u8) {
        match self {
            Self::Legacy { state, .. } => *state = value,
            Self::Ambush { flags, behavior } => {
                *flags = value & FLAGS_MASK;
                behavior.phase = AmbushPhase::decode(value & PHASE_MASK);
            }
            Self::Scene {
                flags, behavior, ..
            } => {
                *flags = value & FLAGS_MASK;
                behavior.set_progress_code(value & PHASE_MASK);
            }
        }
    }
    pub fn set_local_state_code(&mut self, value: u8) {
        match self {
            Self::Legacy { local_state, .. } => *local_state = value,
            Self::Ambush { behavior, .. } => behavior.delay_remaining = value,
            Self::Scene { behavior, .. } => behavior.set_counter_code(value),
        }
    }
    pub fn set_local_counter_code(&mut self, value: u8) {
        match self {
            Self::Legacy { local_counter, .. } => *local_counter = value,
            Self::Ambush { behavior, .. } => behavior.preset_step = value,
            Self::Scene {
                preserved_counter, ..
            } => *preserved_counter = value,
        }
    }
    pub fn ambush_mut(&mut self) -> &mut AmbushBehavior {
        if !matches!(self, Self::Ambush { .. }) {
            let (state, local_state, local_counter) = self.codes();
            *self = Self::from_codes(state, local_state, local_counter, AMBUSH_HANDLER);
        }
        match self {
            Self::Ambush { behavior, .. } => behavior,
            _ => unreachable!("converted ambush owner"),
        }
    }
    /// Preserve the source's same-call successor even if an immediate callback
    /// removed/reclassified the actor before the continuation executes.
    pub fn advance_ambush(&mut self) {
        self.ambush_mut();
        if let Self::Ambush { flags, behavior } = self {
            if behavior.phase.code() == PHASE_MASK {
                *flags = flags.wrapping_add(PHASE_CARRY);
            }
            behavior.phase = behavior.phase.next();
        }
    }
    /// An ordinary family and a scene may share a handler code. Their entered
    /// owner selects the representation explicitly; a handler number alone is
    /// insufficient to infer scene behavior.
    pub fn select_family(&mut self, handler: u8) {
        if matches!(self, Self::Scene { .. }) {
            let (state, local_state, local_counter) = self.codes();
            *self = Self::from_codes(state, local_state, local_counter, handler);
        } else {
            self.select_handler(handler);
        }
    }
    pub fn enter_scene(&mut self, handler: u8) {
        if let Self::Scene { behavior, .. } = self
            && behavior.handler_code() == handler
        {
            return;
        }
        let (state, count, preserved_counter) = self.codes();
        *self = Self::Scene {
            flags: state & FLAGS_MASK,
            behavior: SceneActorState::from_codes(handler, state & PHASE_MASK, count),
            preserved_counter,
        };
    }
    pub fn scene_mut(&mut self, handler: u8) -> &mut SceneActorState {
        self.enter_scene(handler);
        match self {
            Self::Scene { behavior, .. } => behavior,
            _ => unreachable!("entered scene owner"),
        }
    }
    pub fn advance_scene_bounce(&mut self, forward: bool) {
        self.enter_scene(SceneHandler::Bounce as u8);
        if let Self::Scene {
            flags, behavior, ..
        } = self
        {
            let bounce = behavior.bounce_mut();
            let (progress, carry) = bounce.progress.advance(forward);
            bounce.progress = progress;
            if carry > 0 {
                *flags = flags.wrapping_add(PHASE_CARRY);
            } else if carry < 0 {
                *flags = flags.wrapping_sub(PHASE_CARRY);
            }
        }
    }
    pub fn start_scene_move(&mut self) {
        let preserved_counter = self.codes().2;
        *self = Self::Scene {
            flags: crate::actors::ACTIVE,
            behavior: SceneActorState::from_codes(SceneHandler::MoveToMark as u8, 0, 0),
            preserved_counter,
        };
    }
}

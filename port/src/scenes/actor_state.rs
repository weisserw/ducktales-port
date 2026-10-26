//! Scene actor behavior retained natively in the canonical ActorControl owner.
//! Code projections preserve shared legacy callers without retaining a raw twin.
use super::SceneHandler;
use std::num::NonZeroU8;
const PROGRESS_MASK: u8 = 0x0f;
const FALLING_FLAG: u8 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum EntryProgress {
    Entering,
    /// Noncanonical nonzero progress is preserved for partial source writes;
    /// normal native entry selects the first active progress value.
    Active(NonZeroU8),
}
impl EntryProgress {
    fn decode(code: u8) -> Self {
        NonZeroU8::new(code).map_or(Self::Entering, Self::Active)
    }
    fn code(self) -> u8 {
        match self {
            Self::Entering => 0,
            Self::Active(code) => code.get(),
        }
    }
    pub fn is_entering(self) -> bool {
        matches!(self, Self::Entering)
    }
    pub fn activate(&mut self) {
        *self = Self::Active(NonZeroU8::MIN);
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum BounceProgress {
    Entering,
    Rising,
    Falling,
    Compatibility(u8),
}
impl BounceProgress {
    fn decode(code: u8) -> Self {
        match code {
            0 => Self::Entering,
            1 => Self::Rising,
            2 => Self::Falling,
            other => Self::Compatibility(other),
        }
    }
    fn code(self) -> u8 {
        match self {
            Self::Entering => 0,
            Self::Rising => 1,
            Self::Falling => 2,
            Self::Compatibility(code) => code,
        }
    }
    pub fn is_entering(self) -> bool {
        matches!(self, Self::Entering)
    }
    pub fn descending(self) -> bool {
        match self {
            Self::Falling => true,
            Self::Compatibility(code) => code & FALLING_FLAG != 0,
            _ => false,
        }
    }
    /// Return a native phase and any overflow into the preserved actor flags.
    pub fn advance(self, forward: bool) -> (Self, i8) {
        if forward {
            match self {
                Self::Entering => (Self::Rising, 0),
                Self::Rising => (Self::Falling, 0),
                Self::Falling => (Self::Compatibility(3), 0),
                Self::Compatibility(PROGRESS_MASK) => (Self::Entering, 1),
                Self::Compatibility(code) => (Self::decode(code + 1), 0),
            }
        } else {
            match self {
                Self::Entering => (Self::Compatibility(PROGRESS_MASK), -1),
                Self::Rising => (Self::Entering, 0),
                Self::Falling => (Self::Rising, 0),
                Self::Compatibility(code) => (Self::decode(code - 1), 0),
            }
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BounceState {
    pub progress: BounceProgress,
    pub bounces_remaining: u8,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WalkState {
    pub progress: EntryProgress,
    pub pause_remaining: u8,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DelayState {
    pub progress: EntryProgress,
    pub ticks_remaining: u8,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MoveState {
    pub progress: EntryProgress,
    pub wait_remaining: u8,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SceneActorState {
    Bounce(BounceState),
    WalkIn(WalkState),
    Delay(DelayState),
    MoveToMark(MoveState),
    Settle {
        progress: u8,
        retained_count: u8,
    },
    Settled {
        progress: u8,
        retained_count: u8,
    },
    Unsupported {
        handler: u8,
        progress: u8,
        retained_count: u8,
    },
}
impl SceneActorState {
    pub fn from_codes(handler: u8, progress: u8, count: u8) -> Self {
        let progress = progress & PROGRESS_MASK;
        match handler {
            0 => Self::Bounce(BounceState {
                progress: BounceProgress::decode(progress),
                bounces_remaining: count,
            }),
            1 => Self::WalkIn(WalkState {
                progress: EntryProgress::decode(progress),
                pause_remaining: count,
            }),
            2 => Self::Delay(DelayState {
                progress: EntryProgress::decode(progress),
                ticks_remaining: count,
            }),
            3 => Self::MoveToMark(MoveState {
                progress: EntryProgress::decode(progress),
                wait_remaining: count,
            }),
            4 => Self::Settle {
                progress,
                retained_count: count,
            },
            5 => Self::Settled {
                progress,
                retained_count: count,
            },
            handler => Self::Unsupported {
                handler,
                progress,
                retained_count: count,
            },
        }
    }
    pub fn kind(self) -> Option<SceneHandler> {
        Some(match self {
            Self::Bounce(_) => SceneHandler::Bounce,
            Self::WalkIn(_) => SceneHandler::WalkIn,
            Self::Delay(_) => SceneHandler::Delay,
            Self::MoveToMark(_) => SceneHandler::MoveToMark,
            Self::Settle { .. } => SceneHandler::Settle,
            Self::Settled { .. } => SceneHandler::Settled,
            Self::Unsupported { .. } => return None,
        })
    }
    pub fn handler_code(self) -> u8 {
        match self {
            Self::Unsupported { handler, .. } => handler,
            _ => self.kind().expect("known scene behavior") as u8,
        }
    }
    pub fn progress_code(self) -> u8 {
        match self {
            Self::Bounce(state) => state.progress.code(),
            Self::WalkIn(state) => state.progress.code(),
            Self::Delay(state) => state.progress.code(),
            Self::MoveToMark(state) => state.progress.code(),
            Self::Settle { progress, .. }
            | Self::Settled { progress, .. }
            | Self::Unsupported { progress, .. } => progress,
        }
    }
    pub fn counter_code(self) -> u8 {
        match self {
            Self::Bounce(state) => state.bounces_remaining,
            Self::WalkIn(state) => state.pause_remaining,
            Self::Delay(state) => state.ticks_remaining,
            Self::MoveToMark(state) => state.wait_remaining,
            Self::Settle { retained_count, .. }
            | Self::Settled { retained_count, .. }
            | Self::Unsupported { retained_count, .. } => retained_count,
        }
    }
    pub fn set_progress_code(&mut self, progress: u8) {
        *self = Self::from_codes(self.handler_code(), progress, self.counter_code());
    }
    pub fn set_counter_code(&mut self, value: u8) {
        match self {
            Self::Bounce(state) => state.bounces_remaining = value,
            Self::WalkIn(state) => state.pause_remaining = value,
            Self::Delay(state) => state.ticks_remaining = value,
            Self::MoveToMark(state) => state.wait_remaining = value,
            Self::Settle { retained_count, .. }
            | Self::Settled { retained_count, .. }
            | Self::Unsupported { retained_count, .. } => *retained_count = value,
        }
    }
    pub fn bounce_mut(&mut self) -> &mut BounceState {
        match self {
            Self::Bounce(state) => state,
            _ => unreachable!("entered bounce owner"),
        }
    }
    pub fn walk_mut(&mut self) -> &mut WalkState {
        match self {
            Self::WalkIn(state) => state,
            _ => unreachable!("entered walk owner"),
        }
    }
    pub fn delay_mut(&mut self) -> &mut DelayState {
        match self {
            Self::Delay(state) => state,
            _ => unreachable!("entered delay owner"),
        }
    }
    pub fn movement_mut(&mut self) -> &mut MoveState {
        match self {
            Self::MoveToMark(state) => state,
            _ => unreachable!("entered movement owner"),
        }
    }
}

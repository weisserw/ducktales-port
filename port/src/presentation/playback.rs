//! Canonical animation time and frame state. Original packed codes are accepted
//! only at compatibility boundaries; advancement works with explicit modes.
const VALUE_MASK: u8 = 0x7f;
const MODE_BIT: u8 = 0x80;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Clock {
    Running(u8),
    Frozen(u8),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LoopOrigin {
    Beginning,
    Restart,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FrameCursor {
    index: u8,
    origin: LoopOrigin,
}
impl FrameCursor {
    fn decode(code: u8) -> Self {
        Self {
            index: code & VALUE_MASK,
            origin: if code & MODE_BIT == 0 {
                LoopOrigin::Beginning
            } else {
                LoopOrigin::Restart
            },
        }
    }
    fn code(self) -> u8 {
        self.index
            | if self.origin == LoopOrigin::Restart {
                MODE_BIT
            } else {
                0
            }
    }
    fn advance(&mut self) {
        if self.index == VALUE_MASK {
            self.index = 0;
            self.origin = match self.origin {
                LoopOrigin::Beginning => LoopOrigin::Restart,
                LoopOrigin::Restart => LoopOrigin::Beginning,
            };
        } else {
            self.index += 1;
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AnimationPlayback {
    clock: Clock,
    frame: FrameCursor,
    // Restart's origin metadata is not used for playback, but an original
    // partial write/debug observation can distinguish all 256 restart codes.
    restart: FrameCursor,
}
impl Default for AnimationPlayback {
    fn default() -> Self {
        Self::from_codes(0, 0, 0)
    }
}
impl AnimationPlayback {
    pub fn from_codes(tick: u8, frame: u8, restart: u8) -> Self {
        Self {
            clock: Self::decode_clock(tick),
            frame: FrameCursor::decode(frame),
            restart: FrameCursor::decode(restart),
        }
    }
    fn decode_clock(code: u8) -> Clock {
        if code & MODE_BIT == 0 {
            Clock::Running(code)
        } else {
            Clock::Frozen(code & VALUE_MASK)
        }
    }
    pub fn tick_code(self) -> u8 {
        match self.clock {
            Clock::Running(tick) => tick,
            Clock::Frozen(tick) => tick | MODE_BIT,
        }
    }
    pub fn frame_code(self) -> u8 {
        self.frame.code()
    }
    pub fn restart_code(self) -> u8 {
        self.restart.code()
    }
    pub fn set_tick_code(&mut self, code: u8) {
        self.clock = Self::decode_clock(code);
    }
    pub fn set_frame_code(&mut self, code: u8) {
        self.frame = FrameCursor::decode(code);
    }
    pub fn set_restart_code(&mut self, code: u8) {
        self.restart = FrameCursor::decode(code);
    }
    pub fn restart_clock(&mut self) {
        self.clock = Clock::Running(0);
    }
    pub fn freeze_at_start(&mut self) {
        self.clock = Clock::Frozen(0);
    }
    pub fn reset_frame(&mut self) {
        self.frame = FrameCursor::decode(0);
    }
    pub fn reset(&mut self) {
        self.restart_clock();
        self.reset_frame();
    }
    pub fn frame_index(self) -> usize {
        usize::from(self.frame.index)
    }
    /// Gameplay events compare the initial pass, so a looping frame with the
    /// same visible index does not fire those events again.
    pub fn at_initial_frame(self, index: u8) -> bool {
        self.frame.origin == LoopOrigin::Beginning && self.frame.index == index
    }
    pub fn at_running_tick(self, elapsed: u8) -> bool {
        self.clock == Clock::Running(elapsed)
    }
    pub fn at_start(self) -> bool {
        self.at_initial_frame(0) && self.at_running_tick(0)
    }
    pub fn freeze(&mut self) {
        if let Clock::Running(elapsed) = self.clock {
            self.clock = Clock::Frozen(elapsed);
        }
    }
    pub fn resume(&mut self) {
        if let Clock::Frozen(elapsed) = self.clock {
            self.clock = Clock::Running(elapsed);
        }
    }
    pub fn advance_frame(&mut self) {
        self.frame.advance();
    }
    pub fn start_looping_at(&mut self, index: u8) {
        assert!(index <= VALUE_MASK);
        self.frame = FrameCursor {
            index,
            origin: LoopOrigin::Restart,
        };
    }
    /// Source813E..8194 compares the old tick/frame, then decides looping from
    /// the advanced frame's mode. Destruction returns before a loop reset.
    /// Return true when the owning actor must be destroyed.
    pub fn advance(&mut self, last_tick: u8, last_frame: u8, destroy_at_end: bool) -> bool {
        let Clock::Running(elapsed) = self.clock else {
            return false;
        };
        self.clock = if elapsed == VALUE_MASK {
            Clock::Frozen(0)
        } else {
            Clock::Running(elapsed + 1)
        };
        if elapsed != last_tick {
            return false;
        }
        self.restart_clock();
        let old_frame = self.frame.index;
        self.frame.advance();
        if old_frame != last_frame {
            return false;
        }
        if destroy_at_end {
            return true;
        }
        self.frame.index = match self.frame.origin {
            LoopOrigin::Beginning => 0,
            LoopOrigin::Restart => self.restart.index,
        };
        false
    }
}

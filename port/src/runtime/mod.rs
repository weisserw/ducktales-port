//! Deterministic foreground scheduling, sampled controls and source random state.
//!
//! Atomic interrupt requests and cooperative continuations preserve source ordering.
//! No CPU stack, RAM map or mapper execution is part of the native model.
pub const TASK_SLOTS: usize = 4;
pub const CONTROLLER_PORTS: usize = 2;
pub const TITLE_RANDOM_SEED: u8 = 0x88;
pub const RESET_CONTROL: u8 = 0x90;
pub const RESET_BACKGROUND_TILE: u8 = 0x24;
pub const HIDDEN_SPRITE_Y: u8 = 0xf8;
pub const SPLIT_PLAYER_Y_THRESHOLD: u8 = 0x44;

/// Original eight logical button encodings after eight serial samples.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Buttons(pub u8);
impl Buttons {
    pub const NONE: Self = Self(0);
    pub const RIGHT: Self = Self(0x01);
    pub const LEFT: Self = Self(0x02);
    pub const DOWN: Self = Self(0x04);
    pub const UP: Self = Self(0x08);
    pub const START: Self = Self(0x10);
    pub const SELECT: Self = Self(0x20);
    pub const B: Self = Self(0x40);
    pub const A: Self = Self(0x80);
}
impl std::ops::BitOr for Buttons {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
/// The two serial data lanes read by C1A7 are merged before edge detection.
/// Ordinary frontends supply their logical buttons in `standard`, `auxiliary=NONE`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PortSample {
    pub standard: Buttons,
    pub auxiliary: Buttons,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PortInput {
    pub pressed: Buttons,
    pub held: Buttons,
}
#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Controllers {
    /// Result of the actual slot0 resume's final input-filter comparison. None
    /// means no sampling has run since reset; this is not a processor flag.
    pub last_sample_return: Option<SampleReturn>,
    #[serde(with = "crate::snapshot::array")]
    pub ports: [PortInput; CONTROLLER_PORTS],
}
/// WaitTicks preserves this named outcome through its final decrement. Only
/// raw newly pressed port0 edges participate, before direction filtering.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SampleReturn {
    pub primary_opposite_edges: bool,
}
impl Controllers {
    /// Edges use previous *filtered and game-modifiable* held state. Both outputs
    /// then independently discard their entire direction nibble on either conflict.
    pub fn sample(&mut self, samples: [PortSample; CONTROLLER_PORTS]) {
        fn filtered(buttons: u8) -> Buttons {
            const VERTICAL_PAIR: u8 = Buttons::UP.0 | Buttons::DOWN.0;
            const HORIZONTAL_PAIR: u8 = Buttons::LEFT.0 | Buttons::RIGHT.0;
            const ACTIONS: u8 = Buttons::A.0 | Buttons::B.0 | Buttons::SELECT.0 | Buttons::START.0;
            Buttons(
                if buttons & VERTICAL_PAIR == VERTICAL_PAIR
                    || buttons & HORIZONTAL_PAIR == HORIZONTAL_PAIR
                {
                    buttons & ACTIONS
                } else {
                    buttons
                },
            )
        }
        for (index, (port, sample)) in self.ports.iter_mut().zip(samples).enumerate() {
            let acquired = sample.standard.0 | sample.auxiliary.0;
            // Compute edges before either filter: a newly pressed opposite direction
            // can survive in pressed even when held loses its direction nibble.
            let raw_edges = acquired & !port.held.0;
            if index == 0 {
                let vertical = Buttons::UP.0 | Buttons::DOWN.0;
                let horizontal = Buttons::LEFT.0 | Buttons::RIGHT.0;
                self.last_sample_return = Some(SampleReturn {
                    primary_opposite_edges: raw_edges & vertical == vertical
                        || raw_edges & horizontal == horizontal,
                });
            }
            port.pressed = filtered(raw_edges);
            port.held = filtered(acquired);
        }
    }
}
/// Four algorithm words, not an emulated memory region. Consumers in B07/B11/
/// B12/B16/B17 also perform source-specific accumulator updates on these words.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RandomState {
    #[serde(with = "crate::snapshot::array")]
    pub words: [u8; 4],
}
impl RandomState {
    pub fn advance(&mut self) {
        const FEEDBACK_BIT: u8 = 0x02;
        let mut incoming = u8::from((self.words[0] ^ self.words[1]) & FEEDBACK_BIT != 0);
        for word in &mut self.words {
            let outgoing = *word & 1;
            *word = (*word >> 1) | (incoming << 7);
            incoming = outgoing;
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskSlot {
    Foreground,
    AuxiliaryOne,
    AuxiliaryTwo,
    AuxiliaryThree,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TaskState<T> {
    Empty,
    Entry(T),
    Running(T),
    Sleeping {
        continuation: T,
        ticks_remaining: u8,
    },
    Resume(T),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DispatchKind {
    Entry,
    Resume,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Dispatch<T> {
    pub slot: TaskSlot,
    pub continuation: T,
    pub kind: DispatchKind,
}
/// Native continuation values are supplied by the owning scene/system, never
/// source instruction addresses. Four slots and ascending priority are source rules.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(bound(
    serialize = "T: serde::Serialize",
    deserialize = "T: serde::Deserialize<'de>"
))]
pub struct Scheduler<T> {
    #[serde(with = "crate::snapshot::array")]
    pub slots: [TaskState<T>; TASK_SLOTS],
}
impl<T: Copy> Scheduler<T> {
    pub fn empty() -> Self {
        Self {
            slots: [TaskState::Empty; TASK_SLOTS],
        }
    }
    pub fn install(&mut self, slot: TaskSlot, entry: T) {
        self.slots[slot as usize] = TaskState::Entry(entry);
    }
    pub fn dispatch(
        &mut self,
        controllers: &mut Controllers,
        samples: [PortSample; CONTROLLER_PORTS],
    ) -> Option<Dispatch<T>> {
        for (index, slot) in [
            TaskSlot::Foreground,
            TaskSlot::AuxiliaryOne,
            TaskSlot::AuxiliaryTwo,
            TaskSlot::AuxiliaryThree,
        ]
        .into_iter()
        .enumerate()
        {
            let (continuation, kind) = match self.slots[index] {
                TaskState::Entry(value) => (value, DispatchKind::Entry),
                TaskState::Resume(value) => (value, DispatchKind::Resume),
                _ => continue,
            };
            self.slots[index] = TaskState::Running(continuation);
            if index == 0 && kind == DispatchKind::Resume {
                controllers.sample(samples);
            }
            return Some(Dispatch {
                slot,
                continuation,
                kind,
            });
        }
        None
    }
    pub fn yield_one_frame(&mut self, slot: TaskSlot, continuation: T) {
        self.slots[slot as usize] = TaskState::Sleeping {
            continuation,
            ticks_remaining: 1,
        };
    }
    /// Used by NMI. Only Sleeping is decremented; zero wraps to 255.
    pub fn wake(&mut self) {
        for state in &mut self.slots {
            if let TaskState::Sleeping {
                continuation,
                ticks_remaining,
            } = *state
            {
                let remaining = ticks_remaining.wrapping_sub(1);
                *state = if remaining == 0 {
                    TaskState::Resume(continuation)
                } else {
                    TaskState::Sleeping {
                        continuation,
                        ticks_remaining: remaining,
                    }
                };
            }
        }
    }
    pub fn finish(&mut self, slot: TaskSlot) {
        self.slots[slot as usize] = TaskState::Empty;
    }
}
/// Explicit continuation for WaitSchedulerTicks. Resumption happens only after
/// scheduler wake+dispatch; this object records its remaining repeated yields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameWait {
    pub ticks_remaining: u16,
}
impl FrameWait {
    pub fn start(source_count: u8) -> Self {
        const BYTE_WRAP_WAIT: u16 = 256;
        Self {
            ticks_remaining: if source_count == 0 {
                BYTE_WRAP_WAIT
            } else {
                source_count.into()
            },
        }
    }
    /// Return true when the resumed wait is complete, else yield once more.
    pub fn resumed(&mut self) -> bool {
        self.ticks_remaining = self.ticks_remaining.saturating_sub(1);
        self.ticks_remaining == 0
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum BootTask {
    InitializeTitle,
    /// Native main-call continuation after the source's first real yield.
    GameMain,
}
/// Reset requests to presentation, which owns the actual buffers and renderer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResetEffect {
    QuiesceDisplay,
    ClearRuntimeState,
    HideAllSprites,
    ClearFirstNametable,
    ClearSecondNametable,
    ScheduleTitle,
    InstallControlShadow,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResetDisplay {
    pub background_tile: u8,
    pub attribute: u8,
    pub hidden_sprite_y: u8,
    pub control: u8,
    pub effects: Vec<ResetEffect>,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Runtime {
    pub controllers: Controllers,
    pub random: RandomState,
    pub scheduler: Scheduler<BootTask>,
    pub interrupt_tick: u8,
    pub bank: BankContext,
}
impl Runtime {
    /// Cold reset and original reset jump both clear these source-owned fields.
    /// NES startup vblank polling is warm-up, not four playable native ticks.
    pub fn reset() -> (Self, ResetDisplay) {
        let mut scheduler = Scheduler::empty();
        scheduler.install(TaskSlot::Foreground, BootTask::InitializeTitle);
        (
            Self {
                controllers: Controllers::default(),
                random: RandomState::default(),
                scheduler,
                interrupt_tick: 0,
                bank: BankContext {
                    foreground: PrgBank::GraphicsZero,
                    active: PrgBank::GraphicsZero,
                },
            },
            ResetDisplay {
                background_tile: RESET_BACKGROUND_TILE,
                attribute: 0,
                hidden_sprite_y: HIDDEN_SPRITE_Y,
                control: RESET_CONTROL,
                effects: vec![
                    ResetEffect::QuiesceDisplay,
                    ResetEffect::ClearRuntimeState,
                    ResetEffect::HideAllSprites,
                    ResetEffect::ClearFirstNametable,
                    ResetEffect::ClearSecondNametable,
                    ResetEffect::ScheduleTitle,
                    ResetEffect::InstallControlShadow,
                ],
            },
        )
    }
    /// Does not sample controllers, run foreground rules or advance RNG.
    pub fn interrupt(
        &mut self,
        _source: InterruptSource,
        context: InterruptContext,
    ) -> Vec<FrameEffect> {
        let mut effects = Vec::new();
        if context.gameplay_display {
            effects.push(FrameEffect::PrimeSplitSprite);
        }
        let visible = context.display_suspend_depth == 0;
        if visible {
            effects.push(FrameEffect::BeginDisplayCommit);
            if !context.oam_guard {
                effects.push(FrameEffect::CopyOam);
            }
            if context.primary_pending {
                effects.push(FrameEffect::FlushPrimary);
            }
            if context.secondary_pending {
                effects.push(FrameEffect::FlushSecondary);
            }
            if context.palette_pending {
                effects.push(FrameEffect::UploadPalette);
            }
            effects.push(FrameEffect::InstallUiScroll);
            let control = if context.gameplay_display
                && !context.ending_layout
                && context.player_screen_y >= SPLIT_PLAYER_Y_THRESHOLD
            {
                UiControl::ForceGameplayPatterns
            } else {
                UiControl::Shadow
            };
            effects.push(FrameEffect::InstallUiControl(control));
        }
        self.interrupt_tick = self.interrupt_tick.wrapping_add(1);
        self.scheduler.wake();
        effects.push(FrameEffect::WakeTasks);
        if visible && context.gameplay_display {
            if context.alternate_split_address {
                effects.push(FrameEffect::InstallSplitAddress);
            }
            if context.race_intro {
                effects.push(FrameEffect::SpriteOnlyMask);
            }
            effects.push(FrameEffect::InstallPlayfieldScroll);
        }
        self.bank.enter_audio();
        effects.extend([
            FrameEffect::DrainAudioCommands,
            FrameEffect::AdvanceAudioFrame,
        ]);
        self.bank.restore_foreground();
        effects.push(FrameEffect::RestoreForegroundBank);
        effects
    }
}
/// Switch-window catalog identities; fixed-bank code is never selected here.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum PrgBank {
    GraphicsZero,
    GraphicsOne,
    GraphicsTwo,
    Audio,
    ActorsFour,
    PresentationFive,
    ActorsSix,
    FixedSeven,
}
impl PrgBank {
    /// This eight-bank board exposes the selector's low three bank bits.
    /// The value identifies immutable data ownership, never CPU execution.
    pub fn from_selector(selector: u8) -> Self {
        match selector & 7 {
            0 => Self::GraphicsZero,
            1 => Self::GraphicsOne,
            2 => Self::GraphicsTwo,
            3 => Self::Audio,
            4 => Self::ActorsFour,
            5 => Self::PresentationFive,
            6 => Self::ActorsSix,
            _ => Self::FixedSeven,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BankContext {
    pub foreground: PrgBank,
    pub active: PrgBank,
}
impl BankContext {
    pub fn select(&mut self, bank: PrgBank) {
        self.foreground = bank;
        self.active = bank;
    }
    pub fn enter_audio(&mut self) {
        self.active = PrgBank::Audio;
    }
    pub fn restore_foreground(&mut self) {
        self.active = self.foreground;
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterruptSource {
    Nmi,
    Irq,
}

/// Snapshot of other owners' controls at interrupt entry; not another state owner.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct InterruptContext {
    pub display_suspend_depth: u8,
    pub gameplay_display: bool,
    pub oam_guard: bool,
    pub primary_pending: bool,
    pub secondary_pending: bool,
    pub palette_pending: bool,
    pub player_screen_y: u8,
    pub ending_layout: bool,
    pub alternate_split_address: bool,
    pub race_intro: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiControl {
    Shadow,
    ForceGameplayPatterns,
}
/// Requests in exact observable commit order. B03 supplies pixels/queues/split
/// content; B18 supplies ring contents and the once-per-interrupt driver frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameEffect {
    PrimeSplitSprite,
    BeginDisplayCommit,
    CopyOam,
    FlushPrimary,
    FlushSecondary,
    UploadPalette,
    InstallUiScroll,
    InstallUiControl(UiControl),
    WakeTasks,
    InstallSplitAddress,
    SpriteOnlyMask,
    InstallPlayfieldScroll,
    DrainAudioCommands,
    AdvanceAudioFrame,
    RestoreForegroundBank,
}

#[cfg(test)]
mod contracts;

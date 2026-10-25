//! Immutable sequence programs compiled at asset import. Every supplied byte
//! location may be an entry (formats/streams can overlap). Missing operands are
//! retained as bounds faults; a skipped loop destination must never be read.
use super::{Fragment, PRIMARY_HANDLERS, SECONDARY_HANDLERS};
use std::collections::{BTreeMap, BTreeSet};
const PERIOD_TABLE_BASE: u16 = 0x897b;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DurationClass {
    Standard(u8),
    Alternate(u8),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Note {
    pub key: u8,
    pub duration: DurationClass,
}
impl Note {
    fn decode(value: u8, alternate: bool) -> Self {
        let class = (value >> 5) & 7;
        Self {
            key: value & 31,
            duration: if alternate {
                DurationClass::Alternate(class)
            } else {
                DurationClass::Standard(class)
            },
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BranchTarget {
    Known(u16),
    MissingLow,
    MissingHigh,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PrimaryAction {
    Scale(Option<u8>),
    PitchStep(Option<i8>),
    ControlHigh(Option<u8>),
    ControlLow(Option<u8>),
    Repeat {
        count: Option<u8>,
        target: BranchTarget,
    },
    PeriodTable(Option<u16>),
    AlternateNote(Option<Note>),
    Envelope {
        interval: Option<u8>,
        packed: Option<u8>,
    },
    Preset(Option<u8>),
    Stop,
    Deferred(u8),
    Continue,
    Note(Note),
    Invalid(u8),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SecondaryAction {
    Delay(Option<u8>),
    PitchStep(Option<i8>),
    ControlHigh(Option<u8>),
    ControlLow(Option<u8>),
    Repeat {
        count: Option<u8>,
        target: BranchTarget,
    },
    Modulation {
        pitch_interval: Option<u8>,
        pitch_shape: Option<u8>,
        level_interval: Option<u8>,
        level_step: Option<i8>,
    },
    End,
    KeepPeriod(Option<u8>),
    Period(Option<u16>),
    Invalid(u8),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Instruction<T> {
    pub action: T,
    pub dispatch_seed: Option<u8>,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct Programs {
    pub primary: BTreeMap<u16, Instruction<PrimaryAction>>,
    pub secondary: BTreeMap<u16, Instruction<SecondaryAction>>,
}
pub(super) fn source_byte(fragments: &[Fragment], address: u16) -> Option<u8> {
    fragments.iter().find_map(|fragment| {
        address
            .checked_sub(fragment.origin)
            .and_then(|offset| fragment.bytes.get(usize::from(offset)))
            .copied()
    })
}
impl Programs {
    pub fn compile(fragments: &[Fragment]) -> Self {
        let mut positions = BTreeSet::new();
        for fragment in fragments {
            // Bytes beyond FFFF are unreachable through the original resource
            // identity; wrapped reads can enter a separate fragment at0000.
            positions.extend(
                (0..fragment
                    .bytes
                    .len()
                    .min(usize::from(u16::MAX - fragment.origin) + 1))
                    .map(|offset| fragment.origin + offset as u16),
            );
        }
        let mut programs = Self::default();
        for position in positions {
            let value = source_byte(fragments, position).expect("enumerated source entry");
            let operand = |offset: u16| source_byte(fragments, position.wrapping_add(offset));
            let target = match (operand(2), operand(3)) {
                (None, _) => BranchTarget::MissingLow,
                (Some(_), None) => BranchTarget::MissingHigh,
                (Some(low), Some(high)) => BranchTarget::Known(u16::from_le_bytes([low, high])),
            };
            let primary =
                match value {
                    0 => PrimaryAction::Scale(operand(1)),
                    1 => PrimaryAction::PitchStep(operand(1).map(|value| value as i8)),
                    2 => PrimaryAction::ControlHigh(operand(1)),
                    3 => PrimaryAction::ControlLow(operand(1)),
                    4 => PrimaryAction::Repeat {
                        count: operand(1),
                        target,
                    },
                    5 => PrimaryAction::PeriodTable(operand(1).map(|value| {
                        PERIOD_TABLE_BASE.wrapping_add(u16::from(value.wrapping_mul(2)))
                    })),
                    6 => PrimaryAction::AlternateNote(
                        operand(1).map(|value| Note::decode(value, true)),
                    ),
                    7 => PrimaryAction::Envelope {
                        interval: operand(1),
                        packed: operand(2),
                    },
                    8 => PrimaryAction::Preset(operand(1)),
                    9 => PrimaryAction::Stop,
                    10..=15 => PrimaryAction::Invalid(value),
                    _ if value & 0xf0 == 0x20 => PrimaryAction::Deferred(value & 7),
                    _ if value & 0xf0 == 0x30 => PrimaryAction::Continue,
                    _ => PrimaryAction::Note(Note::decode(value, false)),
                };
            programs.primary.insert(
                position,
                Instruction {
                    action: primary,
                    dispatch_seed: PRIMARY_HANDLERS
                        .get(usize::from(value))
                        .map(|target| *target as u8),
                },
            );
            let secondary = match value {
                0 => SecondaryAction::Delay(operand(1)),
                1 => SecondaryAction::PitchStep(operand(1).map(|value| value as i8)),
                2 => SecondaryAction::ControlHigh(operand(1)),
                3 => SecondaryAction::ControlLow(operand(1)),
                4 => SecondaryAction::Repeat {
                    count: operand(1),
                    target,
                },
                5 => SecondaryAction::Modulation {
                    pitch_interval: operand(1),
                    pitch_shape: operand(2),
                    level_interval: operand(3),
                    level_step: operand(4).map(|value| value as i8),
                },
                6 => SecondaryAction::End,
                _ if value & 0x80 == 0 => SecondaryAction::Invalid(value),
                _ if value & 15 == 15 => SecondaryAction::KeepPeriod(operand(1)),
                _ => SecondaryAction::Period(
                    operand(1).map(|low| u16::from_le_bytes([low, value & 7])),
                ),
            };
            programs.secondary.insert(
                position,
                Instruction {
                    action: secondary,
                    dispatch_seed: SECONDARY_HANDLERS
                        .get(usize::from(value))
                        .map(|target| *target as u8),
                },
            );
        }
        programs
    }
}

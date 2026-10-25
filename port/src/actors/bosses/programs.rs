//! Movement programs decoded at the asset boundary. Gameplay consumes named
//! headings, contact conditions and timeline actions instead of source tables.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HorizontalHeading {
    Right,
    Left,
    LegacyNeither,
    LegacyBoth,
}
impl HorizontalHeading {
    fn decode(code: u8) -> Self {
        match code {
            1 => Self::Right,
            2 => Self::Left,
            0 => Self::LegacyNeither,
            _ => Self::LegacyBoth,
        }
    }
    pub fn mirrored(self) -> Self {
        match self {
            Self::Right => Self::Left,
            Self::Left => Self::Right,
            Self::LegacyNeither => Self::LegacyBoth,
            Self::LegacyBoth => Self::LegacyNeither,
        }
    }
    /// Existing shared actor movement/observations retain this boundary code.
    pub fn direction_code(self) -> u8 {
        match self {
            Self::Right => 1,
            Self::Left => 2,
            Self::LegacyNeither => 0,
            Self::LegacyBoth => 3,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerticalHeading {
    Up,
    Down,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerticalCorrection {
    First,
    Second,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContactGate {
    Neither,
    Horizontal,
    Vertical,
    Both,
    /// Source masks selecting nonexistent contact bits can never be satisfied.
    UnreachableMask(u8),
}
impl ContactGate {
    pub fn from_source(mask: u8) -> Self {
        match mask {
            0 => Self::Neither,
            1 => Self::Horizontal,
            2 => Self::Vertical,
            3 => Self::Both,
            other => Self::UnreachableMask(other),
        }
    }
    pub fn accepts(self, horizontal: bool, vertical: bool) -> bool {
        match self {
            Self::Neither => !horizontal && !vertical,
            Self::Horizontal => horizontal,
            Self::Vertical => vertical,
            Self::Both => horizontal && vertical,
            Self::UnreachableMask(_) => false,
        }
    }
    pub fn needs_y_correction(self) -> bool {
        self == Self::Neither
    }
    #[cfg(test)]
    pub fn source_code(self) -> u8 {
        match self {
            Self::Neither => 0,
            Self::Horizontal => 1,
            Self::Vertical => 2,
            Self::Both => 3,
            Self::UnreachableMask(code) => code,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PerimeterStep {
    pub horizontal: HorizontalHeading,
    pub vertical: VerticalHeading,
    pub correction: VerticalCorrection,
    pub gate: ContactGate,
}
impl PerimeterStep {
    pub fn from_source(path: u8, mask: u8) -> Self {
        const HORIZONTAL_MASK: u8 = 3;
        const DOWNWARD_FLAG: u8 = 4;
        const SECOND_CORRECTION_FLAG: u8 = 8;
        Self {
            horizontal: HorizontalHeading::decode(path & HORIZONTAL_MASK),
            vertical: if path & DOWNWARD_FLAG != 0 {
                VerticalHeading::Down
            } else {
                VerticalHeading::Up
            },
            correction: if path & SECOND_CORRECTION_FLAG != 0 {
                VerticalCorrection::Second
            } else {
                VerticalCorrection::First
            },
            gate: ContactGate::from_source(mask),
        }
    }
    /// The current actor motion state is shared with legacy movement helpers.
    /// Decode only source asset records here; encode this projection at handoff.
    pub fn motion_code(self) -> u8 {
        const DOWNWARD_FLAG: u8 = 4;
        const SECOND_CORRECTION_FLAG: u8 = 8;
        (if self.vertical == VerticalHeading::Down {
            DOWNWARD_FLAG
        } else {
            0
        }) | (if self.correction == VerticalCorrection::Second {
            SECOND_CORRECTION_FLAG
        } else {
            0
        })
    }
    #[cfg(test)]
    pub fn source_path_code(self) -> u8 {
        self.horizontal.direction_code() | self.motion_code()
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WaveStep {
    Spawn { x: u8, at_tick: u8 },
    AwaitLastChild,
}
impl WaveStep {
    pub fn from_source(x: u8, tick: u8) -> Self {
        // AAA8 zero-X endpoint branches before consulting AAB6. Its parallel
        // timer byte has no consumer and is deliberately absent from this type.
        if x == 0 {
            Self::AwaitLastChild
        } else {
            Self::Spawn { x, at_tick: tick }
        }
    }
}
pub fn perimeter_program<const N: usize>(paths: [u8; N], masks: [u8; N]) -> [PerimeterStep; N] {
    std::array::from_fn(|index| PerimeterStep::from_source(paths[index], masks[index]))
}
pub fn wave_program<const N: usize>(positions: [u8; N], ticks: [u8; N]) -> [WaveStep; N] {
    std::array::from_fn(|index| WaveStep::from_source(positions[index], ticks[index]))
}

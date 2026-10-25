//! Native health rules, with original sprite records confined to the boundary.
//! Capacity follows segment availability; appearance never drives a game rule.
use super::{OVERLAY_SEGMENTS, TitleRule};
use std::num::NonZeroU8;

/// Original overlay record used only for template import and render/debug output.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HealthSegment {
    pub y: u8,
    pub tile: u8,
    pub depleted: u8,
    pub x: u8,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Depletion {
    Filled,
    /// Noncanonical nonzero attributes remain observable in original overlays.
    Depleted(NonZeroU8),
}
impl Depletion {
    fn from_code(code: u8) -> Self {
        NonZeroU8::new(code).map_or(Self::Filled, Self::Depleted)
    }
    fn code(self) -> u8 {
        match self {
            Self::Filled => 0,
            Self::Depleted(code) => code.get(),
        }
    }
    fn is_depleted(self) -> bool {
        matches!(self, Self::Depleted(_))
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Point {
    available: bool,
    depletion: Depletion,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Appearance {
    y: u8,
    tile: u8,
    x: u8,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct Overlay {
    appearance: [Appearance; OVERLAY_SEGMENTS],
    end: u8,
}
impl Overlay {
    /// Source extension reveals one extra sprite and terminates its successor.
    fn reveal(&mut self, index: usize) {
        const HEALTH_ROW_Y: u8 = 0x20;
        self.appearance[index].y = HEALTH_ROW_Y;
        if index + 1 < OVERLAY_SEGMENTS {
            self.appearance[index + 1].y = u8::MAX;
        } else {
            self.end = u8::MAX;
        }
    }
}
/// Canonical availability and health of each segment, separate from its sprite.
/// Sparse depletion and hidden flags are retained because recovery/payment and
/// full restoration distinguish them. There is no second health-count owner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Health {
    points: [Point; OVERLAY_SEGMENTS],
    overlay: Overlay,
    pub alternating: u8,
}
impl Health {
    pub fn from_overlay(
        segments: [HealthSegment; OVERLAY_SEGMENTS],
        end: u8,
        alternating: u8,
    ) -> Self {
        const LOCKED_Y_BIT: u8 = 0x80;
        Self {
            points: segments.map(|segment| Point {
                available: segment.y & LOCKED_Y_BIT == 0,
                depletion: Depletion::from_code(segment.depleted),
            }),
            overlay: Overlay {
                appearance: segments.map(|segment| Appearance {
                    y: segment.y,
                    tile: segment.tile,
                    x: segment.x,
                }),
                end,
            },
            alternating,
        }
    }
    pub fn capacity(&self) -> usize {
        self.points
            .iter()
            .take_while(|point| point.available)
            .count()
    }
    /// The player's original death gate checks the first flag, even for an
    /// unusual imported layout with no unlocked segments.
    pub fn exhausted(&self) -> bool {
        self.points[0].depletion.is_depleted()
    }
    pub(super) fn needs_paid_restore(&self) -> bool {
        self.points[..self.capacity()]
            .last()
            .is_some_and(|point| point.depletion.is_depleted())
    }
    pub fn damage(&mut self, rule: TitleRule) {
        if rule == TitleRule::AlternatingDamage {
            self.alternating = self.alternating.wrapping_add(1);
            if self.alternating & 1 != 0 {
                return;
            }
        }
        let capacity = self.capacity();
        let unlocked = &mut self.points[..capacity];
        let end = unlocked
            .iter()
            .position(|point| point.depletion.is_depleted())
            .unwrap_or(unlocked.len());
        if let Some(index) = end.checked_sub(1) {
            unlocked[index].depletion = Depletion::from_code(1);
        }
    }
    pub fn recover_one(&mut self) {
        self.alternating = 0;
        let capacity = self.capacity();
        if let Some(point) = self.points[..capacity]
            .iter_mut()
            .find(|point| point.depletion.is_depleted())
        {
            point.depletion = Depletion::Filled;
        }
    }
    pub fn restore(&mut self) {
        self.alternating = 0;
        for point in &mut self.points {
            point.depletion = Depletion::Filled;
        }
    }
    pub fn extend(&mut self) -> bool {
        const INITIAL_CAPACITY: usize = 3;
        let Some(index) =
            (INITIAL_CAPACITY..OVERLAY_SEGMENTS).find(|&index| !self.points[index].available)
        else {
            return false;
        };
        self.points[index].available = true;
        if index + 1 < OVERLAY_SEGMENTS {
            self.points[index + 1].available = false;
        }
        self.overlay.reveal(index);
        self.restore();
        true
    }
    pub fn overlay_segments(&self) -> [HealthSegment; OVERLAY_SEGMENTS] {
        std::array::from_fn(|index| {
            let appearance = self.overlay.appearance[index];
            HealthSegment {
                y: appearance.y,
                tile: appearance.tile,
                depleted: self.points[index].depletion.code(),
                x: appearance.x,
            }
        })
    }
    pub fn overlay_end(&self) -> u8 {
        self.overlay.end
    }
    /// Fixtures author original boundary records without exposing mutable
    /// appearance as gameplay state. Reimport derives availability atomically.
    #[cfg(test)]
    pub fn edit_overlay(&mut self, edit: impl FnOnce(&mut [HealthSegment; OVERLAY_SEGMENTS])) {
        let mut segments = self.overlay_segments();
        edit(&mut segments);
        *self = Self::from_overlay(segments, self.overlay.end, self.alternating);
    }
}

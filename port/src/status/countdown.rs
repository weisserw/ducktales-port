//! Stage time as a quantity. Display codes cross only initialization, rendering
//! and debug boundaries; ordinary countdown/conversion rules operate in seconds.
use super::{DECIMAL_RADIX, EXPIRED, subtract_codes};

const MAX_SECONDS: u16 = 999;
const WARNING_SECONDS: u16 = 30;
const WARNING_TENS_CODE: u8 = 3;
const CONVERSION_SECONDS: u16 = 10;
const HUNDREDS_PLACE: u16 = 100;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
enum Value {
    Seconds(u16),
    Expired,
    /// Game-over overlays and raw stage selectors can supply nondecimal codes.
    /// Preserve those bytes rather than inventing a numeric meaning for them.
    DisplayCodes([u8; 3]),
}

/// Canonical countdown owner. Seconds are always in 0..=999; expiration is a
/// distinct state because death occurs at the next eligible countdown call.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Countdown(Value);
impl Countdown {
    pub fn from_display_codes(codes: [u8; 3]) -> Self {
        Self(if codes == EXPIRED {
            Value::Expired
        } else if codes.iter().all(|&code| code < DECIMAL_RADIX) {
            Value::Seconds(
                u16::from(codes[0]) * HUNDREDS_PLACE
                    + u16::from(codes[1]) * u16::from(DECIMAL_RADIX)
                    + u16::from(codes[2]),
            )
        } else {
            Value::DisplayCodes(codes)
        })
    }
    pub fn display_codes(self) -> [u8; 3] {
        match self.0 {
            Value::Seconds(seconds) => [
                (seconds / HUNDREDS_PLACE) as u8,
                ((seconds / u16::from(DECIMAL_RADIX)) % u16::from(DECIMAL_RADIX)) as u8,
                (seconds % u16::from(DECIMAL_RADIX)) as u8,
            ],
            Value::Expired => EXPIRED,
            Value::DisplayCodes(codes) => codes,
        }
    }
    /// Initial entry copies only the stage's upper pair, retaining the units
    /// from its overlay template. Reload uses a separate complete replacement.
    pub fn replace_upper_codes(&mut self, pair: [u8; 2]) {
        *self = Self::from_display_codes([pair[0], pair[1], self.display_codes()[2]]);
    }
    pub fn expired(self) -> bool {
        match self.0 {
            Value::Expired => true,
            Value::DisplayCodes(codes) => codes[0] == EXPIRED[0],
            Value::Seconds(_) => false,
        }
    }
    pub fn warning(self) -> bool {
        match self.0 {
            Value::Seconds(seconds) => seconds < WARNING_SECONDS,
            Value::Expired => false,
            // The source warning gate ignores the units glyph entirely.
            Value::DisplayCodes(codes) => codes[0] == 0 && codes[1] < WARNING_TENS_CODE,
        }
    }
    /// E2A6: zero wraps to 999; reaching zero installs the expired state.
    /// The caller owns the divider and checks expiration before decrementing.
    pub fn elapse_second(&mut self) {
        self.0 = match self.0 {
            Value::Seconds(0) => Value::Seconds(MAX_SECONDS),
            Value::Seconds(1) => Value::Expired,
            Value::Seconds(seconds) => Value::Seconds(seconds - 1),
            Value::Expired | Value::DisplayCodes(_) => {
                let mut codes = self.display_codes();
                // Preserve the faithful implementation's unusual-code behavior.
                // Its BLANK normalization differs from raw source SBC for some
                // malformed glyph inputs; this iteration does not repair that
                // pre-existing discrepancy or claim original fidelity there.
                subtract_codes(&mut codes, 1);
                if codes == [0; 3] {
                    Value::Expired
                } else {
                    Self::from_display_codes(codes).0
                }
            }
        };
    }
    /// EF70: discard the remaining units, then consume ten seconds.
    /// False means the caller must begin its finish wait on this odd frame.
    /// Consuming the final group returns true; finishing starts on the next call.
    pub fn convert_group(&mut self) -> bool {
        match self.0 {
            Value::Seconds(seconds) => {
                let groups = seconds / CONVERSION_SECONDS;
                if groups == 0 {
                    self.0 = Value::Seconds(0);
                    false
                } else {
                    self.0 = Value::Seconds((groups - 1) * CONVERSION_SECONDS);
                    true
                }
            }
            Value::Expired | Value::DisplayCodes(_) => {
                let mut codes = self.display_codes();
                codes[2] = 0;
                let consumed = codes[0] != 0 || codes[1] != 0;
                if consumed {
                    subtract_codes(&mut codes[..2], 1);
                }
                *self = Self::from_display_codes(codes);
                consumed
            }
        }
    }
}
impl From<[u8; 3]> for Countdown {
    fn from(codes: [u8; 3]) -> Self {
        Self::from_display_codes(codes)
    }
}

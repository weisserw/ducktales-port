//! Monetary quantities and the display distinctions that have observable rules.
//! Ordinary arithmetic uses native amounts. Nondecimal imported codes remain a
//! compatibility boundary; they never force ordinary gameplay back to digit math.
use super::{
    BLANK, DigitTraversal, RewardPayment, add_codes, blank_leading_zeroes,
    blank_literal_zero_prefix, subtract_codes,
};
const RADIX: u32 = 10;
const DIALOGUE_COST: u32 = 30_000;
const RESTORE_COST: u32 = 300_000;
const PAID_PREFIX: usize = 2;
const REWARD_SIGNIFICANT: usize = 4;
const TOTAL_SIGNIFICANT: usize = 5;
const DEPARTURE_REWARD_QUANTUM: u32 = 1_000;
const DEPARTURE_REWARD_DIGIT: u32 = 7;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
enum Value<const N: usize> {
    Amount { amount: u32, blanks: u8 },
    NondecimalCodes(#[serde(with = "crate::snapshot::array")] [u8; N]),
}
/// Numeric amount plus lossless blank-layout metadata. A blank and a displayed
/// zero can have the same amount but different payment/ending outcomes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Money<const N: usize>(Value<N>);
pub type RewardMoney = Money<6>;
pub type TotalMoney = Money<7>;

/// Native denominations credited by actual game rules. Original digit positions
/// are confined to the compatibility adapter below, never supplied by producers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RewardCredit {
    Pickup,
    LargePickup,
    ExtraPickup,
    Treasure,
    Timer,
}
impl RewardCredit {
    pub fn amount(self) -> u32 {
        match self {
            Self::Pickup => 200,
            Self::LargePickup => 1_000,
            Self::ExtraPickup => 5_000,
            Self::Treasure => 100_000,
            Self::Timer => 100,
        }
    }
    fn position_and_units(self) -> (usize, u8) {
        let mut units = self.amount();
        let mut position = 5;
        while units.is_multiple_of(RADIX) {
            units /= RADIX;
            position -= 1;
        }
        (position, units as u8)
    }
}
impl<const N: usize> Money<N> {
    pub fn from_display_codes(codes: [u8; N]) -> Self {
        assert!(N <= 7 && N != 0, "supported monetary display width");
        let mut amount = 0;
        let mut blanks = 0;
        for (index, &code) in codes.iter().enumerate() {
            amount *= RADIX;
            if code == BLANK {
                blanks |= 1 << index;
            } else if u32::from(code) < RADIX {
                amount += u32::from(code);
            } else {
                return Self(Value::NondecimalCodes(codes));
            }
        }
        Self(Value::Amount { amount, blanks })
    }
    /// None identifies nondecimal original data with no invented monetary value.
    pub fn amount(self) -> Option<u32> {
        match self.0 {
            Value::Amount { amount, .. } => Some(amount),
            Value::NondecimalCodes(_) => None,
        }
    }
    pub fn display_codes(self) -> [u8; N] {
        match self.0 {
            Value::Amount { amount, blanks } => std::array::from_fn(|index| {
                if blanks & (1 << index) != 0 {
                    BLANK
                } else {
                    ((amount / RADIX.pow((N - 1 - index) as u32)) % RADIX) as u8
                }
            }),
            Value::NondecimalCodes(codes) => codes,
        }
    }
    fn prefix_mask(width: usize) -> u8 {
        ((1u16 << width) - 1) as u8
    }
    fn blank_at(self, position: usize) -> bool {
        match self.0 {
            Value::Amount { blanks, .. } => blanks & (1 << position) != 0,
            Value::NondecimalCodes(codes) => codes[position] == BLANK,
        }
    }
    fn replace_prefix_with_nines(&mut self, width: usize) {
        match &mut self.0 {
            Value::Amount { amount, blanks } => {
                let quantum = RADIX.pow((N - width) as u32);
                *amount = (RADIX.pow(width as u32) - 1) * quantum + *amount % quantum;
                *blanks &= !Self::prefix_mask(width);
            }
            Value::NondecimalCodes(codes) => {
                codes[..width].fill(9);
                *self = Self::from_display_codes(*codes);
            }
        }
    }
    /// Native addition at the denomination's decimal quantum. The highest
    /// changed place determines which previously blank positions become visible.
    fn add_at(&mut self, position: usize, units: u8, whole_row: bool) -> bool {
        if let Value::Amount { amount, blanks } = &mut self.0
            && u32::from(units) < RADIX
        {
            let width = position + 1;
            let quantum = RADIX.pow((N - width) as u32);
            let limit = RADIX.pow(width as u32);
            let old = *amount / quantum;
            let sum = old + u32::from(units);
            if units != 0 {
                let mut changed = 1;
                let (mut before, mut after) = (old / RADIX, sum / RADIX);
                while before != after && changed < width {
                    changed += 1;
                    before /= RADIX;
                    after /= RADIX;
                }
                *blanks &= !(Self::prefix_mask(width) ^ Self::prefix_mask(width - changed));
            }
            *amount = (sum % limit) * quantum + *amount % quantum;
            return sum >= limit;
        }
        let mut codes = self.display_codes();
        let overflow = add_codes(
            &mut codes[..=position],
            units,
            if whole_row {
                DigitTraversal::WholeRow
            } else {
                DigitTraversal::CarryOnly
            },
        );
        *self = Self::from_display_codes(codes);
        overflow
    }
    fn subtract_prefix(&mut self, width: usize) -> bool {
        if let Value::Amount { amount, blanks } = &mut self.0 {
            let quantum = RADIX.pow((N - width) as u32);
            let prefix = *amount / quantum;
            let borrow = prefix == 0;
            let next = if borrow {
                RADIX.pow(width as u32) - 1
            } else {
                prefix - 1
            };
            *amount = next * quantum + *amount % quantum;
            *blanks &= !Self::prefix_mask(width);
            return borrow;
        }
        let mut codes = self.display_codes();
        let borrow = subtract_codes(&mut codes[..width], 1);
        *self = Self::from_display_codes(codes);
        borrow
    }
    fn debit(&mut self, cost: u32) -> bool {
        if let Value::Amount { amount, blanks } = &mut self.0 {
            if *amount < cost {
                return false;
            }
            *amount -= cost;
            *blanks &= !Self::prefix_mask(PAID_PREFIX);
            return true;
        }
        let mut codes = self.display_codes();
        let mut upper = [codes[0], codes[1]];
        let quantum = RADIX.pow((N - PAID_PREFIX) as u32);
        if subtract_codes(&mut upper, (cost / quantum) as u8) {
            return false;
        }
        codes[..PAID_PREFIX].copy_from_slice(&upper);
        *self = Self::from_display_codes(codes);
        true
    }
    fn leading_zeroes(amount: u32) -> usize {
        if amount == 0 {
            N
        } else {
            N - (amount.ilog10() as usize + 1)
        }
    }
    fn blank_zero_prefix(&mut self, stop_at_existing_blank: bool) -> usize {
        if let Value::Amount { amount, blanks } = &mut self.0 {
            let mut count = Self::leading_zeroes(*amount);
            if stop_at_existing_blank {
                count = count.min(blanks.trailing_zeros() as usize);
            }
            *blanks |= Self::prefix_mask(count);
            return count;
        }
        let mut codes = self.display_codes();
        let count = if stop_at_existing_blank {
            blank_literal_zero_prefix(&mut codes)
        } else {
            blank_leading_zeroes(&mut codes);
            0
        };
        *self = Self::from_display_codes(codes);
        count
    }
    fn zero_suffix(&mut self, width: usize) {
        if let Value::Amount { amount, blanks } = &mut self.0 {
            let quantum = RADIX.pow(width as u32);
            *amount = *amount / quantum * quantum;
            *blanks &= Self::prefix_mask(N - width);
        } else {
            let mut codes = self.display_codes();
            codes[N - width..].fill(0);
            *self = Self::from_display_codes(codes);
        }
    }
    #[cfg(test)]
    pub fn edit_display_codes(&mut self, edit: impl FnOnce(&mut [u8; N])) {
        let mut codes = self.display_codes();
        edit(&mut codes);
        *self = Self::from_display_codes(codes);
    }
}
impl<const N: usize> From<[u8; N]> for Money<N> {
    fn from(codes: [u8; N]) -> Self {
        Self::from_display_codes(codes)
    }
}
impl RewardMoney {
    pub fn credit(&mut self, credit: RewardCredit) {
        let (position, units) = credit.position_and_units();
        self.credit_at(position, units);
    }
    pub(super) fn credit_at(&mut self, position: usize, units: u8) {
        if self.add_at(position, units, false) {
            self.saturate();
        }
    }
    pub fn saturate(&mut self) {
        self.replace_prefix_with_nines(REWARD_SIGNIFICANT);
    }
    pub fn steal(&mut self) {
        const THEFT_PREFIX: usize = 3;
        if self.subtract_prefix(THEFT_PREFIX) {
            *self = [BLANK; 6].into();
        } else {
            self.blank_zero_prefix(false);
        }
    }
    pub fn pay_dialogue(&mut self) -> RewardPayment {
        if !self.debit(DIALOGUE_COST) {
            return RewardPayment::Underfunded;
        }
        match self.blank_zero_prefix(true) {
            0 => RewardPayment::FirstCodeNonzero,
            6 => RewardPayment::AllCodesZero,
            _ => RewardPayment::PartialZeroPrefix,
        }
    }
    pub fn prepare_display(&mut self) {
        if let Value::Amount { blanks, .. } = &mut self.0 {
            *blanks = Self::prefix_mask((blanks.trailing_ones() as usize).min(6));
        } else {
            let mut codes = self.display_codes();
            if let Some(first) = codes.iter().position(|&code| code < BLANK) {
                for code in &mut codes[first..] {
                    if *code == BLANK {
                        *code = 0;
                    }
                }
            }
            *self = Self::from_display_codes(codes);
        }
    }
    pub fn transfer_exhausted(self) -> bool {
        self.blank_at(5)
    }
    pub fn transfer_one(&mut self) {
        self.subtract_prefix(REWARD_SIGNIFICANT);
        self.blank_zero_prefix(false);
    }
    /// The departure's source gate examines the thousands denomination, not a
    /// >= threshold: other high digits and the lower remainder are irrelevant.
    pub fn departure_brakes(self) -> bool {
        match self.0 {
            Value::Amount { amount, .. } => {
                amount / DEPARTURE_REWARD_QUANTUM % RADIX == DEPARTURE_REWARD_DIGIT
            }
            Value::NondecimalCodes(codes) => u32::from(codes[2]) == DEPARTURE_REWARD_DIGIT,
        }
    }
}
impl TotalMoney {
    pub fn saturate(&mut self) {
        self.replace_prefix_with_nines(TOTAL_SIGNIFICANT);
    }
    pub fn pay_restore(&mut self) -> bool {
        if !self.debit(RESTORE_COST) {
            return false;
        }
        self.blank_zero_prefix(true);
        true
    }
    pub fn receive_transfer(&mut self) {
        if self.add_at(TOTAL_SIGNIFICANT - 1, 1, true) {
            self.saturate();
        }
        self.zero_suffix(2);
    }
    pub fn ending_has_top_digit(self) -> bool {
        !self.blank_at(0)
    }
    pub fn ending_has_no_value(self) -> bool {
        self.blank_at(6)
    }
}

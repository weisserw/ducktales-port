//! Encoded status values, overlay health, lives and countdown (B14).
//! Native domain declarations only; all operations await stage4. See trace.md.
mod formats;
use crate::actors::ActorContact;
use anyhow::{Context, Result, ensure};
pub const BLANK: u8 = 0x24;
pub const EXPIRED: [u8; 3] = [0x1e, 0x19, BLANK];
pub const COUNTDOWN_PERIOD: u8 = 60;
pub const MAX_LIVES: u8 = 9;
pub const TRANSFER_BATCH: u8 = 10;
pub const TRANSFER_WAIT: u16 = 2;
pub const AUDIO_SETTLE_WAIT: u16 = 256;
pub const CONVERSION_FINISH_WAIT: u16 = 60;
pub const VALUE_SOUND: u8 = 0x15;
pub const WARNING_SOUND: u8 = 0x36;
pub const STOP_STAGE_SOUND: u8 = 0xff;
pub const OVERLAY_SEGMENTS: usize = 5;
/// The paid dialogue's final leading-zero traversal, needed by the immediately
/// following actor iteration. An existing blank is a nonzero stopping code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RewardPayment {
    Underfunded,
    FirstCodeNonzero,
    PartialZeroPrefix,
    AllCodesZero,
}
impl RewardPayment {
    pub fn accepted(self) -> bool {
        self != Self::Underfunded
    }
    pub fn first_actor_increment(self) -> crate::progression::FirstActorIncrement {
        match self {
            Self::FirstCodeNonzero | Self::AllCodesZero => {
                crate::progression::FirstActorIncrement::One
            }
            Self::Underfunded | Self::PartialZeroPrefix => {
                crate::progression::FirstActorIncrement::Zero
            }
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TitleRule {
    AlternatingDamage,
    OrdinaryDamage,
    AlternateRewards,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HealthSegment {
    pub y: u8,
    pub tile: u8,
    pub depleted: u8,
    pub x: u8,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Health {
    pub segments: [HealthSegment; OVERLAY_SEGMENTS],
    pub end: u8,
    pub alternating: u8,
}
impl Health {
    /// Marks the flag preceding the first depleted segment or negative Y.
    pub fn damage(&mut self, rule: TitleRule) {
        if rule == TitleRule::AlternatingDamage {
            self.alternating = self.alternating.wrapping_add(1);
            if self.alternating & 1 != 0 {
                return;
            }
        }
        let end = self
            .segments
            .iter()
            .position(|segment| segment.y & 0x80 != 0 || segment.depleted != 0)
            .unwrap_or(OVERLAY_SEGMENTS);
        if let Some(index) = end.checked_sub(1) {
            self.segments[index].depleted = 1;
        }
    }
    pub fn recover_one(&mut self) {
        self.alternating = 0;
        for segment in &mut self.segments {
            if segment.y & 0x80 != 0 {
                break;
            }
            if segment.depleted != 0 {
                segment.depleted = 0;
                break;
            }
        }
    }
    pub fn restore(&mut self) {
        self.alternating = 0;
        for segment in &mut self.segments {
            segment.depleted = 0;
        }
    }
    pub fn extend(&mut self) -> bool {
        const INITIAL_SEGMENTS: usize = 3;
        const OVERLAY_Y: u8 = 0x20;
        let Some(index) =
            (INITIAL_SEGMENTS..OVERLAY_SEGMENTS).find(|&index| self.segments[index].y & 0x80 != 0)
        else {
            return false;
        };
        self.segments[index].y = OVERLAY_Y;
        if index + 1 < OVERLAY_SEGMENTS {
            self.segments[index + 1].y = u8::MAX;
        } else {
            self.end = u8::MAX;
        }
        self.restore();
        true
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Status {
    pub reward: [u8; 6],
    pub total: [u8; 7],
    pub lives: u8,
    pub health: Health,
    pub countdown: [u8; 3],
    pub divider: u8,
    pub dirty: u8,
    pub ui_pending: u8,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TimerOutcome {
    pub player_state: Option<u8>,
    pub clear_late_route: bool,
    pub sound: Option<u8>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cost {
    RewardDialogue,
    TotalOverlay,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransferLayout {
    Stage,
    Selection,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferSession {
    pub source: [u8; 6],
    pub started: bool,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TransferOutcome {
    /// None preserves the caller submission fact on blank-source early return;
    /// a converted row batch returns FF from its exhausted total-digit loop.
    pub submission_value: Option<u8>,
    pub moved: u8,
    pub complete: bool,
    pub sound: Option<u8>,
    pub wait_ticks: u16,
    pub upload: Option<Vec<u8>>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConversionPhase {
    Entry,
    AfterAudioWait,
    Running,
    AfterFinishWait,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ConversionOutcome {
    pub wait_ticks: u16,
    pub stage_sound: Option<u8>,
    pub value_sound: Option<u8>,
    pub completion_increment: bool,
    pub freeze_player_animation: bool,
}
impl Status {
    pub fn add_reward(&mut self, digit: usize, amount: u8) {
        if add_codes(
            &mut self.reward[..=digit],
            amount,
            DigitTraversal::CarryOnly,
        ) {
            self.saturate_reward();
        }
        self.dirty = self.dirty.wrapping_add(1);
    }
    pub fn saturate_reward(&mut self) {
        self.reward[..4].fill(9);
    }
    pub fn saturate_total(&mut self) {
        self.total[..5].fill(9);
    }
    pub fn steal(&mut self, actor: &mut ActorContact) {
        actor.local_state = actor.local_state.wrapping_add(1);
        if subtract_codes(&mut self.reward[..3], 1) {
            self.reward.fill(BLANK);
        } else {
            blank_leading_zeroes(&mut self.reward);
        }
        self.dirty = self.dirty.wrapping_add(1);
    }
    pub fn pay(&mut self, cost: Cost, rejected: bool) -> bool {
        if rejected {
            return false;
        }
        match cost {
            Cost::RewardDialogue => self.pay_reward_dialogue().accepted(),
            Cost::TotalOverlay => {
                Self::pay_overlay_parts(&mut self.total, &mut self.health, &mut self.ui_pending)
            }
        }
    }
    /// The player control call already borrows canonical Health. Splitting the
    /// other two fields lets SELECT pay immediately without constructing another
    /// Status owner or deferring restoration until after later collision calls.
    pub fn pay_overlay_parts(
        total: &mut [u8; 7],
        health: &mut Health,
        ui_pending: &mut u8,
    ) -> bool {
        const PAYMENT: u8 = 3;
        let end = health
            .segments
            .iter()
            .position(|segment| segment.y & 0x80 != 0)
            .unwrap_or(OVERLAY_SEGMENTS);
        if end == 0 || health.segments[end - 1].depleted == 0 {
            return false;
        }
        let mut upper = [total[0], total[1]];
        if subtract_codes(&mut upper, PAYMENT) {
            return false;
        }
        total[..2].copy_from_slice(&upper);
        blank_literal_zero_prefix(total);
        health.restore();
        *ui_pending = ui_pending.wrapping_add(1);
        true
    }
    /// EE2F subtracts from the two upper display codes, then EE5A scans
    /// literal zeroes only. Do not reuse general display normalization: that
    /// skips existing blank codes and changes untouched lower display bytes.
    pub fn pay_reward_dialogue(&mut self) -> RewardPayment {
        const PRICE_UPPER_CODES: u8 = 3;
        let mut upper = [self.reward[0], self.reward[1]];
        if subtract_codes(&mut upper, PRICE_UPPER_CODES) {
            return RewardPayment::Underfunded;
        }
        self.reward[..2].copy_from_slice(&upper);
        let zeroes = blank_literal_zero_prefix(&mut self.reward);
        self.dirty = self.dirty.wrapping_add(1);
        if zeroes == 0 {
            RewardPayment::FirstCodeNonzero
        } else if zeroes == self.reward.len() {
            RewardPayment::AllCodesZero
        } else {
            RewardPayment::PartialZeroPrefix
        }
    }
    pub fn gain_life(&mut self) {
        if self.lives != MAX_LIVES {
            self.lives = self.lives.wrapping_add(1);
        }
    }
    pub fn timer_tick(&mut self, player_state: u8, composition_tick: u8) -> TimerOutcome {
        const DEATH: u8 = 0x0d;
        const COMPLETION: u8 = 0x10;
        const WARNING_TENS: u8 = 3;
        const WARNING_TICK_MASK: u8 = 0x0f;
        if matches!(player_state, DEATH | COMPLETION) {
            return TimerOutcome::default();
        }
        if self.countdown[0] == EXPIRED[0] {
            return TimerOutcome {
                player_state: Some(DEATH),
                clear_late_route: true,
                sound: None,
            };
        }
        self.divider = self.divider.wrapping_sub(1);
        if self.divider == 0 {
            self.divider = COUNTDOWN_PERIOD;
            subtract_codes(&mut self.countdown, 1);
            if self.countdown == [0; 3] {
                self.countdown = EXPIRED;
            }
        }
        TimerOutcome {
            sound: (self.countdown[0] == 0
                && self.countdown[1] < WARNING_TENS
                && composition_tick & WARNING_TICK_MASK == 0)
                .then_some(WARNING_SOUND),
            ..TimerOutcome::default()
        }
    }
    pub fn queue_reward(&mut self, primary_busy: bool) -> Option<Vec<u8>> {
        if primary_busy {
            return None;
        }
        if let Some(first) = self.reward.iter().position(|&code| code < BLANK) {
            for code in &mut self.reward[first..] {
                if *code == BLANK {
                    *code = 0;
                }
            }
        }
        self.dirty = 0;
        Some(number_upload(REWARD_UI_ADDRESS, &self.reward))
    }
    pub fn queue_ui(&mut self, primary_busy: bool) -> Option<Vec<u8>> {
        if primary_busy {
            return None;
        }
        self.ui_pending = 0;
        Some(number_upload(TOTAL_UI_ADDRESS, &self.total))
    }
    pub fn transfer_batch(
        &mut self,
        layout: TransferLayout,
        session: &mut TransferSession,
    ) -> TransferOutcome {
        if !session.started && self.reward[5] == BLANK {
            return TransferOutcome {
                complete: true,
                ..TransferOutcome::default()
            };
        }
        session.started = true;
        let mut moved = 0;
        loop {
            subtract_codes(&mut session.source[..4], 1);
            blank_leading_zeroes(&mut session.source);
            if add_codes(&mut self.total[..5], 1, DigitTraversal::WholeRow) {
                self.saturate_total();
            }
            self.total[5..].fill(0);
            moved += 1;
            if session.source[5] == BLANK || moved == TRANSFER_BATCH {
                break;
            }
        }
        let complete = session.source[5] == BLANK;
        let (total_address, reward_address) = match layout {
            TransferLayout::Stage => (0x220a, 0x226b),
            TransferLayout::Selection => (TOTAL_UI_ADDRESS, REWARD_UI_ADDRESS),
        };
        let mut upload = number_upload(total_address, &self.total);
        upload.pop();
        upload.extend(number_upload(reward_address, &session.source));
        TransferOutcome {
            submission_value: Some(u8::MAX),
            moved,
            complete,
            sound: (!complete).then_some(VALUE_SOUND),
            wait_ticks: if complete { 0 } else { TRANSFER_WAIT },
            upload: Some(upload),
        }
    }
    pub fn convert_timer(
        &mut self,
        phase: ConversionPhase,
        stage_sound: u8,
        composition_tick: u8,
    ) -> ConversionOutcome {
        let mut result = ConversionOutcome::default();
        if phase == ConversionPhase::AfterFinishWait {
            result.completion_increment = true;
            return result;
        }
        if phase == ConversionPhase::Entry && stage_sound & 0x80 == 0 {
            result.wait_ticks = AUDIO_SETTLE_WAIT;
            return result;
        }
        if phase == ConversionPhase::AfterAudioWait {
            result.stage_sound = Some(STOP_STAGE_SOUND);
        }
        if composition_tick & 1 == 0 {
            return result;
        }
        result.freeze_player_animation = true;
        self.countdown[2] = 0;
        if self.countdown[0] | self.countdown[1] == 0 {
            result.wait_ticks = CONVERSION_FINISH_WAIT;
            return result;
        }
        subtract_codes(&mut self.countdown[..2], 1);
        self.add_reward(3, 1);
        result.value_sound = Some(VALUE_SOUND);
        result
    }
}
pub fn option_reward(rule: TitleRule, animation: u8) -> u8 {
    const RESTORE: u8 = 0x25;
    const SPECIAL: u8 = 0x26;
    const DIGIT: u8 = 0x28;
    const RECOVER: u8 = 0x24;
    match (rule, animation) {
        (TitleRule::AlternateRewards, RESTORE) => DIGIT,
        (TitleRule::AlternateRewards, SPECIAL) => RECOVER,
        _ => animation,
    }
}
/// Programs resolve runtime fields by name. No source RAM/register representation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Fragment {
    Literal(Vec<u8>),
    Reward,
    Total,
    Conditional {
        progression: usize,
        mask: u8,
        present: Vec<u8>,
        absent: Vec<u8>,
    },
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StreamOp {
    Literal(u8),
    Repeat { tile: u8, count: usize },
    Insert(Fragment),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamRow {
    pub destination: u16,
    pub operations: Vec<StreamOp>,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StatusCatalog {
    pub streams: Vec<Vec<StreamRow>>,
    pub messages: Vec<Vec<(u16, usize)>>,
    pub payload_counts: Vec<Option<u8>>,
    pub payloads: Vec<Option<Vec<u8>>>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RowUpload {
    pub bytes: Vec<u8>,
    pub yield_after: bool,
}
impl StatusCatalog {
    pub fn decode_original(rom: &[u8]) -> Result<Self> {
        formats::decode_original(rom)
    }
    pub fn decode_stream(bytes: &[u8], fragments: &[Fragment]) -> Result<Vec<StreamRow>> {
        const ROW_END: u8 = 0xff;
        const REPEAT_MIN: u8 = 0x80;
        const INSERT_MIN: u8 = 0xa0;
        let mut cursor = 0;
        let mut rows = Vec::new();
        loop {
            let high = *bytes.get(cursor).context("status stream terminator")?;
            if high == 0 {
                return Ok(rows);
            }
            let low = *bytes.get(cursor + 1).context("status destination")?;
            cursor += 2;
            let mut operations = Vec::new();
            loop {
                let tag = *bytes.get(cursor).context("status row terminator")?;
                cursor += 1;
                if tag == ROW_END {
                    break;
                }
                operations.push(if tag < REPEAT_MIN {
                    StreamOp::Literal(tag)
                } else if tag < INSERT_MIN {
                    let tile = *bytes.get(cursor).context("repeat tile")?;
                    cursor += 1;
                    StreamOp::Repeat {
                        tile,
                        count: usize::from(tag & 0x7f) + 1,
                    }
                } else {
                    StreamOp::Insert(
                        fragments
                            .get(usize::from(tag & 0x0f))
                            .context("status fragment selector")?
                            .clone(),
                    )
                });
            }
            rows.push(StreamRow {
                destination: u16::from(high) << 8 | u16::from(low),
                operations,
            });
        }
    }
    pub fn expand(
        rows: &[StreamRow],
        status: &Status,
        progression: &[u8],
    ) -> Result<Vec<RowUpload>> {
        const STATUS_ROW_COUNT: u8 = 0x1d;
        rows.iter()
            .map(|row| {
                let mut bytes = vec![
                    (row.destination >> 8) as u8,
                    row.destination as u8,
                    STATUS_ROW_COUNT,
                ];
                for operation in &row.operations {
                    match operation {
                        StreamOp::Literal(tile) => bytes.push(*tile),
                        StreamOp::Repeat { tile, count } => {
                            bytes.extend(std::iter::repeat_n(*tile, *count))
                        }
                        StreamOp::Insert(fragment) => match fragment {
                            Fragment::Literal(tiles) => bytes.extend_from_slice(tiles),
                            Fragment::Reward => {
                                bytes.push(NUMBER_MARKER);
                                bytes.extend(status.reward);
                                bytes.push(0);
                            }
                            Fragment::Total => {
                                bytes.push(NUMBER_MARKER);
                                bytes.extend(status.total);
                                bytes.push(0);
                            }
                            Fragment::Conditional {
                                progression: index,
                                mask,
                                present,
                                absent,
                            } => {
                                let flags = *progression
                                    .get(*index)
                                    .context("status progression field")?;
                                bytes.extend_from_slice(if flags & mask != 0 {
                                    present
                                } else {
                                    absent
                                });
                            }
                        },
                    }
                }
                bytes.push(u8::MAX);
                Ok(RowUpload {
                    bytes,
                    yield_after: true,
                })
            })
            .collect()
    }
    pub fn update_streams(
        &self,
        selector: usize,
        player_state: u8,
        status: &Status,
        progression: &[u8],
    ) -> Result<Vec<RowUpload>> {
        const CLEAR_STREAM: usize = 4;
        const DIALOGUE_STREAM: usize = 9;
        let mut rows = Self::expand(
            self.streams
                .get(CLEAR_STREAM)
                .context("missing status clear stream")?,
            status,
            progression,
        )?;
        let selector = if player_state == crate::presentation::DIALOGUE_PLAYER_STATE {
            DIALOGUE_STREAM
        } else {
            selector
        };
        rows.extend(Self::expand(
            self.streams
                .get(selector)
                .context("status stream selector")?,
            status,
            progression,
        )?);
        Ok(rows)
    }
    /// Direct B81C entry used by B12 after completion, bypassing B800's actor
    /// clearing. The producer returns its exact pending-submission cursor.
    pub fn write_message(
        &self,
        selector: usize,
        stage: u8,
        status: &Status,
    ) -> Result<MessageOutput> {
        const TOTAL_PAYLOAD: usize = 10;
        const REWARD_PAYLOAD: usize = 11;
        let records = self
            .messages
            .get(selector)
            .context("status message selector")?;
        let mut bytes = Vec::new();
        for &(destination, original_selector) in records {
            let selector = if original_selector == 0 {
                usize::from(stage) + 2
            } else {
                original_selector
            };
            let payload = match selector {
                TOTAL_PAYLOAD => {
                    let mut payload = vec![NUMBER_MARKER];
                    payload.extend(status.total);
                    payload.push(0);
                    payload
                }
                REWARD_PAYLOAD => {
                    let mut payload = vec![NUMBER_MARKER];
                    payload.extend(status.reward);
                    payload.push(0);
                    payload
                }
                _ => self
                    .payloads
                    .get(selector)
                    .and_then(Option::as_ref)
                    .context("status payload selector")?
                    .clone(),
            };
            ensure!(
                !payload.is_empty() && payload.len() <= 129,
                "status payload bounds"
            );
            bytes.extend([
                (destination >> 8) as u8,
                destination as u8,
                (payload.len() - 1) as u8,
            ]);
            bytes.extend(payload);
        }
        let submission_value = bytes.len() as u8;
        bytes.push(u8::MAX);
        Ok(MessageOutput {
            bytes,
            submission_value,
        })
    }
    pub fn build_message(
        &self,
        completion_actor: u8,
        scene_complete: u8,
        special_room: u8,
        stage: u8,
        actors: &mut crate::actors::ActorPool,
        status: &Status,
    ) -> Result<Vec<u8>> {
        let selector = if completion_actor != 0 {
            usize::from(completion_actor)
        } else if scene_complete == 0 {
            0
        } else if special_room == 0 {
            1
        } else {
            2
        };
        if selector != 0 {
            for slot in [0, 2, 3, 4] {
                actors[slot].state = 0;
            }
        }
        Ok(self.write_message(selector, stage, status)?.bytes)
    }
}
#[cfg(test)]
mod contracts;

/// One prepared primary upload and its source submission value. The value is
/// retained across B12 acknowledgement waits; it is not a bool or audio cursor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageOutput {
    pub bytes: Vec<u8>,
    pub submission_value: u8,
}

const DECIMAL_RADIX: u8 = 10;
const REWARD_UI_ADDRESS: u16 = 0x2043;
const TOTAL_UI_ADDRESS: u16 = 0x206c;
const NUMBER_MARKER: u8 = 0x2c;
/// Display blanks behave as zero for subtraction. Return true on final borrow;
/// callers choose transactional rejection, blank exhaustion or literal wrap.
fn subtract_codes(codes: &mut [u8], amount: u8) -> bool {
    let mut subtraction = amount;
    for code in codes.iter_mut().rev() {
        let digit = if *code == BLANK { 0 } else { *code };
        let (value, borrow) = digit.overflowing_sub(subtraction);
        *code = if borrow {
            value.wrapping_add(DECIMAL_RADIX)
        } else {
            value
        };
        subtraction = u8::from(borrow);
    }
    subtraction != 0
}
/// Add with encoded blank normalization and decimal carry. Leading untouched
/// blanks remain blank; a carry beyond the supplied significant width saturates
/// only when the calling operation requests it.
/// Reward additions stop when carry settles. The transfer loop deliberately
/// visits the entire row, including unusual retained display codes.
#[derive(Clone, Copy, Eq, PartialEq)]
enum DigitTraversal {
    CarryOnly,
    WholeRow,
}
fn add_codes(codes: &mut [u8], amount: u8, traversal: DigitTraversal) -> bool {
    let mut carry = amount;
    for code in codes.iter_mut().rev() {
        let mut value = code.wrapping_add(carry);
        if value == BLANK {
            *code = value;
            carry = 0;
            if traversal == DigitTraversal::CarryOnly {
                return false;
            }
            continue;
        }
        if value > BLANK {
            value = value.wrapping_sub(BLANK);
        }
        carry = u8::from(value >= DECIMAL_RADIX);
        *code = if carry != 0 {
            value - DECIMAL_RADIX
        } else {
            value
        };
        if carry == 0 && traversal == DigitTraversal::CarryOnly {
            return false;
        }
    }
    carry != 0
}
/// Payment routines stop at every nonzero tile code, including an existing
/// blank. Their traversal differs from general number display normalization.
fn blank_literal_zero_prefix(codes: &mut [u8]) -> usize {
    let zeroes = codes.iter().take_while(|&&code| code == 0).count();
    codes[..zeroes].fill(BLANK);
    zeroes
}
fn blank_leading_zeroes(codes: &mut [u8]) {
    for code in codes {
        if *code == 0 {
            *code = BLANK;
        } else if *code != BLANK {
            break;
        }
    }
}
fn number_upload(address: u16, codes: &[u8]) -> Vec<u8> {
    let mut bytes = vec![
        (address >> 8) as u8,
        address as u8,
        (codes.len() + 1) as u8,
        NUMBER_MARKER,
    ];
    bytes.extend_from_slice(codes);
    bytes.extend([0, u8::MAX]);
    bytes
}

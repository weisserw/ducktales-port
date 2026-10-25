//! Resumable text and progression-contact policies (B15).
//!
//! Source evidence and exact continuation ordering are in trace.md. State owned
//! elsewhere is always borrowed; no CPU registers, stack or RAM image is stored.
use crate::{
    actors::{ActorPool, SUPPRESSION_BYTES},
    collision::Movement,
    player::Player,
    progression::{OwnerStep, Progression},
    runtime::{Buttons, PortInput},
    status::Status,
    world::{Camera, WorldCache},
};
use anyhow::{Context as _, Result, ensure};

pub const MESSAGE_COUNT: usize = 18;
pub const TEXT_ROWS: usize = 3;
pub const ROW_GLYPHS: usize = 17;
pub const GLYPH_WAIT: u16 = 3;
pub const GLYPH_SOUND: u8 = 0x2e;
pub const BLANK: u8 = 0x24;
pub const MESSAGE_END: u8 = 0xae;
pub const ROW_END: u8 = 0xff;
pub const FINAL_GLYPH: u8 = 0x2e;
pub const POINTERS_PHYSICAL: usize = 0x13898;
pub const STRINGS_PHYSICAL: usize = 0x138bc;
pub const STRINGS_END_PHYSICAL: usize = 0x13c59;
pub const LINES_PHYSICAL: usize = 0x1ef69;
pub const CLEAR_PANEL_PHYSICAL: usize = 0xa900;
pub const RESTORE_PANEL_PHYSICAL: usize = 0x42e0;
pub const PANEL_TEMPLATE_BYTES: usize = 16;
pub const PANEL_DESTINATION: u16 = 0x12d0;
pub const ESCORT_DISTANCE: u8 = 0x38;
pub const ESCORT_PAUSE: u8 = 0xb4;
pub const COMPANION_ANIMATION: u8 = 0x7e;
pub const COMPANION_HANDLER: u8 = 0x3f;
pub const TRANSITION_END_PAGE: u8 = 7;
pub const TRANSITION_END_X: u8 = 0x70;
pub const TRANSITION_SPEED: u16 = 0x0300;
pub const ESCORT_SPEED: u16 = 0x0180;
pub const HIT_LAUNCH_SPEED: u16 = 0x0100;
pub const HIT_LAUNCH_VELOCITY: i16 = 0x054b;
pub const REVEAL_HANDLER: u8 = 8;
pub const REVEAL_ANIMATION: u8 = 0x1b;
pub const REVEAL_PHASE: u8 = 0x0b;
pub const REVEAL_X: u8 = 0x30;
pub const REVEAL_Y: u8 = 0xb8;
pub const PAID_HANDLER: u8 = 0x0f;
pub const PAID_POSITION: u8 = 0x80;
pub const CHOICE_ANIMATION: u8 = 0x33;
pub const CHOICE_REQUEST: u8 = 0x81;
pub const ALL_LANDS: u8 = 0x1f;
pub const COMPLETION_CONTACT_SELECTOR: u8 = 4;
/// Source05A1/05A9 relative to TerrainCollisionCache0580, not attribute base0500.
pub const REVEAL_CACHE_CELLS: [usize; 2] = [0x21, 0x29];

/// Original encoded selectors, named only for outcomes established by consumers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum OutcomeMessage {
    Boundary = 3,
    Reveal = 6,
    EscortPrerequisite = 8,
    PaidReward = 10,
    RaceIntro = 11,
    LifeLoss = 12,
    StageChoice = 15,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Catalog {
    pub messages: Vec<Vec<u8>>,
    pub destinations: [u16; TEXT_ROWS],
    pub row_end_offsets: [u8; TEXT_ROWS],
    pub row_queue_offsets: [u8; TEXT_ROWS],
    pub clear_panel: Vec<u8>,
    pub restore_panel: Vec<u8>,
}
impl Catalog {
    pub fn original(rom: &[u8]) -> Result<Self> {
        let data = |start: usize, count: usize| {
            rom.get(start..start + count)
                .context("dialogue recipe bounds")
        };
        let lines = data(LINES_PHYSICAL, 9)?;
        Ok(Self {
            messages: Self::decode_messages(
                data(POINTERS_PHYSICAL, MESSAGE_COUNT * 2)?,
                data(STRINGS_PHYSICAL, STRINGS_END_PHYSICAL - STRINGS_PHYSICAL)?,
            )?,
            destinations: std::array::from_fn(|i| 0x2000 | u16::from(lines[i])),
            row_end_offsets: lines[3..6].try_into()?,
            row_queue_offsets: lines[6..9].try_into()?,
            clear_panel: data(CLEAR_PANEL_PHYSICAL, PANEL_TEMPLATE_BYTES)?.to_vec(),
            restore_panel: data(RESTORE_PANEL_PHYSICAL, PANEL_TEMPLATE_BYTES)?.to_vec(),
        })
    }
    pub fn decode_messages(pointers: &[u8], strings: &[u8]) -> Result<Vec<Vec<u8>>> {
        ensure!(
            pointers.len() == MESSAGE_COUNT * 2,
            "dialogue pointer plane extent"
        );
        let mut messages = Vec::new();
        for i in 0..MESSAGE_COUNT {
            let pointer =
                usize::from(pointers[i]) | (usize::from(pointers[MESSAGE_COUNT + i]) << 8);
            let offset = pointer
                .checked_sub(0xb8ac)
                .context("dialogue pointer before strings")?;
            let tail = strings
                .get(offset..)
                .context("dialogue pointer outside strings")?;
            let end = tail
                .iter()
                .position(|&byte| byte == MESSAGE_END)
                .context("unterminated dialogue message")?;
            messages.push(tail[..=end].to_vec());
        }
        Ok(messages)
    }
}
/// Borrow the one B03-owned prepared queue. Dialogue owns no second pending byte,
/// encoded buffer or retained row copy. B01 interrupt commits this same queue.
pub use crate::presentation::PreparedPrimary as Panel;
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Phase {
    #[default]
    Entry,
    RaceTerrain,
    PreparePanel,
    ClearPanelYield,
    Compose,
    StatusStreams,
    AddSuppression,
    Render,
    GlyphWait,
    AwaitInput,
    ScrollYield,
    RestoreStatus,
    RestorePanelYield,
    Outcome,
    Done,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Dialogue {
    pub message: u8,
    pub actor_slot: usize,
    pub line: u8,
    pub row_glyphs: u8,
    pub phase: Phase,
    /// Full saved primary-port edge byte survives nested status and panel yields.
    pub acknowledgment: Option<Buttons>,
    pub pending_wait: Option<Work>,
    pub after_scroll: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextBoundary {
    GlyphWait,
    PageComplete,
    MessageComplete,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextStep {
    pub boundary: TextBoundary,
    pub sound: Option<u8>,
    pub wait_ticks: u16,
}
/// Runs until the next real wait/page/message boundary. Blanks do not yield.
/// Source cursor is canonical Progression.dialogue_cursor, not a second counter.
pub fn render_text(
    _dialogue: &mut Dialogue,
    _cursor: &mut u8,
    _panel: &mut Panel,
    _catalog: &Catalog,
) -> Result<TextStep> {
    let message = _catalog
        .messages
        .get(usize::from(_dialogue.message))
        .context("dialogue selector bounds")?;
    _panel.encoded.resize(_panel.encoded.len().max(61), 0);
    loop {
        let line = usize::from(_dialogue.line);
        ensure!(line < TEXT_ROWS, "dialogue line bounds");
        let start = usize::from(_catalog.row_queue_offsets[line]);
        if _dialogue.row_glyphs == 0 {
            _panel.encoded[start] = (_catalog.destinations[line] >> 8) as u8;
            _panel.encoded[start + 1] = _catalog.destinations[line] as u8;
        }
        let glyph = *message
            .get(usize::from(*_cursor))
            .context("dialogue cursor bounds")?;
        let at = start + 3 + usize::from(_dialogue.row_glyphs);
        ensure!(at + 1 < _panel.encoded.len(), "dialogue row payload bounds");
        if glyph == MESSAGE_END {
            _panel.encoded[at] = FINAL_GLYPH;
            _panel.encoded[at + 1] = ROW_END;
            _panel.encoded[start + 2] = _dialogue.row_glyphs;
            _panel.pending = ROW_END;
            *_cursor = 0;
            return Ok(TextStep {
                boundary: TextBoundary::MessageComplete,
                sound: None,
                wait_ticks: 0,
            });
        }
        let mut finish_row = glyph == ROW_END;
        if !finish_row {
            _panel.encoded[at] = glyph;
            _dialogue.row_glyphs = _dialogue.row_glyphs.wrapping_add(1);
            *_cursor = _cursor.wrapping_add(1);
            if glyph == BLANK {
                continue;
            }
            finish_row = _dialogue.row_glyphs == ROW_GLYPHS as u8;
            if !finish_row {
                _panel.encoded[at + 1] = ROW_END;
                _panel.encoded[start + 2] = _dialogue.row_glyphs.wrapping_sub(1);
                _panel.pending = ROW_END;
                return Ok(TextStep {
                    boundary: TextBoundary::GlyphWait,
                    sound: Some(GLYPH_SOUND),
                    wait_ticks: GLYPH_WAIT,
                });
            }
        } else {
            *_cursor = _cursor.wrapping_add(1);
        }
        if finish_row {
            while _dialogue.row_glyphs < ROW_GLYPHS as u8 {
                let at = start + 3 + usize::from(_dialogue.row_glyphs);
                _panel.encoded[at] = BLANK;
                _dialogue.row_glyphs += 1;
            }
            _panel.encoded[usize::from(_catalog.row_end_offsets[line])] = ROW_END;
            _panel.encoded[start + 2] = _dialogue.row_glyphs.wrapping_sub(1);
            _panel.pending = ROW_END;
            if line == 2 {
                return Ok(TextStep {
                    boundary: TextBoundary::PageComplete,
                    sound: None,
                    wait_ticks: 0,
                });
            }
            _dialogue.line += 1;
            _dialogue.row_glyphs = 0;
        }
    }
}
pub fn scroll_rows(_dialogue: &mut Dialogue, _panel: &mut Panel) {
    _panel.encoded.resize(_panel.encoded.len().max(61), 0);
    for index in 0..ROW_GLYPHS {
        _panel.encoded[3 + index] = _panel.encoded[23 + index];
        _panel.encoded[23 + index] = _panel.encoded[43 + index];
        _panel.encoded[43 + index] = BLANK;
    }
    _panel.pending = ROW_END;
    _dialogue.phase = Phase::ScrollYield;
}

pub struct Context<'a> {
    pub dialogue: &'a mut Dialogue,
    pub progression: &'a mut Progression,
    pub player: &'a mut Player,
    pub actors: &'a mut ActorPool,
    pub status: &'a mut Status,
    pub cache: &'a mut WorldCache,
    pub camera: &'a Camera,
    pub suppression: &'a mut [u8; SUPPRESSION_BYTES],
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Work {
    BuildRaceTerrain,
    SelectGraphicsTwo,
    ClearPanel,
    Yield,
    SelectPresentation,
    HideSprites,
    Compose,
    StatusStreams,
    /// B76A entry: OR current rules WITHOUT clearing the existing suppression set.
    AddSuppression,
    SelectTextBank,
    SelectGraphicsOne,
    RestorePanel,
    Sound(u8),
    Wait(u16),
    ChangedBurst(u8),
    RestoreActorBank,
}
/// The final source operation of an outcome, consumed by the enclosing player
/// call before the first actor's random addition. Reveal's tail audio submission
/// is reported by the actual ChangedBurst owner instead.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutcomeReturn {
    MatchedDispatch,
    UnmatchedDispatch(u8),
    PaymentRejected,
    Payment(crate::status::RewardPayment),
    LifeLossVariantOverflow(bool),
}
impl OutcomeReturn {
    pub fn first_actor_increment(self) -> crate::progression::FirstActorIncrement {
        use crate::progression::FirstActorIncrement::{One, Zero};
        match self {
            Self::MatchedDispatch | Self::PaymentRejected => One,
            Self::UnmatchedDispatch(message) => {
                if message >= OutcomeMessage::RaceIntro as u8 {
                    One
                } else {
                    Zero
                }
            }
            Self::Payment(payment) => payment.first_actor_increment(),
            Self::LifeLossVariantOverflow(overflow) => {
                if overflow {
                    One
                } else {
                    Zero
                }
            }
        }
    }
}
/// Owners commit external effects before return. Suspended operations are polled
/// at the same continuation, preserving source ordering through nested yields.
pub trait Systems {
    fn outcome_return(&mut self, _outcome: OutcomeReturn) {}
    fn perform(&mut self, work: Work, context: &mut Context<'_>) -> OwnerStep<()>;
    /// The running dialogue lends its actual canonical primary queue to nested
    /// status/burst owners. No second panel or pending-buffer copy is retained.
    fn perform_with_panel(
        &mut self,
        work: Work,
        context: &mut Context<'_>,
        _panel: &mut Panel,
    ) -> OwnerStep<()> {
        self.perform(work, context)
    }
    fn contact(&mut self, actors: &mut ActorPool, player: &mut Player, slot: usize) -> bool;
    fn distance(&mut self, actors: &ActorPool, slot: usize) -> u8;
    // Separate borrows keep canonical owners visible across immediate callbacks.
    #[allow(clippy::too_many_arguments)]
    fn motion(
        &mut self,
        actors: &mut ActorPool,
        player: &mut Player,
        cache: &WorldCache,
        slot: usize,
        movement: Movement,
        start: u8,
        remaining: u8,
    ) -> u8;
    fn integrate(&mut self, actors: &mut ActorPool, slot: usize, positive: bool);
    /// B10 allocation examines the live pool; only the selected slot is changed.
    fn allocate(&mut self, actors: &ActorPool) -> Option<usize>;
}
pub fn resume(
    _context: &mut Context<'_>,
    _input: &mut PortInput,
    _panel: &mut Panel,
    _catalog: &Catalog,
    _systems: &mut dyn Systems,
) -> Result<OwnerStep<()>> {
    if let Some(work) = _context.dialogue.pending_wait {
        if _systems.perform_with_panel(work, _context, _panel) == OwnerStep::Suspended {
            return Ok(OwnerStep::Suspended);
        }
        _context.dialogue.pending_wait = None;
    }
    loop {
        match _context.dialogue.phase {
            Phase::Entry => {
                _context.dialogue.phase =
                    if _context.dialogue.message == OutcomeMessage::RaceIntro as u8 {
                        Phase::RaceTerrain
                    } else {
                        Phase::PreparePanel
                    };
            }
            Phase::RaceTerrain => {
                if _systems.perform_with_panel(Work::BuildRaceTerrain, _context, _panel)
                    == OwnerStep::Suspended
                {
                    return Ok(OwnerStep::Suspended);
                }
                _context.dialogue.phase = Phase::PreparePanel;
            }
            Phase::PreparePanel => {
                leaf_with_panel(_systems, Work::SelectGraphicsTwo, _context, _panel);
                prepare_panel(_panel, &_catalog.clear_panel)?;
                leaf_with_panel(_systems, Work::ClearPanel, _context, _panel);
                _context.dialogue.phase = Phase::ClearPanelYield;
                request_wait_with_panel(_systems, Work::Yield, _context, _panel);
                return Ok(OwnerStep::Suspended);
            }
            Phase::ClearPanelYield => {
                _context.progression.dialogue_cursor = 0;
                _context.dialogue.line = 0;
                _context.dialogue.row_glyphs = 0;
                leaf_with_panel(_systems, Work::SelectPresentation, _context, _panel);
                leaf_with_panel(_systems, Work::HideSprites, _context, _panel);
                _context.dialogue.phase = Phase::Compose;
            }
            Phase::Compose => {
                if _systems.perform_with_panel(Work::Compose, _context, _panel)
                    == OwnerStep::Suspended
                {
                    return Ok(OwnerStep::Suspended);
                }
                _context.dialogue.phase = Phase::StatusStreams;
            }
            Phase::StatusStreams => {
                if _systems.perform_with_panel(Work::StatusStreams, _context, _panel)
                    == OwnerStep::Suspended
                {
                    return Ok(OwnerStep::Suspended);
                }
                _context.dialogue.phase = Phase::AddSuppression;
            }
            Phase::AddSuppression => {
                leaf_with_panel(_systems, Work::AddSuppression, _context, _panel);
                leaf_with_panel(_systems, Work::SelectTextBank, _context, _panel);
                _context.dialogue.phase = Phase::Render;
            }
            Phase::GlyphWait => _context.dialogue.phase = Phase::Render,
            Phase::ScrollYield => {
                _context.dialogue.line = 2;
                _context.dialogue.row_glyphs = 0;
                _context.dialogue.after_scroll = true;
                _context.dialogue.phase = Phase::Render;
            }
            Phase::Render => {
                let step = render_text(
                    _context.dialogue,
                    &mut _context.progression.dialogue_cursor,
                    _panel,
                    _catalog,
                )?;
                if step.boundary == TextBoundary::GlyphWait {
                    if let Some(sound) = step.sound {
                        leaf_with_panel(_systems, Work::Sound(sound), _context, _panel);
                    }
                    _context.dialogue.phase = Phase::GlyphWait;
                    request_wait_with_panel(
                        _systems,
                        Work::Wait(step.wait_ticks),
                        _context,
                        _panel,
                    );
                    return Ok(OwnerStep::Suspended);
                }
                _context.dialogue.phase = Phase::AwaitInput;
                if _context.dialogue.after_scroll {
                    _context.dialogue.after_scroll = false;
                    request_wait_with_panel(_systems, Work::Yield, _context, _panel);
                    return Ok(OwnerStep::Suspended);
                }
            }
            Phase::AwaitInput => {
                if _input.pressed.0 & 0xc0 == 0 {
                    request_wait_with_panel(_systems, Work::Yield, _context, _panel);
                    return Ok(OwnerStep::Suspended);
                }
                if _context.progression.dialogue_cursor != 0 {
                    scroll_rows(_context.dialogue, _panel);
                    request_wait_with_panel(_systems, Work::Yield, _context, _panel);
                    return Ok(OwnerStep::Suspended);
                }
                _context.dialogue.acknowledgment = Some(_input.pressed);
                _context.player.action = crate::player::Action::Standing;
                leaf_with_panel(_systems, Work::SelectPresentation, _context, _panel);
                _context.dialogue.phase = Phase::RestoreStatus;
            }
            Phase::RestoreStatus => {
                if _systems.perform_with_panel(Work::StatusStreams, _context, _panel)
                    == OwnerStep::Suspended
                {
                    return Ok(OwnerStep::Suspended);
                }
                leaf_with_panel(_systems, Work::SelectGraphicsOne, _context, _panel);
                prepare_panel(_panel, &_catalog.restore_panel)?;
                leaf_with_panel(_systems, Work::RestorePanel, _context, _panel);
                _context.dialogue.phase = Phase::RestorePanelYield;
                request_wait_with_panel(_systems, Work::Yield, _context, _panel);
                return Ok(OwnerStep::Suspended);
            }
            Phase::RestorePanelYield => {
                _input.pressed = _context
                    .dialogue
                    .acknowledgment
                    .expect("saved full acknowledgment edge");
                _context.dialogue.phase = Phase::Outcome;
            }
            Phase::Outcome => {
                apply_outcome_with_panel(_context, _input.pressed, _systems, Some(_panel));
                _context.dialogue.phase = Phase::Done;
                return Ok(OwnerStep::Complete(()));
            }
            Phase::Done => return Ok(OwnerStep::Complete(())),
        }
    }
}
pub fn apply_outcome(_context: &mut Context<'_>, _pressed: Buttons, _systems: &mut dyn Systems) {
    apply_outcome_with_panel(_context, _pressed, _systems, None);
}
fn apply_outcome_with_panel(
    _context: &mut Context<'_>,
    _pressed: Buttons,
    _systems: &mut dyn Systems,
    panel: Option<&mut Panel>,
) {
    let message = _context.dialogue.message;
    let slot = _context.dialogue.actor_slot;
    match message {
        0x0a => {
            if _pressed.0 & Buttons::B.0 != 0 {
                _systems.outcome_return(OutcomeReturn::PaymentRejected);
                return;
            }
            let payment = _context.status.pay_reward_dialogue();
            _systems.outcome_return(OutcomeReturn::Payment(payment));
            if !payment.accepted() {
                return;
            }
            let actor = &mut _context.actors[slot];
            actor.handler = PAID_HANDLER;
            actor.state &= 0xe0;
            actor.position.world_x = (actor.position.world_x & 0xff00) | u16::from(PAID_POSITION);
            actor.position.screen_y = PAID_POSITION;
            actor.profile |= 0x80;
        }
        0x0c => {
            let (base, first_overflow) = _context
                .progression
                .stage
                .0
                .overflowing_add(crate::player::LIFE_LOSS_REQUEST);
            let variant_sum = u16::from(base)
                + u16::from(_context.progression.life_loss_variant)
                + u16::from(first_overflow);
            _systems.outcome_return(OutcomeReturn::LifeLossVariantOverflow(
                variant_sum > u16::from(u8::MAX),
            ));
            let effects = _context.player.commit_life_loss(
                &mut _context.actors[0],
                _context.progression.stage.0,
                _context.progression.life_loss_variant,
                &mut _context.status.health,
            );
            _context.progression.reload_request = effects.reload.expect("life loss request");
        }
        6 => {
            let actor = &mut _context.actors[slot];
            actor.state = 0xa0;
            actor.handler = REVEAL_HANDLER;
            actor.animation = REVEAL_ANIMATION;
            actor.position.screen_y = REVEAL_Y;
            actor.position.world_x = (actor.position.world_x & 0xff00) | u16::from(REVEAL_X);
            actor.animation_frame = 0;
            actor.animation_tick = 0;
            actor.hit_marker = 0;
            actor.local_state = REVEAL_PHASE;
            for cell in REVEAL_CACHE_CELLS {
                _context.cache.collision[cell] = 0;
            }
            if let Some(panel) = panel {
                leaf_with_panel(_systems, Work::ChangedBurst(0), _context, panel);
            } else {
                leaf(_systems, Work::ChangedBurst(0), _context);
            }
        }
        0x0f => {
            _systems.outcome_return(OutcomeReturn::MatchedDispatch);
            if _pressed.0 & Buttons::B.0 == 0 {
                let index = _context.progression.stage.0;
                _context.progression.accepted_stages |= 1u8 << index;
                _context.progression.special_room = CHOICE_REQUEST;
            }
        }
        3 => {
            _systems.outcome_return(OutcomeReturn::MatchedDispatch);
            if _context.progression.reward_progress & 2 == 0 {
                _context.progression.boundary_request =
                    _context.progression.boundary_request.wrapping_add(1);
            }
        }
        0x0b => {
            _systems.outcome_return(OutcomeReturn::MatchedDispatch);
            _context.progression.race_intro = 0;
        }
        _ => _systems.outcome_return(OutcomeReturn::UnmatchedDispatch(message)),
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Family {
    ProgressionContact = 0x2f,
    DialogueContact = 0x35,
    HitActivated = 0x3d,
    Escort = 0x3e,
    RightwardTransition = 0x3f,
    ConditionalMarker = 0x41,
}
pub fn update_actor(
    _family: Family,
    _slot: usize,
    _context: &mut Context<'_>,
    _systems: &mut dyn Systems,
) {
    use crate::player::Action;
    let slot = _slot;
    match _family {
        Family::ProgressionContact => {
            if !_systems.contact(_context.actors, _context.player, slot) {
                return;
            }
            let selector = _context.actors[slot].local_state;
            if selector == 0 && _context.progression.seen_messages[0] & 8 == 0 {
                return;
            }
            if selector == COMPLETION_CONTACT_SELECTOR
                && _context.progression.progress_bits & ALL_LANDS != ALL_LANDS
            {
                _context.progression.boundary_request = 0x80;
            } else {
                _context.progression.reload_request = selector | 0x80;
            }
        }
        Family::DialogueContact => {
            if matches!(_context.player.action, Action::Damage | Action::Death)
                || _context.actors[slot].state & 15 != 0
            {
                return;
            }
            if !_systems.contact(_context.actors, _context.player, slot) {
                return;
            }
            let stage = _context.progression.stage.0;
            if _context.actors[slot].animation == CHOICE_ANIMATION
                && _context.progression.accepted_stages & (1u8 << stage) != 0
            {
                return;
            }
            _context.player.action = Action::Dialogue;
            _context.actors[slot].state = _context.actors[slot].state.wrapping_add(1);
            let message = _context.actors[slot].local_state;
            _context.dialogue.message = message;
            _context.dialogue.actor_slot = slot;
            if message == OutcomeMessage::EscortPrerequisite as u8 {
                _context.progression.escort_prerequisite =
                    _context.progression.escort_prerequisite.wrapping_add(1);
                _context.actors[slot].state = 0;
                suppress(_context, slot);
            }
            _context.progression.seen_messages[usize::from(message >> 3)] |= 1u8 << (message & 7);
        }
        Family::HitActivated => {
            if _context.actors[slot].state & 15 == 0 {
                if _context.actors[slot].hit_marker == 0 {
                    _context.actors[slot].animation_tick = 0;
                    return;
                }
                _context.progression.hit_dialogue_count =
                    _context.progression.hit_dialogue_count.wrapping_add(1);
                _context.actors[slot].vertical_velocity = 0;
                _context.actors[slot].local_counter = 0;
                _context.actors[slot].profile = 0x80;
                _context.actors[slot].state = _context.actors[slot].state.wrapping_add(1);
            }
            if _context.actors[slot].state & 2 == 0 {
                if _systems.motion(
                    _context.actors,
                    _context.player,
                    _context.cache,
                    slot,
                    Movement::Fall,
                    0x2d,
                    1,
                ) & 1
                    != 0
                {
                    _context.actors[slot].horizontal_magnitude = HIT_LAUNCH_SPEED;
                    _context.actors[slot].vertical_velocity = HIT_LAUNCH_VELOCITY;
                    _context.actors[slot].state = _context.actors[slot].state.wrapping_add(1);
                }
            } else {
                if _context.actors[slot].local_counter == 0 {
                    _systems.motion(
                        _context.actors,
                        _context.player,
                        _context.cache,
                        slot,
                        Movement::Rise,
                        0x2f,
                        1,
                    );
                    if _context.actors[slot].vertical_velocity < 0 {
                        _context.actors[slot].local_counter =
                            _context.actors[slot].local_counter.wrapping_add(1);
                        _context.actors[slot].vertical_velocity = 0;
                    }
                } else if _systems.motion(
                    _context.actors,
                    _context.player,
                    _context.cache,
                    slot,
                    Movement::Fall,
                    0x2d,
                    1,
                ) & 1
                    != 0
                {
                    _context.actors[slot].handler = 0;
                    _context.player.action = Action::Dialogue;
                    _context.progression.seen_messages[0] |= 0x40;
                    _context.dialogue.message = _context.actors[slot].local_state;
                    _context.dialogue.actor_slot = slot;
                    suppress(_context, slot);
                }
                _systems.motion(
                    _context.actors,
                    _context.player,
                    _context.cache,
                    slot,
                    Movement::NegativeX,
                    0x2b,
                    1,
                );
            }
            _context.actors[slot].animation_tick = 0;
        }
        Family::Escort => {
            if _context.actors[slot].state & 15 == 0 {
                if _systems.distance(_context.actors, slot) >= ESCORT_DISTANCE
                    || _context.progression.escort_prerequisite == 0
                {
                    return;
                }
                _context.progression.escort_prerequisite |= 0x80;
                suppress(_context, slot);
                _context.player.action = Action::Hold;
                _context.actors[slot].position.world_x = _context.camera.x;
                _context.actors[slot].state &= !0x10;
                _context.actors[slot].state = _context.actors[slot].state.wrapping_add(1);
                _context.actors[slot].horizontal_magnitude = ESCORT_SPEED;
                _context.actors[slot].profile = 0;
                return;
            }
            if _context.actors[slot].state & 2 == 0 {
                _systems.integrate(_context.actors, slot, true);
                _context.actors[slot].render |= 0x40;
                if _systems.distance(_context.actors, slot) >= ESCORT_DISTANCE {
                    return;
                }
                _context.actors[slot].state = _context.actors[slot].state.wrapping_add(1);
                _context.actors[slot].local_state = ESCORT_PAUSE;
                if let Some(child) = _systems.allocate(_context.actors) {
                    let parent = _context.actors[slot];
                    let actor = &mut _context.actors[child];
                    actor.animation = COMPANION_ANIMATION;
                    actor.position.screen_y = parent.position.screen_y;
                    actor.position.world_x = parent.position.world_x;
                    actor.animation_frame = 0;
                    actor.animation_tick = 0;
                    actor.state = 0xa0;
                    actor.render = 0xa0;
                    actor.handler = COMPANION_HANDLER;
                }
                return;
            }
            if _context.actors[slot].local_state != 0 {
                _context.actors[slot].local_state -= 1;
                return;
            }
            if _context.actors[slot].render & 0x80 == 0 {
                _context.actors[slot].state = 0;
                return;
            }
            if _systems.motion(
                _context.actors,
                _context.player,
                _context.cache,
                slot,
                Movement::Fall,
                6,
                2,
            ) & 1
                != 0
            {
                _systems.integrate(_context.actors, slot, false);
                _context.actors[slot].render &= !0x40;
            }
        }
        Family::RightwardTransition => {
            if _context.actors[slot].state & 15 == 0 {
                _context.actors[slot].horizontal_magnitude = TRANSITION_SPEED;
            }
            _systems.integrate(_context.actors, slot, true);
            let x = _context.actors[slot].position.world_x;
            if x >> 8 == u16::from(TRANSITION_END_PAGE) && x as u8 >= TRANSITION_END_X {
                _context.actors[slot].state = 0;
                _context.player.action = Action::Standing;
                leaf(_systems, Work::ChangedBurst(6), _context);
                leaf(_systems, Work::RestoreActorBank, _context);
            }
        }
        Family::ConditionalMarker => {
            if _context.progression.reward_progress & _context.progression.stage.0 == 0 {
                return;
            }
            _context.actors[slot].profile = 0;
            if _systems.contact(_context.actors, _context.player, slot) {
                _context.actors[0].render |= 4;
            }
        }
    }
}
#[cfg(test)]
mod contracts;

fn leaf(systems: &mut dyn Systems, work: Work, context: &mut Context<'_>) {
    assert_eq!(
        systems.perform(work, context),
        OwnerStep::Complete(()),
        "source leaf operation suspended"
    );
}

fn prepare_panel(panel: &mut Panel, template: &[u8]) -> Result<()> {
    let payload = template
        .get(..PANEL_TEMPLATE_BYTES)
        .context("panel template extent")?;
    // The source writes only this prefix. Retain later queue bytes because
    // audio's indexed low-F reset can consume the same canonical storage.
    let length = PANEL_TEMPLATE_BYTES + 4;
    panel.encoded.resize(panel.encoded.len().max(length), 0);
    panel.encoded[..3].copy_from_slice(&[
        (PANEL_DESTINATION >> 8) as u8,
        PANEL_DESTINATION as u8,
        15,
    ]);
    panel.encoded[3..3 + PANEL_TEMPLATE_BYTES].copy_from_slice(payload);
    panel.encoded[length - 1] = ROW_END;
    panel.pending = ROW_END;
    Ok(())
}
fn request_wait_with_panel(
    systems: &mut dyn Systems,
    work: Work,
    context: &mut Context<'_>,
    panel: &mut Panel,
) {
    if systems.perform_with_panel(work, context, panel) == OwnerStep::Suspended {
        context.dialogue.pending_wait = Some(work);
    }
}

fn suppress(context: &mut Context<'_>, slot: usize) {
    if let Some(identity) = context.actors[slot].spawn_identity {
        crate::actors::suppress(context.suppression, identity);
    }
}

fn leaf_with_panel(
    systems: &mut dyn Systems,
    work: Work,
    context: &mut Context<'_>,
    panel: &mut Panel,
) {
    assert_eq!(
        systems.perform_with_panel(work, context, panel),
        OwnerStep::Complete(()),
        "source leaf operation suspended"
    );
}

//! Independent B15 source contracts. trace.md derives every branch and table.
//! Recorder supplies only external owner results; B15 branch/mutation/sequence is
//! the pending production method under test. No mock dialogue implementation.
use super::*;
use crate::{
    actors::{ACTOR_SLOTS, ActorContact},
    collision::Position,
    identifiers::StageId,
    player::Action,
    status::{Health, HealthSegment, OVERLAY_SEGMENTS},
};
use std::collections::VecDeque;
const SOURCE_SLOT: usize = 3;
const CHILD_SLOT: usize = 5;
const RETAINED: u8 = 0x59;
const ACTIVE_WORLD: u8 = 0xa0;
const VISIBLE: u8 = 0x80;
const FLIP: u8 = 0x40;
const CONTACT_BIT: u8 = 1;
const SOURCE_IDENTITY: u8 = 0x9b;
const TEST_GLYPH: u8 = 0x0a;
const SECOND_GLYPH: u8 = 0x0b;
const TEST_STAGE: u8 = 2;
const STAGE_MASK: u8 = 4;
const SPECIAL_SELECTOR: u8 = 4;
const RELOAD_NEGATIVE: u8 = 0x80;
const MARK_PLAYER: u8 = 4;
const MESSAGE_BITS: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];
fn catalog(text: &[u8]) -> Catalog {
    Catalog {
        messages: vec![text.to_vec(); MESSAGE_COUNT],
        destinations: [0x204c, 0x206c, 0x208c],
        row_end_offsets: [0x14, 0x28, 0x3c],
        row_queue_offsets: [0, 0x14, 0x28],
        clear_panel: vec![TEST_GLYPH; PANEL_TEMPLATE_BYTES],
        restore_panel: vec![SECOND_GLYPH; PANEL_TEMPLATE_BYTES],
    }
}
fn status() -> Status {
    Status {
        reward: [BLANK; 6],
        total: [BLANK; 7],
        lives: 3,
        health: Health {
            segments: [HealthSegment {
                y: 0,
                tile: 0,
                depleted: 0,
                x: 0,
            }; OVERLAY_SEGMENTS],
            end: ROW_END,
            alternating: RETAINED,
        },
        countdown: [5, 0, 0],
        divider: 60,
        dirty: 0,
        ui_pending: 0,
    }
}
struct Fixture {
    dialogue: Dialogue,
    progression: Progression,
    player: Player,
    actors: ActorPool,
    status: Status,
    cache: WorldCache,
    camera: Camera,
    suppression: [u8; SUPPRESSION_BYTES],
}
impl Fixture {
    fn new() -> Self {
        let mut actors = [ActorContact::default(); ACTOR_SLOTS];
        actors[SOURCE_SLOT] = ActorContact {
            state: ACTIVE_WORLD,
            render: VISIBLE,
            animation: RETAINED,
            animation_frame: RETAINED,
            animation_tick: RETAINED,
            profile: 0,
            local_state: RETAINED,
            local_counter: RETAINED,
            position: Position {
                world_x: 0x0320,
                screen_y: 0x70,
                x_fraction: RETAINED,
                y_fraction: RETAINED,
            },
            spawn_identity: Some(SOURCE_IDENTITY),
            ..ActorContact::default()
        };
        Self {
            dialogue: Dialogue {
                actor_slot: SOURCE_SLOT,
                ..Dialogue::default()
            },
            progression: Progression {
                stage: StageId(TEST_STAGE),
                ..Progression::default()
            },
            player: Player {
                action: Action::Dialogue,
                ..Player::default()
            },
            actors,
            status: status(),
            cache: WorldCache::default(),
            camera: Camera {
                x: 0x02f0,
                ..Camera::default()
            },
            suppression: [0; SUPPRESSION_BYTES],
        }
    }
    fn context(&mut self) -> Context<'_> {
        Context {
            dialogue: &mut self.dialogue,
            progression: &mut self.progression,
            player: &mut self.player,
            actors: &mut self.actors,
            status: &mut self.status,
            cache: &mut self.cache,
            camera: &self.camera,
            suppression: &mut self.suppression,
        }
    }
    fn outcome(&mut self, message: OutcomeMessage, pressed: Buttons, r: &mut Recorder) {
        self.dialogue.message = message as u8;
        apply_outcome(&mut self.context(), pressed, r);
    }
    fn actor(&mut self, family: Family, r: &mut Recorder) {
        update_actor(family, SOURCE_SLOT, &mut self.context(), r);
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
enum Call {
    Work(Work),
    Contact(usize),
    Distance(usize),
    Motion(Movement, u8, u8),
    Integrate(bool),
    Allocate,
}
#[derive(Default)]
struct Recorder {
    calls: Vec<Call>,
    contact: bool,
    distance: u8,
    motion: u8,
    velocity_after_motion: Option<i16>,
    allocation: Option<usize>,
    suspension: Option<Work>,
    contact_action: Option<Action>,
    observations: Vec<(Work, Action, u8, Buttons)>,
    pressed_at_owner: Buttons,
    ordered_motion: VecDeque<u8>,
    panel_patch: Option<(Work, u8)>,
    outcome_returns: Vec<OutcomeReturn>,
}
impl Systems for Recorder {
    fn outcome_return(&mut self, outcome: OutcomeReturn) {
        self.outcome_returns.push(outcome);
    }
    fn perform_with_panel(
        &mut self,
        work: Work,
        context: &mut Context<'_>,
        panel: &mut Panel,
    ) -> OwnerStep<()> {
        if let Some((expected, value)) = self.panel_patch
            && work == expected
        {
            panel.encoded[0] = value;
        }
        self.perform(work, context)
    }
    fn perform(&mut self, work: Work, c: &mut Context<'_>) -> OwnerStep<()> {
        self.calls.push(Call::Work(work));
        self.observations.push((
            work,
            c.player.action,
            c.progression.dialogue_cursor,
            self.pressed_at_owner,
        ));
        if self.suspension == Some(work) {
            return OwnerStep::Suspended;
        }
        OwnerStep::Complete(())
    }
    fn contact(&mut self, _: &mut ActorPool, p: &mut Player, slot: usize) -> bool {
        self.calls.push(Call::Contact(slot));
        if let Some(action) = self.contact_action {
            p.action = action;
        }
        self.contact
    }
    fn distance(&mut self, _: &ActorPool, slot: usize) -> u8 {
        self.calls.push(Call::Distance(slot));
        self.distance
    }
    fn motion(
        &mut self,
        a: &mut ActorPool,
        _: &mut Player,
        _: &WorldCache,
        slot: usize,
        m: Movement,
        start: u8,
        remaining: u8,
    ) -> u8 {
        self.calls.push(Call::Motion(m, start, remaining));
        if matches!(m, Movement::Rise | Movement::Fall)
            && let Some(v) = self.velocity_after_motion
        {
            a[slot].vertical_velocity = v;
        }
        self.ordered_motion.pop_front().unwrap_or(self.motion)
    }
    fn integrate(&mut self, a: &mut ActorPool, slot: usize, positive: bool) {
        self.calls.push(Call::Integrate(positive));
        // External B05 integration boundary, not the B15 rule under test.
        let delta = a[slot].horizontal_magnitude;
        let old =
            (u32::from(a[slot].position.world_x) << 8) | u32::from(a[slot].position.x_fraction);
        let next = if positive {
            old.wrapping_add(u32::from(delta))
        } else {
            old.wrapping_sub(u32::from(delta))
        };
        a[slot].position.world_x = (next >> 8) as u16;
        a[slot].position.x_fraction = next as u8;
    }
    fn allocate(&mut self, _: &ActorPool) -> Option<usize> {
        self.calls.push(Call::Allocate);
        self.allocation
    }
}
fn run_text(text: &[u8]) -> (Dialogue, u8, Panel, Vec<TextStep>) {
    let mut d = Dialogue::default();
    let mut cursor = 0;
    let mut panel = Panel::default();
    let c = catalog(text);
    let mut steps = Vec::new();
    for _ in 0..256 {
        let step = render_text(&mut d, &mut cursor, &mut panel, &c).unwrap();
        steps.push(step);
        if step.boundary != TextBoundary::GlyphWait {
            return (d, cursor, panel, steps);
        }
    }
    panic!("bounded source text never reaches page or message boundary");
}
#[test]
fn all_eighteen_original_pointer_bounds_and_control_delimiters_are_exact() {
    let rom = std::fs::read("../input/game.rom").unwrap();
    let c = Catalog::original(&rom).unwrap();
    let starts = [
        0x138bc, 0x138e9, 0x13918, 0x1395a, 0x1398c, 0x139a9, 0x139d5, 0x13a03, 0x13a40, 0x13a76,
        0x13aae, 0x13ae1, 0x13b45, 0x13b91, 0x13ba8, 0x13bd2, 0x13c0e, 0x13c26,
    ];
    let lengths = [
        45, 47, 66, 50, 29, 44, 46, 61, 54, 56, 51, 100, 76, 23, 42, 60, 24, 51,
    ];
    assert_eq!(c.messages.len(), MESSAGE_COUNT);
    for i in 0..MESSAGE_COUNT {
        assert_eq!(
            c.messages[i],
            rom[starts[i]..starts[i] + lengths[i]],
            "selector{i:02X}"
        );
        assert_eq!(c.messages[i].last(), Some(&MESSAGE_END));
    }
    assert_eq!(c.destinations, [0x204c, 0x206c, 0x208c]);
    assert_eq!(c.row_queue_offsets, [0, 20, 40]);
    assert_eq!(c.row_end_offsets, [20, 40, 60]);
    assert_eq!(
        c.clear_panel,
        rom[CLEAR_PANEL_PHYSICAL..CLEAR_PANEL_PHYSICAL + PANEL_TEMPLATE_BYTES]
    );
    assert_eq!(
        c.restore_panel,
        rom[RESTORE_PANEL_PHYSICAL..RESTORE_PANEL_PHYSICAL + PANEL_TEMPLATE_BYTES]
    );
}
fn synthetic_pointers() -> Vec<u8> {
    // All selectors intentionally alias one valid string at source CPU B8AC.
    let mut pointers = vec![0xac; MESSAGE_COUNT];
    pointers.extend(vec![0xb8; MESSAGE_COUNT]);
    pointers
}
#[test]
fn split_pointer_aliases_decode_independently_without_a_fixed_word_table() {
    let messages =
        Catalog::decode_messages(&synthetic_pointers(), &[TEST_GLYPH, MESSAGE_END]).unwrap();
    assert_eq!(messages, vec![vec![TEST_GLYPH, MESSAGE_END]; MESSAGE_COUNT]);
}
#[test]
fn truncated_pointer_planes_are_rejected() {
    assert!(
        Catalog::decode_messages(&synthetic_pointers()[..MESSAGE_COUNT], &[MESSAGE_END]).is_err()
    );
}
#[test]
fn pointer_before_text_window_is_rejected() {
    let mut p = synthetic_pointers();
    p[0] = 0xab;
    assert!(Catalog::decode_messages(&p, &[MESSAGE_END]).is_err());
}
#[test]
fn pointer_at_text_end_is_rejected() {
    let mut p = synthetic_pointers();
    p[0] = 0xad;
    assert!(Catalog::decode_messages(&p, &[MESSAGE_END]).is_err());
}
#[test]
fn unterminated_original_format_is_rejected_without_reading_adjacent_gap() {
    assert!(Catalog::decode_messages(&synthetic_pointers(), &[TEST_GLYPH]).is_err());
}
#[test]
fn visible_glyph_submits_only_prepared_row_and_waits_three_scheduler_ticks() {
    let mut d = Dialogue::default();
    let mut cursor = 0;
    let mut p = Panel::default();
    let s = render_text(
        &mut d,
        &mut cursor,
        &mut p,
        &catalog(&[TEST_GLYPH, MESSAGE_END]),
    )
    .unwrap();
    assert_eq!(
        s,
        TextStep {
            boundary: TextBoundary::GlyphWait,
            sound: Some(GLYPH_SOUND),
            wait_ticks: GLYPH_WAIT
        }
    );
    assert_eq!(cursor, 1);
    assert_eq!(p.rows()[0], vec![TEST_GLYPH]);
    assert_eq!(p.pending, ROW_END);
    assert_eq!(&p.encoded[..5], &[0x20, 0x4c, 0, TEST_GLYPH, ROW_END]);
}
#[test]
fn leading_blanks_advance_cursor_without_sound_or_extra_wait() {
    let mut d = Dialogue::default();
    let mut cursor = 0;
    let mut p = Panel::default();
    let s = render_text(
        &mut d,
        &mut cursor,
        &mut p,
        &catalog(&[BLANK, BLANK, TEST_GLYPH, MESSAGE_END]),
    )
    .unwrap();
    assert_eq!(cursor, 3);
    assert_eq!(p.rows()[0], vec![BLANK, BLANK, TEST_GLYPH]);
    assert_eq!(s.sound, Some(GLYPH_SOUND));
    assert_eq!(s.wait_ticks, GLYPH_WAIT);
    assert_eq!(p.encoded[2], 2);
}
#[test]
fn terminal_marker_emits_final_period_without_glyph_sound_or_delay() {
    let (_, cursor, p, steps) = run_text(&[TEST_GLYPH, MESSAGE_END]);
    assert_eq!(cursor, 0);
    assert_eq!(p.rows()[0], vec![TEST_GLYPH, FINAL_GLYPH]);
    assert_eq!(
        steps.last(),
        Some(&TextStep {
            boundary: TextBoundary::MessageComplete,
            sound: None,
            wait_ticks: 0
        })
    );
    assert_eq!(
        &p.encoded[..6],
        &[0x20, 0x4c, 1, TEST_GLYPH, FINAL_GLYPH, ROW_END]
    );
}
#[test]
fn empty_message_uses_wrapped_initial_count_for_one_final_period() {
    let (_, cursor, p, steps) = run_text(&[MESSAGE_END]);
    assert_eq!(cursor, 0);
    assert_eq!(p.rows()[0], vec![FINAL_GLYPH]);
    assert_eq!(p.encoded[2], 0);
    assert_eq!(steps.len(), 1);
}
#[test]
fn blank_only_text_reaches_terminal_without_intermediate_wait() {
    let (_, _, p, steps) = run_text(&[BLANK, BLANK, MESSAGE_END]);
    assert_eq!(steps.len(), 1);
    assert_eq!(p.rows()[0], vec![BLANK, BLANK, FINAL_GLYPH]);
    assert_eq!(p.encoded[2], 2);
}
#[test]
fn delimiter_consumes_cursor_pads_seventeen_cells_and_moves_to_next_row() {
    let (_, _, p, steps) = run_text(&[TEST_GLYPH, ROW_END, SECOND_GLYPH, MESSAGE_END]);
    assert_eq!(p.rows()[0].len(), ROW_GLYPHS);
    assert_eq!(p.rows()[0][0], TEST_GLYPH);
    assert!(p.rows()[0][1..].iter().all(|b| *b == BLANK));
    assert_eq!(p.rows()[1], vec![SECOND_GLYPH, FINAL_GLYPH]);
    assert_eq!(steps.iter().filter(|s| s.sound.is_some()).count(), 2);
    assert_eq!(&p.encoded[..3], &[0x20, 0x4c, 16]);
    assert_eq!(&p.encoded[20..23], &[0x20, 0x6c, 1]);
}
#[test]
fn three_delimiters_stop_at_page_with_nonzero_cursor_and_do_not_consume_next_page() {
    let (d, c, p, steps) = run_text(&[ROW_END, ROW_END, ROW_END, TEST_GLYPH, MESSAGE_END]);
    assert_eq!(d.line, 2);
    assert_eq!(c, 3);
    assert_eq!(
        steps,
        vec![TextStep {
            boundary: TextBoundary::PageComplete,
            sound: None,
            wait_ticks: 0
        }]
    );
    assert!(p.rows().iter().all(|r| r == &vec![BLANK; ROW_GLYPHS]));
}
#[test]
fn seventeenth_visible_glyph_closes_row_before_sound_and_wait() {
    let mut text = vec![TEST_GLYPH; ROW_GLYPHS];
    text.push(MESSAGE_END);
    let (_, _, p, steps) = run_text(&text);
    assert_eq!(
        steps.iter().filter(|s| s.sound.is_some()).count(),
        ROW_GLYPHS - 1
    );
    assert_eq!(p.rows()[0], vec![TEST_GLYPH; ROW_GLYPHS]);
    assert_eq!(p.rows()[1], vec![FINAL_GLYPH]);
}
#[test]
fn sixteenth_glyph_still_waits_then_terminal_occupies_seventeenth_cell() {
    let mut text = vec![TEST_GLYPH; ROW_GLYPHS - 1];
    text.push(MESSAGE_END);
    let (_, _, p, steps) = run_text(&text);
    assert_eq!(
        steps.iter().filter(|s| s.sound.is_some()).count(),
        ROW_GLYPHS - 1
    );
    assert_eq!(p.rows()[0].len(), ROW_GLYPHS);
    assert_eq!(p.rows()[0].last(), Some(&FINAL_GLYPH));
}
#[test]
fn row_delimiter_immediately_after_sixteen_glyphs_pads_one_blank() {
    let mut text = vec![TEST_GLYPH; ROW_GLYPHS - 1];
    text.extend([ROW_END, MESSAGE_END]);
    let (_, _, p, _) = run_text(&text);
    assert_eq!(p.rows()[0].last(), Some(&BLANK));
    assert_eq!(p.rows()[1], vec![FINAL_GLYPH]);
}
#[test]
fn scrolling_copies_all_seventeen_tiles_preserves_headers_and_yields_before_bottom_render() {
    let mut d = Dialogue {
        line: 2,
        phase: Phase::AwaitInput,
        ..Dialogue::default()
    };
    let mut p = three_rows();
    scroll_rows(&mut d, &mut p);
    assert_eq!(
        p.rows(),
        [
            vec![SECOND_GLYPH; ROW_GLYPHS],
            vec![FINAL_GLYPH; ROW_GLYPHS],
            vec![BLANK; ROW_GLYPHS]
        ]
    );
    assert_eq!(p.pending, ROW_END);
    assert_eq!(d.phase, Phase::ScrollYield);
}

#[test]
fn ordinary_messages_have_no_outcome_side_effects_for_all_other_selectors() {
    for message in [0, 1, 2, 4, 5, 7, 8, 9, 13, 14, 16, 17] {
        let mut f = Fixture::new();
        f.dialogue.message = message;
        let p = f.progression.clone();
        let a = f.actors;
        let s = f.status.clone();
        let mut r = Recorder::default();
        apply_outcome(&mut f.context(), Buttons::A | Buttons::B, &mut r);
        assert_eq!(f.progression, p, "message{message}");
        assert_eq!(f.actors, a);
        assert_eq!(f.status, s);
        assert!(r.calls.is_empty());
    }
}
#[test]
fn race_intro_outcome_clears_latch_even_when_b_is_pressed() {
    let mut f = Fixture::new();
    f.progression.race_intro = RETAINED;
    let mut r = Recorder::default();
    f.outcome(OutcomeMessage::RaceIntro, Buttons::B, &mut r);
    assert_eq!(f.progression.race_intro, 0);
    assert!(r.calls.is_empty());
}
#[test]
fn life_loss_outcome_uses_common_reload_and_clears_player_queue_activity_and_alternation() {
    let mut f = Fixture::new();
    f.player.queued = Some(Action::Damage);
    f.actors[0].state = ACTIVE_WORLD;
    f.progression.life_loss_variant = 5;
    f.outcome(
        OutcomeMessage::LifeLoss,
        Buttons::B,
        &mut Recorder::default(),
    );
    assert_eq!(f.progression.reload_request, 0x8e);
    assert_eq!(f.player.action, Action::Standing);
    assert_eq!(f.player.queued, None);
    assert_eq!(f.actors[0].state, 0);
    assert_eq!(f.status.health.alternating, 0);
}
#[test]
fn life_loss_stage_addition_preserves_first_add_carry_into_variant() {
    let mut f = Fixture::new();
    f.progression.stage = StageId(0xff);
    f.progression.life_loss_variant = 1;
    f.outcome(
        OutcomeMessage::LifeLoss,
        Buttons::NONE,
        &mut Recorder::default(),
    );
    assert_eq!(f.progression.reload_request, 0x88);
}
#[test]
fn reveal_reinitializes_only_saved_actor_and_two_collision_cells_before_burst_zero() {
    // Independent source adapter check: collision plane begins0580, not0500.
    assert_eq!(REVEAL_CACHE_CELLS, [0x21, 0x29]);
    let mut f = Fixture::new();
    f.cache.collision.fill(RETAINED);
    f.cache.attributes.fill(RETAINED);
    let old = f.actors;
    let mut r = Recorder::default();
    f.outcome(OutcomeMessage::Reveal, Buttons::B, &mut r);
    let a = &f.actors[SOURCE_SLOT];
    assert_eq!(
        (
            a.state,
            a.handler,
            a.animation,
            a.position.screen_y,
            a.position.world_x
        ),
        (
            ACTIVE_WORLD,
            REVEAL_HANDLER,
            REVEAL_ANIMATION,
            REVEAL_Y,
            0x0330
        )
    );
    assert_eq!(
        (
            a.animation_frame,
            a.animation_tick,
            a.hit_marker,
            a.local_state
        ),
        (0, 0, 0, REVEAL_PHASE)
    );
    assert_eq!(
        (
            a.position.x_fraction,
            a.position.y_fraction,
            a.render,
            a.profile,
            a.spawn_identity
        ),
        (RETAINED, RETAINED, VISIBLE, 0, Some(SOURCE_IDENTITY))
    );
    for i in 0..f.cache.collision.len() {
        assert_eq!(
            f.cache.collision[i],
            if REVEAL_CACHE_CELLS.contains(&i) {
                0
            } else {
                RETAINED
            }
        );
    }
    assert!(f.cache.attributes.iter().all(|v| *v == RETAINED));
    for (i, previous) in old.iter().enumerate() {
        if i != SOURCE_SLOT {
            assert_eq!(&f.actors[i], previous);
        }
    }
    assert_eq!(r.calls, vec![Call::Work(Work::ChangedBurst(0))]);
}
#[test]
fn stage_choice_accepts_a_sets_current_stage_mask_and_special_room_request() {
    let mut f = Fixture::new();
    f.progression.accepted_stages = 0x80;
    f.outcome(
        OutcomeMessage::StageChoice,
        Buttons::A,
        &mut Recorder::default(),
    );
    assert_eq!(f.progression.accepted_stages, 0x84);
    assert_eq!(f.progression.special_room, CHOICE_REQUEST);
}
#[test]
fn stage_choice_rejects_b_even_with_simultaneous_a_and_preserves_requests() {
    for pressed in [Buttons::B, Buttons::A | Buttons::B] {
        let mut f = Fixture::new();
        f.progression.accepted_stages = RETAINED;
        f.progression.special_room = RETAINED;
        f.outcome(
            OutcomeMessage::StageChoice,
            pressed,
            &mut Recorder::default(),
        );
        assert_eq!(f.progression.accepted_stages, RETAINED);
        assert_eq!(f.progression.special_room, RETAINED);
    }
}
#[test]
fn stage_choice_uses_original_eight_bitmask_entries_not_stage_number_itself() {
    for (stage, mask) in MESSAGE_BITS.into_iter().enumerate() {
        let mut f = Fixture::new();
        f.progression.stage = StageId(stage as u8);
        f.outcome(
            OutcomeMessage::StageChoice,
            Buttons::NONE,
            &mut Recorder::default(),
        );
        assert_eq!(f.progression.accepted_stages, mask);
    }
}
#[test]
fn message_three_increments_boundary_request_when_reward_bit_is_clear_regardless_of_b() {
    let mut f = Fixture::new();
    f.progression.reward_progress = 0x80;
    f.progression.boundary_request = RETAINED;
    f.outcome(
        OutcomeMessage::Boundary,
        Buttons::B,
        &mut Recorder::default(),
    );
    assert_eq!(f.progression.boundary_request, RETAINED + 1);
}
#[test]
fn message_three_reward_bit_one_prevents_increment_without_clearing_existing_request() {
    let mut f = Fixture::new();
    f.progression.reward_progress = 2;
    f.progression.boundary_request = RETAINED;
    f.outcome(
        OutcomeMessage::Boundary,
        Buttons::A,
        &mut Recorder::default(),
    );
    assert_eq!(f.progression.boundary_request, RETAINED);
}
#[test]
fn message_three_increment_can_cross_sign_boundary_and_wrap_not_saturate() {
    for (before, after) in [(0x7f, 0x80), (0xff, 0)] {
        let mut f = Fixture::new();
        f.progression.boundary_request = before;
        f.outcome(
            OutcomeMessage::Boundary,
            Buttons::A,
            &mut Recorder::default(),
        );
        assert_eq!(f.progression.boundary_request, after);
    }
}
#[test]
fn paid_reward_rejects_b_without_mutating_status_or_actor() {
    let mut f = Fixture::new();
    f.status.reward = [9; 6];
    let a = f.actors;
    let s = f.status.clone();
    f.outcome(
        OutcomeMessage::PaidReward,
        Buttons::A | Buttons::B,
        &mut Recorder::default(),
    );
    assert_eq!(f.actors, a);
    assert_eq!(f.status, s);
}
#[test]
fn paid_reward_underfunded_two_upper_digits_leave_actor_and_all_digits_unchanged() {
    for upper in [[BLANK, BLANK], [BLANK, 2], [0, 2]] {
        let mut f = Fixture::new();
        f.status.reward = [upper[0], upper[1], 9, 9, 9, 9];
        let a = f.actors;
        let s = f.status.clone();
        f.outcome(
            OutcomeMessage::PaidReward,
            Buttons::A,
            &mut Recorder::default(),
        );
        assert_eq!(f.actors, a);
        assert_eq!(f.status, s);
    }
}
#[test]
fn paid_reward_exact_three_blanks_all_leading_zeroes_and_commits_actor_only_on_success() {
    let mut f = Fixture::new();
    f.status.reward = [BLANK, 3, 0, 0, 0, 0];
    f.actors[SOURCE_SLOT].state = 0xb7;
    f.actors[SOURCE_SLOT].profile = 0x35;
    f.outcome(
        OutcomeMessage::PaidReward,
        Buttons::A,
        &mut Recorder::default(),
    );
    assert_eq!(f.status.reward, [BLANK; 6]);
    assert_eq!(f.status.dirty, 1);
    let a = f.actors[SOURCE_SLOT];
    assert_eq!(
        (
            a.handler,
            a.state,
            a.position.screen_y,
            a.position.world_x,
            a.profile
        ),
        (PAID_HANDLER, 0xa0, PAID_POSITION, 0x0380, 0xb5)
    );
    assert_eq!(
        (
            a.animation,
            a.animation_frame,
            a.animation_tick,
            a.position.x_fraction,
            a.position.y_fraction
        ),
        (RETAINED, RETAINED, RETAINED, RETAINED, RETAINED)
    );
}
#[test]
fn paid_reward_borrows_across_upper_digits_and_preserves_lower_digits_total_and_other_actors() {
    let mut f = Fixture::new();
    f.status.reward = [1, 0, 5, 4, 3, 2];
    f.status.total = [9; 7];
    let old = f.actors;
    f.outcome(
        OutcomeMessage::PaidReward,
        Buttons::A,
        &mut Recorder::default(),
    );
    assert_eq!(f.status.reward, [BLANK, 7, 5, 4, 3, 2]);
    assert_eq!(f.status.total, [9; 7]);
    for (i, previous) in old.iter().enumerate() {
        if i != SOURCE_SLOT {
            assert_eq!(&f.actors[i], previous);
        }
    }
}
#[test]
fn paid_reward_dirty_increment_wraps_and_normalization_stops_at_first_nonzero() {
    let mut f = Fixture::new();
    f.status.reward = [0, 3, 0, 4, 0, 0];
    f.status.dirty = ROW_END;
    f.outcome(
        OutcomeMessage::PaidReward,
        Buttons::A,
        &mut Recorder::default(),
    );
    assert_eq!(f.status.reward, [BLANK, BLANK, BLANK, 4, 0, 0]);
    assert_eq!(f.status.dirty, 0);
}
fn pressed(buttons: Buttons) -> PortInput {
    PortInput {
        pressed: buttons,
        held: buttons,
    }
}
#[test]
fn entry_clears_panel_and_yields_before_resetting_cursor_or_composition() {
    let mut f = Fixture::new();
    f.progression.dialogue_cursor = RETAINED;
    let mut input = pressed(Buttons::NONE);
    let mut panel = Panel::default();
    let mut r = Recorder::default();
    assert_eq!(
        resume(
            &mut f.context(),
            &mut input,
            &mut panel,
            &catalog(&[MESSAGE_END]),
            &mut r
        )
        .unwrap(),
        OwnerStep::Suspended
    );
    assert_eq!(
        r.calls,
        vec![
            Call::Work(Work::SelectGraphicsTwo),
            Call::Work(Work::ClearPanel),
            Call::Work(Work::Yield)
        ]
    );
    assert_eq!(f.progression.dialogue_cursor, RETAINED);
    assert_eq!(f.dialogue.phase, Phase::ClearPanelYield);
    assert_eq!(&panel.encoded[..3], &[0x12, 0xd0, 0x0f]);
    assert_eq!(
        &panel.encoded[3..19],
        &vec![TEST_GLYPH; PANEL_TEMPLATE_BYTES]
    );
    assert_eq!(panel.encoded[19], ROW_END);
    assert_eq!(panel.pending, ROW_END);
}
#[test]
fn race_intro_builds_terrain_before_panel_and_only_at_entry() {
    let mut f = Fixture::new();
    f.dialogue.message = OutcomeMessage::RaceIntro as u8;
    let mut input = pressed(Buttons::NONE);
    let mut panel = Panel::default();
    let mut r = Recorder::default();
    resume(
        &mut f.context(),
        &mut input,
        &mut panel,
        &catalog(&[MESSAGE_END]),
        &mut r,
    )
    .unwrap();
    assert_eq!(r.calls.first(), Some(&Call::Work(Work::BuildRaceTerrain)));
    r.calls.clear();
    r.suspension = Some(Work::StatusStreams);
    resume(
        &mut f.context(),
        &mut input,
        &mut panel,
        &catalog(&[MESSAGE_END]),
        &mut r,
    )
    .unwrap();
    assert!(!r.calls.contains(&Call::Work(Work::BuildRaceTerrain)));
}
#[test]
fn clear_resume_resets_cursor_then_composes_and_suspends_inside_status_without_early_text() {
    let mut f = Fixture::new();
    f.dialogue.phase = Phase::ClearPanelYield;
    f.progression.dialogue_cursor = RETAINED;
    let mut r = Recorder {
        suspension: Some(Work::StatusStreams),
        ..Recorder::default()
    };
    let mut p = Panel::default();
    let mut input = pressed(Buttons::NONE);
    assert_eq!(
        resume(
            &mut f.context(),
            &mut input,
            &mut p,
            &catalog(&[TEST_GLYPH, MESSAGE_END]),
            &mut r
        )
        .unwrap(),
        OwnerStep::Suspended
    );
    assert_eq!(f.progression.dialogue_cursor, 0);
    assert_eq!(f.dialogue.line, 0);
    assert_eq!(f.dialogue.phase, Phase::StatusStreams);
    assert_eq!(
        r.calls,
        vec![
            Call::Work(Work::SelectPresentation),
            Call::Work(Work::HideSprites),
            Call::Work(Work::Compose),
            Call::Work(Work::StatusStreams)
        ]
    );
    assert!(p.rows().iter().all(Vec::is_empty));
}
#[test]
fn status_resume_uses_additive_suppression_before_text_bank_and_first_glyph() {
    let mut f = Fixture::new();
    f.dialogue.phase = Phase::StatusStreams;
    let mut r = Recorder::default();
    let mut p = Panel::default();
    let mut input = pressed(Buttons::NONE);
    resume(
        &mut f.context(),
        &mut input,
        &mut p,
        &catalog(&[TEST_GLYPH, MESSAGE_END]),
        &mut r,
    )
    .unwrap();
    assert_eq!(
        r.calls,
        vec![
            Call::Work(Work::StatusStreams),
            Call::Work(Work::AddSuppression),
            Call::Work(Work::SelectTextBank),
            Call::Work(Work::Sound(GLYPH_SOUND)),
            Call::Work(Work::Wait(GLYPH_WAIT))
        ]
    );
    assert_eq!(f.progression.dialogue_cursor, 1);
}
#[test]
fn awaiting_input_rejects_held_a_and_all_non_acknowledgment_edges() {
    for edge in [
        Buttons::NONE,
        Buttons::START,
        Buttons::SELECT,
        Buttons::LEFT | Buttons::UP,
    ] {
        let mut f = Fixture::new();
        f.dialogue.phase = Phase::AwaitInput;
        let mut input = PortInput {
            pressed: edge,
            held: Buttons::A,
        };
        let mut r = Recorder::default();
        let mut p = Panel::default();
        assert_eq!(
            resume(
                &mut f.context(),
                &mut input,
                &mut p,
                &catalog(&[MESSAGE_END]),
                &mut r
            )
            .unwrap(),
            OwnerStep::Suspended
        );
        assert_eq!(r.calls, vec![Call::Work(Work::Yield)]);
        assert_eq!(f.player.action, Action::Dialogue);
    }
}
#[test]
fn unfinished_page_acknowledgment_scrolls_and_yields_without_outcome_or_player_restore() {
    for edge in [Buttons::A, Buttons::B, Buttons::A | Buttons::B] {
        let mut f = Fixture::new();
        f.dialogue.phase = Phase::AwaitInput;
        f.dialogue.line = 2;
        f.progression.dialogue_cursor = 3;
        let mut input = pressed(edge);
        let mut p = three_rows();
        let mut r = Recorder::default();
        resume(
            &mut f.context(),
            &mut input,
            &mut p,
            &catalog(&[ROW_END, ROW_END, ROW_END, MESSAGE_END]),
            &mut r,
        )
        .unwrap();
        assert_eq!(f.player.action, Action::Dialogue);
        assert_eq!(f.dialogue.phase, Phase::ScrollYield);
        assert_eq!(f.progression.dialogue_cursor, 3);
        assert_eq!(r.calls, vec![Call::Work(Work::Yield)]);
        assert_eq!(p.rows()[0], vec![SECOND_GLYPH; ROW_GLYPHS]);
    }
}
#[test]
fn scroll_resume_renders_bottom_then_yields_before_reusing_input_edge() {
    let mut f = Fixture::new();
    f.dialogue.phase = Phase::ScrollYield;
    f.progression.dialogue_cursor = 3;
    let mut input = pressed(Buttons::A);
    let mut r = Recorder::default();
    let mut p = Panel::default();
    resume(
        &mut f.context(),
        &mut input,
        &mut p,
        &catalog(&[ROW_END, ROW_END, ROW_END, MESSAGE_END]),
        &mut r,
    )
    .unwrap();
    assert_eq!(f.dialogue.line, 2);
    assert_eq!(f.dialogue.phase, Phase::AwaitInput);
    assert_eq!(f.player.action, Action::Dialogue);
    assert_eq!(f.dialogue.acknowledgment, None);
    assert_eq!(p.rows()[2], vec![FINAL_GLYPH]);
    assert_eq!(r.calls, vec![Call::Work(Work::Yield)]);
}
#[test]
fn final_acknowledgment_saves_full_byte_before_restoring_player_and_nested_status_wait() {
    let mut f = Fixture::new();
    f.dialogue.phase = Phase::AwaitInput;
    let edge = Buttons::B | Buttons::START | Buttons::LEFT;
    let mut input = pressed(edge);
    let mut p = Panel::default();
    let mut r = Recorder {
        suspension: Some(Work::StatusStreams),
        ..Recorder::default()
    };
    resume(
        &mut f.context(),
        &mut input,
        &mut p,
        &catalog(&[MESSAGE_END]),
        &mut r,
    )
    .unwrap();
    assert_eq!(f.dialogue.acknowledgment, Some(edge));
    assert_eq!(f.player.action, Action::Standing);
    assert_eq!(f.dialogue.phase, Phase::RestoreStatus);
    assert_eq!(
        r.calls,
        vec![
            Call::Work(Work::SelectPresentation),
            Call::Work(Work::StatusStreams)
        ]
    );
    assert!(
        r.observations
            .iter()
            .all(|(_, action, _, _)| *action == Action::Standing)
    );
}
#[test]
fn restore_status_resume_prepares_original_panel_before_final_yield() {
    let mut f = Fixture::new();
    f.dialogue.phase = Phase::RestoreStatus;
    f.dialogue.acknowledgment = Some(Buttons::A);
    f.player.action = Action::Standing;
    let mut input = pressed(Buttons::NONE);
    let mut p = Panel::default();
    let mut r = Recorder::default();
    resume(
        &mut f.context(),
        &mut input,
        &mut p,
        &catalog(&[MESSAGE_END]),
        &mut r,
    )
    .unwrap();
    assert_eq!(f.dialogue.phase, Phase::RestorePanelYield);
    assert_eq!(
        r.calls,
        vec![
            Call::Work(Work::StatusStreams),
            Call::Work(Work::SelectGraphicsOne),
            Call::Work(Work::RestorePanel),
            Call::Work(Work::Yield)
        ]
    );
    assert_eq!(&p.encoded[..3], &[0x12, 0xd0, 0x0f]);
    assert_eq!(&p.encoded[3..19], &vec![SECOND_GLYPH; PANEL_TEMPLATE_BYTES]);
}
#[test]
fn final_resume_restores_original_pressed_but_keeps_new_held_before_rejected_choice() {
    let mut f = Fixture::new();
    f.dialogue.phase = Phase::RestorePanelYield;
    f.dialogue.message = OutcomeMessage::StageChoice as u8;
    let saved = Buttons::B | Buttons::START;
    f.dialogue.acknowledgment = Some(saved);
    let mut input = PortInput {
        pressed: Buttons::A,
        held: Buttons::RIGHT,
    };
    let mut r = Recorder::default();
    assert_eq!(
        resume(
            &mut f.context(),
            &mut input,
            &mut Panel::default(),
            &catalog(&[MESSAGE_END]),
            &mut r
        )
        .unwrap(),
        OwnerStep::Complete(())
    );
    assert_eq!(input.pressed, saved);
    assert_eq!(input.held, Buttons::RIGHT);
    assert_eq!(f.progression.accepted_stages, 0);
    assert_eq!(f.progression.special_room, 0);
    assert_eq!(f.dialogue.phase, Phase::Done);
}
#[test]
fn final_resume_uses_saved_a_even_when_new_sample_contains_b() {
    let mut f = Fixture::new();
    f.dialogue.phase = Phase::RestorePanelYield;
    f.dialogue.message = OutcomeMessage::StageChoice as u8;
    f.dialogue.acknowledgment = Some(Buttons::A | Buttons::UP);
    let mut input = pressed(Buttons::B);
    resume(
        &mut f.context(),
        &mut input,
        &mut Panel::default(),
        &catalog(&[MESSAGE_END]),
        &mut Recorder::default(),
    )
    .unwrap();
    assert_eq!(input.pressed, Buttons::A | Buttons::UP);
    assert_eq!(f.progression.accepted_stages, STAGE_MASK);
    assert_eq!(f.progression.special_room, CHOICE_REQUEST);
}

fn three_rows() -> Panel {
    let mut encoded = vec![0x20, 0x4c, 16];
    encoded.extend([TEST_GLYPH; ROW_GLYPHS]);
    encoded.extend([0x20, 0x6c, 16]);
    encoded.extend([SECOND_GLYPH; ROW_GLYPHS]);
    encoded.extend([0x20, 0x8c, 16]);
    encoded.extend([FINAL_GLYPH; ROW_GLYPHS]);
    encoded.push(ROW_END);
    Panel {
        encoded,
        pending: 0,
    }
}

#[test]
fn progression_contact_returns_without_requests_on_rejected_overlap() {
    let mut f = Fixture::new();
    f.actors[SOURCE_SLOT].local_state = SPECIAL_SELECTOR;
    let p = f.progression.clone();
    let mut r = Recorder::default();
    f.actor(Family::ProgressionContact, &mut r);
    assert_eq!(f.progression, p);
    assert_eq!(r.calls, vec![Call::Contact(SOURCE_SLOT)]);
}
#[test]
fn progression_contact_zero_selector_requires_seen_message_three_flag() {
    for (seen, expected) in [
        (0, 0),
        (4, 0),
        (8, RELOAD_NEGATIVE),
        (0xff, RELOAD_NEGATIVE),
    ] {
        let mut f = Fixture::new();
        f.actors[SOURCE_SLOT].local_state = 0;
        f.progression.seen_messages[0] = seen;
        let mut r = Recorder {
            contact: true,
            ..Recorder::default()
        };
        f.actor(Family::ProgressionContact, &mut r);
        assert_eq!(f.progression.reload_request, expected);
    }
}
#[test]
fn progression_contact_ordinary_nonzero_selectors_or_negative_without_nibble_masking() {
    for selector in [1, 2, 3, 5, 0x7f, 0x80, 0xff] {
        let mut f = Fixture::new();
        f.actors[SOURCE_SLOT].local_state = selector;
        let mut r = Recorder {
            contact: true,
            ..Recorder::default()
        };
        f.actor(Family::ProgressionContact, &mut r);
        assert_eq!(f.progression.reload_request, selector | RELOAD_NEGATIVE);
    }
}
#[test]
fn progression_contact_selector_four_requires_only_low_five_complete_bits() {
    for progress in [ALL_LANDS, 0x3f, 0x7f, 0xff] {
        let mut f = Fixture::new();
        f.actors[SOURCE_SLOT].local_state = SPECIAL_SELECTOR;
        f.progression.progress_bits = progress;
        let mut r = Recorder {
            contact: true,
            ..Recorder::default()
        };
        f.actor(Family::ProgressionContact, &mut r);
        assert_eq!(f.progression.reload_request, 0x84);
        assert_eq!(f.progression.boundary_request, 0);
    }
}
#[test]
fn progression_contact_incomplete_selector_four_sets_negative_boundary_and_preserves_reload() {
    for progress in [0, 0x1e, 0x0f, 0xe0] {
        let mut f = Fixture::new();
        f.actors[SOURCE_SLOT].local_state = SPECIAL_SELECTOR;
        f.progression.progress_bits = progress;
        f.progression.reload_request = RETAINED;
        let mut r = Recorder {
            contact: true,
            ..Recorder::default()
        };
        f.actor(Family::ProgressionContact, &mut r);
        assert_eq!(f.progression.boundary_request, RELOAD_NEGATIVE);
        assert_eq!(f.progression.reload_request, RETAINED);
    }
}
#[test]
fn dialogue_contact_damage_and_death_return_before_overlap_or_phase_change() {
    for action in [Action::Damage, Action::Death] {
        let mut f = Fixture::new();
        f.player.action = action;
        let a = f.actors;
        let mut r = Recorder {
            contact: true,
            ..Recorder::default()
        };
        f.actor(Family::DialogueContact, &mut r);
        assert!(r.calls.is_empty());
        assert_eq!(f.actors, a);
        assert_eq!(f.player.action, action);
    }
}
#[test]
fn dialogue_contact_any_used_low_phase_returns_before_overlap() {
    for phase in 1..=15 {
        let mut f = Fixture::new();
        f.actors[SOURCE_SLOT].state = ACTIVE_WORLD | phase;
        let a = f.actors;
        let mut r = Recorder {
            contact: true,
            ..Recorder::default()
        };
        f.actor(Family::DialogueContact, &mut r);
        assert_eq!(f.actors, a);
        assert!(r.calls.is_empty());
    }
}
#[test]
fn dialogue_contact_failed_qualification_preserves_source_state_even_if_helper_reacts() {
    let mut f = Fixture::new();
    let a = f.actors;
    let mut r = Recorder {
        contact: false,
        contact_action: Some(Action::Damage),
        ..Recorder::default()
    };
    f.actor(Family::DialogueContact, &mut r);
    assert_eq!(f.actors, a);
    assert_eq!(f.player.action, Action::Damage);
    assert_eq!(f.dialogue.message, 0);
}
#[test]
fn choice_animation_contact_is_blocked_only_by_current_stage_accepted_bit() {
    for (accepted, blocked) in [(STAGE_MASK, true), (0x80, false), (0, false)] {
        let mut f = Fixture::new();
        f.player.action = Action::Standing;
        f.actors[SOURCE_SLOT].animation = CHOICE_ANIMATION;
        f.actors[SOURCE_SLOT].local_state = OutcomeMessage::StageChoice as u8;
        f.progression.accepted_stages = accepted;
        let mut r = Recorder {
            contact: true,
            ..Recorder::default()
        };
        f.actor(Family::DialogueContact, &mut r);
        assert_eq!(
            f.player.action,
            if blocked {
                Action::Standing
            } else {
                Action::Dialogue
            }
        );
        assert_eq!(
            f.actors[SOURCE_SLOT].state,
            if blocked {
                ACTIVE_WORLD
            } else {
                ACTIVE_WORLD + 1
            }
        );
    }
}
#[test]
fn nonchoice_animation_does_not_consult_accepted_stages() {
    let mut f = Fixture::new();
    f.progression.accepted_stages = ROW_END;
    f.actors[SOURCE_SLOT].local_state = 1;
    let mut r = Recorder {
        contact: true,
        ..Recorder::default()
    };
    f.actor(Family::DialogueContact, &mut r);
    assert_eq!(f.actors[SOURCE_SLOT].state, ACTIVE_WORLD + 1);
    assert_eq!(f.dialogue.message, 1);
}
#[test]
fn all_eighteen_contact_messages_write_exact_bit_plane_and_saved_slot() {
    for message in 0..MESSAGE_COUNT as u8 {
        let mut f = Fixture::new();
        f.dialogue.actor_slot = CHILD_SLOT;
        f.actors[SOURCE_SLOT].local_state = message;
        f.progression.seen_messages = [0x20, 0x40, 0x80];
        let mut expected = f.progression.seen_messages;
        expected[usize::from(message / 8)] |= MESSAGE_BITS[usize::from(message % 8)];
        let mut r = Recorder {
            contact: true,
            ..Recorder::default()
        };
        f.actor(Family::DialogueContact, &mut r);
        assert_eq!(f.dialogue.message, message);
        assert_eq!(f.dialogue.actor_slot, SOURCE_SLOT);
        assert_eq!(f.progression.seen_messages, expected);
        assert_eq!(f.player.action, Action::Dialogue);
    }
}
#[test]
fn message_eight_contact_increments_prerequisite_removes_and_suppresses_original_identity() {
    let mut f = Fixture::new();
    f.actors[SOURCE_SLOT].local_state = OutcomeMessage::EscortPrerequisite as u8;
    f.progression.escort_prerequisite = ROW_END;
    let mut r = Recorder {
        contact: true,
        ..Recorder::default()
    };
    f.actor(Family::DialogueContact, &mut r);
    assert_eq!(f.progression.escort_prerequisite, 0);
    assert_eq!(f.actors[SOURCE_SLOT].state, 0);
    assert_eq!(f.suppression[19], 8);
    assert_eq!(f.progression.seen_messages, [0, 1, 0]);
    assert_eq!(f.actors[SOURCE_SLOT].spawn_identity, Some(SOURCE_IDENTITY));
}
#[test]
fn ordinary_contact_is_not_suppressed_or_removed_and_can_override_helper_damage_on_success() {
    let mut f = Fixture::new();
    f.actors[SOURCE_SLOT].local_state = 6;
    f.player.protection = RETAINED;
    let mut r = Recorder {
        contact: true,
        contact_action: Some(Action::Damage),
        ..Recorder::default()
    };
    f.actor(Family::DialogueContact, &mut r);
    assert_eq!(f.actors[SOURCE_SLOT].state, ACTIVE_WORLD + 1);
    assert_eq!(f.player.action, Action::Dialogue);
    assert!(f.suppression.iter().all(|b| *b == 0));
}
#[test]
fn conditional_marker_uses_bitwise_stage_encoding_not_one_shifted_by_stage() {
    for (stage, reward, qualifies) in [(2, 2, true), (2, 4, false), (0, 0xff, false), (3, 1, true)]
    {
        let mut f = Fixture::new();
        f.progression.stage = StageId(stage);
        f.progression.reward_progress = reward;
        f.actors[SOURCE_SLOT].profile = RETAINED;
        let mut r = Recorder {
            contact: true,
            ..Recorder::default()
        };
        f.actor(Family::ConditionalMarker, &mut r);
        assert_eq!(
            f.actors[SOURCE_SLOT].profile,
            if qualifies { 0 } else { RETAINED }
        );
        assert_eq!(f.actors[0].render, if qualifies { MARK_PLAYER } else { 0 });
        assert_eq!(r.calls.len(), usize::from(qualifies));
    }
}
#[test]
fn conditional_marker_profile_is_cleared_before_failed_overlap_without_player_mark() {
    let mut f = Fixture::new();
    f.progression.reward_progress = TEST_STAGE;
    f.actors[SOURCE_SLOT].profile = RETAINED;
    f.actors[0].render = FLIP;
    let mut r = Recorder::default();
    f.actor(Family::ConditionalMarker, &mut r);
    assert_eq!(f.actors[SOURCE_SLOT].profile, 0);
    assert_eq!(f.actors[0].render, FLIP);
}
#[test]
fn conditional_marker_or_preserves_all_other_player_render_bits() {
    let mut f = Fixture::new();
    f.progression.reward_progress = TEST_STAGE;
    f.actors[0].render = 0xa2;
    let mut r = Recorder {
        contact: true,
        ..Recorder::default()
    };
    f.actor(Family::ConditionalMarker, &mut r);
    assert_eq!(f.actors[0].render, 0xa6);
}
#[test]
fn hit_actor_untriggered_freezes_animation_tick_only() {
    let mut f = Fixture::new();
    f.actors[SOURCE_SLOT].hit_marker = 0;
    let old = f.actors[SOURCE_SLOT];
    let mut r = Recorder::default();
    f.actor(Family::HitActivated, &mut r);
    let mut expected = old;
    expected.animation_tick = 0;
    assert_eq!(f.actors[SOURCE_SLOT], expected);
    assert_eq!(f.progression.hit_dialogue_count, 0);
    assert!(r.calls.is_empty());
}
#[test]
fn first_hit_increments_wrapping_count_initializes_then_falls_in_same_invocation() {
    let mut f = Fixture::new();
    f.actors[SOURCE_SLOT].hit_marker = 1;
    f.actors[SOURCE_SLOT].vertical_velocity = -1;
    f.progression.hit_dialogue_count = ROW_END;
    let mut r = Recorder::default();
    f.actor(Family::HitActivated, &mut r);
    let a = f.actors[SOURCE_SLOT];
    assert_eq!(f.progression.hit_dialogue_count, 0);
    assert_eq!(
        (
            a.state,
            a.profile,
            a.vertical_velocity,
            a.local_counter,
            a.animation_tick
        ),
        (0xa1, 0x80, 0, 0, 0)
    );
    assert_eq!(r.calls, vec![Call::Motion(Movement::Fall, 0x2d, 1)]);
}
#[test]
fn first_fall_ground_contact_initializes_launch_without_rise_or_horizontal_motion_yet() {
    let mut f = Fixture::new();
    f.actors[SOURCE_SLOT].state = 0xa1;
    let mut r = Recorder {
        motion: CONTACT_BIT,
        ..Recorder::default()
    };
    f.actor(Family::HitActivated, &mut r);
    let a = f.actors[SOURCE_SLOT];
    assert_eq!(
        (
            a.state,
            a.horizontal_magnitude,
            a.vertical_velocity,
            a.animation_tick
        ),
        (0xa2, HIT_LAUNCH_SPEED, HIT_LAUNCH_VELOCITY, 0)
    );
    assert_eq!(r.calls, vec![Call::Motion(Movement::Fall, 0x2d, 1)]);
}
#[test]
fn first_fall_requires_solid_bit_not_any_collision_result() {
    let mut f = Fixture::new();
    f.actors[SOURCE_SLOT].state = 0xa1;
    let mut r = Recorder {
        motion: 2,
        ..Recorder::default()
    };
    f.actor(Family::HitActivated, &mut r);
    assert_eq!(f.actors[SOURCE_SLOT].state, 0xa1);
    assert_eq!(r.calls, vec![Call::Motion(Movement::Fall, 0x2d, 1)]);
}
#[test]
fn hit_arc_positive_velocity_keeps_rising_then_moves_left_and_freezes_animation() {
    for v in [0, 1, HIT_LAUNCH_VELOCITY, 0x7fff] {
        let mut f = Fixture::new();
        f.actors[SOURCE_SLOT].state = 0xa2;
        f.actors[SOURCE_SLOT].local_counter = 0;
        let mut r = Recorder {
            velocity_after_motion: Some(v),
            ..Recorder::default()
        };
        f.actor(Family::HitActivated, &mut r);
        assert_eq!(f.actors[SOURCE_SLOT].local_counter, 0);
        assert_eq!(f.actors[SOURCE_SLOT].animation_tick, 0);
        assert_eq!(
            r.calls,
            vec![
                Call::Motion(Movement::Rise, 0x2f, 1),
                Call::Motion(Movement::NegativeX, 0x2b, 1)
            ]
        );
    }
}
#[test]
fn hit_arc_negative_post_rise_velocity_selects_fall_for_next_invocation() {
    let mut f = Fixture::new();
    f.actors[SOURCE_SLOT].state = 0xa2;
    f.actors[SOURCE_SLOT].local_counter = 0;
    let mut r = Recorder {
        velocity_after_motion: Some(-1),
        ..Recorder::default()
    };
    f.actor(Family::HitActivated, &mut r);
    assert_eq!(f.actors[SOURCE_SLOT].local_counter, 1);
    assert_eq!(f.actors[SOURCE_SLOT].vertical_velocity, 0);
    // The following horizontal boundary may change vertical velocity in a real
    // contact response; this fixture leaves it unchanged to inspect apex clear.
    assert_eq!(
        r.calls,
        vec![
            Call::Motion(Movement::Rise, 0x2f, 1),
            Call::Motion(Movement::NegativeX, 0x2b, 1)
        ]
    );
}
#[test]
fn hit_arc_falling_without_ground_continues_left_without_opening_dialogue() {
    let mut f = Fixture::new();
    f.player.action = Action::Standing;
    f.actors[SOURCE_SLOT].state = 0xa2;
    f.actors[SOURCE_SLOT].local_counter = 1;
    let mut r = Recorder::default();
    f.actor(Family::HitActivated, &mut r);
    assert_eq!(f.player.action, Action::Standing);
    assert_eq!(
        r.calls,
        vec![
            Call::Motion(Movement::Fall, 0x2d, 1),
            Call::Motion(Movement::NegativeX, 0x2b, 1)
        ]
    );
}
#[test]
fn hit_arc_landing_disables_handler_marks_reveal_bit_suppresses_and_still_moves_left() {
    let mut f = Fixture::new();
    f.actors[SOURCE_SLOT].state = 0xa2;
    f.actors[SOURCE_SLOT].handler = Family::HitActivated as u8;
    f.actors[SOURCE_SLOT].local_counter = 1;
    f.actors[SOURCE_SLOT].local_state = 6;
    f.progression.seen_messages[0] = 8;
    let mut r = Recorder {
        ordered_motion: VecDeque::from([CONTACT_BIT, 0]),
        ..Recorder::default()
    };
    f.actor(Family::HitActivated, &mut r);
    assert_eq!(f.actors[SOURCE_SLOT].handler, 0);
    assert_eq!(f.actors[SOURCE_SLOT].state, 0xa2);
    assert_eq!(f.player.action, Action::Dialogue);
    assert_eq!(f.progression.seen_messages[0], 0x48);
    assert_eq!(f.dialogue.message, 6);
    assert_eq!(f.dialogue.actor_slot, SOURCE_SLOT);
    assert_eq!(f.suppression[19], 8);
    assert_eq!(
        r.calls,
        vec![
            Call::Motion(Movement::Fall, 0x2d, 1),
            Call::Motion(Movement::NegativeX, 0x2b, 1)
        ]
    );
}
#[test]
fn escort_initial_distance_at_threshold_waits_even_with_prerequisite() {
    for distance in [ESCORT_DISTANCE, ESCORT_DISTANCE + 1, ROW_END] {
        let mut f = Fixture::new();
        f.progression.escort_prerequisite = 1;
        let a = f.actors;
        let mut r = Recorder {
            distance,
            ..Recorder::default()
        };
        f.actor(Family::Escort, &mut r);
        assert_eq!(f.actors, a);
        assert_eq!(r.calls, vec![Call::Distance(SOURCE_SLOT)]);
    }
}
#[test]
fn escort_close_without_prerequisite_waits_without_suppression() {
    let mut f = Fixture::new();
    let a = f.actors;
    let mut r = Recorder {
        distance: ESCORT_DISTANCE - 1,
        ..Recorder::default()
    };
    f.actor(Family::Escort, &mut r);
    assert_eq!(f.actors, a);
    assert!(f.suppression.iter().all(|b| *b == 0));
    assert_eq!(f.progression.escort_prerequisite, 0);
}
#[test]
fn escort_start_marks_prerequisite_holds_player_copies_camera_and_suppresses_before_return() {
    for prerequisite in [1, 0x7f, 0x80, 0xff] {
        let mut f = Fixture::new();
        f.progression.escort_prerequisite = prerequisite;
        f.actors[SOURCE_SLOT].state = 0xb0;
        let mut r = Recorder {
            distance: ESCORT_DISTANCE - 1,
            ..Recorder::default()
        };
        f.actor(Family::Escort, &mut r);
        let a = f.actors[SOURCE_SLOT];
        assert_eq!(
            f.progression.escort_prerequisite,
            prerequisite | RELOAD_NEGATIVE
        );
        assert_eq!(f.player.action, Action::Hold);
        assert_eq!(
            (
                a.state,
                a.position.world_x,
                a.position.x_fraction,
                a.horizontal_magnitude,
                a.profile
            ),
            (0xa1, f.camera.x, RETAINED, ESCORT_SPEED, 0)
        );
        assert_eq!(f.suppression[19], 8);
        assert_eq!(r.calls, vec![Call::Distance(SOURCE_SLOT)]);
    }
}
#[test]
fn escort_approach_moves_and_sets_flip_before_close_distance_test() {
    let mut f = Fixture::new();
    f.actors[SOURCE_SLOT].state = 0xa1;
    f.actors[SOURCE_SLOT].horizontal_magnitude = ESCORT_SPEED;
    let mut r = Recorder {
        distance: ESCORT_DISTANCE,
        ..Recorder::default()
    };
    f.actor(Family::Escort, &mut r);
    assert_eq!(f.actors[SOURCE_SLOT].render, VISIBLE | FLIP);
    assert_eq!(f.actors[SOURCE_SLOT].position.world_x, 0x0321);
    assert_eq!(f.actors[SOURCE_SLOT].position.x_fraction, 0xd9);
    assert_eq!(
        r.calls,
        vec![Call::Integrate(true), Call::Distance(SOURCE_SLOT)]
    );
}
#[test]
fn escort_close_allocation_failure_still_sets_full_pause_and_never_restores_player() {
    let mut f = Fixture::new();
    f.actors[SOURCE_SLOT].state = 0xa1;
    f.player.action = Action::Hold;
    let old = f.actors;
    let mut r = Recorder {
        distance: ESCORT_DISTANCE - 1,
        allocation: None,
        ..Recorder::default()
    };
    f.actor(Family::Escort, &mut r);
    assert_eq!(f.actors[SOURCE_SLOT].state, 0xa2);
    assert_eq!(f.actors[SOURCE_SLOT].local_state, ESCORT_PAUSE);
    assert_eq!(f.player.action, Action::Hold);
    for (i, previous) in old.iter().enumerate() {
        if i != SOURCE_SLOT {
            assert_eq!(&f.actors[i], previous);
        }
    }
    assert_eq!(
        r.calls,
        vec![
            Call::Integrate(true),
            Call::Distance(SOURCE_SLOT),
            Call::Allocate
        ]
    );
}
#[test]
fn escort_companion_initialization_preserves_unwritten_slot_fields_and_source_slot() {
    let mut f = Fixture::new();
    f.actors[SOURCE_SLOT].state = 0xa1;
    f.actors[SOURCE_SLOT].horizontal_magnitude = ESCORT_SPEED;
    f.actors[CHILD_SLOT] = ActorContact {
        position: Position {
            world_x: 0xffff,
            screen_y: RETAINED,
            x_fraction: RETAINED,
            y_fraction: RETAINED,
        },
        profile: RETAINED,
        local_state: RETAINED,
        spawn_identity: Some(ROW_END),
        vertical_velocity: -1,
        ..ActorContact::default()
    };
    let old = f.actors[CHILD_SLOT];
    let mut r = Recorder {
        distance: 0,
        allocation: Some(CHILD_SLOT),
        ..Recorder::default()
    };
    f.actor(Family::Escort, &mut r);
    let a = f.actors[CHILD_SLOT];
    let mut expected = old;
    expected.animation = COMPANION_ANIMATION;
    expected.position.world_x = f.actors[SOURCE_SLOT].position.world_x;
    expected.position.screen_y = f.actors[SOURCE_SLOT].position.screen_y;
    expected.animation_frame = 0;
    expected.animation_tick = 0;
    expected.state = ACTIVE_WORLD;
    expected.render = ACTIVE_WORLD;
    expected.handler = COMPANION_HANDLER;
    assert_eq!(a, expected);
    assert_eq!(f.actors[SOURCE_SLOT].state, 0xa2);
    assert_eq!(f.actors[SOURCE_SLOT].local_state, ESCORT_PAUSE);
    assert_eq!(f.dialogue.actor_slot, SOURCE_SLOT);
}
#[test]
fn escort_pause_decrements_once_and_defers_exit_even_on_last_tick() {
    for before in [ESCORT_PAUSE, 1] {
        let mut f = Fixture::new();
        f.actors[SOURCE_SLOT].state = 0xa2;
        f.actors[SOURCE_SLOT].local_state = before;
        let mut r = Recorder::default();
        f.actor(Family::Escort, &mut r);
        assert_eq!(f.actors[SOURCE_SLOT].local_state, before - 1);
        assert!(r.calls.is_empty());
    }
}
#[test]
fn escort_after_pause_offscreen_deactivates_without_collision_or_render_reset() {
    for render in [0, 0x7f] {
        let mut f = Fixture::new();
        f.actors[SOURCE_SLOT].state = 0xa2;
        f.actors[SOURCE_SLOT].local_state = 0;
        f.actors[SOURCE_SLOT].render = render;
        let mut r = Recorder::default();
        f.actor(Family::Escort, &mut r);
        assert_eq!(f.actors[SOURCE_SLOT].state, 0);
        assert_eq!(f.actors[SOURCE_SLOT].render, render);
        assert!(r.calls.is_empty());
    }
}
#[test]
fn escort_after_pause_visible_falls_but_does_not_walk_without_ground_bit() {
    for bits in [0, 2] {
        let mut f = Fixture::new();
        f.actors[SOURCE_SLOT].state = 0xa2;
        f.actors[SOURCE_SLOT].local_state = 0;
        f.actors[SOURCE_SLOT].render = VISIBLE | FLIP;
        let mut r = Recorder {
            motion: bits,
            ..Recorder::default()
        };
        f.actor(Family::Escort, &mut r);
        assert_eq!(r.calls, vec![Call::Motion(Movement::Fall, 6, 2)]);
        assert_eq!(f.actors[SOURCE_SLOT].render, VISIBLE | FLIP);
    }
}
#[test]
fn escort_after_pause_grounded_walks_left_and_clears_only_flip() {
    let mut f = Fixture::new();
    f.actors[SOURCE_SLOT].state = 0xa2;
    f.actors[SOURCE_SLOT].local_state = 0;
    f.actors[SOURCE_SLOT].render = 0xe5;
    f.actors[SOURCE_SLOT].horizontal_magnitude = ESCORT_SPEED;
    let mut r = Recorder {
        motion: CONTACT_BIT,
        ..Recorder::default()
    };
    f.actor(Family::Escort, &mut r);
    assert_eq!(
        r.calls,
        vec![Call::Motion(Movement::Fall, 6, 2), Call::Integrate(false)]
    );
    assert_eq!(f.actors[SOURCE_SLOT].render, 0xa5);
    assert_eq!(f.actors[SOURCE_SLOT].position.world_x, 0x031e);
    assert_eq!(f.actors[SOURCE_SLOT].position.x_fraction, 0xd9);
}
#[test]
fn rightward_transition_phase_zero_reloads_speed_three_and_moves_without_phase_increment() {
    let mut f = Fixture::new();
    f.actors[SOURCE_SLOT].horizontal_magnitude = 0xffff;
    let mut r = Recorder::default();
    f.actor(Family::RightwardTransition, &mut r);
    assert_eq!(f.actors[SOURCE_SLOT].horizontal_magnitude, TRANSITION_SPEED);
    assert_eq!(f.actors[SOURCE_SLOT].state, ACTIVE_WORLD);
    assert_eq!(f.actors[SOURCE_SLOT].position.world_x, 0x0323);
    assert_eq!(r.calls, vec![Call::Integrate(true)]);
}
#[test]
fn rightward_transition_nonzero_phase_retains_speed_and_tests_post_integration_position() {
    let mut f = Fixture::new();
    f.actors[SOURCE_SLOT].state = 0xa1;
    f.actors[SOURCE_SLOT].horizontal_magnitude = ESCORT_SPEED;
    let mut r = Recorder::default();
    f.actor(Family::RightwardTransition, &mut r);
    assert_eq!(f.actors[SOURCE_SLOT].horizontal_magnitude, ESCORT_SPEED);
    assert_eq!(f.actors[SOURCE_SLOT].position.world_x, 0x0321);
}
#[test]
fn rightward_transition_requires_exact_page_seven_and_low_x_at_least_seventy_hex() {
    for x in [0x066f, 0x0670, 0x076f, 0x0870, 0x08ff] {
        let mut f = Fixture::new();
        f.actors[SOURCE_SLOT].state = 0xa1;
        f.actors[SOURCE_SLOT].horizontal_magnitude = 0;
        f.actors[SOURCE_SLOT].position.world_x = x;
        let mut r = Recorder::default();
        f.actor(Family::RightwardTransition, &mut r);
        assert_eq!(f.actors[SOURCE_SLOT].state, 0xa1);
        assert_eq!(f.player.action, Action::Dialogue);
        assert_eq!(r.calls, vec![Call::Integrate(true)]);
    }
}
#[test]
fn rightward_transition_finishes_at_threshold_restores_player_then_burst_six_then_actor_bank() {
    for low in [TRANSITION_END_X, ROW_END] {
        let mut f = Fixture::new();
        f.actors[SOURCE_SLOT].state = 0xa1;
        f.actors[SOURCE_SLOT].horizontal_magnitude = 0;
        f.actors[SOURCE_SLOT].position.world_x =
            u16::from(TRANSITION_END_PAGE) << 8 | u16::from(low);
        let mut r = Recorder::default();
        f.actor(Family::RightwardTransition, &mut r);
        assert_eq!(f.actors[SOURCE_SLOT].state, 0);
        assert_eq!(f.player.action, Action::Standing);
        assert_eq!(
            r.calls,
            vec![
                Call::Integrate(true),
                Call::Work(Work::ChangedBurst(6)),
                Call::Work(Work::RestoreActorBank)
            ]
        );
        assert!(
            r.observations
                .iter()
                .all(|(_, action, _, _)| *action == Action::Standing)
        );
    }
}

fn assert_original_render(selector: u8, sound_count: usize, scroll_count: usize) {
    let rom = std::fs::read("../input/game.rom").unwrap();
    let c = Catalog::original(&rom).unwrap();
    let mut d = Dialogue {
        message: selector,
        ..Dialogue::default()
    };
    let mut cursor = 0;
    let mut p = Panel::default();
    let mut sounds = 0;
    let mut scrolls = 0;
    let mut done = false;
    for _ in 0..256 {
        let step = render_text(&mut d, &mut cursor, &mut p, &c).unwrap();
        if step.sound.is_some() {
            assert_eq!(step.sound, Some(GLYPH_SOUND));
            assert_eq!(step.wait_ticks, GLYPH_WAIT);
            sounds += 1;
        }
        match step.boundary {
            TextBoundary::MessageComplete => {
                done = true;
                break;
            }
            TextBoundary::PageComplete => {
                assert_ne!(cursor, 0);
                scrolls += 1;
                scroll_rows(&mut d, &mut p);
                // Direct renderer harness resumes the prescribed one-frame scroll
                // continuation; full scheduler/input ordering is tested in resume.
                d.phase = Phase::Render;
                d.line = 2;
                d.row_glyphs = 0;
            }
            TextBoundary::GlyphWait => {}
        }
    }
    assert!(
        done,
        "selector{selector:02X} exceeded bounded original text"
    );
    assert_eq!(cursor, 0);
    assert_eq!(sounds, sound_count);
    assert_eq!(scrolls, scroll_count);
    let original = &c.messages[usize::from(selector)];
    let last = original[..original.len() - 1]
        .split(|b| *b == ROW_END)
        .next_back()
        .unwrap();
    let mut expected = last.to_vec();
    expected.push(FINAL_GLYPH);
    assert_eq!(p.rows()[usize::from(d.line)], expected);
    assert_eq!(p.pending, ROW_END);
}
macro_rules! original_message {
    ($name:ident,$selector:expr,$sounds:expr,$scrolls:expr) => {
        #[test]
        fn $name() {
            assert_original_render($selector, $sounds, $scrolls);
        }
    };
}
// Counts independently enumerated from original delimited streams; no asset
// payload is embedded. Each selector is a separately discovered executable case.
original_message!(
    original_message_00_full_glyph_wait_and_page_sequence,
    0,
    37,
    1
);
original_message!(
    original_message_01_full_glyph_wait_and_page_sequence,
    1,
    39,
    1
);
original_message!(
    original_message_02_full_glyph_wait_and_page_sequence,
    2,
    55,
    2
);
original_message!(
    original_message_03_full_glyph_wait_and_page_sequence,
    3,
    40,
    1
);
original_message!(
    original_message_04_full_glyph_wait_and_page_sequence,
    4,
    24,
    0
);
original_message!(
    original_message_05_full_glyph_wait_and_page_sequence,
    5,
    36,
    0
);
original_message!(
    original_message_06_full_glyph_wait_and_page_sequence,
    6,
    37,
    1
);
original_message!(
    original_message_07_full_glyph_wait_and_page_sequence,
    7,
    50,
    2
);
original_message!(
    original_message_08_full_glyph_wait_and_page_sequence,
    8,
    44,
    2
);
original_message!(
    original_message_09_full_glyph_wait_and_page_sequence,
    9,
    44,
    1
);
original_message!(
    original_message_0a_full_glyph_wait_and_page_sequence,
    10,
    38,
    2
);
original_message!(
    original_message_0b_full_glyph_wait_and_page_sequence,
    11,
    80,
    4
);
original_message!(
    original_message_0c_full_glyph_wait_and_page_sequence,
    12,
    63,
    3
);
original_message!(
    original_message_0d_full_glyph_wait_and_page_sequence,
    13,
    19,
    0
);
original_message!(
    original_message_0e_full_glyph_wait_and_page_sequence,
    14,
    34,
    1
);
original_message!(
    original_message_0f_full_glyph_wait_and_page_sequence,
    15,
    43,
    2
);
original_message!(
    original_message_10_full_glyph_wait_and_page_sequence,
    16,
    18,
    0
);
original_message!(
    original_message_11_full_glyph_wait_and_page_sequence,
    17,
    40,
    1
);

#[test]
fn race_terrain_nested_yields_finish_before_dialogue_panel_setup() {
    let mut f = Fixture::new();
    f.dialogue.message = OutcomeMessage::RaceIntro as u8;
    let mut recorder = Recorder {
        suspension: Some(Work::BuildRaceTerrain),
        ..Default::default()
    };
    let mut input = pressed(Buttons::NONE);
    let mut panel = Panel::default();
    assert_eq!(
        resume(
            &mut f.context(),
            &mut input,
            &mut panel,
            &catalog(&[MESSAGE_END]),
            &mut recorder
        )
        .unwrap(),
        OwnerStep::Suspended
    );
    assert_eq!(f.dialogue.phase, Phase::RaceTerrain);
    assert!(panel.encoded.is_empty());
    recorder.suspension = None;
    resume(
        &mut f.context(),
        &mut input,
        &mut panel,
        &catalog(&[MESSAGE_END]),
        &mut recorder,
    )
    .unwrap();
    assert_eq!(
        recorder.calls,
        [
            Call::Work(Work::BuildRaceTerrain),
            Call::Work(Work::BuildRaceTerrain),
            Call::Work(Work::SelectGraphicsTwo),
            Call::Work(Work::ClearPanel),
            Call::Work(Work::Yield)
        ]
    );
    assert_eq!(f.dialogue.phase, Phase::ClearPanelYield);
}
#[test]
fn panel_preparation_preserves_unwritten_canonical_queue_tail_for_audio_aliases() {
    let mut panel = Panel {
        encoded: vec![0x79; 64],
        pending: 0,
    };
    prepare_panel(&mut panel, &[TEST_GLYPH; PANEL_TEMPLATE_BYTES]).unwrap();
    assert_eq!(panel.encoded.len(), 64);
    assert_eq!(panel.encoded[PANEL_TEMPLATE_BYTES + 3], ROW_END);
    assert_eq!(&panel.encoded[PANEL_TEMPLATE_BYTES + 4..], &[0x79; 44]);
}

#[test]
fn outcome_burst_receives_the_current_panel_owner_without_losing_its_retained_tail() {
    let mut f = Fixture::new();
    f.dialogue.phase = Phase::Outcome;
    f.dialogue.message = OutcomeMessage::Reveal as u8;
    let mut panel = Panel {
        encoded: vec![0x55; 64],
        pending: 0,
    };
    let mut recorder = Recorder {
        panel_patch: Some((Work::ChangedBurst(0), 0x22)),
        ..Recorder::default()
    };
    let result = resume(
        &mut f.context(),
        &mut pressed(Buttons::A),
        &mut panel,
        &catalog(&[MESSAGE_END]),
        &mut recorder,
    )
    .unwrap();
    assert_eq!(result, OwnerStep::Complete(()));
    assert_eq!(recorder.calls, [Call::Work(Work::ChangedBurst(0))]);
    assert_eq!((panel.encoded[0], panel.encoded[63]), (0x22, 0x55));
}

#[test]
fn outcome_exposes_actual_payment_and_life_loss_final_operations() {
    use crate::progression::FirstActorIncrement::{One, Zero};
    let mut f = Fixture::new();
    let mut r = Recorder::default();
    for (codes, expected) in [
        ([1, 3, 0, 0, 0, 0], One),
        ([0, 3, 0, 0, 0, 0], One),
        ([0, 3, BLANK, 0, 0, 7], Zero),
        ([0, 2, 0, 0, 0, 0], Zero),
    ] {
        f.status.reward = codes;
        f.outcome(OutcomeMessage::PaidReward, Buttons::A, &mut r);
        assert_eq!(
            r.outcome_returns.last().unwrap().first_actor_increment(),
            expected
        );
    }
    f.outcome(OutcomeMessage::PaidReward, Buttons::B, &mut r);
    assert_eq!(
        r.outcome_returns.last(),
        Some(&OutcomeReturn::PaymentRejected)
    );
    f.progression.stage.0 = 0xff;
    f.progression.life_loss_variant = 1;
    f.outcome(OutcomeMessage::LifeLoss, Buttons::A, &mut r);
    assert_eq!(
        r.outcome_returns.last(),
        Some(&OutcomeReturn::LifeLossVariantOverflow(false))
    );
    f.progression.life_loss_variant = 0xff;
    f.outcome(OutcomeMessage::LifeLoss, Buttons::A, &mut r);
    assert_eq!(
        r.outcome_returns.last(),
        Some(&OutcomeReturn::LifeLossVariantOverflow(true))
    );
}

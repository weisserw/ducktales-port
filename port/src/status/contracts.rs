//! Source-derived B14 contracts; detailed per-branch arithmetic is in trace.md.
//! Fixtures are synthetic, except matching-local-ROM format metadata checks.
use super::*;
use crate::{actors::ActorContact, config};
use romatlas_port::assets::Rom;
const REWARD_HUNDREDS_DIGIT: usize = 3;
const REWARD_THOUSANDS_DIGIT: usize = 2;
const REWARD_TOP_DIGIT: usize = 0;
const UNIT_INCREMENT: u8 = 1;
const EXTRA_REWARD_INCREMENT: u8 = 5;
const SPECIAL_ANIMATION: u8 = 0x25;
const EXTRA_ANIMATION: u8 = 0x26;
const ALTERNATE_SPECIAL_ANIMATION: u8 = 0x28;
const ALTERNATE_EXTRA_ANIMATION: u8 = 0x24;
const ORDINARY_PLAYER: u8 = 0;
const DEATH_PLAYER: u8 = 0x0d;
const COMPLETION_PLAYER: u8 = 0x10;
const OVERLAY_Y: u8 = 0x20;
const OVERLAY_TILE: u8 = 0x2d;
const OVERLAY_FIRST_X: u8 = 0x30;
const OVERLAY_X_STEP: u8 = 8;
const END_OVERLAY: u8 = 0xff;
const WARNING_BOUNDARY: u8 = 3;
fn health() -> Health {
    Health {
        segments: std::array::from_fn(|i| HealthSegment {
            y: if i == 3 { END_OVERLAY } else { OVERLAY_Y },
            tile: OVERLAY_TILE,
            depleted: 0,
            x: OVERLAY_FIRST_X + i as u8 * OVERLAY_X_STEP,
        }),
        end: END_OVERLAY,
        alternating: 0,
    }
}
fn state() -> Status {
    Status {
        reward: [BLANK; 6],
        total: [BLANK; 7],
        lives: 3,
        health: health(),
        countdown: [5, 0, 0],
        divider: COUNTDOWN_PERIOD,
        dirty: 0,
        ui_pending: 0,
    }
}
fn rom() -> Vec<u8> {
    let bytes =
        std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../input/game.rom"))
            .expect("local matching ROM fixture");
    Rom::from_bytes(bytes.clone(), config::ROM_SHA256).expect("matching ROM");
    bytes
}
#[test]
fn reward_add_preserves_leading_blank_codes_and_trailing_display_digits() {
    let mut s = state();
    s.reward = [BLANK, BLANK, BLANK, 1, 0, 0];
    s.add_reward(REWARD_HUNDREDS_DIGIT, UNIT_INCREMENT);
    assert_eq!(s.reward, [BLANK, BLANK, BLANK, 2, 0, 0]);
    assert_eq!(s.dirty, 1);
}
#[test]
fn reward_add_to_blank_normalizes_encoded_sum_above_blank() {
    let mut s = state();
    s.add_reward(REWARD_THOUSANDS_DIGIT, EXTRA_REWARD_INCREMENT);
    assert_eq!(s.reward, [BLANK, BLANK, 5, BLANK, BLANK, BLANK]);
    assert_eq!(s.dirty, 1);
}
#[test]
fn reward_add_zero_at_blank_takes_equal_blank_exit() {
    let mut s = state();
    s.add_reward(REWARD_TOP_DIGIT, 0);
    assert_eq!(s.reward, [BLANK; 6]);
    assert_eq!(s.dirty, 1);
}
#[test]
fn reward_carry_crosses_every_higher_digit_and_saturates_upper_four() {
    let cases = [
        ([BLANK, BLANK, 9, 9, 0, 0], [BLANK, 1, 0, 0, 0, 0]),
        ([9, 9, 9, 9, 7, 8], [9, 9, 9, 9, 7, 8]),
        ([BLANK, 9, 9, 9, 0, 0], [1, 0, 0, 0, 0, 0]),
    ];
    for (before, after) in cases {
        let mut s = state();
        s.reward = before;
        s.add_reward(REWARD_HUNDREDS_DIGIT, UNIT_INCREMENT);
        assert_eq!(s.reward, after);
        assert_eq!(s.dirty, 1);
    }
}
#[test]
fn saturators_fill_only_source_owned_upper_widths_without_dirty_increment() {
    let mut s = state();
    s.reward = [1, 2, 3, 4, 5, 6];
    s.total = [1, 2, 3, 4, 5, 6, 7];
    s.saturate_reward();
    s.saturate_total();
    assert_eq!(s.reward, [9, 9, 9, 9, 5, 6]);
    assert_eq!(s.total, [9, 9, 9, 9, 9, 6, 7]);
    assert_eq!(s.dirty, 0);
}
#[test]
fn dirty_request_wraps_as_source_byte() {
    let mut s = state();
    s.dirty = u8::MAX;
    s.add_reward(REWARD_TOP_DIGIT, UNIT_INCREMENT);
    assert_eq!(s.dirty, 0);
}
#[test]
fn theft_subtracts_one_at_third_digit_and_increments_same_actor_phase() {
    let mut s = state();
    s.reward = [BLANK, 1, 0, 6, 0, 0];
    let mut a = ActorContact {
        local_state: u8::MAX,
        ..Default::default()
    };
    s.steal(&mut a);
    assert_eq!(s.reward, [BLANK, BLANK, 9, 6, 0, 0]);
    assert_eq!(a.local_state, 0);
    assert_eq!(s.dirty, 1);
}
#[test]
fn theft_borrow_crosses_blank_upper_digits_and_underflow_blanks_all_six() {
    for reward in [[BLANK; 6], [0, 0, 0, 9, 9, 9], [BLANK, BLANK, 0, 1, 2, 3]] {
        let mut s = state();
        s.reward = reward;
        s.steal(&mut ActorContact::default());
        assert_eq!(s.reward, [BLANK; 6]);
        assert_eq!(s.dirty, 1);
    }
}
#[test]
fn theft_zero_result_blanks_leading_run_but_retains_lower_nonzero() {
    let mut s = state();
    s.reward = [0, 0, 1, 0, 2, 0];
    s.steal(&mut ActorContact::default());
    assert_eq!(s.reward, [BLANK, BLANK, BLANK, BLANK, 2, 0]);
}
#[test]
fn theft_exact_exhaustion_blanks_all_digits() {
    let mut s = state();
    s.reward = [BLANK, BLANK, 1, 0, 0, 0];
    s.steal(&mut ActorContact::default());
    assert_eq!(s.reward, [BLANK; 6]);
}
#[test]
fn dialogue_payment_is_atomic_on_insufficient_upper_two_digits() {
    for reward in [[BLANK; 6], [0, 2, 9, 9, 9, 9], [BLANK, 2, 0, 0, 0, 0]] {
        let mut s = state();
        s.reward = reward;
        let old = s.clone();
        assert!(!s.pay(Cost::RewardDialogue, false));
        assert_eq!(s, old);
    }
}
#[test]
fn dialogue_payment_exact_and_borrow_cases_preserve_lower_value() {
    for (before, after) in [
        ([0, 3, 0, 0, 0, 0], [BLANK; 6]),
        ([1, 0, 4, 0, 0, 0], [BLANK, 7, 4, 0, 0, 0]),
        ([BLANK, 3, 0, 2, 0, 0], [BLANK, BLANK, BLANK, 2, 0, 0]),
    ] {
        let mut s = state();
        s.reward = before;
        assert!(s.pay(Cost::RewardDialogue, false));
        assert_eq!(s.reward, after);
        assert_eq!(s.dirty, 1);
    }
}
#[test]
fn rejected_dialogue_choice_does_not_pay_or_dirty() {
    let mut s = state();
    s.reward = [9; 6];
    let old = s.clone();
    assert!(!s.pay(Cost::RewardDialogue, true));
    assert_eq!(s, old);
}
#[test]
fn total_overlay_payment_requires_preceding_damage_flag() {
    let mut s = state();
    s.total = [9; 7];
    let old = s.clone();
    assert!(!s.pay(Cost::TotalOverlay, false));
    assert_eq!(s, old);
}
#[test]
fn total_overlay_payment_checks_upper_two_digits_and_commits_all_or_none() {
    let mut s = state();
    s.health.segments[2].depleted = 1;
    s.total = [BLANK, 2, 9, 9, 9, 9, 9];
    let old = s.clone();
    assert!(!s.pay(Cost::TotalOverlay, false));
    assert_eq!(s, old);
    s.total = [1, 0, 0, 0, 0, 0, 0];
    assert!(s.pay(Cost::TotalOverlay, false));
    assert_eq!(s.total, [BLANK, 7, 0, 0, 0, 0, 0]);
    assert!(s.health.segments.iter().all(|x| x.depleted == 0));
    assert_eq!(s.ui_pending, 1);
    assert_eq!(s.dirty, 0);
}
#[test]
fn total_payment_normalizes_seven_codes_and_resets_alternation() {
    let mut s = state();
    s.health.segments[2].depleted = 1;
    s.health.alternating = 7;
    s.total = [BLANK, 3, 0, 0, 0, 0, 0];
    assert!(s.pay(Cost::TotalOverlay, false));
    assert_eq!(s.total, [BLANK; 7]);
    assert_eq!(s.health.alternating, 0);
}
#[test]
fn ordinary_damage_marks_last_available_segment_then_walks_toward_first() {
    let mut h = health();
    for expected in [[0, 0, 1, 0, 0], [0, 1, 1, 0, 0], [1, 1, 1, 0, 0]] {
        h.damage(TitleRule::OrdinaryDamage);
        assert_eq!(h.segments.map(|s| s.depleted), expected);
    }
}
#[test]
fn alternating_damage_skips_odd_events_and_wraps_latch() {
    let mut h = health();
    h.damage(TitleRule::AlternatingDamage);
    assert_eq!(h.alternating, 1);
    assert_eq!(h.segments[2].depleted, 0);
    h.damage(TitleRule::AlternatingDamage);
    assert_eq!(h.alternating, 2);
    assert_eq!(h.segments[2].depleted, 1);
    h.alternating = u8::MAX;
    h.damage(TitleRule::AlternatingDamage);
    assert_eq!(h.alternating, 0);
    assert_eq!(h.segments[1].depleted, 1);
}
#[test]
fn alternate_rewards_option_uses_each_damage_event_without_alternating_latch() {
    let mut h = health();
    h.alternating = 5;
    h.damage(TitleRule::AlternateRewards);
    assert_eq!(h.segments[2].depleted, 1);
    assert_eq!(h.alternating, 5);
}
#[test]
fn recover_one_clears_first_depleted_flag_and_alternation() {
    let mut h = health();
    h.alternating = 3;
    h.segments[1].depleted = 1;
    h.segments[2].depleted = 1;
    h.recover_one();
    assert_eq!(h.segments.map(|s| s.depleted), [0, 0, 1, 0, 0]);
    assert_eq!(h.alternating, 0);
}
#[test]
fn recovery_stops_at_negative_y_and_does_not_reveal_hidden_segments() {
    let mut h = health();
    h.segments[4].depleted = 1;
    h.recover_one();
    assert_eq!(h.segments[4].depleted, 1);
    assert_eq!(h.segments[3].y, END_OVERLAY);
}
#[test]
fn restore_clears_five_flags_but_preserves_geometry_and_terminator() {
    let mut h = health();
    for s in &mut h.segments {
        s.depleted = 3;
    }
    h.alternating = 9;
    let mut expected = h.clone();
    for s in &mut expected.segments {
        s.depleted = 0;
    }
    expected.alternating = 0;
    h.restore();
    assert_eq!(h, expected);
}
#[test]
fn extra_health_reveals_fourth_then_fifth_segment_and_stops_at_capacity() {
    let mut h = health();
    assert!(h.extend());
    assert_eq!(h.segments[3].y, OVERLAY_Y);
    assert_eq!(h.segments[4].y, END_OVERLAY);
    assert!(h.extend());
    assert_eq!(h.segments[4].y, OVERLAY_Y);
    assert_eq!(h.end, END_OVERLAY);
    let before = h.clone();
    assert!(!h.extend());
    assert_eq!(h, before);
}
#[test]
fn alternate_rule_remaps_only_two_animation_selectors() {
    for rule in [
        TitleRule::AlternatingDamage,
        TitleRule::OrdinaryDamage,
        TitleRule::AlternateRewards,
    ] {
        for animation in 0..=u8::MAX {
            let expected = match (rule, animation) {
                (TitleRule::AlternateRewards, SPECIAL_ANIMATION) => ALTERNATE_SPECIAL_ANIMATION,
                (TitleRule::AlternateRewards, EXTRA_ANIMATION) => ALTERNATE_EXTRA_ANIMATION,
                _ => animation,
            };
            assert_eq!(option_reward(rule, animation), expected);
        }
    }
}
#[test]
fn life_reward_stops_at_exactly_nine_not_generic_clamping() {
    for (before, after) in [
        (0, 1),
        (8, MAX_LIVES),
        (MAX_LIVES, MAX_LIVES),
        (10, 11),
        (u8::MAX, 0),
    ] {
        let mut s = state();
        s.lives = before;
        s.gain_life();
        assert_eq!(s.lives, after);
    }
}
#[test]
fn countdown_exempt_states_preserve_expiration_and_divider() {
    for player in [DEATH_PLAYER, COMPLETION_PLAYER] {
        let mut s = state();
        s.countdown = EXPIRED;
        s.divider = 1;
        let before = s.clone();
        assert_eq!(s.timer_tick(player, 0), TimerOutcome::default());
        assert_eq!(s, before);
    }
}
#[test]
fn expired_marker_enters_death_and_clears_late_route_without_decrement() {
    let mut s = state();
    s.countdown = EXPIRED;
    s.divider = 1;
    assert_eq!(
        s.timer_tick(ORDINARY_PLAYER, 0),
        TimerOutcome {
            player_state: Some(DEATH_PLAYER),
            clear_late_route: true,
            sound: None
        }
    );
    assert_eq!(s.divider, 1);
}
#[test]
fn countdown_divider_decrements_before_zero_test_and_zero_wraps() {
    for (before, after, digits) in [
        (2, 1, [5, 0, 0]),
        (0, u8::MAX, [5, 0, 0]),
        (1, COUNTDOWN_PERIOD, [4, 9, 9]),
    ] {
        let mut s = state();
        s.divider = before;
        s.timer_tick(ORDINARY_PLAYER, 1);
        assert_eq!(s.divider, after);
        assert_eq!(s.countdown, digits);
    }
}
#[test]
fn countdown_borrow_and_exact_zero_install_non_numeric_expiration() {
    for (before, after) in [
        ([1, 0, 0], [0, 9, 9]),
        ([0, 1, 0], [0, 0, 9]),
        ([0, 0, 1], EXPIRED),
        ([0, 0, 0], [9, 9, 9]),
    ] {
        let mut s = state();
        s.countdown = before;
        s.divider = 1;
        s.timer_tick(ORDINARY_PLAYER, 1);
        assert_eq!(s.countdown, after);
    }
}
#[test]
fn warning_uses_post_decrement_digits_and_composition_modulo_sixteen() {
    for (digits, tick, sound) in [
        ([0, 2, 9], 0, Some(WARNING_SOUND)),
        ([0, WARNING_BOUNDARY, 0], 0, None),
        ([1, 0, 0], 0, None),
        ([0, 2, 0], 15, None),
        ([0, 0, 0], 16, Some(WARNING_SOUND)),
    ] {
        let mut s = state();
        s.countdown = digits;
        s.divider = 2;
        assert_eq!(s.timer_tick(ORDINARY_PLAYER, tick).sound, sound);
    }
}
#[test]
fn countdown_crossing_thirty_to_twenty_nine_can_warn_same_tick() {
    let mut s = state();
    s.countdown = [0, 3, 0];
    s.divider = 1;
    assert_eq!(s.timer_tick(ORDINARY_PLAYER, 0).sound, Some(WARNING_SOUND));
    assert_eq!(s.countdown, [0, 2, 9]);
}
#[test]
fn reward_refresh_busy_is_transactional_and_retains_dirty() {
    let mut s = state();
    s.reward = [BLANK, 0, BLANK, 1, BLANK, 0];
    s.dirty = 4;
    let old = s.clone();
    assert_eq!(s.queue_reward(true), None);
    assert_eq!(s, old);
}
#[test]
fn refresh_preserves_initial_blanks_and_zeroes_later_blank_codes() {
    let mut s = state();
    s.reward = [BLANK, BLANK, 1, BLANK, 0, BLANK];
    s.dirty = 3;
    assert_eq!(
        s.queue_reward(false),
        Some(vec![0x20, 0x43, 7, 0x2c, BLANK, BLANK, 1, 0, 0, 0, 0, 0xff])
    );
    assert_eq!(s.reward, [BLANK, BLANK, 1, 0, 0, 0]);
    assert_eq!(s.dirty, 0);
}
#[test]
fn refresh_all_blank_still_queues_template_without_normalization() {
    let mut s = state();
    s.dirty = 1;
    let upload = s.queue_reward(false).unwrap();
    assert_eq!(upload.len(), 12);
    assert_eq!(&upload[4..10], &[BLANK; 6]);
    assert_eq!(s.reward, [BLANK; 6]);
    assert_eq!(s.dirty, 0);
}
#[test]
fn stage_ui_busy_retains_latch_and_free_upload_copies_thirteen_bytes() {
    let mut s = state();
    s.total = [1, 2, 3, 4, 5, 6, 7];
    s.ui_pending = 2;
    assert_eq!(s.queue_ui(true), None);
    assert_eq!(s.ui_pending, 2);
    assert_eq!(
        s.queue_ui(false),
        Some(vec![0x20, 0x6c, 8, 0x2c, 1, 2, 3, 4, 5, 6, 7, 0, 0xff])
    );
    assert_eq!(s.ui_pending, 0);
}
#[test]
fn transfer_blank_final_source_code_returns_before_arithmetic_or_upload() {
    let mut s = state();
    s.reward = [1, 2, 3, 4, 0, BLANK];
    let before = s.clone();
    let mut session = TransferSession {
        source: s.reward,
        started: false,
    };
    let out = s.transfer_batch(TransferLayout::Stage, &mut session);
    assert!(out.complete);
    assert_eq!(out.moved, 0);
    assert_eq!(out.submission_value, None);
    assert_eq!(out.upload, None);
    assert_eq!(s, before);
}
#[test]
fn transfer_final_partial_batch_has_no_sound_or_two_frame_wait() {
    let mut s = state();
    s.reward = [BLANK, BLANK, BLANK, 2, 0, 0];
    let mut session = TransferSession {
        source: s.reward,
        started: false,
    };
    let out = s.transfer_batch(TransferLayout::Stage, &mut session);
    assert_eq!(out.moved, 2);
    assert_eq!(out.submission_value, Some(0xff));
    assert!(out.complete);
    assert_eq!(out.sound, None);
    assert_eq!(out.wait_ticks, 0);
    assert_eq!(s.total, [BLANK, BLANK, BLANK, BLANK, 2, 0, 0]);
    assert_eq!(session.source, [BLANK; 6]);
    assert_eq!(s.reward, [BLANK, BLANK, BLANK, 2, 0, 0]);
    let upload = out.upload.unwrap();
    assert_eq!(&upload[..2], &[0x22, 0x0a]);
    assert_eq!(&upload[12..14], &[0x22, 0x6b]);
}
#[test]
fn transfer_ten_units_submits_sound_and_yields_two_frames_until_next_batch() {
    let mut s = state();
    s.reward = [BLANK, BLANK, 1, 1, 0, 0];
    let mut session = TransferSession {
        source: s.reward,
        started: false,
    };
    let out = s.transfer_batch(TransferLayout::Selection, &mut session);
    assert_eq!(out.moved, TRANSFER_BATCH);
    assert!(!out.complete);
    assert_eq!(out.sound, Some(VALUE_SOUND));
    assert_eq!(out.wait_ticks, TRANSFER_WAIT);
    let upload = out.upload.unwrap();
    assert_eq!(&upload[..2], &[0x20, 0x6c]);
    assert_eq!(&upload[12..14], &[0x20, 0x43]);
    assert_eq!(session.source, [BLANK, BLANK, BLANK, 1, 0, 0]);
    assert_eq!(s.reward, [BLANK, BLANK, 1, 1, 0, 0]);
    assert_eq!(s.total, [BLANK, BLANK, BLANK, 1, 0, 0, 0]);
}
#[test]
fn transfer_exact_tenth_exhaustion_returns_without_full_batch_sound() {
    let mut s = state();
    s.reward = [BLANK, BLANK, 1, 0, 0, 0];
    let mut session = TransferSession {
        source: s.reward,
        started: false,
    };
    let out = s.transfer_batch(TransferLayout::Stage, &mut session);
    assert_eq!(out.moved, TRANSFER_BATCH);
    assert!(out.complete);
    assert_eq!(out.sound, None);
    assert_eq!(out.wait_ticks, 0);
}
#[test]
fn transfer_total_carry_saturates_five_digits_and_keeps_trailing_zeroes() {
    let mut s = state();
    s.reward = [BLANK, BLANK, BLANK, 1, 0, 0];
    s.total = [9, 9, 9, 9, 9, 7, 8];
    let mut session = TransferSession {
        source: s.reward,
        started: false,
    };
    let out = s.transfer_batch(TransferLayout::Stage, &mut session);
    assert!(out.complete);
    assert_eq!(s.total, [9, 9, 9, 9, 9, 0, 0]);
}
#[test]
fn conversion_nonnegative_stage_audio_waits_full_wrapped_zero_count() {
    let mut s = state();
    let old = s.clone();
    let out = s.convert_timer(ConversionPhase::Entry, 0, 1);
    assert_eq!(out.wait_ticks, AUDIO_SETTLE_WAIT);
    assert_eq!(out.stage_sound, None);
    assert_eq!(s, old);
}
#[test]
fn conversion_resumes_audio_wait_with_stop_command_even_on_even_tick() {
    let mut s = state();
    let out = s.convert_timer(ConversionPhase::AfterAudioWait, 0, 0);
    assert_eq!(out.stage_sound, Some(STOP_STAGE_SOUND));
    assert!(!out.freeze_player_animation);
    assert_eq!(s.countdown, [5, 0, 0]);
}
#[test]
fn conversion_even_composition_tick_preserves_digits_and_dirty() {
    let mut s = state();
    let old = s.clone();
    assert_eq!(
        s.convert_timer(ConversionPhase::Running, STOP_STAGE_SOUND, 2),
        ConversionOutcome::default()
    );
    assert_eq!(s, old);
}
#[test]
fn conversion_odd_tick_discards_units_borrows_upper_pair_and_adds_reward() {
    let mut s = state();
    s.countdown = [1, 0, 9];
    let out = s.convert_timer(ConversionPhase::Running, STOP_STAGE_SOUND, 1);
    assert_eq!(s.countdown, [0, 9, 0]);
    assert_eq!(s.reward, [BLANK, BLANK, BLANK, 1, BLANK, BLANK]);
    assert_eq!(s.dirty, 1);
    assert_eq!(out.value_sound, Some(VALUE_SOUND));
    assert!(out.freeze_player_animation);
}
#[test]
fn conversion_upper_pair_empty_waits_sixty_then_signals_only_after_wait() {
    let mut s = state();
    s.countdown = [0, 0, 9];
    let out = s.convert_timer(ConversionPhase::Running, STOP_STAGE_SOUND, 1);
    assert_eq!(out.wait_ticks, CONVERSION_FINISH_WAIT);
    assert!(!out.completion_increment);
    assert_eq!(out.value_sound, None);
    assert_eq!(s.countdown, [0, 0, 0]);
    let finish = s.convert_timer(ConversionPhase::AfterFinishWait, STOP_STAGE_SOUND, 0);
    assert!(finish.completion_increment);
}
#[test]
fn conversion_overflow_saturates_reward_upper_four_and_increments_dirty_byte() {
    let mut s = state();
    s.countdown = [0, 1, 0];
    s.reward = [9, 9, 9, 9, 0, 0];
    s.dirty = u8::MAX;
    s.convert_timer(ConversionPhase::Running, STOP_STAGE_SOUND, 1);
    assert_eq!(s.reward, [9, 9, 9, 9, 0, 0]);
    assert_eq!(s.dirty, 0);
    assert_eq!(s.countdown, [0, 0, 0]);
}
#[test]
fn extending_health_restores_all_flags_and_alternating_damage() {
    let mut h = health();
    h.alternating = 9;
    for s in &mut h.segments {
        s.depleted = 1;
    }
    assert!(h.extend());
    assert_eq!(h.segments.map(|s| s.depleted), [0; OVERLAY_SEGMENTS]);
    assert_eq!(h.alternating, 0);
}
#[test]
fn original_all_stream_rows_and_conditional_fragment_locations_are_decoded() {
    let c = StatusCatalog::decode_original(&rom()).unwrap();
    assert_eq!(c.streams.len(), 10);
    for selector in [0, 1, 4, 9] {
        assert_eq!(c.streams[selector].len(), 5);
    }
    for selector in [2, 3, 5, 6, 7, 8] {
        assert!(c.streams[selector].is_empty());
    }
    assert_eq!(
        c.streams[0]
            .iter()
            .map(|r| r.destination)
            .collect::<Vec<_>>(),
        vec![0x2021, 0x2041, 0x2061, 0x2081, 0x20a1]
    );
    assert_eq!(
        c.streams[4]
            .iter()
            .map(|r| r.destination)
            .collect::<Vec<_>>(),
        vec![0x20a1, 0x2081, 0x2061, 0x2041, 0x2021]
    );
    let conditionals = c.streams[1]
        .iter()
        .flat_map(|r| &r.operations)
        .filter_map(|op| match op {
            StreamOp::Insert(Fragment::Conditional {
                progression,
                mask,
                present,
                absent,
            }) => Some((*progression, *mask, present.len(), absent.len())),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        conditionals,
        vec![(1, 2, 14, 14), (1, 4, 14, 14), (0, 0xff, 14, 14)]
    );
}
#[test]
fn original_all_twenty_three_payload_selector_counts_include_runtime_aliases() {
    let c = StatusCatalog::decode_original(&rom()).unwrap();
    assert_eq!(
        c.payload_counts,
        vec![
            Some(0),
            Some(12),
            Some(10),
            Some(11),
            Some(12),
            Some(12),
            Some(10),
            Some(9),
            Some(10),
            Some(10),
            None,
            None,
            Some(11),
            Some(12),
            Some(8),
            Some(8),
            Some(10),
            Some(10),
            Some(2),
            Some(8),
            Some(7),
            Some(11),
            Some(5)
        ]
    );
}
#[test]
fn original_all_five_message_modes_preserve_record_destinations_and_payload_selectors() {
    let c = StatusCatalog::decode_original(&rom()).unwrap();
    assert_eq!(
        c.messages,
        vec![
            vec![
                (0x218a, 1),
                (0x21aa, 12),
                (0x21ca, 2),
                (0x21ea, 3),
                (0x220a, 4),
                (0x222a, 5),
                (0x224a, 6),
                (0x226a, 12)
            ],
            vec![
                (0x218a, 0),
                (0x21ab, 7),
                (0x21ea, 8),
                (0x220a, 10),
                (0x224a, 9),
                (0x226b, 11)
            ],
            vec![
                (0x218a, 0),
                (0x21ea, 8),
                (0x220a, 10),
                (0x224a, 9),
                (0x226b, 11)
            ],
            vec![
                (0x218a, 13),
                (0x21aa, 12),
                (0x21ca, 14),
                (0x21ea, 12),
                (0x220a, 12),
                (0x224a, 12),
                (0x226a, 12)
            ],
            vec![
                (0x218a, 15),
                (0x21aa, 16),
                (0x21ca, 17),
                (0x21ea, 18),
                (0x220a, 19),
                (0x222a, 20),
                (0x224a, 21),
                (0x226a, 22)
            ]
        ]
    );
}
#[test]
fn stream_zero_destination_is_terminal_before_low_byte_read() {
    assert_eq!(StatusCatalog::decode_stream(&[0], &[]).unwrap(), vec![]);
}
#[test]
fn stream_literal_repeat_min_max_and_next_row_decode_without_pointer_loss() {
    let rows = StatusCatalog::decode_stream(
        &[
            0x20, 0x41, 0x7f, 0x80, 0x2a, 0x9f, 0x2b, 0xff, 0x21, 0x61, 0x11, 0xff, 0,
        ],
        &[],
    )
    .unwrap();
    assert_eq!(
        rows,
        vec![
            StreamRow {
                destination: 0x2041,
                operations: vec![
                    StreamOp::Literal(0x7f),
                    StreamOp::Repeat {
                        tile: 0x2a,
                        count: 1
                    },
                    StreamOp::Repeat {
                        tile: 0x2b,
                        count: 32
                    }
                ]
            },
            StreamRow {
                destination: 0x2161,
                operations: vec![StreamOp::Literal(0x11)]
            }
        ]
    );
}
#[test]
fn stream_shared_insertion_restores_outer_cursor_after_each_fragment() {
    let fragments = vec![
        Fragment::Literal(vec![4, 5]),
        Fragment::Literal(vec![6]),
        Fragment::Reward,
        Fragment::Total,
        Fragment::Literal(vec![7]),
    ];
    let rows = StatusCatalog::decode_stream(
        &[0x20, 0x21, 0xa0, 0x13, 0xa1, 0xa2, 0xa3, 0xa4, 0xff, 0],
        &fragments,
    )
    .unwrap();
    assert_eq!(
        rows[0].operations,
        vec![
            StreamOp::Insert(fragments[0].clone()),
            StreamOp::Literal(0x13),
            StreamOp::Insert(fragments[1].clone()),
            StreamOp::Insert(Fragment::Reward),
            StreamOp::Insert(Fragment::Total),
            StreamOp::Insert(fragments[4].clone())
        ]
    );
}
#[test]
fn stream_expansion_keeps_fixed_count_thirty_and_ff_terminator_per_yield() {
    let rows = vec![
        StreamRow {
            destination: 0x2041,
            operations: vec![
                StreamOp::Repeat {
                    tile: 0x2a,
                    count: 2,
                },
                StreamOp::Literal(0x11),
            ],
        },
        StreamRow {
            destination: 0x2061,
            operations: vec![StreamOp::Literal(0x12)],
        },
    ];
    let out = StatusCatalog::expand(&rows, &state(), &[0, 0]).unwrap();
    assert_eq!(
        out,
        vec![
            RowUpload {
                bytes: vec![0x20, 0x41, 0x1d, 0x2a, 0x2a, 0x11, 0xff],
                yield_after: true
            },
            RowUpload {
                bytes: vec![0x20, 0x61, 0x1d, 0x12, 0xff],
                yield_after: true
            }
        ]
    );
}
#[test]
fn stream_runtime_value_fragments_use_counts_plus_one_and_marker_byte() {
    let mut s = state();
    s.reward = [1, 2, 3, 4, 5, 6];
    s.total = [1, 2, 3, 4, 5, 6, 7];
    let rows = vec![StreamRow {
        destination: 0x2041,
        operations: vec![
            StreamOp::Insert(Fragment::Reward),
            StreamOp::Insert(Fragment::Total),
        ],
    }];
    let out = StatusCatalog::expand(&rows, &s, &[0, 0]).unwrap();
    assert_eq!(
        out[0].bytes,
        vec![
            0x20, 0x41, 0x1d, 0x2c, 1, 2, 3, 4, 5, 6, 0, 0x2c, 1, 2, 3, 4, 5, 6, 7, 0, 0xff
        ]
    );
}
#[test]
fn conditional_fragments_select_by_mask_and_preserve_following_literal() {
    let rows = vec![StreamRow {
        destination: 0x2041,
        operations: vec![
            StreamOp::Insert(Fragment::Conditional {
                progression: 1,
                mask: 2,
                present: vec![8, 9],
                absent: vec![BLANK, BLANK],
            }),
            StreamOp::Literal(4),
        ],
    }];
    for (flags, expected) in [
        (0, vec![BLANK, BLANK]),
        (1, vec![BLANK, BLANK]),
        (2, vec![8, 9]),
        (3, vec![8, 9]),
    ] {
        let out = StatusCatalog::expand(&rows, &state(), &[0, flags]).unwrap();
        assert_eq!(&out[0].bytes[3..5], &expected);
        assert_eq!(out[0].bytes[5], 4);
    }
}
#[test]
fn every_original_stream_expands_each_row_to_thirty_tiles_then_ff() {
    let c = StatusCatalog::decode_original(&rom()).unwrap();
    for selector in [0, 1, 4, 9] {
        for flags in [[0, 0], [0xff, 0xff]] {
            let out = StatusCatalog::expand(&c.streams[selector], &state(), &flags).unwrap();
            assert_eq!(out.len(), 5);
            for row in out {
                assert_eq!(row.bytes.len(), 34);
                assert_eq!(row.bytes[2], 0x1d);
                assert_eq!(row.bytes[33], 0xff);
                assert!(row.yield_after);
            }
        }
    }
}
#[test]
fn update_streams_always_clears_five_rows_before_selected_five_rows() {
    let c = StatusCatalog::decode_original(&rom()).unwrap();
    for selector in [0, 1] {
        let out = c
            .update_streams(selector, ORDINARY_PLAYER, &state(), &[0, 0])
            .unwrap();
        assert_eq!(out.len(), 10);
        assert_eq!(&out[0].bytes[..2], &[0x20, 0xa1]);
        assert_eq!(&out[5].bytes[..2], &[0x20, 0x21]);
    }
}
#[test]
fn dialogue_player_forces_stream_nine_regardless_of_status_selector() {
    let c = StatusCatalog::decode_original(&rom()).unwrap();
    let expected = c
        .update_streams(9, ORDINARY_PLAYER, &state(), &[0, 0])
        .unwrap();
    for selector in [0, 1] {
        assert_eq!(
            c.update_streams(selector, 0x0f, &state(), &[0, 0]).unwrap(),
            expected
        );
    }
}
#[test]
fn message_mode_zero_preserves_actors_and_all_other_modes_clear_only_four_slots() {
    let c = StatusCatalog::decode_original(&rom()).unwrap();
    for (actor_latch, scene, special, mode) in [
        (0, 0, 0, 0),
        (0, 1, 0, 1),
        (0, 1, 0x80, 2),
        (3, 0, 0, 3),
        (4, 0, 0, 4),
    ] {
        let mut actors = [ActorContact {
            state: 0xa0,
            ..Default::default()
        }; crate::actors::ACTOR_SLOTS];
        let out = c
            .build_message(actor_latch, scene, special, 0, &mut actors, &state())
            .unwrap();
        assert_eq!(&out[..2], &[0x21, 0x8a]);
        assert_eq!(out.last(), Some(&0xff));
        for (slot, a) in actors.iter().enumerate() {
            assert_eq!(
                a.state,
                if mode != 0 && [0, 2, 3, 4].contains(&slot) {
                    0
                } else {
                    0xa0
                }
            );
        }
    }
}
#[test]
fn message_zero_selector_resolves_stage_plus_two_and_runtime_value_payloads() {
    let c = StatusCatalog::decode_original(&rom()).unwrap();
    for stage in 0..5 {
        let mut actors = [ActorContact::default(); crate::actors::ACTOR_SLOTS];
        let mut s = state();
        s.total = [1, 2, 3, 4, 5, 6, 7];
        s.reward = [7, 6, 5, 4, 3, 2];
        let out = c.build_message(0, 1, 0, stage, &mut actors, &s).unwrap();
        assert_eq!(out[2], c.payload_counts[usize::from(stage) + 2].unwrap());
        assert!(out.windows(9).any(|v| v == [0x2c, 1, 2, 3, 4, 5, 6, 7, 0]));
        assert!(out.windows(8).any(|v| v == [0x2c, 7, 6, 5, 4, 3, 2, 0]));
    }
}

#[test]
fn reward_addition_stops_at_settled_carry_without_normalizing_higher_codes() {
    let mut s = state();
    s.reward = [0x35, BLANK, 2, 3, 0, 0];
    s.add_reward(3, 1);
    assert_eq!(s.reward, [0x35, BLANK, 2, 4, 0, 0]);
    s.reward[3] = BLANK.wrapping_sub(1);
    s.add_reward(3, 1);
    assert_eq!(s.reward, [0x35, BLANK, 2, BLANK, 0, 0]);
}

#[test]
fn dialogue_payment_stops_zero_normalization_at_an_existing_blank_code() {
    let mut s = state();
    s.reward = [0, 3, BLANK, 0, 0, 7];
    assert!(s.pay(Cost::RewardDialogue, false));
    assert_eq!(s.reward, [BLANK, BLANK, BLANK, 0, 0, 7]);
}

#[test]
fn overlay_payment_stops_at_existing_blank_and_preserves_following_zeroes() {
    let mut s = state();
    s.total = [0, 3, BLANK, 0, 0, 0, 7];
    s.health.segments[2].depleted = 1;
    assert!(s.pay(Cost::TotalOverlay, false));
    assert_eq!(s.total, [BLANK, BLANK, BLANK, 0, 0, 0, 7]);
}

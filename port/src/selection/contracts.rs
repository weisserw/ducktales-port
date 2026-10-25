//! Independent source expectations; derivations and caller constraints: trace.md.
use super::*;
use crate::{
    actors::ActorContact,
    config,
    runtime::{Buttons, TITLE_RANDOM_SEED},
};
use romatlas_port::assets::Rom;
const OPTIONS: [TitleOption; 3] = [
    TitleOption::Easy,
    TitleOption::Normal,
    TitleOption::Difficult,
];
const LANDS: [Land; 5] = [
    Land::Amazon,
    Land::Transylvania,
    Land::AfricanMines,
    Land::Himalayas,
    Land::Moon,
];
fn placement(animation: u8, y: u8, x: u8) -> ActorPlacement {
    ActorPlacement { animation, y, x }
}
/// Only semantic parameters are authored here; no extracted graphics/text assets.
fn catalog() -> Catalog {
    Catalog {
        title_actors: [placement(0x86, 0x32, 0x54), placement(0x43, 0x88, 0x50)],
        title_marker_x: [0x20, 0x50, 0x90],
        title_steps: [1, -1],
        title_wrap: [-3, 3],
        land_actors: [
            placement(0x43, 0x70, 0x46),
            placement(0x78, 0xbc, 0x80),
            placement(0x6f, 0x96, 0xd0),
            placement(0x7d, 0x28, 0x80),
            placement(0x6f, 0x96, 0x30),
            placement(0x52, 0xd8, 0x98),
            placement(0x51, 0xd8, 0xa8),
            placement(0x52, 0xc8, 0xd0),
        ],
        cursor_y: [0x70, 0x78, 0x80, 0x88, 0x90],
        name_y: [0x28, 0x20, 0x30, 0x28, 0x30],
        name_x: [0x80, 0x60, 0x88, 0x68, 0xa0],
        completion_masks: [1, 2, 4, 8, 16, 32, 64, 128],
        land_steps: [-1, 1],
        land_wrap: [5, -5],
    }
}
fn input(pressed: Buttons, held: Buttons) -> PortInput {
    PortInput { pressed, held }
}
fn pool() -> ActorPool {
    [ActorContact::default(); crate::actors::ACTOR_SLOTS]
}
fn rom() -> Vec<u8> {
    let bytes =
        std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../input/game.rom"))
            .expect("local matching ROM");
    Rom::from_bytes(bytes.clone(), config::ROM_SHA256).expect("validated matching source fixture");
    bytes
}
#[test]
fn all_original_parameter_records_decode_with_signed_interleaved_steps() {
    let bytes = rom();
    let actual = Catalog::decode(
        &bytes[TITLE_PARAMETER_OFFSET..TITLE_PARAMETER_OFFSET + TITLE_PARAMETER_BYTES],
        &bytes[LAND_PARAMETER_OFFSET..LAND_PARAMETER_OFFSET + LAND_PARAMETER_BYTES],
    )
    .unwrap();
    assert_eq!(actual, catalog());
}
#[test]
fn parameter_decoder_rejects_each_truncated_recipe_without_partial_catalog() {
    let bytes = rom();
    let t = &bytes[TITLE_PARAMETER_OFFSET..TITLE_PARAMETER_OFFSET + TITLE_PARAMETER_BYTES];
    let l = &bytes[LAND_PARAMETER_OFFSET..LAND_PARAMETER_OFFSET + LAND_PARAMETER_BYTES];
    for end in 0..t.len() {
        assert!(
            Catalog::decode(&t[..end], l).is_err(),
            "title truncation {end}"
        );
    }
    for end in 0..l.len() {
        assert!(
            Catalog::decode(t, &l[..end]).is_err(),
            "land truncation {end}"
        );
    }
}
#[test]
fn source_script_labels_and_b14_rule_join_cover_all_three_choices() {
    for (option, selector, label, rule) in [
        (TitleOption::Easy, 0, "EASY", TitleRule::AlternatingDamage),
        (TitleOption::Normal, 1, "NORMAL", TitleRule::OrdinaryDamage),
        (
            TitleOption::Difficult,
            2,
            "DIFFICULT",
            TitleRule::AlternateRewards,
        ),
    ] {
        assert_eq!(option.source_selector(), selector);
        assert_eq!(option.display_label(), label);
        assert_eq!(option.status_rule(), rule);
    }
}
#[test]
fn fresh_title_entry_yields_before_seed_or_any_menu_input() {
    let mut a = pool();
    let before = a;
    let mut r = RandomState {
        words: [0x12, 0x34, 0x56, 0x78],
    };
    let mut o = TitleOption::Difficult;
    let b = title_resume(
        TitlePhase::Entry,
        &mut TitleContext {
            actors: &mut a,
            random: &mut r,
            option: &mut o,
        },
        &catalog(),
    );
    assert_eq!(
        b,
        TitleBatch {
            operations: vec![Operation::InstallControlShadow, Operation::Yield],
            next: TitlePhase::AfterEntryYield
        }
    );
    assert_eq!(r.words, [0x12, 0x34, 0x56, 0x78]);
    assert_eq!(a, before);
    assert_eq!(o, TitleOption::Difficult);
}
#[test]
fn first_title_resume_sets_only_first_random_word_before_overlay_audio_and_fade() {
    let mut a = pool();
    let mut r = RandomState {
        words: [0x12, 0x34, 0x56, 0x78],
    };
    let mut o = TitleOption::Easy;
    let b = title_resume(
        TitlePhase::AfterEntryYield,
        &mut TitleContext {
            actors: &mut a,
            random: &mut r,
            option: &mut o,
        },
        &catalog(),
    );
    assert_eq!(r.words, [TITLE_RANDOM_SEED, 0x34, 0x56, 0x78]);
    assert_eq!(o, TitleOption::Easy);
    assert_eq!(
        b,
        TitleBatch {
            operations: vec![
                Operation::CopyBaseOverlays,
                Operation::StageAudio(TITLE_AUDIO),
                Operation::FadeOut
            ],
            next: TitlePhase::AfterFadeOut
        }
    );
}
fn title_batch(phase: TitlePhase) -> (TitleBatch, ActorPool, TitleOption) {
    let mut a = pool();
    let mut r = RandomState::default();
    let mut o = TitleOption::Difficult;
    let b = title_resume(
        phase,
        &mut TitleContext {
            actors: &mut a,
            random: &mut r,
            option: &mut o,
        },
        &catalog(),
    );
    (b, a, o)
}
#[test]
fn completed_title_fade_clears_display_and_protection_then_yields_before_disable() {
    assert_eq!(
        title_batch(TitlePhase::AfterFadeOut).0,
        TitleBatch {
            operations: vec![Operation::ClearGameplayAndProtection, Operation::Yield],
            next: TitlePhase::AfterDisplayYield
        }
    );
}
#[test]
fn title_suspended_setup_orders_clear_script_chr_palette_actors_and_enable() {
    let (b, mut a, o) = title_batch(TitlePhase::AfterDisplayYield);
    assert_eq!(
        b,
        TitleBatch {
            operations: vec![
                Operation::DisableRendering,
                Operation::ResetTitleScroll,
                Operation::FillFirstNametable,
                Operation::HideSprites,
                Operation::ClearNonplayerActors,
                Operation::ScreenScript(TITLE_SCRIPT),
                Operation::Chr(TITLE_CHR),
                Operation::TitlePalette,
                Operation::InitializeTitleActors,
                Operation::TitlePaletteTail,
                Operation::EnableRendering,
                Operation::Yield
            ],
            next: TitlePhase::AfterEnableYield
        }
    );
    assert_eq!(o, TitleOption::Difficult);
    assert_eq!(
        a,
        pool(),
        "batch construction must not initialize actors before ordered effects"
    );
    initialize_title_actors(&mut a, &catalog());
    for (slot, p) in catalog().title_actors.iter().enumerate() {
        assert_eq!(
            (
                a[slot].state,
                a[slot].render,
                a[slot].animation,
                a[slot].position.screen_y,
                a[slot].position.world_x,
                a[slot].animation_tick,
                a[slot].animation_frame
            ),
            (0x80, 0x80, p.animation, p.y, u16::from(p.x), 0, 0)
        );
    }
}
#[test]
fn title_fade_in_is_requested_only_after_enable_yield() {
    assert_eq!(
        title_batch(TitlePhase::AfterEnableYield).0,
        TitleBatch {
            operations: vec![Operation::FadeIn],
            next: TitlePhase::AfterFadeIn
        }
    );
}
#[test]
fn title_overlay_copy_has_its_own_yield_before_default_option_is_written() {
    let (b, _, o) = title_batch(TitlePhase::AfterFadeIn);
    assert_eq!(o, TitleOption::Difficult);
    assert_eq!(
        b,
        TitleBatch {
            operations: vec![Operation::CopyTitleOverlay, Operation::Yield],
            next: TitlePhase::AfterOverlayYield
        }
    );
    let (b, _, o) = title_batch(TitlePhase::AfterOverlayYield);
    assert_eq!(o, TitleOption::Normal);
    assert_eq!(
        b,
        TitleBatch {
            operations: vec![],
            next: TitlePhase::Input
        }
    );
}
#[test]
fn all_title_edge_combinations_obey_start_priority_select_right_increment_left_decrement() {
    // The exhaustive table is a finite input fixture: 0/1/2 choices, 256 edge bytes.
    // ADC/compare wrap maps are written explicitly, independent of implementation.
    let increment = [1, 2, 0];
    let decrement = [2, 0, 1];
    for (index, original) in OPTIONS.into_iter().enumerate() {
        for bits in 0..=u8::MAX {
            let mut option = original;
            let mut a = pool();
            a[1].position.world_x = 0x3456;
            let result = title_input(
                &mut option,
                &mut a,
                input(Buttons(bits), Buttons::NONE),
                &catalog(),
            );
            if bits & Buttons::START.0 != 0 {
                assert_eq!(
                    result,
                    InputResult {
                        decision: Decision::LandSelection,
                        operations: vec![]
                    }
                );
                assert_eq!(option, original);
                assert_eq!(a[1].position.world_x, 0x3456);
            } else if bits & 0x23 != 0 {
                let target = if bits & Buttons::LEFT.0 != 0 {
                    decrement[index]
                } else {
                    increment[index]
                };
                assert_eq!(option, OPTIONS[target]);
                assert_eq!(
                    a[1].position.world_x,
                    0x3400 | u16::from(catalog().title_marker_x[target])
                );
                assert_eq!(
                    result,
                    InputResult {
                        decision: Decision::Frame,
                        operations: vec![Operation::Sound(MENU_SOUND)]
                    }
                );
            } else {
                assert_eq!(option, original);
                assert_eq!(a[1].position.world_x, 0x3456);
                assert_eq!(
                    result,
                    InputResult {
                        decision: Decision::Frame,
                        operations: vec![]
                    }
                );
            }
        }
    }
}
#[test]
fn held_title_buttons_do_not_repeat_or_confirm_without_edges() {
    for held in [
        Buttons::START,
        Buttons::LEFT,
        Buttons::RIGHT,
        Buttons::SELECT,
        Buttons::A,
        Buttons(u8::MAX),
    ] {
        let mut o = TitleOption::Normal;
        let mut a = pool();
        let out = title_input(&mut o, &mut a, input(Buttons::NONE, held), &catalog());
        assert_eq!(o, TitleOption::Normal);
        assert_eq!(
            out,
            InputResult {
                decision: Decision::Frame,
                operations: vec![]
            }
        );
    }
}
#[test]
fn title_input_uses_edge_byte_independently_of_conflicting_held_byte() {
    let mut o = TitleOption::Normal;
    let mut a = pool();
    let out = title_input(
        &mut o,
        &mut a,
        input(Buttons::LEFT, Buttons::RIGHT),
        &catalog(),
    );
    assert_eq!(o, TitleOption::Easy);
    assert_eq!(out.operations, vec![Operation::Sound(MENU_SOUND)]);
}
fn animated(
    frame: u8,
    tick: u8,
    clock: u8,
    words: [u8; 4],
) -> (ActorPool, RandomState, Composition, Vec<Operation>) {
    let mut a = pool();
    a[0].animation_frame = frame;
    a[0].animation_tick = tick;
    let mut r = RandomState { words };
    let mut c = Composition {
        clock,
        ..Composition::default()
    };
    let effects = title_animation(&mut a, &mut r, &mut c);
    (a, r, c, effects)
}
#[test]
fn title_animation_all_nonzero_frames_bypass_tick_and_random_gates() {
    for frame in 1..=u8::MAX {
        let (a, r, c, e) = animated(frame, 1, TITLE_RANDOM_PERIOD, [0x88, 0x11, 0x22, 0x33]);
        assert_eq!(a[0].animation_frame, frame);
        assert_eq!(a[0].animation_tick, 1);
        assert_eq!(r.words, [0x88, 0x11, 0x22, 0x33]);
        assert_eq!(c.clock, TITLE_RANDOM_PERIOD);
        assert_eq!(
            e,
            vec![Operation::HideSprites, Operation::Compose, Operation::Yield]
        );
    }
}
#[test]
fn title_animation_requires_exact_tick_one_not_nonzero_or_masked_tick() {
    for tick in 0..=u8::MAX {
        if tick == 1 {
            continue;
        }
        let (a, r, c, _) = animated(0, tick, TITLE_RANDOM_PERIOD, [0x88, 0x11, 0x22, 0x33]);
        assert_eq!(a[0].animation_tick, tick);
        assert_eq!(a[0].animation_frame, 0);
        assert_eq!(r.words, [0x88, 0x11, 0x22, 0x33]);
        assert_eq!(c.clock, TITLE_RANDOM_PERIOD);
    }
}
#[test]
fn title_tick_resets_at_frame_zero_tick_one_even_when_clock_misses_random_period() {
    for clock in 0..=u8::MAX {
        if clock == TITLE_RANDOM_PERIOD {
            continue;
        }
        let (a, r, c, _) = animated(0, 1, clock, [0x88, 0x11, 0x22, 0x33]);
        assert_eq!(a[0].animation_tick, 0);
        assert_eq!(a[0].animation_frame, 0);
        assert_eq!(r.words, [0x88, 0x11, 0x22, 0x33]);
        assert_eq!(c.clock, clock);
    }
}
#[test]
fn title_random_adds_compare_equality_carry_wraps_and_selects_low_bit_before_composition() {
    for (words, result, frame) in [
        ([0, 0, 0x22, 0x33], 1, 1),
        ([0x88, 0x11, 0x22, 0x33], 0x9a, 0),
        ([0xff, 0, 0x22, 0x33], 0, 0),
        ([0xff, 0xff, 0x22, 0x33], 0xff, 1),
    ] {
        let (a, r, c, e) = animated(0, 1, TITLE_RANDOM_PERIOD, words);
        assert_eq!(a[0].animation_tick, 0);
        assert_eq!(a[0].animation_frame, frame);
        assert_eq!(r.words, [result, words[1], words[2], words[3]]);
        assert_eq!(c.clock, 0);
        assert_eq!(
            e,
            vec![Operation::HideSprites, Operation::Compose, Operation::Yield]
        );
    }
}
#[test]
fn land_entry_fades_out_before_touching_display_or_actors() {
    assert_eq!(
        land_resume(LandPhase::Entry),
        LandBatch {
            operations: vec![Operation::FadeOut],
            next: LandPhase::AfterFadeOut
        }
    );
}
#[test]
fn land_setup_saves_stage_before_layout_five_and_restores_before_initializing_actors() {
    assert_eq!(
        land_resume(LandPhase::AfterFadeOut),
        LandBatch {
            operations: vec![
                Operation::ClearGameplay,
                Operation::DisableRendering,
                Operation::HideSprites,
                Operation::ClearNonplayerActors,
                Operation::SaveStageSelector,
                Operation::UseLandLayoutSelector,
                Operation::Chr(LAND_CHR),
                Operation::LoadLandPresentation,
                Operation::BuildWholeTerrainRing,
                Operation::RestoreStageSelector,
                Operation::InitializeLandActors,
                Operation::EnableRendering,
                Operation::StatusMessage,
                Operation::Yield
            ],
            next: LandPhase::AfterStatusYield
        }
    );
}
#[test]
fn land_status_yield_precedes_fade_in_and_overlay_gate_clear_follows_fade() {
    assert_eq!(
        land_resume(LandPhase::AfterStatusYield),
        LandBatch {
            operations: vec![Operation::FadeIn],
            next: LandPhase::AfterFadeIn
        }
    );
    assert_eq!(
        land_resume(LandPhase::AfterFadeIn),
        LandBatch {
            operations: vec![Operation::ClearOverlayGate],
            next: LandPhase::Completion
        }
    );
}
#[test]
fn land_actor_initialization_copies_all_eight_records_and_preserves_unwritten_fields() {
    let mut a = pool();
    for actor in &mut a {
        actor.handler = 0x37;
        actor.local_state = 0x48;
        actor.position.x_fraction = 0x59;
        actor.position.y_fraction = 0x6a;
        actor.animation_restart = 0x7b;
    }
    let before = a;
    initialize_land_actors(&mut a, &catalog());
    for (slot, p) in catalog().land_actors.iter().enumerate() {
        assert_eq!(
            (
                a[slot].state,
                a[slot].render,
                a[slot].animation,
                a[slot].position.screen_y,
                a[slot].position.world_x,
                a[slot].animation_frame,
                a[slot].animation_tick
            ),
            (0x80, 0x90, p.animation, p.y, u16::from(p.x), 0, 0)
        );
        assert_eq!(
            (
                a[slot].handler,
                a[slot].local_state,
                a[slot].position.x_fraction,
                a[slot].position.y_fraction,
                a[slot].animation_restart
            ),
            (0x37, 0x48, 0x59, 0x6a, 0x7b)
        );
    }
    assert_eq!(&a[8..], &before[8..]);
}
#[test]
fn no_completion_signals_enter_available_land_even_with_all_five_progress_bits() {
    for progress_bits in 0..=u8::MAX {
        assert_eq!(
            completion_plan(CompletionInput {
                completion_actor: 0,
                scene_signal: 0,
                progress_bits
            }),
            CompletionPlan {
                operations: vec![],
                route: CompletionRoute::AvailableLand
            }
        );
    }
}
#[test]
fn any_nonzero_completion_actor_bypasses_transfer_progress_test_and_first_message() {
    for latch in 1..=u8::MAX {
        let out = completion_plan(CompletionInput {
            completion_actor: latch,
            scene_signal: 0,
            progress_bits: 0,
        });
        assert_eq!(
            out,
            CompletionPlan {
                operations: vec![
                    Operation::Wait(COMPLETION_PAUSE),
                    Operation::CompletionActor,
                    Operation::Message(4),
                    Operation::Acknowledge
                ],
                route: CompletionRoute::LateStage
            }
        );
    }
}
#[test]
fn scene_completion_incomplete_masks_transfer_then_acknowledge_restore_message_and_yield() {
    for progress_bits in 0..=u8::MAX {
        if progress_bits & ORIGINAL_LAND_BITS == ORIGINAL_LAND_BITS {
            continue;
        }
        let out = completion_plan(CompletionInput {
            completion_actor: 0,
            scene_signal: 1,
            progress_bits,
        });
        assert_eq!(
            out,
            CompletionPlan {
                operations: vec![
                    Operation::Compose,
                    Operation::Wait(COMPLETION_PAUSE),
                    Operation::TransferDisplayedValue,
                    Operation::Acknowledge,
                    Operation::Message(0),
                    Operation::Yield
                ],
                route: CompletionRoute::OrdinaryAcknowledgement
            }
        );
    }
}
#[test]
fn all_five_completed_ignores_upper_bits_and_has_two_distinct_sixty_tick_waits() {
    for upper in [0, 0x20, 0x40, 0x60, 0x80, 0xa0, 0xc0, 0xe0] {
        let out = completion_plan(CompletionInput {
            completion_actor: 0,
            scene_signal: 0xff,
            progress_bits: upper | ORIGINAL_LAND_BITS,
        });
        assert_eq!(
            out,
            CompletionPlan {
                operations: vec![
                    Operation::Compose,
                    Operation::Wait(COMPLETION_PAUSE),
                    Operation::TransferDisplayedValue,
                    Operation::Message(3),
                    Operation::Wait(COMPLETION_PAUSE),
                    Operation::CompletionActor,
                    Operation::Message(4),
                    Operation::Acknowledge
                ],
                route: CompletionRoute::LateStage
            }
        );
    }
}
#[test]
fn first_available_scans_all_eight_source_masks_including_alternate_selector_five() {
    // FF would read beyond the eight-mask table. Its caller precondition is
    // deliberately not silently changed into a five-choice early return.
    for (bits, expected) in [
        (0, 0),
        (1, 1),
        (3, 2),
        (7, 3),
        (15, 4),
        (31, 5),
        (63, 6),
        (127, 7),
        (0xfe, 0),
    ] {
        assert_eq!(
            first_available_selector(bits, &catalog()),
            Some(StageId(expected))
        );
    }
}
#[test]
fn every_original_incomplete_mask_selects_lowest_unfinished_land() {
    for bits in 0..ORIGINAL_LAND_BITS {
        let expected = match bits {
            0 | 2 | 4 | 6 | 8 | 10 | 12 | 14 | 16 | 18 | 20 | 22 | 24 | 26 | 28 | 30 => 0,
            1 | 5 | 9 | 13 | 17 | 21 | 25 | 29 => 1,
            3 | 11 | 19 | 27 => 2,
            7 | 23 => 3,
            15 => 4,
            _ => unreachable!(),
        };
        assert_eq!(
            first_available_selector(bits, &catalog()),
            Some(StageId(expected))
        );
    }
}
#[test]
fn positioning_all_five_lands_preserves_animation_clocks_but_sets_exact_four_active_slots() {
    for (index, land) in LANDS.into_iter().enumerate() {
        let mut a = pool();
        for actor in &mut a {
            actor.animation_tick = 0x23;
            actor.animation_frame = 0x34;
            actor.render = 0x56;
            actor.position.world_x = 0x8900;
        }
        position_land(land, &mut a, &catalog());
        assert_eq!(a[0].position.screen_y, catalog().cursor_y[index]);
        for slot in [2, 4] {
            assert_eq!(a[slot].animation, LAND_NAME_ANIMATION_BASE + index as u8);
            assert_eq!(a[slot].animation_tick, 0x23);
            assert_eq!(a[slot].animation_frame, 0x34);
        }
        assert_eq!(a[3].position.screen_y, catalog().name_y[index]);
        assert_eq!(
            a[3].position.world_x,
            0x8900 | u16::from(catalog().name_x[index])
        );
        for slot in [0, 2, 3, 4] {
            assert_eq!(a[slot].state, 0x80);
        }
        assert_eq!(a[3].render, 0x90);
        for slot in [0, 2, 4] {
            assert_eq!(a[slot].render, 0x56);
        }
    }
}
#[test]
fn land_up_and_down_wrap_all_five_source_entries_and_reset_name_clocks() {
    let up = [4, 0, 1, 2, 3];
    let down = [1, 2, 3, 4, 0];
    for (index, land) in LANDS.into_iter().enumerate() {
        for (edge, target) in [(Buttons::UP, up[index]), (Buttons::DOWN, down[index])] {
            let mut selected = land;
            let mut a = pool();
            a[2].animation_tick = 3;
            a[2].animation_frame = 4;
            a[4].animation_tick = 5;
            a[4].animation_frame = 6;
            let out = land_input(
                &mut selected,
                &mut a,
                0,
                input(edge, Buttons::NONE),
                &catalog(),
            );
            assert_eq!(selected, LANDS[target]);
            assert_eq!(
                out,
                InputResult {
                    decision: Decision::Frame,
                    operations: vec![
                        Operation::Sound(MENU_SOUND),
                        Operation::HideSprites,
                        Operation::Compose,
                        Operation::Yield
                    ]
                }
            );
            for slot in [2, 4] {
                assert_eq!(a[slot].animation_frame, 0);
                assert_eq!(a[slot].animation_tick, 0);
                assert_eq!(a[slot].animation, LAND_NAME_ANIMATION_BASE + target as u8);
            }
            assert_eq!(a[0].position.screen_y, catalog().cursor_y[target]);
        }
    }
}
#[test]
fn all_progress_masks_skip_completed_in_each_direction_without_yielding_between_candidates() {
    // Explicit cyclic orders are fixtures; no production stepping algorithm is
    // reimplemented. Every0..30 mask and every initial land are exercised.
    let up = [
        [4, 3, 2, 1, 0],
        [0, 4, 3, 2, 1],
        [1, 0, 4, 3, 2],
        [2, 1, 0, 4, 3],
        [3, 2, 1, 0, 4],
    ];
    let down = [
        [1, 2, 3, 4, 0],
        [2, 3, 4, 0, 1],
        [3, 4, 0, 1, 2],
        [4, 0, 1, 2, 3],
        [0, 1, 2, 3, 4],
    ];
    for bits in 0..ORIGINAL_LAND_BITS {
        for index in 0..LANDS.len() {
            for (edge, order) in [(Buttons::UP, up[index]), (Buttons::DOWN, down[index])] {
                let expected = *order
                    .iter()
                    .find(|&&candidate| bits & catalog().completion_masks[candidate] == 0)
                    .unwrap();
                let mut land = LANDS[index];
                let mut a = pool();
                let out = land_input(
                    &mut land,
                    &mut a,
                    bits,
                    input(edge, Buttons::NONE),
                    &catalog(),
                );
                assert_eq!(land, LANDS[expected], "bits={bits:02x} start={index}");
                assert_eq!(
                    out.operations,
                    vec![
                        Operation::Sound(MENU_SOUND),
                        Operation::HideSprites,
                        Operation::Compose,
                        Operation::Yield
                    ]
                );
            }
        }
    }
}
#[test]
fn land_direction_edges_take_priority_over_a_even_if_skip_returns_to_same_land() {
    let mut land = Land::Amazon;
    let mut a = pool();
    let out = land_input(
        &mut land,
        &mut a,
        0x1e,
        input(Buttons::DOWN | Buttons::A, Buttons::A),
        &catalog(),
    );
    assert_eq!(land, Land::Amazon);
    assert_eq!(out.decision, Decision::Frame);
    assert_eq!(
        out.operations,
        vec![
            Operation::Sound(MENU_SOUND),
            Operation::HideSprites,
            Operation::Compose,
            Operation::Yield
        ]
    );
}
#[test]
fn land_both_direction_edges_choose_down_when_given_source_game_written_edges() {
    // B01 normally filters conflicts. The consumer does not resample/filter a
    // game-written edge field: LSR/AND02 selects the DOWN-indexed entry here.
    let mut land = Land::Amazon;
    let mut a = pool();
    land_input(
        &mut land,
        &mut a,
        0,
        input(Buttons::UP | Buttons::DOWN, Buttons::NONE),
        &catalog(),
    );
    assert_eq!(land, Land::Transylvania);
}
#[test]
fn land_a_edge_confirms_but_b_start_select_and_horizontal_have_no_back_route() {
    for edge in [
        Buttons::B,
        Buttons::START,
        Buttons::SELECT,
        Buttons::LEFT,
        Buttons::RIGHT,
        Buttons::NONE,
    ] {
        let mut land = Land::Moon;
        let mut a = pool();
        let out = land_input(&mut land, &mut a, 0, input(edge, Buttons::A), &catalog());
        assert_eq!(land, Land::Moon);
        assert_eq!(
            out,
            InputResult {
                decision: Decision::Frame,
                operations: vec![Operation::HideSprites, Operation::Compose, Operation::Yield]
            }
        );
    }
    let mut land = Land::Moon;
    let mut a = pool();
    let out = land_input(
        &mut land,
        &mut a,
        0,
        input(Buttons::A, Buttons::NONE),
        &catalog(),
    );
    assert_eq!(
        out,
        InputResult {
            decision: Decision::ConfirmStage,
            operations: vec![]
        }
    );
}
#[test]
fn acknowledge_reads_only_a_edge_and_retains_producer_value_for_shared_submission_tail() {
    for bits in 0..=u8::MAX {
        for cursor in [0, 1, 0x7f, 0xff] {
            let snapshot = input(Buttons(bits), Buttons::A);
            let expected = if bits & Buttons::A.0 != 0 {
                Acknowledgement::Accepted {
                    primary_upload_pending: cursor,
                }
            } else {
                Acknowledgement::WaitOneFrame
            };
            assert_eq!(acknowledge(snapshot, cursor), expected);
            assert_eq!(snapshot, input(Buttons(bits), Buttons::A));
        }
    }
}
#[test]
fn confirmation_has_two_terrain_column_builds_separated_by_ui_and_chr() {
    assert_eq!(
        confirmation_plan(),
        vec![
            Operation::FadeOut,
            Operation::ClearGameplay,
            Operation::Yield,
            Operation::DisableRendering,
            Operation::HideSprites,
            Operation::ClearActorOne,
            Operation::FillFirstNametable,
            Operation::LoadStagePresentation,
            Operation::ClearNonplayerActors,
            Operation::BuildInitialTerrainColumns,
            Operation::ScreenScript(STAGE_UI_SCRIPT),
            Operation::StageChr,
            Operation::BuildInitialTerrainColumns,
            Operation::EnterStageRuntime
        ]
    );
}
fn named_catalog() -> DecodedAssets {
    use crate::assets::catalog::LevelDefinition;
    let mut levels = Vec::new();
    // Added entries deliberately precede/interleave originals; selector lookup
    // cannot be substituted with a fixed-length Vec index.
    for selector in 0..12 {
        levels.push(LevelDefinition {
            id: AssetId(format!("added-{selector}")),
            original_selector: None,
            rooms: AssetId("rooms".into()),
            terrain: AssetId("terrain".into()),
            spawns: None,
            graphics: vec![],
            audio: vec![],
        });
        if selector < 5 {
            levels.push(LevelDefinition {
                id: AssetId(format!("source-{selector}")),
                original_selector: Some(StageId(selector)),
                rooms: AssetId("rooms".into()),
                terrain: AssetId("terrain".into()),
                spawns: None,
                graphics: vec![],
                audio: vec![],
            });
        }
    }
    DecodedAssets { levels }
}
#[test]
fn each_original_land_resolves_named_catalog_without_capping_or_exposing_extensions_in_menu() {
    let c = named_catalog();
    for (selector, land) in LANDS.into_iter().enumerate() {
        assert_eq!(land.source_selector(), StageId(selector as u8));
        assert_eq!(
            land.level(&c).unwrap(),
            AssetId(format!("source-{selector}"))
        );
    }
    assert_eq!(c.levels.len(), 17);
}
#[test]
fn missing_original_named_level_is_an_error_not_an_arbitrary_extension_fallback() {
    let mut c = named_catalog();
    c.levels.retain(|l| l.original_selector != Some(StageId(3)));
    assert!(Land::Himalayas.level(&c).is_err());
}
#[test]
fn late_completion_selects_stage_one_and_increments_late_route_with_byte_wrap() {
    let c = named_catalog();
    for (before, after) in [(0, 1), (1, 2), (0xff, 0)] {
        let mut route = before;
        let request = late_stage_request(&mut route, &c).unwrap();
        assert_eq!(route, after);
        assert_eq!(
            request,
            StageRequest {
                level: AssetId("source-1".into()),
                selector: StageId(1),
                late_route: after
            }
        );
    }
}

#[test]
fn title_palette_overrides_write_both_shadows_in_source_order_around_actor_setup() {
    assert_eq!(
        title_palette_edits(),
        TitlePaletteEdits {
            before_actors: vec![(16, 0x11)],
            after_actors: vec![(26, 0x2c), (30, 0x20), (31, 0x2c)]
        }
    );
}
#[test]
fn title_base_overlays_copy_thirteen_and_twenty_one_bytes_without_stopping_at_ff() {
    let mut stage: Vec<u8> = (0..20).collect();
    stage[1] = 0xff;
    let mut main: Vec<u8> = (40..70).collect();
    main[2] = 0xff;
    let (ui, overlay) = prepare_base_overlays(&stage, &main).unwrap();
    assert_eq!(ui, stage[..13]);
    assert_eq!(overlay, main[..21]);
    for len in 0..13 {
        assert!(prepare_base_overlays(&stage[..len], &main).is_err());
    }
    for len in 0..21 {
        assert!(prepare_base_overlays(&stage, &main[..len]).is_err());
    }
}
#[test]
fn title_overlay_copy_includes_all_thirteen_bytes_and_keeps_source_unmodified() {
    let mut template: Vec<u8> = (70..90).collect();
    template[4] = 0xff;
    let before = template.clone();
    assert_eq!(prepare_title_overlay(&template).unwrap(), template[..13]);
    assert_eq!(template, before);
    for len in 0..13 {
        assert!(prepare_title_overlay(&template[..len]).is_err());
    }
}
#[test]
fn repeated_idle_title_frames_have_no_demo_timeout_or_implicit_land_request() {
    let mut option = TitleOption::Normal;
    let mut a = pool();
    // More than two complete8-bit composition-clock periods and the B4 random
    // threshold. Only explicit START can leave TitleInputLoop.
    for _ in 0..520 {
        let out = title_input(
            &mut option,
            &mut a,
            input(Buttons::NONE, Buttons::START),
            &catalog(),
        );
        assert_eq!(
            out,
            InputResult {
                decision: Decision::Frame,
                operations: vec![]
            }
        );
    }
    assert_eq!(option, TitleOption::Normal);
}
#[test]
fn title_actor_initialization_preserves_unwritten_fraction_handler_and_local_fields() {
    let mut a = pool();
    for actor in &mut a {
        actor.position.x_fraction = 0x34;
        actor.position.y_fraction = 0x45;
        actor.local_state = 0x56;
        actor.handler = 0x67;
        actor.animation_restart = 0x78;
    }
    let before = a;
    initialize_title_actors(&mut a, &catalog());
    for actor in a.iter().take(2) {
        assert_eq!(
            (
                actor.position.x_fraction,
                actor.position.y_fraction,
                actor.local_state,
                actor.handler,
                actor.animation_restart
            ),
            (0x34, 0x45, 0x56, 0x67, 0x78)
        );
    }
    assert_eq!(&a[2..], &before[2..]);
}

#[test]
fn completion_message_owner_returns_exact_pending_cursor_before_acknowledgement() {
    let catalog = crate::status::StatusCatalog::decode_original(&rom()).unwrap();
    let status = crate::status::Status {
        reward: [crate::status::BLANK; 6],
        total: [crate::status::BLANK; 7],
        lives: 0,
        health: crate::status::Health {
            segments: [crate::status::HealthSegment {
                y: 0,
                tile: 0,
                depleted: 0,
                x: 0,
            }; crate::status::OVERLAY_SEGMENTS],
            end: 0xff,
            alternating: 0,
        },
        countdown: [0; 3],
        divider: 0,
        dirty: 0,
        ui_pending: 0,
    };
    for (selector, expected) in [(3, 0x67), (4, 0x5d)] {
        let message = catalog.write_message(selector, 0, &status).unwrap();
        assert_eq!(message.submission_value, expected);
        assert_eq!(message.bytes.last(), Some(&0xff));
        assert_eq!(message.bytes.len(), usize::from(expected) + 1);
        assert_eq!(
            acknowledge(input(Buttons::A, Buttons::NONE), message.submission_value),
            Acknowledgement::Accepted {
                primary_upload_pending: expected
            }
        );
    }
}

#[test]
fn raw_initial_scan_and_position_preserve_adjacent_source_consumers() {
    let bytes = rom();
    let raw = RawLandCatalog::original(&bytes).unwrap();
    for (progress, selector) in [(0x1f, 5), (0x3f, 6), (0x7f, 7), (0xff, 21)] {
        assert_eq!(raw.first_available(progress), StageId(selector));
        let mut actors = pool();
        raw.position(StageId(selector), &mut actors, true);
        let index = usize::from(selector);
        assert_eq!(
            actors[0].position.screen_y,
            bytes[LAND_PARAMETER_OFFSET + 4 + index]
        );
        assert_eq!(
            actors[3].position.screen_y,
            bytes[LAND_PARAMETER_OFFSET + 41 + index]
        );
        assert_eq!(
            actors[3].position.world_x as u8,
            bytes[LAND_PARAMETER_OFFSET + 46 + index]
        );
        assert_eq!(
            actors[2].animation,
            LAND_NAME_ANIMATION_BASE.wrapping_add(selector)
        );
    }
}
#[test]
fn raw_navigation_retains_single_wrap_and_detects_source_non_yielding_cycle() {
    let raw = RawLandCatalog::original(&rom()).unwrap();
    assert_eq!(
        raw.navigate(StageId(5), 0, true, &catalog()),
        Some(StageId(1))
    );
    assert_eq!(
        raw.navigate(StageId(5), 0, false, &catalog()),
        Some(StageId(4))
    );
    assert_eq!(raw.navigate(StageId(0), 0x1f, true, &catalog()), None);
}

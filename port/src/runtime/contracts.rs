//! B01 executable behavior contracts; derivations and source ranges: trace.md.
//! Fixtures are synthetic logical input/state, never extracted original payloads.
use super::*;

const NO_SAMPLES: [PortSample; CONTROLLER_PORTS] = [PortSample {
    standard: Buttons::NONE,
    auxiliary: Buttons::NONE,
}; CONTROLLER_PORTS];
fn sample(first: Buttons, second: Buttons) -> [PortSample; CONTROLLER_PORTS] {
    [
        PortSample {
            standard: first,
            auxiliary: Buttons::NONE,
        },
        PortSample {
            standard: second,
            auxiliary: Buttons::NONE,
        },
    ]
}
fn expect_port(controllers: &Controllers, port: usize, held: Buttons, pressed: Buttons) {
    assert_eq!(controllers.ports[port], PortInput { held, pressed });
}

#[test]
fn reset_clears_runtime_and_requests_display_before_fresh_title_entry() {
    let (runtime, display) = Runtime::reset();
    assert_eq!(runtime.interrupt_tick, 0);
    assert_eq!(runtime.controllers, Controllers::default());
    assert_eq!(
        runtime.random.words, [0; 4],
        "88 is written after the title's first yield, not at reset"
    );
    assert_eq!(
        runtime.scheduler.slots,
        [
            TaskState::Entry(BootTask::InitializeTitle),
            TaskState::Empty,
            TaskState::Empty,
            TaskState::Empty
        ]
    );
    assert_eq!(
        runtime.bank,
        BankContext {
            foreground: PrgBank::GraphicsZero,
            active: PrgBank::GraphicsZero
        }
    );
    assert_eq!(
        display,
        ResetDisplay {
            background_tile: RESET_BACKGROUND_TILE,
            attribute: 0,
            hidden_sprite_y: HIDDEN_SPRITE_Y,
            control: RESET_CONTROL,
            effects: vec![
                ResetEffect::QuiesceDisplay,
                ResetEffect::ClearRuntimeState,
                ResetEffect::HideAllSprites,
                ResetEffect::ClearFirstNametable,
                ResetEffect::ClearSecondNametable,
                ResetEffect::ScheduleTitle,
                ResetEffect::InstallControlShadow
            ]
        }
    );
}
#[test]
fn both_ports_preserve_all_action_edges_and_release_repress_independently() {
    let actions = Buttons::A | Buttons::B | Buttons::START | Buttons::SELECT;
    let mut controls = Controllers::default();
    controls.sample(sample(actions, Buttons::UP | Buttons::RIGHT));
    expect_port(&controls, 0, actions, actions);
    expect_port(
        &controls,
        1,
        Buttons::UP | Buttons::RIGHT,
        Buttons::UP | Buttons::RIGHT,
    );
    controls.sample(sample(actions, Buttons::UP));
    expect_port(&controls, 0, actions, Buttons::NONE);
    expect_port(&controls, 1, Buttons::UP, Buttons::NONE);
    controls.sample(NO_SAMPLES);
    expect_port(&controls, 0, Buttons::NONE, Buttons::NONE);
    controls.sample(sample(actions, Buttons::UP));
    expect_port(&controls, 0, actions, actions);
    expect_port(&controls, 1, Buttons::UP, Buttons::UP);
}
#[test]
fn every_button_has_its_source_serial_position_on_both_ports() {
    for button in [
        Buttons::A,
        Buttons::B,
        Buttons::SELECT,
        Buttons::START,
        Buttons::UP,
        Buttons::DOWN,
        Buttons::LEFT,
        Buttons::RIGHT,
    ] {
        let mut controls = Controllers::default();
        controls.sample(sample(button, button));
        for port in 0..CONTROLLER_PORTS {
            expect_port(&controls, port, button, button);
        }
    }
}
#[test]
fn auxiliary_serial_lane_merges_before_edges_and_opposite_filter() {
    let mut controls = Controllers::default();
    let samples = [
        PortSample {
            standard: Buttons::A | Buttons::UP,
            auxiliary: Buttons::SELECT | Buttons::DOWN,
        },
        PortSample {
            standard: Buttons::RIGHT,
            auxiliary: Buttons::B,
        },
    ];
    controls.sample(samples);
    expect_port(
        &controls,
        0,
        Buttons::A | Buttons::SELECT,
        Buttons::A | Buttons::SELECT,
    );
    expect_port(
        &controls,
        1,
        Buttons::RIGHT | Buttons::B,
        Buttons::RIGHT | Buttons::B,
    );
    controls.sample(samples);
    expect_port(&controls, 0, Buttons::A | Buttons::SELECT, Buttons::NONE);
    expect_port(&controls, 1, Buttons::RIGHT | Buttons::B, Buttons::NONE);
}
#[test]
fn all_direction_combinations_discard_entire_nibble_on_either_conflict() {
    // C1E8 tests 0C before 03. Even an unrelated third direction is discarded.
    const FILTERED_DIRECTIONS: [u8; 16] = [0, 1, 2, 0, 4, 5, 6, 0, 8, 9, 10, 0, 0, 0, 0, 0];
    for (raw, expected) in FILTERED_DIRECTIONS.into_iter().enumerate() {
        let buttons = Buttons(raw as u8) | Buttons::A | Buttons::START;
        let expected = Buttons(expected) | Buttons::A | Buttons::START;
        let mut controls = Controllers::default();
        controls.sample(sample(buttons, buttons));
        for port in 0..CONTROLLER_PORTS {
            expect_port(&controls, port, expected, expected);
        }
    }
}
#[test]
fn edge_filter_is_independent_of_held_filter_and_uses_filtered_previous_held() {
    let mut controls = Controllers::default();
    controls.sample(sample(Buttons::UP, Buttons::LEFT));
    controls.sample(sample(
        Buttons::UP | Buttons::DOWN | Buttons::A,
        Buttons::LEFT | Buttons::RIGHT | Buttons::B,
    ));
    expect_port(&controls, 0, Buttons::A, Buttons::A | Buttons::DOWN);
    expect_port(&controls, 1, Buttons::B, Buttons::B | Buttons::RIGHT);
    // Conflict destroyed previous directional held; removing one side reedges.
    controls.sample(sample(Buttons::UP | Buttons::A, Buttons::LEFT | Buttons::B));
    expect_port(&controls, 0, Buttons::UP | Buttons::A, Buttons::UP);
    expect_port(&controls, 1, Buttons::LEFT | Buttons::B, Buttons::LEFT);
}
#[test]
fn sampling_observes_game_written_held_and_pressed_is_overwritten_not_accumulated() {
    let mut controls = Controllers::default();
    controls.ports[0] = PortInput {
        held: Buttons::DOWN,
        pressed: Buttons::A,
    };
    controls.ports[1] = PortInput {
        held: Buttons::B | Buttons::A,
        pressed: Buttons::START,
    };
    controls.sample(sample(Buttons::DOWN | Buttons::A, Buttons::B | Buttons::A));
    expect_port(&controls, 0, Buttons::DOWN | Buttons::A, Buttons::A);
    expect_port(&controls, 1, Buttons::B | Buttons::A, Buttons::NONE);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SyntheticContinuation {
    TitleEntry,
    AfterYield,
    SceneEntry,
    SceneResume,
}
use SyntheticContinuation::*;
#[test]
fn scheduler_installs_replaces_and_cancels_each_of_four_slots() {
    for (index, slot) in [
        TaskSlot::Foreground,
        TaskSlot::AuxiliaryOne,
        TaskSlot::AuxiliaryTwo,
        TaskSlot::AuxiliaryThree,
    ]
    .into_iter()
    .enumerate()
    {
        let mut scheduler = Scheduler::empty();
        scheduler.install(slot, TitleEntry);
        assert_eq!(scheduler.slots[index], TaskState::Entry(TitleEntry));
        for old in [
            TaskState::Entry(TitleEntry),
            TaskState::Running(TitleEntry),
            TaskState::Sleeping {
                continuation: AfterYield,
                ticks_remaining: u8::MAX,
            },
            TaskState::Resume(AfterYield),
        ] {
            scheduler.slots[index] = old;
            scheduler.install(slot, SceneEntry);
            assert_eq!(
                scheduler.slots[index],
                TaskState::Entry(SceneEntry),
                "replacement discards old continuation/delay"
            );
        }
        scheduler.finish(slot);
        assert_eq!(scheduler.slots, [TaskState::Empty; TASK_SLOTS]);
    }
}
#[test]
fn scheduler_chooses_ascending_priority_and_never_dispatches_running_sleeping_empty() {
    let mut scheduler = Scheduler {
        slots: [
            TaskState::Sleeping {
                continuation: AfterYield,
                ticks_remaining: 2,
            },
            TaskState::Running(SceneEntry),
            TaskState::Resume(SceneResume),
            TaskState::Entry(TitleEntry),
        ],
    };
    let mut controls = Controllers::default();
    assert_eq!(
        scheduler.dispatch(&mut controls, NO_SAMPLES),
        Some(Dispatch {
            slot: TaskSlot::AuxiliaryTwo,
            continuation: SceneResume,
            kind: DispatchKind::Resume
        })
    );
    assert_eq!(scheduler.slots[2], TaskState::Running(SceneResume));
    assert_eq!(
        scheduler.dispatch(&mut controls, NO_SAMPLES),
        Some(Dispatch {
            slot: TaskSlot::AuxiliaryThree,
            continuation: TitleEntry,
            kind: DispatchKind::Entry
        })
    );
    assert_eq!(scheduler.dispatch(&mut controls, NO_SAMPLES), None);
}
#[test]
fn entry_and_other_slot_resume_do_not_sample_but_foreground_resume_does() {
    let mut scheduler = Scheduler::empty();
    let mut controls = Controllers::default();
    let held = sample(Buttons::A, Buttons::UP);
    scheduler.install(TaskSlot::Foreground, TitleEntry);
    scheduler.dispatch(&mut controls, held);
    assert_eq!(
        controls,
        Controllers::default(),
        "fresh entry jumps before ReadControllers"
    );
    scheduler.yield_one_frame(TaskSlot::Foreground, AfterYield);
    assert_eq!(scheduler.dispatch(&mut controls, held), None);
    scheduler.wake();
    assert_eq!(
        scheduler.dispatch(&mut controls, held),
        Some(Dispatch {
            slot: TaskSlot::Foreground,
            continuation: AfterYield,
            kind: DispatchKind::Resume
        })
    );
    expect_port(&controls, 0, Buttons::A, Buttons::A);
    expect_port(&controls, 1, Buttons::UP, Buttons::UP);
    scheduler.install(TaskSlot::AuxiliaryOne, SceneEntry);
    scheduler.dispatch(&mut controls, NO_SAMPLES);
    scheduler.yield_one_frame(TaskSlot::AuxiliaryOne, SceneResume);
    scheduler.wake();
    scheduler.dispatch(&mut controls, NO_SAMPLES);
    expect_port(&controls, 0, Buttons::A, Buttons::A);
    expect_port(&controls, 1, Buttons::UP, Buttons::UP);
}
#[test]
fn yield_preserves_native_continuation_and_wakes_only_after_interrupt() {
    let mut scheduler = Scheduler::empty();
    let mut controls = Controllers::default();
    scheduler.install(TaskSlot::Foreground, TitleEntry);
    scheduler.dispatch(&mut controls, NO_SAMPLES);
    scheduler.yield_one_frame(TaskSlot::Foreground, AfterYield);
    assert_eq!(
        scheduler.slots[0],
        TaskState::Sleeping {
            continuation: AfterYield,
            ticks_remaining: 1
        }
    );
    assert_eq!(scheduler.dispatch(&mut controls, NO_SAMPLES), None);
    scheduler.wake();
    assert_eq!(scheduler.slots[0], TaskState::Resume(AfterYield));
    scheduler.dispatch(&mut controls, NO_SAMPLES);
    assert_eq!(scheduler.slots[0], TaskState::Running(AfterYield));
}
#[test]
fn nmi_wake_decrements_only_sleepers_and_zero_delay_wraps() {
    let mut scheduler = Scheduler {
        slots: [
            TaskState::Sleeping {
                continuation: AfterYield,
                ticks_remaining: 0,
            },
            TaskState::Sleeping {
                continuation: SceneResume,
                ticks_remaining: 1,
            },
            TaskState::Sleeping {
                continuation: TitleEntry,
                ticks_remaining: u8::MAX,
            },
            TaskState::Entry(SceneEntry),
        ],
    };
    scheduler.wake();
    assert_eq!(
        scheduler.slots,
        [
            TaskState::Sleeping {
                continuation: AfterYield,
                ticks_remaining: u8::MAX
            },
            TaskState::Resume(SceneResume),
            TaskState::Sleeping {
                continuation: TitleEntry,
                ticks_remaining: 254
            },
            TaskState::Entry(SceneEntry)
        ]
    );
    for _ in 0..254 {
        scheduler.wake();
    }
    assert_eq!(
        scheduler.slots[0],
        TaskState::Sleeping {
            continuation: AfterYield,
            ticks_remaining: 1
        }
    );
    assert_eq!(scheduler.slots[2], TaskState::Resume(TitleEntry));
    scheduler.wake();
    assert_eq!(scheduler.slots[0], TaskState::Resume(AfterYield));
}
#[test]
fn wait_count_one_255_and_zero_require_exact_resumed_frames_and_sample_each_frame() {
    for (encoded, frames) in [(1, 1_u16), (u8::MAX, 255), (0, 256)] {
        let mut wait = FrameWait::start(encoded);
        let mut scheduler = Scheduler::empty();
        let mut controls = Controllers::default();
        scheduler.install(TaskSlot::Foreground, TitleEntry);
        scheduler.dispatch(&mut controls, NO_SAMPLES);
        assert_eq!(wait.ticks_remaining, frames);
        for frame in 1..=frames {
            scheduler.yield_one_frame(TaskSlot::Foreground, AfterYield);
            assert_eq!(scheduler.dispatch(&mut controls, NO_SAMPLES), None);
            scheduler.wake();
            let input = if frame % 2 == 1 {
                sample(Buttons::A, Buttons::B)
            } else {
                NO_SAMPLES
            };
            scheduler.dispatch(&mut controls, input);
            expect_port(&controls, 0, input[0].standard, input[0].standard);
            assert_eq!(
                wait.resumed(),
                frame == frames,
                "encoded {encoded}, frame {frame}"
            );
            assert_eq!(wait.ticks_remaining, frames - frame);
        }
    }
}
#[test]
fn simultaneous_wakes_resume_foreground_first_without_new_installation() {
    let mut scheduler = Scheduler {
        slots: [TaskState::Sleeping {
            continuation: AfterYield,
            ticks_remaining: 1,
        }; TASK_SLOTS],
    };
    let mut controls = Controllers::default();
    scheduler.wake();
    for slot in [
        TaskSlot::Foreground,
        TaskSlot::AuxiliaryOne,
        TaskSlot::AuxiliaryTwo,
        TaskSlot::AuxiliaryThree,
    ] {
        assert_eq!(
            scheduler.dispatch(&mut controls, NO_SAMPLES),
            Some(Dispatch {
                slot,
                continuation: AfterYield,
                kind: DispatchKind::Resume
            })
        );
    }
    assert_eq!(scheduler.dispatch(&mut controls, NO_SAMPLES), None);
}

#[test]
fn random_zero_lock_and_title_seed_sequence_are_call_count_exact() {
    let mut zero = RandomState::default();
    zero.advance();
    assert_eq!(zero.words, [0; 4]);
    let mut seeded = RandomState {
        words: [TITLE_RANDOM_SEED, 0, 0, 0],
    };
    // Manual C2E6 ROR chain; the third and seventh calls feed bit1 into first word bit7.
    for expected in [
        [0x44, 0, 0, 0],
        [0x22, 0, 0, 0],
        [0x91, 0, 0, 0],
        [0x48, 0x80, 0, 0],
        [0x24, 0x40, 0, 0],
        [0x12, 0x20, 0, 0],
        [0x89, 0x10, 0, 0],
        [0x44, 0x88, 0, 0],
    ] {
        seeded.advance();
        assert_eq!(seeded.words, expected);
    }
}
#[test]
fn random_feedback_xor_and_cross_word_carry_are_not_recomputed_per_byte() {
    for (before, after) in [
        ([0x02, 0, 0, 0], [0x81, 0, 0, 0]),
        ([0, 0x02, 0, 0], [0x80, 0x01, 0, 0]),
        ([0x02, 0x02, 0, 0], [0x01, 0x01, 0, 0]),
        ([0x01, 0x01, 0x01, 0x01], [0, 0x80, 0x80, 0x80]),
        ([0xff; 4], [0x7f, 0xff, 0xff, 0xff]),
    ] {
        let mut random = RandomState { words: before };
        random.advance();
        assert_eq!(random.words, after);
    }
}

fn runtime_fixture() -> Runtime {
    Runtime {
        controllers: Controllers::default(),
        random: RandomState {
            words: [TITLE_RANDOM_SEED, 0, 0, 0],
        },
        scheduler: Scheduler {
            slots: [TaskState::Empty; TASK_SLOTS],
        },
        interrupt_tick: 0,
        bank: BankContext {
            foreground: PrgBank::ActorsSix,
            active: PrgBank::ActorsSix,
        },
    }
}
fn audio_tail() -> Vec<FrameEffect> {
    vec![
        FrameEffect::DrainAudioCommands,
        FrameEffect::AdvanceAudioFrame,
        FrameEffect::RestoreForegroundBank,
    ]
}
#[test]
fn suspended_display_keeps_split_sprite_prime_but_wakes_tasks_and_runs_audio() {
    for depth in [1, u8::MAX] {
        for gameplay in [false, true] {
            let mut runtime = runtime_fixture();
            runtime.scheduler.slots[0] = TaskState::Sleeping {
                continuation: BootTask::InitializeTitle,
                ticks_remaining: 1,
            };
            let effects = runtime.interrupt(
                InterruptSource::Nmi,
                InterruptContext {
                    display_suspend_depth: depth,
                    gameplay_display: gameplay,
                    primary_pending: true,
                    secondary_pending: true,
                    palette_pending: true,
                    ..Default::default()
                },
            );
            let mut expected = Vec::new();
            if gameplay {
                expected.push(FrameEffect::PrimeSplitSprite);
            }
            expected.push(FrameEffect::WakeTasks);
            expected.extend(audio_tail());
            assert_eq!(effects, expected);
            assert_eq!(runtime.interrupt_tick, 1);
            assert_eq!(
                runtime.scheduler.slots[0],
                TaskState::Resume(BootTask::InitializeTitle)
            );
            assert_eq!(runtime.random.words, [TITLE_RANDOM_SEED, 0, 0, 0]);
            assert_eq!(runtime.controllers, Controllers::default());
            assert_eq!(runtime.bank.active, runtime.bank.foreground);
        }
    }
}
#[test]
fn normal_interrupt_commit_order_upload_flags_dma_guard_and_audio_tail() {
    // All eight flag combinations and both OAM guard states; no inferred uploads.
    for guard in [false, true] {
        for flags in 0..8 {
            let context = InterruptContext {
                oam_guard: guard,
                primary_pending: flags & 1 != 0,
                secondary_pending: flags & 2 != 0,
                palette_pending: flags & 4 != 0,
                ..Default::default()
            };
            let mut expected = vec![FrameEffect::BeginDisplayCommit];
            if !guard {
                expected.push(FrameEffect::CopyOam);
            }
            if context.primary_pending {
                expected.push(FrameEffect::FlushPrimary);
            }
            if context.secondary_pending {
                expected.push(FrameEffect::FlushSecondary);
            }
            if context.palette_pending {
                expected.push(FrameEffect::UploadPalette);
            }
            expected.extend([
                FrameEffect::InstallUiScroll,
                FrameEffect::InstallUiControl(UiControl::Shadow),
                FrameEffect::WakeTasks,
            ]);
            expected.extend(audio_tail());
            assert_eq!(
                runtime_fixture().interrupt(InterruptSource::Nmi, context),
                expected
            );
        }
    }
}
#[test]
fn gameplay_split_threshold_ending_exception_address_and_race_mask_are_ordered() {
    for (y, ending, control) in [
        (SPLIT_PLAYER_Y_THRESHOLD - 1, false, UiControl::Shadow),
        (
            SPLIT_PLAYER_Y_THRESHOLD,
            false,
            UiControl::ForceGameplayPatterns,
        ),
        (u8::MAX, false, UiControl::ForceGameplayPatterns),
        (SPLIT_PLAYER_Y_THRESHOLD, true, UiControl::Shadow),
    ] {
        for alternate in [false, true] {
            for race in [false, true] {
                let context = InterruptContext {
                    gameplay_display: true,
                    player_screen_y: y,
                    ending_layout: ending,
                    alternate_split_address: alternate,
                    race_intro: race,
                    ..Default::default()
                };
                let mut expected = vec![
                    FrameEffect::PrimeSplitSprite,
                    FrameEffect::BeginDisplayCommit,
                    FrameEffect::CopyOam,
                    FrameEffect::InstallUiScroll,
                    FrameEffect::InstallUiControl(control),
                    FrameEffect::WakeTasks,
                ];
                if alternate {
                    expected.push(FrameEffect::InstallSplitAddress);
                }
                if race {
                    expected.push(FrameEffect::SpriteOnlyMask);
                }
                expected.push(FrameEffect::InstallPlayfieldScroll);
                expected.extend(audio_tail());
                assert_eq!(
                    runtime_fixture().interrupt(InterruptSource::Nmi, context),
                    expected
                );
            }
        }
    }
}
#[test]
fn interrupt_tick_wraps_and_irq_runs_the_identical_service_without_sampling_rng() {
    let context = InterruptContext {
        gameplay_display: true,
        race_intro: true,
        ..Default::default()
    };
    let mut nmi = runtime_fixture();
    nmi.interrupt_tick = u8::MAX;
    let mut irq = nmi.clone();
    assert_eq!(
        nmi.interrupt(InterruptSource::Nmi, context),
        irq.interrupt(InterruptSource::Irq, context)
    );
    assert_eq!(nmi, irq);
    assert_eq!(nmi.interrupt_tick, 0);
    assert_eq!(nmi.random.words, [TITLE_RANDOM_SEED, 0, 0, 0]);
    assert_eq!(nmi.controllers, Controllers::default());
}
#[test]
fn temporary_audio_bank_never_replaces_foreground_context() {
    for bank in [
        PrgBank::GraphicsZero,
        PrgBank::GraphicsOne,
        PrgBank::GraphicsTwo,
        PrgBank::Audio,
        PrgBank::ActorsFour,
        PrgBank::PresentationFive,
        PrgBank::ActorsSix,
    ] {
        let mut context = BankContext {
            foreground: PrgBank::GraphicsZero,
            active: PrgBank::GraphicsZero,
        };
        context.select(bank);
        assert_eq!(
            context,
            BankContext {
                foreground: bank,
                active: bank
            }
        );
        context.enter_audio();
        assert_eq!(
            context,
            BankContext {
                foreground: bank,
                active: PrgBank::Audio
            }
        );
        context.restore_foreground();
        assert_eq!(
            context,
            BankContext {
                foreground: bank,
                active: bank
            }
        );
    }
}

#[test]
fn sampling_return_uses_raw_primary_pressed_conflicts_instead_of_held_or_filtered_bits() {
    let mut controllers = Controllers::default();
    assert_eq!(controllers.last_sample_return, None);
    controllers.sample(sample(Buttons::RIGHT, Buttons::UP | Buttons::DOWN));
    assert_eq!(
        controllers.last_sample_return,
        Some(SampleReturn {
            primary_opposite_edges: false
        })
    );
    controllers.sample(sample(Buttons::RIGHT | Buttons::LEFT, Buttons::NONE));
    assert_eq!(controllers.ports[0].held, Buttons::NONE);
    assert_eq!(controllers.ports[0].pressed, Buttons::LEFT);
    assert_eq!(
        controllers.last_sample_return,
        Some(SampleReturn {
            primary_opposite_edges: false
        }),
        "held conflict alone does not determine the final comparison"
    );
    controllers.sample(sample(Buttons::RIGHT | Buttons::LEFT, Buttons::NONE));
    assert_eq!(controllers.ports[0].pressed, Buttons::NONE);
    assert_eq!(
        controllers.last_sample_return,
        Some(SampleReturn {
            primary_opposite_edges: true
        }),
        "filtering must not erase the producer outcome"
    );
}

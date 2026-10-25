//! Executable source contracts, independently derived in trace.md (frozen revision).
use super::*;
const PRIMARY_ORIGIN: u16 = 0xd000;
const SECONDARY_ORIGIN: u16 = 0xd100;
const PERIOD_ORIGIN: u16 = 0x9000;
const PRESET_ORIGIN: u16 = 0x91fe;
const NOTE: u8 = 0x41; // duration class2, note1; two scale units make four source ticks.
const STOP_PRIMARY: u8 = 9;
const STOP_SECONDARY: u8 = 6;
const TEMPO: u8 = 0xfc;
const FADE: u8 = 0xfd;
const STOP_EFFECT: u8 = 0xfe;
const STOP_MUSIC: u8 = 0xff;
const PULSE_CONTROL: u16 = 0x4000;
const PULSE_PERIOD: u16 = 0x4002;
const PULSE_LENGTH: u16 = 0x4003;
fn catalog(primary: &[u8], secondary: &[u8]) -> Catalog {
    let mut c = Catalog::from_fragments(vec![
        Fragment {
            origin: PRIMARY_ORIGIN,
            bytes: primary.to_vec(),
        },
        Fragment {
            origin: SECONDARY_ORIGIN,
            bytes: secondary.to_vec(),
        },
        Fragment {
            origin: PERIOD_ORIGIN,
            bytes: vec![0, 0, 0x34, 0x02, 0x78, 0x01],
        },
        Fragment {
            origin: PRESET_ORIGIN,
            bytes: vec![1, 2, 3, 4, 5, 6, 7, 8],
        },
    ]);
    c.headers = vec![
        Header::Primary {
            priority: 7,
            cursors: [PRIMARY_ORIGIN; CHANNELS],
            preset_base: PRESET_ORIGIN,
        },
        Header::Secondary {
            priority: 0x50,
            mask: 0x0a,
            cursor: SECONDARY_ORIGIN,
        },
    ];
    c.duration = [0, 0, 2, 4, 8, 16, 32, 64];
    c.alternate_duration = [0, 0, 3, 6, 12, 24, 48, 96];
    c
}
fn driver() -> Driver {
    let mut d = Driver::default();
    d.channels[0].primary = Primary {
        cursor: PRIMARY_ORIGIN,
        scale: 2,
        period_table: PERIOD_ORIGIN,
        control: 0x3f,
        ..Primary::default()
    };
    d.secondary_cursor = SECONDARY_ORIGIN;
    d
}
fn w(address: u16, value: u8) -> RegisterWrite {
    RegisterWrite { address, value }
}
#[test]
fn queue_acceptance_preserves_cmp_equal_increment_and_wraps_producer() {
    let mut q = CommandQueue {
        producer: 7,
        ..CommandQueue::default()
    };
    let r = q.submit(0x15);
    assert_eq!(
        r,
        Submission {
            accepted: true,
            comparison: ProducerComparison::Empty
        }
    );
    assert!(r.random_increment());
    assert_eq!(q.slots[7], 0x15);
    assert_eq!(q.producer, 0);
}
#[test]
fn occupied_normal_drops_without_advancing_and_does_not_increment_random() {
    let mut q = CommandQueue::default();
    q.slots[0] = 0x15;
    let before = q.clone();
    let r = q.submit(0x16);
    assert_eq!(
        r,
        Submission {
            accepted: false,
            comparison: ProducerComparison::BelowEmpty
        }
    );
    assert!(!r.random_increment());
    assert_eq!(q, before);
}
#[test]
fn occupied_ff_drops_but_comparison_still_increments_random() {
    let mut q = CommandQueue::default();
    q.slots[0] = STOP_MUSIC;
    let r = q.submit(0x15);
    assert_eq!(
        r,
        Submission {
            accepted: false,
            comparison: ProducerComparison::AboveEmpty
        }
    );
    assert!(r.random_increment());
    assert_eq!(q.producer, 0);
    assert_eq!(q.slots[0], STOP_MUSIC);
}
#[test]
fn each_enqueue_observes_fresh_producer_not_a_copied_context() {
    let mut q = CommandQueue::default();
    for _ in 0..QUEUE_SLOTS {
        assert!(q.submit(0x15).random_increment());
    }
    let r = q.submit(0x16);
    assert!(!r.accepted);
    assert!(!r.random_increment());
    assert_eq!(q.producer, 0);
}
#[test]
fn stage_command_is_remembered_before_full_queue_rejection() {
    let mut q = CommandQueue::default();
    q.slots[0] = 0x15;
    assert!(!q.submit_stage(0x37).accepted);
    assert_eq!(q.stage_command, 0x37);
    assert_eq!(q.slots[0], 0x15);
}
#[test]
fn sentinel_submission_is_accepted_creates_hole_and_stops_consumer() {
    let mut q = CommandQueue::default();
    assert!(q.submit(EMPTY_COMMAND).accepted);
    assert!(q.submit(0x15).accepted);
    assert_eq!(q.producer, 2);
    assert_eq!(q.drain(), Vec::<u8>::new());
    assert_eq!(q.consumer, 0);
    assert_eq!(q.slots[1], 0x15);
}
#[test]
fn ring_drain_wraps_clears_each_slot_and_stops_at_first_hole() {
    let mut q = CommandQueue {
        consumer: 7,
        producer: 2,
        ..CommandQueue::default()
    };
    q.slots[7] = 1;
    q.slots[0] = 2;
    q.slots[2] = 3;
    assert_eq!(q.drain(), vec![1, 2]);
    assert_eq!(q.consumer, 1);
    assert_eq!(q.slots[7], EMPTY_COMMAND);
    assert_eq!(q.slots[0], EMPTY_COMMAND);
    assert_eq!(q.slots[2], 3);
}
#[test]
fn all_eight_commands_drain_in_fifo_order_without_count_state() {
    let mut q = CommandQueue::default();
    for c in 0..QUEUE_SLOTS as u8 {
        q.submit(c);
    }
    assert_eq!(q.drain(), (0..QUEUE_SLOTS as u8).collect::<Vec<_>>());
    assert_eq!(q.slots, [EMPTY_COMMAND; QUEUE_SLOTS]);
    assert_eq!(q.consumer, 0);
}
#[test]
fn interrupt_drains_in_order_before_advancing_exactly_one_driver_frame() {
    let c = catalog(&[NOTE], &[STOP_SECONDARY]);
    let mut d = Driver::default();
    let mut q = CommandQueue::default();
    q.submit(0);
    q.submit(STOP_MUSIC);
    d.interrupt(&mut q, &c, 0);
    assert_eq!(d.frames, 1);
    assert_eq!(d.phase, 1);
    assert_eq!(d.priority, 0);
    assert!(d.channels.iter().all(|v| v.primary.cursor == 0));
    assert_eq!(q.slots, [EMPTY_COMMAND; QUEUE_SLOTS]);
}
#[test]
fn primary_priority_rejects_lower_accepts_equal_and_preserves_secondary_nibble() {
    let mut c = catalog(&[NOTE], &[]);
    c.headers[0] = Header::Primary {
        priority: 6,
        cursors: [PRIMARY_ORIGIN; CHANNELS],
        preset_base: PRESET_ORIGIN,
    };
    let mut d = Driver {
        priority: 0x57,
        ..Driver::default()
    };
    let before = d.clone();
    assert_eq!(d.command(&c, 0, 0, &mut vec![]), Install::Rejected);
    assert_eq!(d, before);
    d.priority = 0x56;
    assert_eq!(d.command(&c, 0, 0, &mut vec![]), Install::Primary);
    assert_eq!(d.priority, 0x56);
}
#[test]
fn primary_header_compares_full_byte_then_ors_it_into_priority() {
    let mut c = catalog(&[NOTE], &[]);
    c.headers[0] = Header::Primary {
        priority: 0x21,
        cursors: [PRIMARY_ORIGIN; CHANNELS],
        preset_base: PRESET_ORIGIN,
    };
    let mut d = Driver {
        priority: 0x87,
        ..Driver::default()
    };
    assert_eq!(d.command(&c, 0, 0, &mut vec![]), Install::Primary);
    assert_eq!(d.priority, 0xa1);
}
#[test]
fn primary_install_resets_prefix_and_controls_preserves_masked_secondary_work() {
    let c = catalog(&[NOTE], &[]);
    let mut d = Driver {
        secondary_mask: 1,
        tempo: 4,
        fade: 3,
        ..Driver::default()
    };
    d.channels[0].secondary.period = 0x4321;
    d.channels[1].secondary.period = 0x4321;
    d.command(&c, 0, 0, &mut vec![]);
    assert_eq!(d.channels[0].secondary.period, 0x4321);
    assert_eq!(d.channels[1].secondary.period, 0);
    assert_eq!(d.channels[0].primary.cursor, PRIMARY_ORIGIN);
    assert_eq!(d.channels[0].primary.scale, 0);
    assert_eq!(
        (d.tempo, d.fade, d.secondary_mask, d.preset_base, d.busy),
        (0, 0, 1, PRESET_ORIGIN, false)
    );
}
#[test]
fn secondary_priority_compares_high_nibble_and_preserves_primary() {
    let c = catalog(&[], &[STOP_SECONDARY]);
    let mut d = Driver {
        priority: 0x67,
        ..Driver::default()
    };
    assert_eq!(d.command(&c, 1, 0, &mut vec![]), Install::Rejected);
    d.priority = 0x57;
    assert_eq!(d.command(&c, 1, 0, &mut vec![]), Install::Secondary);
    assert_eq!(d.priority, 0x57);
    assert_eq!(d.secondary_mask, 0x0a);
    assert_eq!(d.secondary_cursor, SECONDARY_ORIGIN);
    assert_eq!(d.shared_delay, 0);
}
#[test]
fn secondary_install_only_clears_selected_work_and_restores_displaced_primary() {
    let c = catalog(&[], &[STOP_SECONDARY]);
    let mut d = driver();
    d.secondary_mask = 1;
    d.channels[0].primary.period = 0x1234;
    d.channels[0].modulation.cached_high = 2;
    d.channels[1].secondary.period = 0x4321;
    d.preset_base = PRESET_ORIGIN;
    d.command(&c, 1, 0, &mut vec![]);
    assert_eq!(d.channels[0].modulation.cached_high, 0xff);
    assert_eq!(d.channels[0].primary.period, 0x1234);
    assert_eq!(d.channels[1].secondary.period, 0);
    assert_eq!(d.secondary_mask, 0x0a);
}
#[test]
fn tempo_fc_uses_wrapping_parameter_plus_one_without_installing_stream() {
    let c = catalog(&[], &[]);
    for (parameter, expected) in [(0, 1), (3, 4), (0xff, 0)] {
        let mut d = driver();
        let before = d.channels;
        assert_eq!(d.command(&c, TEMPO, parameter, &mut vec![]), Install::Tempo);
        assert_eq!(d.tempo, expected);
        assert_eq!(d.channels, before);
    }
}
#[test]
fn fade_fd_retains_parameter_resets_level_and_only_phase_low_bit() {
    let c = catalog(&[], &[]);
    let mut d = Driver {
        phase: 0xff,
        ..Driver::default()
    };
    assert_eq!(d.command(&c, FADE, 0x84, &mut vec![]), Install::Fade);
    assert_eq!((d.fade, d.fade_level, d.phase), (0x84, 1, 1));
}
#[test]
fn fe_stops_secondary_restores_primary_without_clearing_low_priority() {
    let c = catalog(&[], &[]);
    let mut d = driver();
    d.priority = 0x57;
    d.secondary_mask = 1;
    d.preset_base = PRESET_ORIGIN;
    d.channels[0].primary.period = 0x0234;
    d.command(&c, STOP_EFFECT, 0, &mut vec![]);
    assert_eq!(d.priority, 7);
    assert_eq!(d.secondary_mask, 0x10);
    assert!(!d.busy);
    assert_eq!(d.channels[0].modulation.cached_high, 0xff);
}
#[test]
fn ff_clears_all_primary_cursors_controls_but_retains_secondary_work() {
    let c = catalog(&[], &[]);
    let mut d = driver();
    d.priority = 0x57;
    d.tempo = 2;
    d.fade = 3;
    d.secondary_mask = 0x0a;
    d.channels[0].secondary.period = 0x1234;
    d.command(&c, STOP_MUSIC, 0, &mut vec![]);
    assert!(d.channels.iter().all(|c| c.primary.cursor == 0));
    assert_eq!(
        (d.priority, d.tempo, d.fade, d.secondary_mask),
        (0x50, 0, 0, 0x0a)
    );
    assert_eq!(d.channels[0].secondary.period, 0x1234);
}
#[test]
fn busy_frame_still_writes_sweep_enables_and_wraps_phase_but_skips_counters() {
    let mut d = Driver {
        busy: true,
        phase: 0xff,
        shared_delay: 3,
        ..Driver::default()
    };
    let writes = d.frame(&catalog(&[], &[]), 0);
    assert_eq!(writes, vec![w(0x4001, 8), w(0x4005, 8), w(0x4015, 15)]);
    assert_eq!((d.phase, d.shared_delay), (0, 3));
    assert_eq!(d.channels[0].modulation.pitch_counter, 0);
}
#[test]
fn frame_advances_all_modulation_counters_with_byte_wrap() {
    let mut d = Driver::default();
    d.channels[0].modulation.pitch_counter = 0xff;
    d.channels[0].modulation.level_counter = 0xff;
    d.frame(&catalog(&[], &[]), 0);
    assert_eq!(d.channels[0].modulation.pitch_counter, 0);
    assert_eq!(d.channels[0].modulation.level_counter, 0);
    assert_eq!(d.channels[3].modulation.pitch_counter, 1);
}
#[test]
fn status_selector_bit_zero_suppresses_primary_but_not_secondary_or_delay() {
    let mut d = driver();
    d.secondary_mask = 1;
    d.shared_delay = 2;
    d.channels[0].secondary.period = 0x0234;
    d.channels[0].secondary.control = 0x3f;
    d.frame(&catalog(&[NOTE], &[]), 1);
    assert_eq!(d.channels[0].primary.cursor, PRIMARY_ORIGIN);
    assert_eq!(d.shared_delay, 1);
    assert_eq!(d.secondary_mask, 1);
    assert!(d.channels[0].modulation.cached_high != 0xff);
}
#[test]
fn unowned_triangle_silences_control_period_and_length_in_source_order() {
    let writes = Driver::default().frame(&catalog(&[], &[]), 0);
    let triangle: Vec<_> = writes
        .into_iter()
        .filter(|w| (0x4008..=0x400b).contains(&w.address))
        .collect();
    assert_eq!(triangle, vec![w(0x4008, 0), w(0x400a, 0), w(0x400b, 0)]);
}
#[test]
fn primary_duration_four_expires_zero_and_decodes_same_call() {
    let mut d = driver();
    d.channels[0].primary.duration = 4;
    d.primary_step(&catalog(&[NOTE], &[]), Voice::PulseOne);
    assert_eq!(d.channels[0].primary.duration, 4);
    assert_eq!(d.channels[0].primary.cursor, PRIMARY_ORIGIN + 1);
    assert_eq!(d.channels[0].primary.period, 0x0234);
}
#[test]
fn primary_duration_underflow_wraps_and_does_not_expire() {
    let mut d = driver();
    d.channels[0].primary.duration = 2;
    d.primary_step(&catalog(&[NOTE], &[]), Voice::PulseOne);
    assert_eq!(d.channels[0].primary.duration, 0xfffe);
    assert_eq!(d.channels[0].primary.cursor, PRIMARY_ORIGIN);
}
#[test]
fn tempo_repeats_whole_duration_step_exact_number_of_times() {
    let mut d = driver();
    d.tempo = 3;
    d.channels[0].primary.duration = 20;
    d.primary_step(&catalog(&[NOTE], &[]), Voice::PulseOne);
    assert_eq!(d.channels[0].primary.duration, 8);
}
#[test]
fn continuation_takes_second_tick_only_on_odd_phase() {
    for (phase, expected) in [(0, 16), (1, 12)] {
        let mut d = driver();
        d.phase = phase;
        d.channels[0].primary.duration = 20;
        d.channels[0].primary.repeat_control = 0x80;
        d.primary_step(&catalog(&[], &[]), Voice::PulseOne);
        assert_eq!(d.channels[0].primary.duration, expected);
    }
}
#[test]
fn primary_command_zero_sets_duration_scale_then_decodes_note() {
    let mut d = driver();
    d.primary_step(&catalog(&[0, 6, NOTE], &[]), Voice::PulseOne);
    assert_eq!(d.channels[0].primary.scale, 6);
    assert_eq!(d.channels[0].primary.duration, 12);
    assert_eq!(d.channels[0].primary.cursor, PRIMARY_ORIGIN + 3);
}
#[test]
fn duration_scale_zero_means_256_additions_not_zero() {
    let mut d = driver();
    d.channels[0].primary.scale = 0;
    d.primary_step(&catalog(&[NOTE], &[]), Voice::PulseOne);
    assert_eq!(d.channels[0].primary.duration, 512);
}
#[test]
fn primary_command_one_sets_signed_pitch_step() {
    let mut d = driver();
    d.primary_step(&catalog(&[1, 0xff, NOTE], &[]), Voice::PulseOne);
    assert_eq!(d.channels[0].primary.pitch_step, -1);
}
#[test]
fn primary_control_commands_preserve_opposite_bit_groups() {
    let mut d = driver();
    d.primary_step(&catalog(&[2, 0x80, 3, 0x15, NOTE], &[]), Voice::PulseOne);
    assert_eq!(d.channels[0].primary.control, 0x95);
}
#[test]
fn triangle_low_control_replaces_full_byte_instead_of_merging() {
    let mut d = driver();
    d.channels[2].primary = d.channels[0].primary;
    d.primary_step(&catalog(&[2, 0xc0, 3, 0x21, NOTE], &[]), Voice::Triangle);
    assert_eq!(d.channels[2].primary.control, 0x21);
}
#[test]
fn primary_loop_exhaustion_clears_count_preserves_continue_and_skips_word_with_carry() {
    let mut d = driver();
    d.channels[0].primary.cursor = 0xd0fc;
    d.channels[0].primary.repeat_control = 2;
    let mut c = catalog(&[], &[]);
    c.add_fragment(Fragment {
        origin: 0xd0fc,
        bytes: vec![4, 2, 0x34, 0x12, NOTE],
    });
    d.primary_step(&c, Voice::PulseOne);
    assert_eq!(d.channels[0].primary.cursor, 0xd101);
    assert_eq!(d.channels[0].primary.repeat_control, 0);
}
#[test]
fn exhausted_primary_loop_skips_unavailable_destination_bytes_without_reading_them() {
    // AudioPrimaryLoop_Continue_883C advances two positions without a read.
    let mut c = catalog(&[4, 1], &[]);
    c.add_fragment(Fragment {
        origin: PRIMARY_ORIGIN + 4,
        bytes: vec![NOTE],
    });
    let mut d = driver();
    d.channels[0].primary.repeat_control = 1;
    d.primary_step(&c, Voice::PulseOne);
    assert_eq!(d.channels[0].primary.cursor, PRIMARY_ORIGIN + 5);
    assert_eq!(d.channels[0].primary.repeat_control, 0);
}
#[test]
fn exhausted_secondary_loop_skips_unavailable_destination_bytes_without_reading_them() {
    // AudioSecondaryLoop_Clear_862A has the same skip, and no read seed.
    let mut c = catalog(&[], &[4, 1]);
    c.add_fragment(Fragment {
        origin: SECONDARY_ORIGIN + 4,
        bytes: vec![0x82, 0x34],
    });
    let mut d = driver();
    d.secondary_repeat = 1;
    d.secondary_step(&c, Voice::PulseOne);
    assert_eq!(d.secondary_cursor, SECONDARY_ORIGIN + 6);
    assert_eq!(d.channels[0].secondary.period, 0x0234);
    assert_eq!(d.secondary_repeat, 0);
}
#[test]
fn compiled_programs_keep_overlapping_entries_and_native_note_meanings() {
    let c = catalog(&[0x21, 0x30, 5, 0xff, 6, 0xe1, 9], &[]);
    assert_eq!(
        c.programs.primary[&PRIMARY_ORIGIN].action,
        PrimaryAction::Deferred(1)
    );
    assert_eq!(
        c.programs.primary[&(PRIMARY_ORIGIN + 1)].action,
        PrimaryAction::Continue
    );
    assert_eq!(
        c.programs.primary[&(PRIMARY_ORIGIN + 2)].action,
        PrimaryAction::PeriodTable(Some(0x8a79))
    );
    let note = Note {
        key: 1,
        duration: DurationClass::Alternate(7),
    };
    assert_eq!(
        c.programs.primary[&(PRIMARY_ORIGIN + 4)].action,
        PrimaryAction::AlternateNote(Some(note))
    );
    assert_eq!(
        c.programs.primary[&(PRIMARY_ORIGIN + 5)].action,
        PrimaryAction::Note(Note {
            duration: DurationClass::Standard(7),
            ..note
        })
    );
    assert_eq!(
        c.programs.secondary[&(PRIMARY_ORIGIN + 5)].action,
        SecondaryAction::Period(Some(0x0109))
    );
}
#[test]
fn explicit_asset_overrides_rebuild_both_instruction_roles_without_stale_entries() {
    let mut c = catalog(&[NOTE], &[]);
    let original = c.clone();
    c.prepend_fragment(Fragment {
        origin: PRIMARY_ORIGIN,
        bytes: vec![0x8f, 0x77],
    });
    assert_eq!(c.read(PRIMARY_ORIGIN).unwrap(), 0x8f);
    assert_eq!(
        c.programs.secondary[&PRIMARY_ORIGIN].action,
        SecondaryAction::KeepPeriod(Some(0x77))
    );
    assert_eq!(
        c.programs.primary[&PRIMARY_ORIGIN].action,
        PrimaryAction::Note(Note {
            key: 15,
            duration: DurationClass::Standard(4)
        })
    );
    assert_eq!(
        original.programs.primary[&PRIMARY_ORIGIN].action,
        PrimaryAction::Note(Note {
            key: 1,
            duration: DurationClass::Standard(2)
        })
    );
}
#[test]
fn native_program_positions_wrap_across_ffff_for_both_stream_roles() {
    let mut c = catalog(&[], &[]);
    c.add_fragment(Fragment {
        origin: 0xffff,
        bytes: vec![1],
    });
    c.add_fragment(Fragment {
        origin: 0,
        bytes: vec![0xfe, NOTE],
    });
    let mut d = driver();
    d.channels[0].primary.cursor = 0xffff;
    d.primary_step(&c, Voice::PulseOne);
    assert_eq!(
        (
            d.channels[0].primary.cursor,
            d.channels[0].primary.pitch_step
        ),
        (2, -2)
    );
    let mut c = catalog(&[], &[]);
    c.add_fragment(Fragment {
        origin: 0xffff,
        bytes: vec![0x82],
    });
    c.add_fragment(Fragment {
        origin: 0,
        bytes: vec![0x34],
    });
    let mut d = driver();
    d.secondary_cursor = 0xffff;
    d.secondary_step(&c, Voice::PulseOne);
    assert_eq!(
        (d.secondary_cursor, d.channels[0].secondary.period),
        (1, 0x0234)
    );
}
#[test]
fn primary_loop_zero_jumps_without_incrementing_repeat_count() {
    let mut d = driver();
    let c = catalog(&[4, 0, 0x04, 0xd0, NOTE], &[]);
    d.primary_step(&c, Voice::PulseOne);
    assert_eq!(d.channels[0].primary.cursor, PRIMARY_ORIGIN + 5);
    assert_eq!(d.channels[0].primary.repeat_control, 0);
}
#[test]
fn primary_loop_nonzero_increments_before_jump_and_note_returns() {
    let mut d = driver();
    d.primary_step(&catalog(&[4, 2, 0x04, 0xd0, NOTE], &[]), Voice::PulseOne);
    assert_eq!(d.channels[0].primary.repeat_control, 1);
    assert_eq!(d.channels[0].primary.cursor, PRIMARY_ORIGIN + 5);
}
#[test]
fn primary_period_selector_doubles_in_byte_width_then_adds_source_base() {
    let mut d = driver();
    d.primary_step(&catalog(&[5, 0x80, 0x40], &[]), Voice::PulseOne);
    assert_eq!(d.channels[0].primary.period_table, 0x897b);
    assert_eq!(d.channels[0].primary.period, 0);
}
#[test]
fn alternate_duration_command_uses_second_table_and_its_own_note_bits() {
    let mut d = driver();
    d.primary_step(&catalog(&[6, NOTE], &[]), Voice::PulseOne);
    assert_eq!(d.channels[0].primary.duration, 6);
    assert_eq!(d.channels[0].primary.period, 0x0234);
}
#[test]
fn primary_envelope_command_initializes_sign_endpoint_and_preserves_step() {
    for (interval, expected) in [(2, 0x30), (0x82, 0x3f), (0, 0x35)] {
        let mut d = driver();
        d.primary_step(&catalog(&[7, interval, 0x35, NOTE], &[]), Voice::PulseOne);
        assert_eq!(d.channels[0].primary.envelope.step_level, expected);
        assert_eq!(d.channels[0].primary.envelope.counter, 0);
    }
}
#[test]
fn primary_preset_loads_four_bytes_across_page_boundary() {
    let mut d = driver();
    d.preset_base = PRESET_ORIGIN;
    d.primary_step(&catalog(&[8, 0, NOTE], &[]), Voice::PulseOne);
    assert_eq!(
        (
            d.channels[0].modulation.pitch_interval,
            d.channels[0].modulation.pitch_shape,
            d.channels[0].modulation.level_interval,
            d.channels[0].modulation.level_step
        ),
        (1, 2, 3, 4)
    );
}
#[test]
fn masked_preset_command_updates_index_without_overwriting_effect_modulation() {
    let mut d = driver();
    d.secondary_mask = 1;
    d.channels[0].modulation.pitch_interval = 9;
    d.primary_step(&catalog(&[8, 1, NOTE], &[]), Voice::PulseOne);
    assert_eq!(d.channels[0].primary.note_control, 1);
    assert_eq!(d.channels[0].modulation.pitch_interval, 9);
}
#[test]
fn primary_stop_clears_cursor_low_priority_and_silences_only_unmasked() {
    for (mask, expected) in [(0, vec![w(PULSE_CONTROL, 0x30)]), (1, vec![])] {
        let mut d = driver();
        d.priority = 0x57;
        d.secondary_mask = mask;
        let writes = d.primary_step(&catalog(&[STOP_PRIMARY], &[]), Voice::PulseOne);
        assert_eq!(d.channels[0].primary.cursor, 0);
        assert_eq!(d.priority, 0x50);
        assert_eq!(writes, expected);
    }
}
#[test]
fn primary_rest_sets_zero_period_and_silences_current_voice() {
    let mut d = driver();
    d.channels[0].primary.period = 0x1234;
    let writes = d.primary_step(&catalog(&[0x40], &[]), Voice::PulseOne);
    assert_eq!(d.channels[0].primary.period, 0);
    assert_eq!(writes.last(), Some(&w(PULSE_CONTROL, 0x30)));
}
#[test]
fn noise_note_uses_low_five_command_bits_directly_without_period_table() {
    let mut d = driver();
    d.channels[3].primary = d.channels[0].primary;
    d.primary_step(&catalog(&[0x5b], &[]), Voice::Noise);
    assert_eq!(d.channels[3].primary.period, 0x001b);
}
#[test]
fn note_flag_prefix_applies_after_nested_note_and_retains_preset() {
    let mut d = driver();
    d.channels[0].primary.note_control = 5;
    d.primary_step(&catalog(&[0x23, NOTE], &[]), Voice::PulseOne);
    assert_eq!(d.channels[0].primary.note_control, 0x65);
    assert_eq!(d.channels[0].primary.period, 0x0234);
}
#[test]
fn deferred_note_counter_retains_period_and_decrements_high_bits() {
    let mut d = driver();
    d.channels[0].primary.note_control = 0x45;
    d.channels[0].primary.period = 0x1234;
    d.primary_step(&catalog(&[NOTE], &[]), Voice::PulseOne);
    assert_eq!(d.channels[0].primary.note_control, 0x25);
    assert_eq!(d.channels[0].primary.period, 0x1234);
}
#[test]
fn primary_continuation_command_sets_bit_before_next_note() {
    let mut d = driver();
    d.primary_step(&catalog(&[0x30, NOTE], &[]), Voice::PulseOne);
    assert_eq!(d.channels[0].primary.repeat_control, 0x80);
}
#[test]
fn primary_byte_read_carries_cursor_across_page() {
    let mut d = driver();
    d.channels[0].primary.cursor = 0xd0ff;
    let mut c = catalog(&[], &[]);
    c.add_fragment(Fragment {
        origin: 0xd0ff,
        bytes: vec![NOTE],
    });
    d.primary_step(&c, Voice::PulseOne);
    assert_eq!(d.channels[0].primary.cursor, 0xd100);
}
#[test]
fn secondary_delay_command_keeps_decoding_until_note_and_delay_is_shared() {
    let mut d = driver();
    d.secondary_mask = 1;
    let writes = d.secondary_step(&catalog(&[], &[0, 3, 0x82, 0x34]), Voice::PulseOne);
    assert_eq!(d.shared_delay, 3);
    assert_eq!(d.channels[0].secondary.period, 0x0234);
    assert_eq!(d.secondary_cursor, SECONDARY_ORIGIN + 4);
    assert_eq!(writes.last(), Some(&w(PULSE_CONTROL, 0x30)));
}
#[test]
fn secondary_pitch_and_control_commands_use_actual_all_three_table_entries() {
    let mut d = driver();
    d.secondary_mask = 1;
    d.secondary_step(
        &catalog(&[], &[1, 0xfe, 2, 0x80, 3, 0x15, 0x82, 0x34]),
        Voice::PulseOne,
    );
    assert_eq!(d.channels[0].secondary.pitch_step, -2);
    assert_eq!(d.channels[0].secondary.control, 0x95);
}
#[test]
fn secondary_triangle_control_low_is_full_byte() {
    let mut d = driver();
    d.secondary_mask = 4;
    d.secondary_step(
        &catalog(&[], &[2, 0xc0, 3, 0x21, 0x82, 0x34]),
        Voice::Triangle,
    );
    assert_eq!(d.channels[2].secondary.control, 0x21);
}
#[test]
fn secondary_loop_finite_count_jumps_then_exhaustion_skips_word() {
    let mut d = driver();
    d.secondary_mask = 1;
    let c = catalog(&[], &[4, 1, 0x06, 0xd1, 0x81, 0x55, 0x82, 0x34]);
    d.secondary_step(&c, Voice::PulseOne);
    assert_eq!(d.secondary_repeat, 1);
    assert_eq!(d.channels[0].secondary.period, 0x0234);
    d.secondary_cursor = SECONDARY_ORIGIN;
    d.secondary_step(&c, Voice::PulseOne);
    assert_eq!(d.secondary_repeat, 0);
    assert_eq!(d.channels[0].secondary.period, 0x0155);
}
#[test]
fn secondary_loop_zero_never_increments_shared_repeat() {
    let mut d = driver();
    d.secondary_mask = 1;
    d.secondary_step(
        &catalog(&[], &[4, 0, 0x04, 0xd1, 0x82, 0x34]),
        Voice::PulseOne,
    );
    assert_eq!(d.secondary_repeat, 0);
    assert_eq!(d.channels[0].secondary.period, 0x0234);
}
#[test]
fn secondary_modulation_command_copies_exactly_four_bytes_then_decodes_note() {
    let mut d = driver();
    d.secondary_mask = 1;
    d.secondary_step(
        &catalog(&[], &[5, 1, 2, 3, 0xfe, 0x82, 0x34]),
        Voice::PulseOne,
    );
    assert_eq!(
        (
            d.channels[0].modulation.pitch_interval,
            d.channels[0].modulation.pitch_shape,
            d.channels[0].modulation.level_interval,
            d.channels[0].modulation.level_step
        ),
        (1, 2, 3, -2)
    );
    assert_eq!(d.secondary_cursor, SECONDARY_ORIGIN + 7);
}
#[test]
fn secondary_end_backs_cursor_clears_current_mask_and_high_priority_restores_preset() {
    let mut d = driver();
    d.priority = 0x57;
    d.secondary_mask = 3;
    d.preset_base = PRESET_ORIGIN;
    d.channels[0].primary.period = 0x0234;
    d.secondary_step(&catalog(&[], &[STOP_SECONDARY]), Voice::PulseOne);
    assert_eq!(d.secondary_cursor, SECONDARY_ORIGIN);
    assert_eq!(d.priority, 7);
    assert_eq!(d.secondary_mask, 2);
    assert_eq!(d.channels[0].modulation.cached_high, 0xff);
    assert_eq!(d.channels[0].modulation.pitch_interval, 1);
}
#[test]
fn secondary_end_with_no_primary_cursor_or_period_silences_and_returns() {
    let mut d = Driver {
        secondary_cursor: SECONDARY_ORIGIN,
        secondary_mask: 1,
        priority: 0x50,
        ..Driver::default()
    };
    assert_eq!(
        d.secondary_step(&catalog(&[], &[STOP_SECONDARY]), Voice::PulseOne),
        vec![w(PULSE_CONTROL, 0x30)]
    );
    assert_eq!(d.secondary_mask, 0);
}
#[test]
fn secondary_high_low_fifteen_consumes_parameter_without_period_change_and_keeps_cache() {
    let mut d = driver();
    d.channels[0].secondary.period = 0x1234;
    d.channels[0].modulation.cached_high = 0x12;
    d.secondary_step(&catalog(&[], &[0x8f, 0x77]), Voice::PulseOne);
    assert_eq!(d.secondary_cursor, SECONDARY_ORIGIN + 2);
    assert_eq!(d.channels[0].secondary.period, 0x1234);
    assert_eq!(d.channels[0].modulation.cached_high, 0x12);
}
#[test]
fn secondary_note_uses_only_three_high_period_bits_and_reads_with_page_carry() {
    let mut d = driver();
    d.secondary_cursor = 0xd1ff;
    let mut c = catalog(&[], &[]);
    c.add_fragment(Fragment {
        origin: 0xd1ff,
        bytes: vec![0xfe, 0x34],
    });
    d.secondary_step(&c, Voice::PulseOne);
    assert_eq!(d.channels[0].secondary.period, 0x0634);
    assert_eq!(d.secondary_cursor, 0xd201);
}
#[test]
fn held_secondary_null_period_returns_without_consuming_stream() {
    let mut d = driver();
    d.shared_delay = 2;
    assert!(
        d.secondary_step(&catalog(&[], &[]), Voice::PulseOne)
            .is_empty()
    );
    assert_eq!(d.secondary_cursor, SECONDARY_ORIGIN);
}
#[test]
fn secondary_held_output_uses_its_level_and_primary_step_remains_hidden() {
    let mut d = driver();
    d.shared_delay = 2;
    d.secondary_mask = 1;
    d.channels[0].secondary.control = 0xbf;
    d.channels[0].secondary.period = 0x0234;
    let writes = d.secondary_step(&catalog(&[], &[]), Voice::PulseOne);
    assert_eq!(writes[0], w(PULSE_CONTROL, 0xbf));
    assert_eq!(writes[1], w(PULSE_PERIOD, 0x34));
    assert_eq!(writes[2], w(PULSE_LENGTH, 0x0a));
}
#[test]
fn fade_positive_caps_level_negative_decrements_without_crossing_zero() {
    for (fade, level, expected) in [(2, 3, 3), (2, 12, 5), (0x82, 3, 0), (0x82, 12, 7)] {
        let mut d = driver();
        d.fade = fade;
        d.fade_level = 5;
        d.channels[0].primary.control = 0x30 | level;
        assert_eq!(d.level(Voice::PulseOne) & 15, expected);
    }
}
#[test]
fn triangle_fade_scales_accumulator_by_thirteen_with_byte_wrap() {
    let mut d = driver();
    d.fade = 2;
    d.fade_level = 3;
    d.channels[2].primary.control = 100;
    assert_eq!(d.level(Voice::Triangle), 39);
}
#[test]
fn fade_phase_advances_only_at_equality_and_saturates_positive_mode() {
    let mut d = Driver {
        phase: 2,
        fade: 3,
        fade_level: 15,
        ..Driver::default()
    };
    d.frame(&catalog(&[], &[]), 0);
    assert_eq!((d.phase, d.fade, d.fade_level), (1, 0, 15));
}
#[test]
fn negative_fade_retains_mode_and_next_equality_reclamps_fifteen() {
    let mut d = Driver {
        phase: 2,
        fade: 0x83,
        fade_level: 15,
        ..Driver::default()
    };
    d.frame(&catalog(&[], &[]), 0);
    assert_eq!((d.phase, d.fade, d.fade_level), (1, 0x83, 15));
}
#[test]
fn envelope_equality_applies_signed_high_nibble_then_clamps_to_level() {
    for (interval, packed, expected) in [
        (2, 0x35, 8),
        (0x82, 0x35, 2),
        (0x82, 0x31, 0),
        (2, 0x3e, 15),
    ] {
        let mut d = driver();
        d.channels[0].primary.envelope = Envelope {
            interval,
            counter: 2,
            step_level: packed,
        };
        assert_eq!(d.level(Voice::PulseOne) & 15, expected);
        assert_eq!(d.channels[0].primary.envelope.counter, 0);
    }
}
#[test]
fn envelope_nonmatching_counter_keeps_level_and_triangle_skips_envelope() {
    let mut d = driver();
    d.channels[0].primary.envelope = Envelope {
        interval: 3,
        counter: 2,
        step_level: 0x35,
    };
    assert_eq!(d.level(Voice::PulseOne) & 15, 5);
    d.channels[2].primary.control = 100;
    d.channels[2].primary.envelope = d.channels[0].primary.envelope;
    assert_eq!(d.level(Voice::Triangle), 100);
}
#[test]
fn level_modulation_low_endpoint_clamps_one_and_reverses_step() {
    let mut d = driver();
    d.channels[0].modulation = Modulation {
        level_interval: 2,
        level_counter: 2,
        level: 1,
        level_step: -2,
        ..Modulation::default()
    };
    assert_eq!(d.level(Voice::PulseOne) & 15, 1);
    assert_eq!(d.channels[0].modulation.level_step, 2);
    assert_eq!(d.channels[0].modulation.level_counter, 0);
}
#[test]
fn level_modulation_high_endpoint_clamps_fifteen_and_reverses_step() {
    let mut d = driver();
    d.channels[0].modulation = Modulation {
        level_interval: 2,
        level_counter: 2,
        level: 14,
        level_step: 3,
        ..Modulation::default()
    };
    assert_eq!(d.level(Voice::PulseOne) & 15, 15);
    assert_eq!(d.channels[0].modulation.level_step, -3);
    assert_eq!(d.channels[0].modulation.level, 15);
}
#[test]
fn signed_pitch_step_integrates_with_carry_and_borrow_in_sixteen_bits() {
    for (period, step, expected) in [(0x0100, -1, 0x00ff), (0x00ff, 1, 0x0100), (0, -1, 0xffff)] {
        let mut d = driver();
        d.channels[0].primary.period = period;
        d.channels[0].primary.pitch_step = step;
        d.pitch(Voice::PulseOne);
        assert_eq!(d.channels[0].primary.period, expected);
    }
}
#[test]
fn masked_primary_integrates_hidden_period_without_writing_hardware() {
    let mut d = driver();
    d.secondary_mask = 1;
    d.channels[0].primary.period = 0x0100;
    d.channels[0].primary.pitch_step = -1;
    assert!(d.pitch(Voice::PulseOne).is_empty());
    assert_eq!(d.channels[0].primary.period, 0x00ff);
}
#[test]
fn pitch_oscillator_interval_adds_three_bit_magnitude_and_toggles_direction() {
    let mut d = driver();
    d.channels[0].modulation = Modulation {
        pitch_interval: 2,
        pitch_shape: 0x41,
        pitch_counter: 2,
        ..Modulation::default()
    };
    d.pitch(Voice::PulseOne);
    assert_eq!(d.channels[0].modulation.pitch_offset, 2);
    assert_eq!(d.channels[0].modulation.pitch_interval, 0x82);
    assert_eq!(d.channels[0].modulation.pitch_phase, 0x80);
}
#[test]
fn pitch_oscillator_reverse_span_clears_interval_marker_without_second_direction_toggle() {
    let mut d = driver();
    d.channels[0].modulation = Modulation {
        pitch_interval: 0x82,
        pitch_shape: 0x41,
        pitch_counter: 2,
        pitch_phase: 0x80,
        ..Modulation::default()
    };
    d.pitch(Voice::PulseOne);
    assert_eq!(d.channels[0].modulation.pitch_offset, 0xfffe);
    assert_eq!(d.channels[0].modulation.pitch_interval, 2);
    assert_eq!(d.channels[0].modulation.pitch_phase, 0x80);
}
#[test]
fn pitch_oscillator_nonmatching_counter_keeps_offset_and_phase() {
    let mut d = driver();
    d.channels[0].modulation = Modulation {
        pitch_interval: 2,
        pitch_shape: 0x41,
        pitch_counter: 1,
        pitch_offset: 4,
        pitch_phase: 3,
        ..Modulation::default()
    };
    d.pitch(Voice::PulseOne);
    assert_eq!(d.channels[0].modulation.pitch_offset, 4);
    assert_eq!(d.channels[0].modulation.pitch_phase, 3);
}
#[test]
fn period_write_always_writes_low_but_only_retriggers_changed_high() {
    let mut d = driver();
    d.channels[0].primary.period = 0x0234;
    d.channels[0].modulation.cached_high = 2;
    assert_eq!(d.pitch(Voice::PulseOne), vec![w(PULSE_PERIOD, 0x34)]);
    d.channels[0].modulation.pitch_offset = 0x0100;
    assert_eq!(
        d.pitch(Voice::PulseOne),
        vec![w(PULSE_PERIOD, 0x34), w(PULSE_LENGTH, 0x0b)]
    );
    assert_eq!(d.channels[0].modulation.cached_high, 3);
}
#[test]
fn noise_output_masks_low_nibble_and_takes_mode_from_control_byte() {
    let mut d = driver();
    d.channels[3].primary.period = 0x123b;
    d.channels[3].primary.control = 0xb0;
    d.channels[3].modulation.cached_high = 1;
    assert_eq!(d.pitch(Voice::Noise), vec![w(0x400e, 0x8b), w(0x400f, 8)]);
    assert_eq!(d.channels[3].modulation.cached_high, 0);
}
#[test]
fn modulation_reset_clears_six_counters_seeds_optional_level_and_invalidates_cache() {
    let mut d = driver();
    d.channels[0].modulation = Modulation {
        pitch_interval: 0x82,
        level_interval: 0x83,
        pitch_counter: 9,
        pitch_phase: 0x80,
        pitch_offset: 0x1234,
        cached_high: 2,
        level_counter: 3,
        level: 7,
        ..Modulation::default()
    };
    d.reset_modulation(
        Voice::PulseOne,
        ResetLevel { value: 0xbf },
        Reset::InvalidatePeriod,
    );
    let m = d.channels[0].modulation;
    assert_eq!(
        (
            m.pitch_interval,
            m.pitch_counter,
            m.pitch_phase,
            m.pitch_offset,
            m.cached_high,
            m.level_counter,
            m.level
        ),
        (2, 0, 0, 0, 0xff, 0, 15)
    );
}
#[test]
fn reset_keep_period_preserves_high_cache_and_nonseeded_level() {
    let mut d = driver();
    d.channels[0].modulation.cached_high = 2;
    d.channels[0].modulation.level = 7;
    d.reset_modulation(
        Voice::PulseOne,
        ResetLevel { value: 0xbf },
        Reset::KeepPeriod,
    );
    assert_eq!(d.channels[0].modulation.cached_high, 2);
    assert_eq!(d.channels[0].modulation.level, 7);
}
#[test]
fn preset_offset_is_four_times_index_and_carries_across_page() {
    let mut d = driver();
    d.preset_base = PRESET_ORIGIN;
    d.load_preset(&catalog(&[], &[]), Voice::PulseOne, 1);
    let m = d.channels[0].modulation;
    assert_eq!(
        (
            m.pitch_interval,
            m.pitch_shape,
            m.level_interval,
            m.level_step
        ),
        (5, 6, 7, 8)
    );
}
#[test]
fn actual_rom_loader_resolves_all_headers_and_actual_inline_targets() {
    let rom = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom"))
        .expect("matching local ROM");
    let c = Catalog::from_rom(&rom).unwrap();
    assert_eq!(c.headers.len(), HEADER_COUNT);
    assert_eq!(c.primary_targets(), PRIMARY_HANDLERS);
    assert_eq!(c.secondary_targets(), SECONDARY_HANDLERS);
    assert_eq!(c.duration, [0, 0, 2, 4, 8, 16, 32, 64]);
    assert_eq!(c.alternate_duration, [0, 0, 3, 6, 12, 24, 48, 96]);
    for (command, header) in c.headers.iter().enumerate() {
        match header {
            Header::Primary {
                priority, cursors, ..
            } => {
                assert!(command < 16);
                assert_eq!(*priority, 7);
                assert_ne!(cursors[0], 0);
            }
            Header::Secondary {
                priority,
                mask,
                cursor,
            } => {
                assert!(command >= 16);
                assert_eq!(*priority & 15, 0);
                assert_ne!(*mask, 0);
                assert_ne!(*cursor, 0);
            }
        }
    }
}
#[test]
fn bounded_audio_asset_lookup_does_not_claim_unknown_c522_as_stream_or_preset() {
    let rom = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap();
    let c = Catalog::from_rom(&rom).unwrap();
    assert!(c.read(0x8512).is_err());
    assert!(c.read(0x8513).is_err());
    assert!(c.read(0x8ab0).is_ok());
}
#[test]
fn actual_driver_writes_feed_independent_apu_and_partitioned_samples_match() {
    let mut d = driver();
    d.channels[0].primary.duration = 8;
    d.channels[0].primary.period = 0x0234;
    d.channels[0].modulation.cached_high = 0xff;
    let writes = d.frame(&catalog(&[], &[]), 0);
    assert!(writes.contains(&w(0x4015, 15)));
    assert!(writes.contains(&w(PULSE_LENGTH, 0x0a)));
    let before = d.clone();
    let mut whole = Synthesizer::default();
    whole.apply(&writes);
    let mut split = whole.clone();
    let mut a = vec![0.0; 2048];
    whole.render(&mut a, 48000);
    let mut b = vec![0.0; 2048];
    for chunk in b.chunks_mut(128) {
        split.render(chunk, 48000);
    }
    assert_eq!(a, b);
    assert!(a.iter().any(|sample| *sample != 0.0));
    assert!(a.iter().all(|sample| sample.is_finite()));
    assert_eq!(whole.elapsed_clocks, split.elapsed_clocks);
    assert_eq!(d, before);
}
#[test]
fn audio_sampling_does_not_consume_queue_or_advance_driver_even_across_long_output() {
    let d = driver();
    let before = d.clone();
    let q = CommandQueue {
        slots: [1; QUEUE_SLOTS],
        ..CommandQueue::default()
    };
    let queue_before = q.clone();
    let mut s = Synthesizer::default();
    s.render(&mut [0.0; 2048], 44100);
    assert_eq!(d, before);
    assert_eq!(q, queue_before);
    assert!(s.elapsed_clocks > 0);
}

#[test]
fn first_nmi_command_parameter_is_zero_even_when_display_is_suspended() {
    let mut q = CommandQueue::default();
    q.submit(TEMPO);
    let mut d = Driver::default();
    d.interrupt(&mut q, &catalog(&[], &[]), 0);
    assert_eq!(d.tempo, 1);
    assert_eq!(d.frames, 1);
}
#[test]
fn drained_primary_install_supplies_three_to_following_fade_command() {
    let mut q = CommandQueue::default();
    q.submit(0);
    q.submit(FADE);
    let mut d = Driver::default();
    // A newly installed primary record has a zero table pointer. Original
    // streams select their period table before their first tonal note.
    let mut c = catalog(&[5, 0, NOTE], &[]);
    c.add_fragment(Fragment {
        origin: 0x897b,
        bytes: vec![0, 0, 0x34, 0x02],
    });
    d.interrupt(&mut q, &c, 0);
    assert_eq!(d.fade, 3);
    assert_eq!(d.fade_level, 1);
}
#[test]
fn repeated_tempo_commands_carry_their_own_incremented_parameter_through_drain() {
    let mut q = CommandQueue::default();
    q.submit(TEMPO);
    q.submit(TEMPO);
    q.submit(FADE);
    let mut d = Driver::default();
    d.interrupt(&mut q, &catalog(&[], &[]), 0);
    assert_eq!(d.tempo, 2);
    assert_eq!(d.fade, 2);
}
#[test]
fn reveal_final_audio_submission_maps_both_comparison_outcomes_to_typed_actor_input() {
    use crate::progression::FirstActorIncrement::{One, Zero};
    for (occupied, expected) in [(EMPTY_COMMAND, One), (0x15, Zero), (STOP_MUSIC, One)] {
        let mut q = CommandQueue::default();
        q.slots[0] = occupied;
        let result = q.submit(0x15);
        assert_eq!(result.first_actor_increment(), expected);
    }
}
#[test]
fn secondary_end_is_reread_by_every_remaining_selected_channel_in_same_frame() {
    let mut d = Driver {
        secondary_mask: 0x0f,
        secondary_cursor: SECONDARY_ORIGIN,
        priority: 0x50,
        ..Driver::default()
    };
    d.frame(&catalog(&[], &[STOP_SECONDARY]), 0);
    assert_eq!(d.secondary_mask, 0);
    assert_eq!(d.priority, 0);
    assert_eq!(d.secondary_cursor, SECONDARY_ORIGIN);
}
#[test]
fn shared_secondary_delay_is_decremented_only_after_all_selected_channels_output() {
    let mut d = Driver {
        secondary_mask: 0x0f,
        secondary_cursor: SECONDARY_ORIGIN,
        shared_delay: 1,
        ..Driver::default()
    };
    for channel in &mut d.channels {
        channel.secondary.period = 0x0234;
        channel.secondary.control = 0x3f;
    }
    let writes = d.frame(&catalog(&[], &[STOP_SECONDARY]), 0);
    assert_eq!(d.shared_delay, 0);
    assert_eq!(d.secondary_mask, 0x0f);
    assert_eq!(d.secondary_cursor, SECONDARY_ORIGIN);
    for address in [0x4000, 0x4004, 0x4008, 0x400c] {
        assert!(
            writes
                .iter()
                .any(|write| write.address == address && write.value != 0x30)
        );
    }
}
#[test]
fn fe_rotated_mask_does_not_hide_restored_primary_in_next_frame() {
    let mut d = driver();
    d.secondary_mask = 1;
    d.preset_base = PRESET_ORIGIN;
    d.channels[0].primary.period = 0x0234;
    d.channels[0].primary.duration = 8;
    let c = catalog(&[], &[]);
    d.command(&c, STOP_EFFECT, 0, &mut vec![]);
    assert_eq!(d.secondary_mask, 0x10);
    let writes = d.frame(&c, 0);
    assert_eq!(d.secondary_mask, 0);
    assert!(writes.iter().any(|write| write.address == PULSE_PERIOD));
}
#[test]
fn nondefault_derived_reset_seed_is_used_instead_of_inventing_secondary_control() {
    for (voice, expected) in [(Voice::PulseOne, 11), (Voice::Triangle, 0xab)] {
        let mut d = driver();
        let slot = if voice == Voice::Triangle { 2 } else { 0 };
        d.channels[slot].modulation.level_interval = 0x80;
        d.channels[slot].secondary.control = 0x32;
        d.reset_modulation(voice, ResetLevel { value: 0xab }, Reset::KeepPeriod);
        assert_eq!(d.channels[slot].modulation.level, expected);
    }
}
#[test]
fn zero_span_pitch_oscillator_waits_for_low_seven_phase_wrap() {
    let mut d = driver();
    d.channels[0].modulation = Modulation {
        pitch_interval: 1,
        pitch_shape: 0x40,
        pitch_counter: 1,
        pitch_phase: 0x7f,
        ..Modulation::default()
    };
    d.pitch(Voice::PulseOne);
    assert_eq!(d.channels[0].modulation.pitch_phase, 0);
    assert_eq!(d.channels[0].modulation.pitch_interval, 0x81);
}
#[test]
fn all_fifty_six_headers_decode_exact_cursors_preset_masks_and_priorities_from_rom() {
    let rom = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap();
    let c = Catalog::from_rom(&rom).unwrap();
    let read_word = |offset: usize| u16::from_le_bytes([rom[offset], rom[offset + 1]]);
    for command in 0..HEADER_COUNT {
        let encoded = read_word(HEADER_WORDS_PHYSICAL + command * 2);
        let physical = BANK_PHYSICAL_START + usize::from(encoded - BANK_CPU_START);
        let priority = rom[physical];
        if priority & 15 != 0 {
            let cursors = std::array::from_fn(|channel| read_word(physical + 1 + channel * 2));
            assert_eq!(
                c.headers[command],
                Header::Primary {
                    priority,
                    cursors,
                    preset_base: read_word(physical + 9)
                }
            );
        } else {
            assert_eq!(
                c.headers[command],
                Header::Secondary {
                    priority,
                    mask: rom[physical + 1] & 15,
                    cursor: encoded + 2
                }
            );
        }
    }
}
#[test]
fn canonical_audio_loader_honors_named_file_override_and_preserves_mixed_graphics_bytes() {
    use crate::{assets::Assets, config};
    use romatlas_port::assets::{MemoryAssets, Origin, Rom};
    let rom = Rom::from_bytes(
        std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap(),
        config::ROM_SHA256,
    )
    .unwrap();
    let mut assets = Assets::load(&rom, &MemoryAssets::default()).unwrap();
    let duration = assets
        .source
        .assets
        .iter_mut()
        .find(|a| a.id == "AudioDurationAndPeriodData_0C97BBytes")
        .unwrap();
    duration.bytes[2] = 5;
    duration.origin = Origin::File("synthetic-duration.bin".into());
    let mixed = assets
        .source
        .assets
        .iter_mut()
        .find(|a| a.id == "AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes")
        .unwrap();
    let graphics_offset = 0xe910 - MIXED_PAYLOAD_PHYSICAL.start;
    mixed.bytes[graphics_offset] = 0x5a;
    let before = mixed.bytes.clone();
    let c = Catalog::from_assets(&assets).unwrap();
    assert_eq!(c.duration[2], 5);
    assert_eq!(
        assets
            .source
            .get("AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes")
            .unwrap()
            .bytes,
        before
    );
    assert!(c.read(0x8512).is_err());
}
#[test]
fn named_audio_loader_rejects_missing_format_record_without_rom_fallback() {
    use crate::{assets::Assets, config};
    use romatlas_port::assets::{MemoryAssets, Rom};
    let rom = Rom::from_bytes(
        std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap(),
        config::ROM_SHA256,
    )
    .unwrap();
    let mut assets = Assets::load(&rom, &MemoryAssets::default()).unwrap();
    assets
        .source
        .assets
        .retain(|a| a.id != "AudioCommandHeaderPointers_0CA50Bytes");
    assert!(Catalog::from_assets(&assets).is_err());
}
#[test]
fn actual_scheduled_audio_preserves_fractional_phase_at_irregular_partition_and_rate_change() {
    let mut d = driver();
    d.channels[0].primary.duration = 8;
    d.channels[0].primary.period = 0x0234;
    d.channels[0].modulation.cached_high = 0xff;
    let writes = d.frame(&catalog(&[], &[]), 0);
    let mut whole = Synthesizer::default();
    whole.apply(&writes);
    let mut split = whole.clone();
    for rate in [44100, 48000, 44100] {
        let mut a = vec![0.0; 2066];
        let mut b = vec![0.0; 2066];
        whole.render(&mut a, rate);
        let mut at = 0;
        for length in [2, 6, 18, 34, 514, 1492] {
            split.render(&mut b[at..at + length], rate);
            at += length;
        }
        assert_eq!(a, b);
        assert!(a.iter().any(|sample| *sample != 0.0));
        assert_eq!(whole.elapsed_clocks, split.elapsed_clocks);
    }
    assert_eq!(d.frames, 1);
}

#[test]
fn negative_fade_initial_zero_level_wraps_before_its_zero_stop_test() {
    //8337 DEC happens before BEQ. Zero is not an entry guard: five decrements
    //produce FB, whose bits survive OR with ordinary control upper nibble30.
    let mut d = driver();
    d.fade = 0x82;
    d.fade_level = 5;
    d.channels[0].primary.control = 0x30;
    assert_eq!(d.level(Voice::PulseOne), 0xfb);
}
#[test]
fn streamed_preset_uses_full_index_byte_while_restore_uses_low_five_bits() {
    let mut d = driver();
    d.preset_base = PRESET_ORIGIN;
    let mut c = catalog(&[8, 0xe1, NOTE], &[]);
    //225*4 wraps to132, not4. The unmasked command's X retains raw E1.
    c.add_fragment(Fragment {
        origin: PRESET_ORIGIN + 132,
        bytes: vec![9, 10, 11, 12],
    });
    d.primary_step(&c, Voice::PulseOne);
    assert_eq!(d.channels[0].primary.note_control, 0xc1);
    assert_eq!(d.channels[0].modulation.pitch_interval, 9);
    assert_eq!(d.channels[0].modulation.level_step, 12);
}
#[test]
fn fe_empty_primary_period_silences_then_restores_preset_only_for_nonnull_cursor() {
    for cursor in [0, PRIMARY_ORIGIN] {
        let mut d = driver();
        d.secondary_mask = 1;
        d.preset_base = PRESET_ORIGIN;
        d.channels[0].primary.cursor = cursor;
        let mut writes = vec![];
        d.command(&catalog(&[], &[]), STOP_EFFECT, 0, &mut writes);
        assert_eq!(writes, vec![w(PULSE_CONTROL, 0x30)]);
        assert_eq!(d.channels[0].modulation.cached_high, 0);
        assert_eq!(
            d.channels[0].modulation.pitch_interval,
            if cursor == 0 { 0 } else { 1 }
        );
    }
}
#[test]
fn secondary_install_parameter_depends_on_selected_channel_path_before_following_fade() {
    for (mask, expected_parameter) in [(0, 1), (0x0a, 0)] {
        let mut c = catalog(&[], &[STOP_SECONDARY]);
        c.headers[1] = Header::Secondary {
            priority: 0x50,
            mask,
            cursor: SECONDARY_ORIGIN,
        };
        let mut q = CommandQueue::default();
        q.submit(1);
        q.submit(FADE);
        let mut d = Driver::default();
        d.interrupt(&mut q, &c, 0);
        assert_eq!(d.fade, expected_parameter);
    }
}

#[test]
fn stopped_secondary_high_nibble_is_discarded_at_frame_tail() {
    let mut d = Driver {
        secondary_mask: 1,
        ..Driver::default()
    };
    d.command(&catalog(&[], &[]), STOP_EFFECT, 0, &mut vec![]);
    assert_eq!(d.secondary_mask, 0x10);
    d.frame(&catalog(&[], &[]), 0);
    assert_eq!(d.secondary_mask, 0);
}
#[test]
fn low_f_reset_consumes_previous_dispatch_selector_from_adjacent_channel() {
    let mut d = driver();
    d.channels[0].modulation.level_interval = 0x80;
    // Secondary command1's inline target85D9 leaves D9 as the indexed selector.
    // 0370+D9=0449 belongs to the canonical secondary display queue.
    let mut observed = vec![];
    d.secondary_step_with_alias(
        &catalog(&[], &[1, 0, 0x8f, 0]),
        Voice::PulseOne,
        &mut |address| {
            observed.push(address);
            0xbc
        },
    );
    assert_eq!(observed, vec![0x449]);
    assert_eq!(d.channels[0].modulation.level, 12);
    assert_eq!(
        d.reset_selector,
        SeedSelector {
            selector: 0xd9,
            producer: SeedProducer::InlineTarget
        }
    );
}
#[test]
fn low_f_reset_reads_adjacent_canonical_channel_control_without_external_projection() {
    let mut d = driver();
    d.channels[0].modulation.level_interval = 0x80;
    // Control command2 parameter43 decimal selects channel1's primary control:
    // one complete31-byte record plus offset12.
    d.channels[1].primary.control = 0xb7;
    d.secondary_step(&catalog(&[], &[2, 43, 0x8f, 0]), Voice::PulseOne);
    assert_eq!(d.channels[0].modulation.level, 7);
}
#[test]
fn cold_reset_zero_selector_and_no_command_frames_preserve_the_actual_producer() {
    let mut d = Driver::default();
    assert_eq!(
        d.reset_selector,
        SeedSelector {
            selector: 0,
            producer: SeedProducer::ResetZero
        }
    );
    d.secondary_mask = 1;
    d.secondary_cursor = SECONDARY_ORIGIN;
    d.channels[0].modulation.level_interval = 0x80;
    d.channels[1].primary.control = 0xb7;
    let c = catalog(&[], &[2, 43, 0x8f, 0, 0x8f, 0]);
    let mut queue = CommandQueue::default();
    d.interrupt(&mut queue, &c, 1);
    assert_eq!(d.channels[0].modulation.level, 7);
    assert_eq!(
        d.reset_selector,
        SeedSelector {
            selector: 43,
            producer: SeedProducer::ControlParameter
        }
    );
    d.channels[0].modulation.level_interval |= 0x80;
    d.channels[1].primary.control = 0xb9;
    d.interrupt(&mut queue, &c, 1);
    assert_eq!(d.channels[0].modulation.level, 9);
    assert_eq!(d.reset_selector.producer, SeedProducer::ControlParameter);
}
#[test]
fn command_preset_restore_preserves_reset_producer_and_returns_indexed_copy_parameter() {
    for index in [0, 1] {
        let mut c = catalog(&[], &[STOP_SECONDARY]);
        c.headers[1] = Header::Secondary {
            priority: 0x50,
            mask: 0,
            cursor: SECONDARY_ORIGIN,
        };
        let mut d = Driver {
            secondary_mask: 1,
            preset_base: PRESET_ORIGIN,
            ..Driver::default()
        };
        d.channels[0].primary.period = 0x234;
        d.channels[0].primary.note_control = index;
        d.reset_selector = SeedSelector {
            selector: 43,
            producer: SeedProducer::ControlParameter,
        };
        let mut queue = CommandQueue::default();
        queue.submit(1);
        queue.submit(FADE);
        d.interrupt(&mut queue, &c, 1);
        assert_eq!(d.fade, index * 4 + 4);
        assert_eq!(
            d.reset_selector,
            SeedSelector {
                selector: 43,
                producer: SeedProducer::ControlParameter
            }
        );
    }
}
#[test]
fn zero_fade_counter_preserves_positive_level_and_runs_negative_byte_loop_to_zero() {
    for level in [0, 5] {
        let mut d = driver();
        d.channels[0].primary.control = 0x30 | level;
        d.fade = 2;
        d.fade_level = 0;
        assert_eq!(d.level(Voice::PulseOne), 0x30 | level);
        d.fade = 0x82;
        assert_eq!(d.level(Voice::PulseOne), 0x30);
    }
}

#[test]
fn semantic_journal_records_real_submit_attempts_without_changing_ring_equality() {
    let events = crate::events::Events::new(4);
    let mut q = CommandQueue::default();
    q.observe_with(events.clone());
    q.slots[1] = 0x11;
    q.submit(0x13);
    q.submit(0x18);
    let before = q.clone();
    q.submit_stage(0x2d);
    assert_eq!(q.slots, before.slots);
    assert_eq!(q.producer, before.producer);
    assert_eq!(q.stage_command, 0x2d);
    let snapshot = events.observe();
    let entries = snapshot["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0]["payload"]["producer"], 0);
    assert_eq!(entries[0]["payload"]["accepted"], true);
    assert_eq!(entries[1]["payload"]["producer"], 1);
    assert_eq!(entries[1]["payload"]["accepted"], false);
    assert_eq!(entries[1]["payload"]["comparison"], "BelowEmpty");
    assert_eq!(entries[2]["payload"]["stage_command"], true);
    assert_eq!(events.observe(), snapshot);
    let before_drop = q.clone();
    q.submit(0x18);
    assert_eq!(
        q, before_drop,
        "observer metadata is not canonical source ring state"
    );
}

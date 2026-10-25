//! B03 independent synthetic fixtures. Each expected result follows trace.md;
//! no implementation output is used to manufacture expected values.
use super::*;
fn actor() -> ActorVisual {
    ActorVisual {
        state: ACTIVE,
        render: VISIBLE,
        world_x: 100,
        screen_x: 100,
        screen_y: 100,
        spawn_identity: Some(7),
        ..Default::default()
    }
}
fn sequence() -> Animation {
    Animation {
        last_frame: 1,
        last_tick: 2,
        frames: vec![9, 12],
    }
}
fn gate() -> OverlayGate {
    OverlayGate {
        gameplay_latch: ACTIVE,
        player_y: OVERLAY_PLAYER_MIN_Y,
        ..Default::default()
    }
}
#[test]
fn player_transient_flags_preserve_other_bits_and_combine_both_protection_sources() {
    for (protection, special, expected) in [(0, 0, 0xfc), (1, 0, 0xfe), (0, 1, 0xfe), (1, 1, 0xfe)]
    {
        let mut a = actor();
        a.render = 0xff;
        Composition::player_flags(&mut a, protection, special);
        assert_eq!(a.render, expected);
    }
}
#[test]
fn overlays_require_every_gate_and_copy_three_byte_sentinel_lists_in_order() {
    let overlays: [&[u8]; 3] = [
        &[1, 2, 3, 4, 0xff],
        &[5, 6, 7, 8, 0xff],
        &[9, 10, 11, 12, 0xff],
    ];
    let mut c = Composition::default();
    c.begin(gate(), overlays);
    assert_eq!(c.cursor, 16);
    assert_eq!(
        c.oam[1],
        Sprite {
            y: 1,
            tile: 2,
            attributes: 3,
            x: 4
        }
    );
    assert_eq!(
        c.oam[3],
        Sprite {
            y: 9,
            tile: 10,
            attributes: 11,
            x: 12
        }
    );
    for blocked in [
        OverlayGate {
            gameplay_latch: 0,
            ..gate()
        },
        OverlayGate {
            dialogue_cursor: 1,
            ..gate()
        },
        OverlayGate {
            ending: 1,
            ..gate()
        },
        OverlayGate {
            status_stream: 1,
            ..gate()
        },
        OverlayGate {
            player_state: DIALOGUE_PLAYER_STATE,
            ..gate()
        },
        OverlayGate {
            player_y: OVERLAY_PLAYER_MIN_Y - 1,
            ..gate()
        },
    ] {
        let mut c = Composition::default();
        c.begin(blocked, overlays);
        assert_eq!(c.cursor, 4);
    }
}
#[test]
fn composition_clock_increments_before_alternating_all_sixteen_slots_and_wraps() {
    let mut c = Composition::default();
    assert_eq!(
        c.begin(OverlayGate::default(), [&[0xff]; 3]),
        (0..ACTOR_SLOTS).rev().collect::<Vec<_>>()
    );
    assert_eq!(
        c.begin(OverlayGate::default(), [&[0xff]; 3]),
        (0..ACTOR_SLOTS).collect::<Vec<_>>()
    );
    c.clock = 0xff;
    assert_eq!(
        c.begin(OverlayGate::default(), [&[0xff]; 3]),
        (0..ACTOR_SLOTS).collect::<Vec<_>>()
    );
    assert_eq!(c.clock, 0);
}
#[test]
fn screen_positioned_actors_copy_low_world_x_without_camera_projection() {
    let mut c = Composition::default();
    let mut a = actor();
    a.world_x = 0x1234;
    a.render = 0;
    assert!(c.project(&mut a, 0xffff));
    assert_eq!((a.screen_x, a.render), (0x34, VISIBLE));
}
#[test]
fn world_projection_borrows_low_byte_and_retains_on_page_visibility() {
    for (world, camera, expected) in [
        (0x110, 0xf0, 0x20),
        (0x1ef, 0xf0, 0xff),
        (0xffff, 0xff00, 0xff),
        (0, 0xffff, 1),
    ] {
        let mut c = Composition::default();
        let mut a = actor();
        a.state |= WORLD_POSITIONED;
        a.world_x = world;
        assert!(c.project(&mut a, camera));
        assert_eq!(a.screen_x, expected);
        assert_eq!(a.render & VISIBLE, VISIBLE);
    }
}
#[test]
fn offpage_margin_uses_low_byte_absolute_distance_not_sixteen_bit_distance() {
    for (world, expected_active) in [
        (0xffff, true),
        (0xffc1, true),
        (0xffc0, false),
        (0x13f, true),
        (0x140, false),
        (0x100, true),
    ] {
        let mut a = actor();
        a.state |= WORLD_POSITIONED;
        a.world_x = world;
        let mut c = Composition::default();
        assert!(!c.project(&mut a, 0));
        assert_eq!(a.state != 0, expected_active);
        assert_eq!(a.render & VISIBLE, 0);
        if expected_active {
            assert_eq!(a.screen_x, 0);
            assert_eq!(a.spawn_identity, Some(7));
        } else {
            assert_eq!(a.spawn_identity, None);
        }
    }
}
#[test]
fn persistent_animation_keeps_distant_actor_without_composition() {
    let mut a = actor();
    a.state |= WORLD_POSITIONED;
    a.world_x = 0x180;
    a.animation = OFFPAGE_PERSISTENT_ANIMATION;
    let mut c = Composition::default();
    assert!(!c.project(&mut a, 0));
    assert_ne!(a.state, 0);
    assert_eq!(a.screen_x, 0);
}
#[test]
fn world_projection_only_state_skips_animation_but_screen_positioned_state_does_not() {
    let mut c = Composition::default();
    let mut a = actor();
    a.state |= WORLD_POSITIONED | PROJECTION_ONLY;
    assert!(!c.project(&mut a, 0));
    assert_eq!(a.render & VISIBLE, VISIBLE);
    a.state = ACTIVE | PROJECTION_ONLY;
    assert!(c.project(&mut a, 0));
}
#[test]
fn protection_flash_uses_composition_bit_one_and_suppresses_animation() {
    for (clock, expected) in [
        (0, false),
        (1, false),
        (2, true),
        (3, true),
        (4, false),
        (0xff, true),
    ] {
        let mut c = Composition {
            clock,
            ..Default::default()
        };
        let mut a = actor();
        a.render |= FLASH;
        assert_eq!(c.project(&mut a, 0), expected);
        assert_eq!(a.playback.tick_code(), 0);
    }
}
#[test]
fn animation_compares_old_tick_and_uses_duration_plus_one() {
    let mut a = actor();
    let s = sequence();
    assert_eq!(Composition::advance(&mut a, &s, 0), Some(9));
    assert_eq!((a.playback.tick_code(), a.playback.frame_code()), (1, 0));
    assert_eq!(Composition::advance(&mut a, &s, 0), Some(9));
    assert_eq!((a.playback.tick_code(), a.playback.frame_code()), (2, 0));
    assert_eq!(Composition::advance(&mut a, &s, 0), Some(12));
    assert_eq!((a.playback.tick_code(), a.playback.frame_code()), (0, 1));
}
#[test]
fn zero_tick_duration_advances_on_first_composition_and_loops_last_frame() {
    let mut a = actor();
    let s = Animation {
        last_tick: 0,
        ..sequence()
    };
    assert_eq!(Composition::advance(&mut a, &s, 0), Some(12));
    assert_eq!(Composition::advance(&mut a, &s, 0), Some(9));
    assert_eq!(a.playback.frame_code(), 0);
}
#[test]
fn dialogue_and_negative_animation_tick_freeze_but_still_select_frame() {
    for (tick, player) in [(0, DIALOGUE_PLAYER_STATE), (0x80, 0), (0xff, 0)] {
        let mut a = actor();
        {
            let animation_value = tick;
            a.playback.set_tick_code(animation_value);
        };
        {
            let animation_value = 1;
            a.playback.set_frame_code(animation_value);
        };
        assert_eq!(Composition::advance(&mut a, &sequence(), player), Some(12));
        assert_eq!((a.playback.tick_code(), a.playback.frame_code()), (tick, 1));
    }
}
#[test]
fn animation_destroy_clears_active_render_and_spawn_at_sequence_end() {
    let mut a = actor();
    a.render |= DESTROY_AT_END;
    {
        let animation_value = 2;
        a.playback.set_tick_code(animation_value);
    };
    {
        let animation_value = 1;
        a.playback.set_frame_code(animation_value);
    };
    assert_eq!(Composition::advance(&mut a, &sequence(), 0), None);
    assert_eq!((a.state, a.render, a.spawn_identity), (0, 0, None));
}
#[test]
fn negative_frame_restarts_only_when_post_increment_remains_negative() {
    for (frame, last, restart, expected) in [(0x81, 1, 0, 0x80), (0xff, 0x7f, 1, 0), (1, 1, 1, 0)] {
        let mut a = actor();
        {
            let animation_value = 2;
            a.playback.set_tick_code(animation_value);
        };
        {
            let animation_value = frame;
            a.playback.set_frame_code(animation_value);
        };
        {
            let animation_value = restart;
            a.playback.set_restart_code(animation_value);
        };
        let s = Animation {
            last_frame: last,
            frames: vec![9; 128],
            ..sequence()
        };
        Composition::advance(&mut a, &s, 0);
        assert_eq!(a.playback.frame_code(), expected);
    }
}
#[test]
fn animation_tick_overflow_freezes_unless_old_tick_matches_duration() {
    // AdvanceObjectAnimation increments before comparing the OLD tick. A
    // nonmatching 127 crosses the hold bit; the matching duration resets it.
    for (last_tick, expected_tick, expected_frame) in [(126, 128, 0), (127, 0, 1)] {
        let mut a = actor();
        {
            let animation_value = 127;
            a.playback.set_tick_code(animation_value);
        };
        let s = Animation {
            last_tick,
            ..sequence()
        };
        assert_eq!(
            Composition::advance(&mut a, &s, 0),
            Some(if expected_frame == 0 { 9 } else { 12 })
        );
        assert_eq!(
            (a.playback.tick_code(), a.playback.frame_code()),
            (expected_tick, expected_frame)
        );
        if expected_tick == 128 {
            Composition::advance(&mut a, &s, 0);
            assert_eq!((a.playback.tick_code(), a.playback.frame_code()), (128, 0));
        }
    }
}
#[test]
fn animation_frame_overflow_selects_loop_origin_after_increment() {
    // Source817F tests the post-increment sign, including 127->128 and255->0.
    for (frame, restart, expected) in [(127, 5, 133), (127, 133, 133), (255, 5, 0)] {
        let mut a = actor();
        {
            let animation_value = frame;
            a.playback.set_frame_code(animation_value);
        };
        {
            let animation_value = restart;
            a.playback.set_restart_code(animation_value);
        };
        let s = Animation {
            last_tick: 0,
            last_frame: 127,
            frames: (0..128).collect(),
        };
        assert_eq!(Composition::advance(&mut a, &s, 0), Some(expected & 127));
        assert_eq!(
            (
                a.playback.tick_code(),
                a.playback.frame_code(),
                a.playback.restart_code()
            ),
            (0, expected, restart)
        );
    }
}
#[test]
fn animation_destruction_retains_incremented_playback_before_culling() {
    for frame in [1, 127, 129, 255] {
        let mut a = actor();
        a.render |= DESTROY_AT_END;
        {
            let animation_value = 127;
            a.playback.set_tick_code(animation_value);
        };
        {
            let animation_value = frame;
            a.playback.set_frame_code(animation_value);
        };
        {
            let animation_value = 219;
            a.playback.set_restart_code(animation_value);
        };
        let s = Animation {
            last_tick: 127,
            last_frame: frame & 127,
            frames: vec![9; 128],
        };
        assert_eq!(Composition::advance(&mut a, &s, 0), None);
        assert_eq!(
            (
                a.playback.tick_code(),
                a.playback.frame_code(),
                a.playback.restart_code()
            ),
            (0, frame.wrapping_add(1), 219)
        );
        assert_eq!((a.state, a.render, a.spawn_identity), (0, 0, None));
    }
}
#[test]
fn native_playback_preserves_all_codes_and_independent_partial_writes() {
    for tick in 0..=255 {
        for frame in 0..=255 {
            let restart = tick ^ frame;
            let mut playback = AnimationPlayback::from_codes(tick, frame, restart);
            assert_eq!(
                (
                    playback.tick_code(),
                    playback.frame_code(),
                    playback.restart_code()
                ),
                (tick, frame, restart)
            );
            let original = playback;
            playback.set_tick_code(frame);
            assert_eq!(
                (
                    playback.tick_code(),
                    playback.frame_code(),
                    playback.restart_code()
                ),
                (frame, frame, restart)
            );
            playback.set_frame_code(tick);
            playback.set_restart_code(frame);
            assert_eq!(
                (
                    playback.tick_code(),
                    playback.frame_code(),
                    playback.restart_code()
                ),
                (frame, tick, frame)
            );
            assert_eq!(
                (
                    original.tick_code(),
                    original.frame_code(),
                    original.restart_code()
                ),
                (tick, frame, restart)
            );
        }
    }
}
#[test]
fn native_playback_event_gates_distinguish_looping_frames_and_frozen_ticks() {
    for index in 0..128 {
        let mut playback = AnimationPlayback::from_codes(index, index, 0);
        assert!(playback.at_initial_frame(index));
        assert!(playback.at_running_tick(index));
        playback.freeze();
        assert!(!playback.at_running_tick(index));
        assert_eq!(playback.tick_code(), index | 128);
        playback.resume();
        assert!(playback.at_running_tick(index));
        playback.start_looping_at(index);
        assert!(!playback.at_initial_frame(index));
        assert_eq!(playback.frame_index(), usize::from(index));
    }
}
fn piece(dy: i8, dx: i8) -> Composite {
    Composite {
        pieces: vec![Piece {
            tile: 13,
            attributes: 3,
            dy,
            dx,
        }],
    }
}
#[test]
fn accepted_piece_emits_palette_flip_priority_and_advances_one_oam_entry() {
    let mut c = Composition::default();
    let mut a = actor();
    a.render |= FLIP_HORIZONTAL | PRIORITY;
    c.emit(1, &a, &piece(-2, 3));
    assert_eq!(
        c.oam[1],
        Sprite {
            y: 98,
            tile: 13,
            attributes: 0x63,
            x: 103
        }
    );
    assert_eq!(c.cursor, 8);
}
#[test]
fn signed_offset_wrap_rejection_uses_adc_carry_for_both_axes() {
    for (x, y, dy, dx, accepted) in [
        (0, 0, -1, 0, false),
        (0, 0, 0, -1, false),
        (255, 255, 1, 0, false),
        (255, 255, 0, 1, false),
        (1, 1, -1, -1, true),
        (254, 254, 1, 1, true),
    ] {
        let mut a = actor();
        a.screen_x = x;
        a.screen_y = y;
        let mut c = Composition::default();
        c.emit(0, &a, &piece(dy, dx));
        assert_eq!(c.cursor, if accepted { 8 } else { 4 });
        if !accepted {
            assert_eq!(c.oam[1].y, crate::runtime::HIDDEN_SPRITE_Y);
        }
    }
}
#[test]
fn status_cutoff_exempts_player_and_explicit_above_status_flag() {
    for (slot, render, y, accepted) in [
        (1, VISIBLE, STATUS_TOP - 1, false),
        (1, VISIBLE, STATUS_TOP, true),
        (0, VISIBLE, 0, true),
        (1, VISIBLE | ABOVE_STATUS, 0, true),
    ] {
        let mut a = actor();
        a.screen_y = y;
        a.render = render;
        let mut c = Composition::default();
        c.emit(slot, &a, &piece(0, 0));
        assert_eq!(c.cursor, if accepted { 8 } else { 4 });
    }
}
#[test]
fn rejected_pieces_reuse_slot_and_keep_stream_alignment() {
    let mut a = actor();
    a.screen_x = 0;
    let mut c = Composition::default();
    let f = Composite {
        pieces: vec![
            Piece {
                tile: 1,
                attributes: 2,
                dy: 0,
                dx: -1,
            },
            Piece {
                tile: 7,
                attributes: 1,
                dy: 1,
                dx: 1,
            },
        ],
    };
    c.emit(1, &a, &f);
    assert_eq!(c.cursor, 8);
    assert_eq!(
        c.oam[1],
        Sprite {
            y: 101,
            tile: 7,
            attributes: 1,
            x: 1
        }
    );
}
#[test]
fn oam_capacity_wrap_stops_and_zero_cursor_never_emits() {
    let mut c = Composition {
        cursor: 252,
        ..Default::default()
    };
    c.emit(
        0,
        &actor(),
        &Composite {
            pieces: vec![piece(0, 0).pieces[0]; 2],
        },
    );
    assert_eq!(c.cursor, 0);
    assert_eq!(c.oam[63].tile, 13);
    let before = c.clone();
    c.emit(0, &actor(), &piece(0, 0));
    assert_eq!(c, before);
}
#[test]
fn hide_preserves_split_sprite_only_during_gameplay_and_only_changes_y() {
    for gameplay in [false, true] {
        let mut c = Composition {
            oam: [Sprite {
                y: 7,
                tile: 8,
                attributes: 9,
                x: 10,
            }; OAM_ENTRIES],
            ..Default::default()
        };
        c.hide(gameplay);
        assert_eq!(
            c.oam[0].y,
            if gameplay {
                7
            } else {
                crate::runtime::HIDDEN_SPRITE_Y
            }
        );
        assert_eq!(
            c.oam[63],
            Sprite {
                y: crate::runtime::HIDDEN_SPRITE_Y,
                tile: 8,
                attributes: 9,
                x: 10
            }
        );
    }
}
#[test]
fn snapshot_does_not_advance_clocks_animation_or_drain_sprites() {
    let c = Composition::default();
    let before = c.clone();
    assert_eq!(c.snapshot(), c.snapshot());
    assert_eq!(c, before);
}
#[test]
fn queue_records_count_minus_one_and_negative_destination_end_for_both_queues() {
    for queue in [Queue::Primary, Queue::Secondary] {
        let mut d = Display {
            primary: PreparedPrimary {
                pending: 0xff,
                ..PreparedPrimary::default()
            },
            secondary_pending: true,
            ..Default::default()
        };
        d.flush(
            queue,
            &[0x20, 0x10, 0, 7, 0x24, 0x30, 1, 8, 9, 0x80],
            Stride::Across,
        );
        assert_eq!(
            d.writes,
            vec![
                VideoWrite {
                    address: 0x2010,
                    value: 7
                },
                VideoWrite {
                    address: 0x2430,
                    value: 8
                },
                VideoWrite {
                    address: 0x2431,
                    value: 9
                }
            ]
        );
        assert_eq!(
            ((d.primary.pending != 0), d.secondary_pending),
            if queue == Queue::Primary {
                (false, true)
            } else {
                (true, false)
            }
        );
    }
}
#[test]
fn queue_negative_count_still_copies_first_byte_and_down_stride_is_retained() {
    let mut d = Display::default();
    d.flush(Queue::Primary, &[0x20, 0, 0xff, 6, 0x80], Stride::Down);
    assert_eq!(
        d.writes,
        vec![VideoWrite {
            address: 0x2000,
            value: 6
        }]
    );
    let mut d = Display::default();
    d.flush(Queue::Secondary, &[0x20, 0xf0, 1, 6, 7, 0xff], Stride::Down);
    assert_eq!(
        d.writes,
        vec![
            VideoWrite {
                address: 0x20f0,
                value: 6
            },
            VideoWrite {
                address: 0x2110,
                value: 7
            }
        ]
    );
}
#[test]
fn empty_queue_negative_high_reads_no_destination_or_count() {
    let mut d = Display {
        primary: PreparedPrimary {
            pending: 0xff,
            ..PreparedPrimary::default()
        },
        ..Default::default()
    };
    d.flush(Queue::Primary, &[0xff], Stride::Across);
    assert!(d.writes.is_empty());
    assert!((d.primary.pending == 0));
}
#[test]
fn screen_scripts_fe_ends_payload_zero_ends_script_zero_payload_is_data() {
    let mut d = Display::default();
    d.script(
        0,
        &[0x20, 0, 0, 7, 0xfe, 0x24, 1, 8, 0xfe, 0],
        &[0; 8],
        Stride::Across,
    );
    assert_eq!(
        d.writes,
        vec![
            VideoWrite {
                address: 0x2000,
                value: 0
            },
            VideoWrite {
                address: 0x2001,
                value: 7
            },
            VideoWrite {
                address: 0x2401,
                value: 8
            }
        ]
    );
}
#[test]
fn screen_selector_eleven_appends_exactly_eight_status_bytes_at_current_address() {
    let mut d = Display::default();
    d.script(
        11,
        &[0x20, 0, 6, 0xfe, 0],
        &[1, 2, 3, 4, 5, 6, 7, 8],
        Stride::Across,
    );
    assert_eq!(d.writes.len(), 9);
    assert_eq!(
        d.writes[1],
        VideoWrite {
            address: 0x2001,
            value: 1
        }
    );
    assert_eq!(
        d.writes[8],
        VideoWrite {
            address: 0x2008,
            value: 8
        }
    );
}
#[test]
fn long_screen_script_carries_pointer_page_without_terminating_on_zero_y() {
    let mut stream = vec![0x20, 0];
    stream.extend([7; 260]);
    stream.extend([0xfe, 0]);
    let mut d = Display::default();
    d.script(0, &stream, &[0; 8], Stride::Across);
    assert_eq!(d.writes.len(), 260);
    assert_eq!(
        d.writes[259],
        VideoWrite {
            address: 0x2103,
            value: 7
        }
    );
}
#[test]
fn suspend_depth_wraps_and_enable_unconditionally_installs_mask() {
    let mut d = Display::default();
    d.enable();
    assert_eq!((d.suspended, d.mask), (0xff, 0x1e));
    d.disable();
    assert_eq!((d.suspended, d.mask), (0, 0));
    d.disable();
    d.disable();
    d.enable();
    assert_eq!((d.suspended, d.mask), (1, 0x1e));
}
#[test]
fn nmi_control_helpers_mask_exact_bits_then_restore_nmi_only() {
    let mut d = Display {
        control: 0xff,
        ..Default::default()
    };
    d.disable_nmi();
    assert_eq!(d.control, 0x11);
    d.enable_nmi();
    assert_eq!(d.control, 0x91);
}
#[test]
fn nametable_fill_writes_four_pages_then_overlays_sixtyfour_attributes() {
    let mut d = Display::default();
    d.fill(0x20, 0x24, 3, Stride::Across);
    assert_eq!(d.writes.len(), 1088);
    assert_eq!(
        d.writes[1023],
        VideoWrite {
            address: 0x23ff,
            value: 0x24
        }
    );
    assert_eq!(
        d.writes[1024],
        VideoWrite {
            address: 0x23c0,
            value: 3
        }
    );
    assert_eq!(
        d.writes[1087],
        VideoWrite {
            address: 0x23ff,
            value: 3
        }
    );
}
#[test]
fn lower_fill_uses_requested_pages_and_does_not_emit_attributes() {
    let mut d = Display::default();
    d.fill(0x10, 9, 2, Stride::Down);
    assert_eq!(d.writes.len(), 512);
    assert_eq!(
        d.writes[1],
        VideoWrite {
            address: 0x1020,
            value: 9
        }
    );
}
#[test]
fn sequential_producers_preserve_address_wrap_stride_and_attribute_overwrite_order() {
    // C129, BAC0 and C209 all retain control-bit2 stride and 14-bit output.
    for (stride, step) in [(Stride::Across, 1_u32), (Stride::Down, 32)] {
        for address in [0x0100_u16, 0x3ffe, 0x3fff, 0x7fff, 0xfffe, 0xffff] {
            let expected: Vec<_> = [7, 8, 9]
                .into_iter()
                .enumerate()
                .map(|(i, value)| VideoWrite {
                    address: ((u32::from(address) + i as u32 * step) % 16384) as u16,
                    value,
                })
                .collect();
            let mut script = Display::default();
            script.script(
                0,
                &[(address >> 8) as u8, address as u8, 7, 8, 9, 0xfe, 0],
                &[0; 8],
                stride,
            );
            assert_eq!(script.writes, expected);
            if address < 0x8000 {
                let mut queue = Display::default();
                queue.flush(
                    Queue::Primary,
                    &[(address >> 8) as u8, address as u8, 2, 7, 8, 9, 0xff],
                    stride,
                );
                assert_eq!(queue.writes, expected);
            }
        }
        let mut fill = Display::default();
        fill.fill(0xff, 7, 9, stride);
        assert_eq!(fill.writes.len(), 1024 + 64);
        for (i, write) in fill.writes.iter().enumerate() {
            let (start, offset, value) = if i < 1024 {
                (0xff00, i, 7)
            } else {
                (0x02c0, i - 1024, 9)
            };
            assert_eq!(
                *write,
                VideoWrite {
                    address: ((start + offset as u32 * step) % 16384) as u16,
                    value,
                }
            );
        }
    }
}
#[test]
fn fades_have_four_immediate_steps_with_four_resumed_yields_per_step() {
    for (direction, offsets) in [
        (FadeDirection::In, [0x30, 0x20, 0x10, 0]),
        (FadeDirection::Out, [0x10, 0x20, 0x30, 0x40]),
    ] {
        let mut f = Fade::start(direction);
        for offset in offsets {
            assert_eq!(f.subtraction, offset);
            let mut d = Display::default();
            f.publish(&mut d);
            assert!(d.palette_pending);
            for _ in 0..3 {
                assert!(!f.resumed());
            }
            assert!(f.resumed());
        }
        assert!(f.complete);
    }
}
#[test]
fn fade_subtraction_clamps_negative_result_to_black_and_keeps_base() {
    let mut d = Display::default();
    d.base[..8].copy_from_slice(&[0, 0x0f, 0x10, 0x1f, 0x20, 0x2f, 0x30, 0x3f]);
    let base = d.base;
    let f = Fade::start(FadeDirection::Out);
    f.publish(&mut d);
    assert_eq!(
        &d.active[..8],
        &[BLACK, BLACK, 0, 0x0f, 0x10, 0x1f, 0x20, 0x2f]
    );
    assert_eq!(d.base, base);
}
#[test]
fn scene_palette_backwards_endpoints_universal_color_and_option_override() {
    let colors: Vec<u8> = (0..64).collect();
    let options = [[9, 8, 7, 6], [5, 4, 3, 2], [1, 0, 1, 0]];
    for (sprite_end, option, last) in [
        (31, 0, [28, 29, 30, 31]),
        (0x9f, 0, options[0]),
        (0x9f, 1, options[1]),
        (0x9f, 2, options[2]),
    ] {
        let mut d = Display::default();
        d.scene_palette(&colors, 15, sprite_end, option, &options);
        assert_eq!(&d.base[..16], &(0..16).collect::<Vec<_>>());
        assert_eq!(d.base[16], 0);
        assert_eq!(&d.base[28..], &last);
        assert_eq!(d.base, d.active);
        assert!(!d.palette_pending);
    }
}
#[test]
fn palette_commit_copies_all_thirtytwo_and_clears_only_its_pending_flag() {
    let mut d = Display {
        palette_pending: true,
        primary: PreparedPrimary {
            pending: 0xff,
            ..PreparedPrimary::default()
        },
        secondary_pending: true,
        ..Default::default()
    };
    d.active = [7; PALETTE_COLORS];
    d.commit_palette();
    assert_eq!(d.writes.len(), 32);
    assert_eq!(
        d.writes[31],
        VideoWrite {
            address: 0x3f1f,
            value: 7
        }
    );
    assert_eq!(
        (
            d.palette_pending,
            (d.primary.pending != 0),
            d.secondary_pending
        ),
        (false, true, true)
    );
}
#[test]
fn original_catalog_decodes_every_animation_frame_coordinate_and_screen_script() {
    let rom = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom"))
        .expect("matching local ROM");
    let catalog = PresentationCatalog::from_original(&rom, &[0, 0, 9, 3]);
    assert_eq!(catalog.animations.len(), 144);
    assert_eq!(catalog.composites.len(), 248);
    assert_eq!(catalog.coordinate_variants, 224);
    assert_eq!(catalog.scripts.len(), 12);
    assert_eq!(
        catalog.composites[0],
        Composite {
            pieces: vec![Piece {
                tile: 9,
                attributes: 3,
                dy: 0,
                dx: 0
            }]
        }
    );
}
#[test]
fn inactive_slots_release_identity_hidden_render_skips_without_mutating_actor() {
    let mut actors: [ActorVisual; ACTOR_SLOTS] = std::array::from_fn(|_| actor());
    actors[2].state = 0;
    actors[2].render = 0x55;
    actors[3].render = HIDDEN_RENDER;
    let hidden = actors[3].clone();
    let mut c = Composition::default();
    let eligible = c.eligible(&mut actors, &(0..ACTOR_SLOTS).collect::<Vec<_>>());
    assert_eq!(
        eligible,
        vec![0, 1, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
    );
    assert_eq!((actors[2].render, actors[2].spawn_identity), (0, None));
    assert_eq!(actors[3], hidden);
}
#[test]
fn raster_effects_prime_split_and_apply_source_ui_then_playfield_values() {
    use crate::runtime::{FrameEffect as E, UiControl};
    let input = RasterInputs {
        split_y: 0x2f,
        ui_x: 3,
        ui_y: 4,
        camera_x: 0x234,
        playfield_y: 7,
        control: 0x90,
        mask: 0x1e,
        split_address: 0x2540,
    };
    let mut raster = RasterSnapshot::default();
    raster.apply(
        &[
            E::PrimeSplitSprite,
            E::InstallUiScroll,
            E::InstallUiControl(UiControl::ForceGameplayPatterns),
            E::InstallSplitAddress,
            E::SpriteOnlyMask,
            E::InstallPlayfieldScroll,
        ],
        input,
    );
    assert_eq!(
        raster.split_sprite,
        Sprite {
            y: 0x2f,
            tile: 0x21,
            attributes: 0x20,
            x: 8
        }
    );
    assert_eq!(raster.ui_scroll, (3, 4));
    assert_eq!(raster.ui_control, 0x98);
    assert_eq!(raster.playfield_scroll, Some((0x34, 7)));
    assert_eq!(raster.playfield_control, Some(0x90));
    assert_eq!(raster.split_address, Some(0x2540));
    assert_eq!(raster.playfield_mask, 0x10);
}
#[test]
fn suspended_boundary_can_prime_sprite_but_leaves_scroll_palette_and_split_uncommitted() {
    use crate::runtime::FrameEffect as E;
    let mut raster = RasterSnapshot {
        ui_control: 0x81,
        ..Default::default()
    };
    raster.apply(
        &[E::PrimeSplitSprite, E::WakeTasks, E::AdvanceAudioFrame],
        RasterInputs {
            split_y: 70,
            ..Default::default()
        },
    );
    assert_eq!(raster.split_sprite.y, 70);
    assert_eq!(raster.ui_control, 0x81);
    assert_eq!(raster.playfield_scroll, None);
    assert_eq!(raster.split_address, None);
}
#[test]
fn nonsplit_raster_uses_shadow_control_and_camera_page_parity() {
    use crate::runtime::{FrameEffect as E, UiControl};
    for (camera, expected) in [(0xff, 0x90), (0x100, 0x91), (0x200, 0x90), (0xffff, 0x91)] {
        let mut r = RasterSnapshot::default();
        r.apply(
            &[
                E::InstallUiScroll,
                E::InstallUiControl(UiControl::Shadow),
                E::InstallPlayfieldScroll,
            ],
            RasterInputs {
                camera_x: camera,
                control: 0x81,
                mask: 0x1e,
                ..Default::default()
            },
        );
        assert_eq!(r.ui_control, 0x81);
        assert_eq!(r.playfield_control, Some(expected));
        assert_eq!(r.playfield_mask, 0x1e);
        assert_eq!(r.split_address, None);
    }
}
#[test]
fn queue_count_sign_boundary_80_copies129_bytes_7f_copies128() {
    for (count, total) in [(0x7f, 128), (0x80, 129), (0x81, 1), (0xfe, 1)] {
        let mut encoded = vec![0x20, 0, count];
        encoded.extend(vec![9; total]);
        encoded.push(0xff);
        let mut d = Display::default();
        d.flush(Queue::Primary, &encoded, Stride::Across);
        assert_eq!(d.writes.len(), total);
    }
}
#[test]
fn original_all_animation_headers_frames_and_rom_composite_piece_counts_match_independent_raw_records()
 {
    let rom = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom"))
        .expect("local ROM");
    let catalog = PresentationCatalog::from_original(&rom, &[0, 0, 9, 3]);
    // Loader locations only: mapper5 pointers; these reads are fixture facts, not
    // a second animation/composition engine. Every original selector is checked.
    const BANK_PHYSICAL: usize = 0x14010;
    const CPU_WINDOW: usize = 0x8000;
    const ANIMATION_LOW: usize = 0xb1c7;
    const ANIMATION_HIGH: usize = 0xb257;
    const FRAME_LOW: usize = 0x9681;
    const FRAME_HIGH: usize = 0x9779;
    for selector in 0..144 {
        let address = usize::from(rom[BANK_PHYSICAL + ANIMATION_LOW - CPU_WINDOW + selector])
            + 256 * usize::from(rom[BANK_PHYSICAL + ANIMATION_HIGH - CPU_WINDOW + selector]);
        let offset = BANK_PHYSICAL + address - CPU_WINDOW;
        let expected = &catalog.animations[selector];
        assert_eq!(
            (expected.last_frame, expected.last_tick),
            (rom[offset], rom[offset + 1]),
            "animation {selector}"
        );
        assert_eq!(
            expected.frames,
            rom[offset + 2..offset + 3 + usize::from(rom[offset])]
        );
    }
    for selector in 4..248 {
        let address = usize::from(rom[BANK_PHYSICAL + FRAME_LOW - CPU_WINDOW + selector])
            + 256 * usize::from(rom[BANK_PHYSICAL + FRAME_HIGH - CPU_WINDOW + selector]);
        let offset = BANK_PHYSICAL + address - CPU_WINDOW;
        let pieces = &catalog.composites[selector].pieces;
        assert_eq!(
            pieces.len(),
            usize::from(rom[offset]) + 1,
            "frame {selector}"
        );
        for (piece, raw) in pieces.iter().zip(rom[offset + 2..].chunks_exact(2)) {
            assert_eq!((piece.tile, piece.attributes), (raw[0], raw[1]));
        }
    }
    for selector in 0..4 {
        assert_eq!(catalog.composites[selector], catalog.composites[0]);
    }
}
#[test]
fn all_twelve_original_script_record_destinations_and_lengths_are_preserved() {
    let rom = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom"))
        .expect("local ROM");
    let c = PresentationCatalog::from_original(&rom, &[0, 0, 9, 3]);
    let totals = [89, 0, 315, 147, 158, 150, 87, 100, 16, 32, 31, 31];
    let first = [
        0x2021, 0, 0x20b2, 0x2504, 0x2504, 0x2504, 0x2204, 0x2244, 0x26ce, 0x26ce, 0x26ce, 0x2486,
    ];
    for selector in 0..12 {
        assert_eq!(
            c.scripts[selector].len(),
            totals[selector],
            "script {selector}"
        );
        if totals[selector] != 0 {
            assert_eq!(c.scripts[selector][0].address, first[selector]);
        }
    }
}
#[test]
fn coordinate_flip_selects_adjacent_stream_without_arithmetic_negation() {
    let streams = vec![vec![1, 2, 0xff, 0xfe], vec![3, 4, 0xfd, 0xfc]];
    let record = [1, 0, 7, 1, 8, 2];
    assert_eq!(
        decode_composite(&record, &streams, false),
        Composite {
            pieces: vec![
                Piece {
                    tile: 7,
                    attributes: 1,
                    dy: 1,
                    dx: 2
                },
                Piece {
                    tile: 8,
                    attributes: 2,
                    dy: -1,
                    dx: -2
                }
            ]
        }
    );
    assert_eq!(
        decode_composite(&record, &streams, true),
        Composite {
            pieces: vec![
                Piece {
                    tile: 7,
                    attributes: 1,
                    dy: 3,
                    dx: 4
                },
                Piece {
                    tile: 8,
                    attributes: 2,
                    dy: -3,
                    dx: -4
                }
            ]
        }
    );
}
#[test]
fn counted_record_header_is_last_index_and_last_tick_not_length_or_duration() {
    assert_eq!(
        decode_animation(&[2, 3, 7, 8, 9]),
        Animation {
            last_frame: 2,
            last_tick: 3,
            frames: vec![7, 8, 9]
        }
    );
    assert_eq!(
        decode_animation(&[0, 0, 6]),
        Animation {
            last_frame: 0,
            last_tick: 0,
            frames: vec![6]
        }
    );
}
#[test]
fn attribute_flip_is_xor_while_priority_is_or_and_palette_bits_stay_piece_owned() {
    for (piece_attr, render, expected) in [
        (0x40, FLIP_HORIZONTAL, 0),
        (2, FLIP_HORIZONTAL, 0x42),
        (0x20, PRIORITY, 0x20),
        (3, 0, 3),
    ] {
        let mut a = actor();
        a.render = render;
        let mut c = Composition::default();
        c.emit(
            0,
            &a,
            &Composite {
                pieces: vec![Piece {
                    tile: 1,
                    attributes: piece_attr,
                    dy: 0,
                    dx: 0,
                }],
            },
        );
        assert_eq!(c.oam[1].attributes, expected);
    }
}
#[test]
fn maximal_nonfrozen_tick_wraps_and_bit_seven_becomes_a_freeze_latch() {
    let mut a = actor();
    {
        let animation_value = 0x7f;
        a.playback.set_tick_code(animation_value);
    };
    let s = Animation {
        last_tick: 0x7e,
        ..sequence()
    };
    assert_eq!(Composition::advance(&mut a, &s, 0), Some(9));
    assert_eq!(a.playback.tick_code(), 0x80);
    assert_eq!(Composition::advance(&mut a, &s, 0), Some(9));
    assert_eq!(a.playback.tick_code(), 0x80);
}
#[test]
fn zero_lower_fill_page_count_wraps_to_twohundredfiftysix_pages() {
    let mut d = Display::default();
    d.fill(0, 2, 0, Stride::Across);
    assert_eq!(d.writes.len(), 65536);
    assert_eq!(
        d.writes[65535],
        VideoWrite {
            address: 0x3fff,
            value: 2
        }
    );
}
#[test]
fn every_rom_frame_both_coordinate_variants_preserve_signed_pairs_and_shared_pointer_aliases() {
    let rom = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom"))
        .expect("local ROM");
    let c = PresentationCatalog::from_original(&rom, &[0, 0, 9, 3]);
    const BANK_PHYSICAL: usize = 0x14010;
    const CPU_WINDOW: usize = 0x8000;
    const FRAME_LOW: usize = 0x9681;
    const FRAME_HIGH: usize = 0x9779;
    const COORD_LOW: usize = 0xade1;
    const COORD_HIGH: usize = 0xaec1;
    for selector in 4..248 {
        let address = usize::from(rom[BANK_PHYSICAL + FRAME_LOW - CPU_WINDOW + selector])
            + 256 * usize::from(rom[BANK_PHYSICAL + FRAME_HIGH - CPU_WINDOW + selector]);
        let offset = BANK_PHYSICAL + address - CPU_WINDOW;
        for (flip, frames) in [(0, &c.composites), (1, &c.flipped_composites)] {
            let coordinate_selector = usize::from(rom[offset + 1]) + flip;
            let address =
                usize::from(rom[BANK_PHYSICAL + COORD_LOW - CPU_WINDOW + coordinate_selector])
                    + 256
                        * usize::from(
                            rom[BANK_PHYSICAL + COORD_HIGH - CPU_WINDOW + coordinate_selector],
                        );
            let coordinate_offset = BANK_PHYSICAL + address - CPU_WINDOW;
            for (index, piece) in frames[selector].pieces.iter().enumerate() {
                assert_eq!(
                    (piece.dy, piece.dx),
                    (
                        rom[coordinate_offset + 2 * index] as i8,
                        rom[coordinate_offset + 2 * index + 1] as i8
                    ),
                    "frame {selector} flip {flip} piece {index}"
                );
            }
        }
    }
}
#[test]
fn fade_in_hides_sprites_while_fade_out_preserves_existing_oam() {
    for direction in [FadeDirection::In, FadeDirection::Out] {
        let mut d = Display::default();
        let mut c = Composition {
            oam: [Sprite {
                y: 7,
                ..Sprite::default()
            }; OAM_ENTRIES],
            ..Default::default()
        };
        let f = d.begin_fade(direction, &mut c, true);
        assert_eq!(f.direction, direction);
        assert_eq!(c.oam[0].y, 7);
        assert_eq!(
            c.oam[1].y,
            if direction == FadeDirection::In {
                crate::runtime::HIDDEN_SPRITE_Y
            } else {
                7
            }
        );
    }
}
#[test]
fn queue_cursor_wrapping_exactly_zero_exits_without_reading_another_header() {
    let mut encoded = vec![0x20, 0, 0x7f];
    encoded.extend([1; 128]);
    encoded.extend([0x24, 0, 121]);
    encoded.extend([2; 122]);
    assert_eq!(encoded.len(), 256);
    let mut d = Display::default();
    d.flush(Queue::Secondary, &encoded, Stride::Across);
    assert_eq!(d.writes.len(), 250);
    assert_eq!(
        d.writes[249],
        VideoWrite {
            address: 0x2479,
            value: 2
        }
    );
}
#[test]
fn prepared_chr_copies_preserve_order_stride_and_control_shadow() {
    let p = crate::assets::formats::ChrPreparation {
        control: 0x94,
        stride: 32,
        copies: vec![(0, vec![1, 2]), (0, vec![3])],
        selected_bank: Some(2),
    };
    let mut d = Display {
        control: 0x95,
        ..Default::default()
    };
    d.apply_chr(&p);
    assert_eq!(d.control, 0x95);
    assert_eq!(
        d.writes,
        vec![
            VideoWrite {
                address: 0,
                value: 1
            },
            VideoWrite {
                address: 32,
                value: 2
            },
            VideoWrite {
                address: 0,
                value: 3
            }
        ]
    );
}
#[test]
fn logical_video_memory_preserves_chr_vertical_nametable_and_palette_aliases() {
    let mut v = VideoMemory::blank();
    for (address, value) in [
        (0, 3),
        (0x1fff, 4),
        (0x2000, 5),
        (0x2400, 6),
        (0x3f10, 7),
        (0x3f14, 8),
    ] {
        v.write(VideoWrite { address, value });
    }
    for (address, expected) in [
        (0, 3),
        (0x1fff, 4),
        (0x2000, 5),
        (0x2800, 5),
        (0x3000, 5),
        (0x2400, 6),
        (0x2c00, 6),
        (0x3f00, 7),
        (0x3f04, 8),
        (0x3f34, 8),
    ] {
        assert_eq!(v.read(address), expected);
    }
    let before = v.clone();
    v.read(0);
    assert_eq!(v, before);
}

#[test]
fn canonical_primary_exact_submission_byte_commits_once_then_clears_without_losing_prepared_data() {
    // B12 message producer returns5D; B15 owns a borrow of this same queue.
    const STATUS_SUBMISSION: u8 = 0x5d;
    let encoded = vec![0x20, 0x4c, 1, 0x0a, 0x2e, 0xff];
    let mut d = Display {
        primary: PreparedPrimary {
            encoded: encoded.clone(),
            pending: STATUS_SUBMISSION,
        },
        ..Display::default()
    };
    d.commit_primary(Stride::Across);
    assert_eq!(
        d.writes,
        vec![
            VideoWrite {
                address: 0x204c,
                value: 0x0a
            },
            VideoWrite {
                address: 0x204d,
                value: 0x2e
            }
        ]
    );
    assert_eq!(d.primary.pending, 0);
    assert_eq!(d.primary.encoded, encoded);
    d.commit_primary(Stride::Across);
    assert_eq!(d.writes.len(), 2);
}
#[test]
fn canonical_primary_zero_submission_keeps_prepared_text_uncommitted() {
    let encoded = vec![0x20, 0x4c, 0, 0x0a, 0xff];
    let mut d = Display {
        primary: PreparedPrimary {
            encoded: encoded.clone(),
            pending: 0,
        },
        ..Display::default()
    };
    d.commit_primary(Stride::Across);
    assert!(d.writes.is_empty());
    assert_eq!(d.primary.encoded, encoded);
    assert_eq!(d.primary.pending, 0);
}
#[test]
fn fallible_presentation_loader_rejects_malformed_pointer_and_composite_overrides() {
    let original =
        std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap();
    let mut pointer = original.clone();
    pointer[0x171d7] = 0;
    pointer[0x17267] = 0;
    assert!(PresentationCatalog::try_from_original(&pointer, &[0; 4]).is_err());
    let mut coordinate = original.clone();
    let frame = usize::from(coordinate[0x15691 + 4]) | (usize::from(coordinate[0x15789 + 4]) << 8);
    coordinate[0x14010 + frame - 0x8000 + 1] = 0xff;
    assert!(PresentationCatalog::try_from_original(&coordinate, &[0; 4]).is_err());
    assert!(PresentationCatalog::try_from_original(&original, &[0]).is_err());
}

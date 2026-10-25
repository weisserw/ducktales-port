//! Source-derived contracts, with branch/flag derivations in adjacent trace.md.
//! Fixtures are synthetic. Original table bytes are read only from matching local
//! ROM, never embedded or executed. No test helper implements collision rules.
use super::*;
use crate::{config, world::CACHE_CELLS};
use romatlas_port::assets::Rom;
const TEST_WIDTH: u8 = 10;
const TEST_HEIGHT: u8 = 20;
const TEST_X: u16 = 0x0160;
const TEST_Y: u8 = 0x60;
const TEST_ROW: u8 = 2;
const ONE_SAMPLE: ProbeRange = ProbeRange {
    start: 0,
    remaining: 0,
};
fn geometry() -> Geometry {
    Geometry {
        offsets: vec![SampleOffset::default(); SAMPLE_GEOMETRIES],
        extents: [Extent {
            half_width: TEST_WIDTH,
            half_height: TEST_HEIGHT,
        }; COLLISION_PROFILES],
        responses: [TerrainResponse::Solid; 16],
        velocity_presets: [0; 2],
        particle_x: [0; 4],
        particle_y: [0; 4],
        particle_ticks: [0; 4],
    }
}
fn actor() -> ActorContact {
    ActorContact {
        control: crate::actors::ActorControl::from_codes(ACTIVE, 0, 0, 0),
        position: Position {
            world_x: TEST_X,
            screen_y: TEST_Y,
            ..Default::default()
        },
        render: VISIBLE,
        profile: PROFILE_ENABLED,
        spawn_identity: Some(7),
        ..Default::default()
    }
}
fn actors() -> [ActorContact; ACTOR_SLOTS] {
    [ActorContact::default(); ACTOR_SLOTS]
}
fn cache(class: u8) -> WorldCache {
    WorldCache {
        collision: [class * 0x55; CACHE_CELLS],
        ..Default::default()
    }
}
fn rom() -> Vec<u8> {
    let bytes =
        std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../input/game.rom"))
            .expect("matching local ROM fixture");
    Rom::from_bytes(bytes.clone(), config::ROM_SHA256).expect("identity checked");
    bytes
}
fn special_probe(
    slot: usize,
    response: TerrainResponse,
    range: ProbeRange,
    a: &mut ActorContact,
    p: &mut PlayerContact,
) -> ProbeReport {
    let mut g = geometry();
    g.responses = [response; 16];
    probe(slot, a, p, &cache(3), TEST_ROW, &g, range, &mut |_| 0)
}
fn move_case(
    a: &mut [ActorContact; ACTOR_SLOTS],
    movement: Movement,
    class: u8,
    step: u8,
    latch: &mut u8,
) -> MovementReport {
    move_with_collision(
        2,
        a,
        &mut PlayerContact::default(),
        &cache(class),
        TEST_ROW,
        &geometry(),
        ONE_SAMPLE,
        movement,
        step,
        latch,
        &mut |_| 0,
    )
}
#[test]
fn original_geometry_decodes_every_signed_sample_and_both_extent_planes() {
    let bytes = rom();
    let g = Geometry::original(&bytes).unwrap();
    assert_eq!(g.offsets.len(), SAMPLE_GEOMETRIES);
    for index in 0..SAMPLE_GEOMETRIES {
        assert_eq!(
            g.offsets[index],
            SampleOffset {
                x: bytes[SAMPLE_X_PHYSICAL_START + index] as i8,
                y: bytes[SAMPLE_Y_PHYSICAL_START + index] as i8
            }
        );
    }
    for index in 0..COLLISION_PROFILES {
        assert_eq!(
            g.extents[index],
            Extent {
                half_width: bytes[WIDTHS_PHYSICAL_START + index],
                half_height: bytes[HEIGHTS_PHYSICAL_START + index]
            }
        );
    }
    assert_eq!(
        g.particle_y.as_slice(),
        &bytes[PARTICLES_PHYSICAL_START..PARTICLES_PHYSICAL_START + 4]
    );
    assert_eq!(
        g.particle_x.as_slice(),
        &bytes[PARTICLES_PHYSICAL_START + 4..PARTICLES_PHYSICAL_START + 8]
    );
    assert_eq!(
        g.particle_ticks.as_slice(),
        &bytes[PARTICLES_PHYSICAL_START + 8..PARTICLES_PHYSICAL_START + 12]
    );
}
#[test]
fn all_sixteen_split_terrain_dispatch_entries_have_independently_traced_semantics() {
    use TerrainResponse::*;
    assert_eq!(
        Geometry::original(&rom()).unwrap().responses,
        [
            Pose,
            Pose,
            Pose,
            Pose,
            Mark,
            Mark,
            Mark,
            Mark,
            Momentum,
            Damage,
            LowExit,
            LowExit,
            Damage,
            Damage,
            MarkMomentum,
            Solid
        ]
    );
}
#[test]
fn original_preset_consumer_includes_reachable_instruction_byte_word_without_execution() {
    let bytes = rom();
    let g = Geometry::original(&bytes).unwrap();
    assert_eq!(PRESET_CONSUMER_BYTES, 4);
    let mut a = actor();
    set_velocity_preset(&mut a, &g, 0, false);
    let low = bytes[PRESET_CONSUMER_PHYSICAL_START];
    let high = bytes[PRESET_CONSUMER_PHYSICAL_START + 1];
    assert_eq!(a.horizontal_magnitude, u16::from_le_bytes([low, high]));
    // ACCF loads A=1 then INC state (does not alter A); F313 doubles it.
    set_velocity_preset(&mut a, &g, 1, false);
    assert_eq!(
        a.horizontal_magnitude,
        u16::from_le_bytes([
            bytes[PRESET_CONSUMER_PHYSICAL_START + 2],
            bytes[PRESET_CONSUMER_PHYSICAL_START + 3]
        ])
    );
    set_velocity_preset(&mut a, &g, 1, true);
    assert_eq!(a.vertical_velocity as u16, a.horizontal_magnitude);
}
#[test]
fn positive_x_carries_fraction_low_integer_and_page_without_false_boundary() {
    let mut a = actor();
    a.position.world_x = 0x00ff;
    a.position.x_fraction = 0xf0;
    a.horizontal_magnitude = 0x0030;
    assert!(!integrate_horizontal(&mut a, HorizontalDirection::Positive));
    assert_eq!((a.position.world_x, a.position.x_fraction), (0x0100, 0x20));
}
#[test]
fn negative_x_borrows_across_fraction_low_integer_and_page_without_false_boundary() {
    let mut a = actor();
    a.position.world_x = 0x0100;
    a.position.x_fraction = 0x10;
    a.horizontal_magnitude = 0x0030;
    assert!(!integrate_horizontal(&mut a, HorizontalDirection::Negative));
    assert_eq!((a.position.world_x, a.position.x_fraction), (0x00ff, 0xe0));
}
#[test]
fn horizontal_magnitude_is_unsigned_and_full_world_overflow_is_reported() {
    let mut a = actor();
    a.position.world_x = 0xff80;
    a.horizontal_magnitude = 0x8000;
    assert!(integrate_horizontal(&mut a, HorizontalDirection::Positive));
    assert_eq!(a.position.world_x, 0);
    a.position.world_x = 0;
    a.horizontal_magnitude = 1;
    a.position.x_fraction = 0;
    assert!(integrate_horizontal(&mut a, HorizontalDirection::Negative));
    assert_eq!((a.position.world_x, a.position.x_fraction), (0xffff, 0xff));
}
#[test]
fn zero_horizontal_speed_preserves_fractions_and_extreme_pages() {
    for direction in [HorizontalDirection::Positive, HorizontalDirection::Negative] {
        let mut a = actor();
        a.position.world_x = 0xffff;
        a.position.x_fraction = 0xff;
        let old = a;
        assert!(!integrate_horizontal(&mut a, direction));
        assert_eq!(a, old);
    }
}
#[test]
fn vertical_encoded_addition_and_subtraction_preserve_fractional_wrap() {
    for (operation, y, fraction, velocity, expected_y, expected_fraction, wrap) in [
        (
            VerticalOperation::AddEncodedVelocity,
            0xff,
            0xf0,
            0x0030,
            0,
            0x20,
            true,
        ),
        (
            VerticalOperation::SubtractEncodedVelocity,
            0,
            0x10,
            0x0030,
            0xff,
            0xe0,
            true,
        ),
        (
            VerticalOperation::SubtractEncodedVelocity,
            100,
            0,
            0x0133,
            98,
            0xcd,
            false,
        ),
        (
            VerticalOperation::SubtractEncodedVelocity,
            100,
            0,
            -0x0040,
            100,
            0x40,
            true,
        ),
        (
            VerticalOperation::SubtractEncodedVelocity,
            0xff,
            0,
            -0x0100,
            0,
            0,
            false,
        ),
    ] {
        let mut a = actor();
        a.position.screen_y = y;
        a.position.y_fraction = fraction;
        a.vertical_velocity = velocity;
        assert_eq!(integrate_vertical(&mut a, operation), wrap);
        assert_eq!(
            (a.position.screen_y, a.position.y_fraction),
            (expected_y, expected_fraction)
        );
    }
}
#[test]
fn gravity_subtracts_fraction_with_borrow_and_wraps_signed_velocity() {
    for (before, step, expected) in [
        (0x0010, 0x20, -0x0010),
        (0, 0x40, -0x0040),
        (i16::MIN, 1, i16::MAX),
        (123, 0, 123),
    ] {
        let mut a = actor();
        a.vertical_velocity = before;
        gravity(&mut a, step);
        assert_eq!(a.vertical_velocity, expected);
    }
}
#[test]
fn every_cache_cell_and_quadrant_uses_page_parity_and_source_coordinate_transpose() {
    let mut c = WorldCache::default();
    for page in 0..2u16 {
        for row in 0..8u8 {
            for col in 0..8u8 {
                for quadrant in 0..4u8 {
                    let index = usize::from(page) * 64 + usize::from(row) * 8 + usize::from(col);
                    c.collision.fill(0);
                    c.collision[index] = SECONDARY_CONTACT << (quadrant * 2);
                    let p = Position {
                        world_x: page * 256 + u16::from(col) * 32 + u16::from(quadrant & 1) * 16,
                        screen_y: row * 32 + (quadrant >> 1) * 16,
                        ..Default::default()
                    };
                    let s = sample(&c, p, TEST_ROW, SampleOffset::default());
                    assert_eq!(
                        (s.class, s.cache_index, s.quadrant),
                        (SECONDARY_CONTACT, Some(index), Some(quadrant))
                    );
                    assert_eq!(
                        s.descriptor,
                        Some(TerrainProbe {
                            room_row: TEST_ROW,
                            world_page: page as u8,
                            block_coordinate: row * 8 + col,
                            quadrant
                        })
                    );
                }
            }
        }
    }
}
#[test]
fn signed_probe_offsets_carry_into_world_page_and_select_final_descriptor_coordinates() {
    let p = Position {
        world_x: 0x00fe,
        screen_y: 0x21,
        ..Default::default()
    };
    let s = sample(&cache(2), p, TEST_ROW, SampleOffset { x: 3, y: -2 });
    assert_eq!(
        (s.world_x, s.screen_y, s.cache_index, s.quadrant),
        (0x0101, 0x1f, Some(64), Some(2))
    );
    assert_eq!(
        s.descriptor,
        Some(TerrainProbe {
            room_row: TEST_ROW,
            world_page: 1,
            block_coordinate: 0,
            quadrant: 2
        })
    );
}
#[test]
fn horizontal_negative_page_and_sign_bit_pages_are_empty_without_descriptor_reads() {
    for (x, offset) in [(0, -1), (0x8000, 0), (0x7fff, 1)] {
        let s = sample(
            &cache(3),
            Position {
                world_x: x,
                ..Default::default()
            },
            TEST_ROW,
            SampleOffset { x: offset, y: 0 },
        );
        assert_eq!(s.class, 0);
        assert!(s.descriptor.is_none());
        assert!(s.cache_index.is_none());
    }
}
#[test]
fn horizontal_sampling_has_no_layout_width_clamp_and_wraps_positive_high_page() {
    for world_x in [0x0200, 0x7f00] {
        let s = sample(
            &cache(2),
            Position {
                world_x,
                screen_y: 0,
                ..Default::default()
            },
            TEST_ROW,
            SampleOffset::default(),
        );
        assert_eq!(s.class, 2);
    }
    let s = sample(
        &cache(2),
        Position {
            world_x: 0xffff,
            screen_y: 0,
            ..Default::default()
        },
        TEST_ROW,
        SampleOffset { x: 1, y: 0 },
    );
    assert_eq!((s.class, s.world_x), (2, 0));
}
#[test]
fn vertical_probe_underflow_returns_solid_while_overflow_returns_empty_for_every_room_row() {
    for room_row in [0, 1, 0xff] {
        for (y, offset, class) in [(0, -1, 1), (0xff, 1, 0)] {
            let s = sample(
                &cache(3),
                Position {
                    world_x: 0x0100,
                    screen_y: y,
                    ..Default::default()
                },
                room_row,
                SampleOffset { x: 0, y: offset },
            );
            assert_eq!(s.class, class);
            assert!(s.descriptor.is_none());
        }
    }
}
#[test]
fn ordinary_probe_runs_count_plus_one_reverses_result_indices_and_preserves_class_two() {
    let mut a = actor();
    a.render |= TERRAIN_MARK;
    let mut g = geometry();
    g.offsets[0] = SampleOffset { x: 0, y: 0 };
    g.offsets[1] = SampleOffset { x: 16, y: 0 };
    g.offsets[2] = SampleOffset { x: 32, y: 0 };
    let mut c = WorldCache::default(); // TEST_X=160? fixture is0x160: page1, column3,row3 ->91.
    c.collision[91] = 2;
    c.collision[92] = 1;
    let mut reads = 0;
    let r = probe(
        0,
        &mut a,
        &mut PlayerContact::default(),
        &c,
        TEST_ROW,
        &g,
        ProbeRange {
            start: 0,
            remaining: 2,
        },
        &mut |_| {
            reads += 1;
            0
        },
    );
    assert_eq!(r.visited, vec![0, 1, 2]);
    assert_eq!(r.results, vec![(2, 2), (1, 0), (0, 1)]);
    assert_eq!(r.bits, 3);
    assert_eq!(
        r.final_range,
        ProbeRange {
            start: 3,
            remaining: 0xff
        }
    );
    assert_eq!(reads, 0);
    assert_eq!(a.render & TERRAIN_MARK, 0);
}
#[test]
fn nonplayer_stops_at_first_solid_but_not_class_two_and_keeps_unvisited_results_absent() {
    let mut a = actor();
    let mut g = geometry();
    g.offsets[1].x = 32;
    let mut c = WorldCache::default();
    c.collision[91] = 2;
    c.collision[92] = 1;
    let r = probe(
        2,
        &mut a,
        &mut PlayerContact::default(),
        &c,
        TEST_ROW,
        &g,
        ProbeRange {
            start: 0,
            remaining: 2,
        },
        &mut |_| panic!("no class3"),
    );
    assert_eq!(r.bits, 3);
    assert_eq!(r.visited, vec![0, 1]);
    assert_eq!(r.results, vec![(2, 2), (1, 1)]);
    assert_eq!(
        r.final_range,
        ProbeRange {
            start: 1,
            remaining: 1
        }
    );
}
#[test]
fn special_class_dispatch_uses_descriptor_high_nibble_and_records_original_class() {
    let mut a = actor();
    let mut p = PlayerContact::default();
    let mut g = geometry();
    g.responses[14] = TerrainResponse::MarkMomentum;
    let mut reads = Vec::new();
    let r = probe(
        0,
        &mut a,
        &mut p,
        &cache(3),
        TEST_ROW,
        &g,
        ProbeRange {
            start: 0,
            remaining: 1,
        },
        &mut |q| {
            reads.push(q);
            0xe7
        },
    );
    assert_eq!(r.results, vec![(1, 3), (0, 3)]);
    assert_eq!(r.bits, SOLID_CONTACT);
    assert_eq!(p.momentum_control, 1);
    assert_eq!(a.render & TERRAIN_MARK, TERRAIN_MARK);
    assert_eq!(reads.len(), 2);
}
#[test]
fn mark_response_is_nonsolid_but_nonplayer_early_exit_uses_render_low_bit() {
    for (render, visits) in [(VISIBLE, 2), (VISIBLE | 1, 1)] {
        let mut a = actor();
        a.render = render;
        let r = special_probe(
            2,
            TerrainResponse::Mark,
            ProbeRange {
                start: 0,
                remaining: 1,
            },
            &mut a,
            &mut PlayerContact::default(),
        );
        assert_eq!(r.bits, 0);
        assert_eq!(r.visited.len(), visits);
        assert_eq!(a.render, render | TERRAIN_MARK);
    }
}
#[test]
fn damage_response_requires_player_unprotected_mode_sample_remaining_and_pogo_exception() {
    for (slot, protection, special, animation, start, remaining, expected) in [
        (0, 0, 0, 0, 0, 1, DAMAGE_STATE),
        (1, 0, 0, 0, 0, 1, 0),
        (0, 1, 0, 0, 0, 1, 0),
        (0, 0, 1, 0, 0, 1, 0),
        (0, 0, 0, POGO_POSE, 0, 1, 0),
        (0, 0, 0, POGO_POSE, DAMAGE_POGO_SAMPLE, 1, DAMAGE_STATE),
        (0, 0, 0, 0, 0, 0, 0),
    ] {
        let mut a = actor();
        let mut p = PlayerContact {
            animation,
            protection,
            special_mode: special,
            ..Default::default()
        };
        let r = special_probe(
            slot,
            TerrainResponse::Damage,
            ProbeRange { start, remaining },
            &mut a,
            &mut p,
        );
        assert_eq!(p.queued_state, expected);
        assert_eq!(p.immediate_state, 0);
        assert_eq!(r.bits, SOLID_CONTACT);
    }
}
#[test]
fn pose_response_queues_seven_only_for_exact_player_pose_sample_and_is_nonsolid_on_success() {
    for (slot, start, animation, queued, bits) in [
        (0, POSE_TERRAIN_SAMPLE, POGO_POSE, POSE_TERRAIN_STATE, 0),
        (1, POSE_TERRAIN_SAMPLE, POGO_POSE, 0, 1),
        (0, 6, POGO_POSE, 0, 1),
        (0, POSE_TERRAIN_SAMPLE, 0, 0, 1),
    ] {
        let mut a = actor();
        let mut p = PlayerContact {
            animation,
            ..Default::default()
        };
        let r = special_probe(
            slot,
            TerrainResponse::Pose,
            ProbeRange {
                start,
                remaining: 0,
            },
            &mut a,
            &mut p,
        );
        assert_eq!((p.queued_state, r.bits), (queued, bits));
    }
}
#[test]
fn momentum_response_wrap_zero_falls_through_exit_instead_of_marking_solid() {
    for (before, y, expected_latch, bits, queued) in [
        (0, TEST_Y, 1, 1, 0),
        (0xff, LOW_EXIT_Y - 1, 0, 1, 0),
        (0xff, LOW_EXIT_Y, 0, 1, LOW_EXIT_STATE),
    ] {
        // Two samples: remaining1 wraps latch, remaining0 supplies common solid.
        let mut a = actor();
        a.position.screen_y = y;
        let mut p = PlayerContact {
            momentum_control: before,
            ..Default::default()
        };
        let r = special_probe(
            0,
            TerrainResponse::Momentum,
            ProbeRange {
                start: 0,
                remaining: 1,
            },
            &mut a,
            &mut p,
        );
        assert_eq!(
            (p.momentum_control, r.bits, p.queued_state),
            (expected_latch, bits, queued)
        );
    }
}
#[test]
fn momentum_only_mutates_player_at_remaining_one_and_mark_variant_retains_other_render_bits() {
    for (slot, remaining, expected) in [(0, 0, 0), (0, 1, 1), (1, 1, 0)] {
        let mut a = actor();
        a.render = 0xff;
        let mut p = PlayerContact::default();
        let r = special_probe(
            slot,
            TerrainResponse::MarkMomentum,
            ProbeRange {
                start: 0,
                remaining,
            },
            &mut a,
            &mut p,
        );
        assert_eq!(p.momentum_control, expected);
        assert_eq!(r.bits, 1);
        assert_eq!(a.render, 0xff);
    }
}
#[test]
fn low_exit_response_queues_eight_at_inclusive_y_d0_without_solid_and_actor_stops_after_one() {
    for (slot, y, queued, visits) in [
        (0, LOW_EXIT_Y - 1, 0, 2),
        (0, LOW_EXIT_Y, LOW_EXIT_STATE, 2),
        (0, 0xff, LOW_EXIT_STATE, 2),
        (1, LOW_EXIT_Y, 0, 1),
    ] {
        let mut a = actor();
        a.position.screen_y = y;
        let mut p = PlayerContact::default();
        let r = special_probe(
            slot,
            TerrainResponse::LowExit,
            ProbeRange {
                start: 0,
                remaining: 1,
            },
            &mut a,
            &mut p,
        );
        assert_eq!(
            (r.bits, p.queued_state, r.visited.len()),
            (0, queued, visits)
        );
    }
}
#[test]
fn snap_adjusts_integer_coordinates_only_and_negative_direction_moves_full_cell_at_zero_nibble() {
    let s = Sample {
        class: 1,
        world_x: 0x0100,
        screen_y: 0,
        cache_index: None,
        quadrant: None,
        descriptor: None,
    };
    for (direction, x, y) in [
        (Snap::PositiveX, 0xff, 0xf8),
        (Snap::NegativeX, 0x010f, 0xf8),
        (Snap::PositiveY, 0xff, 0xf8),
        (Snap::NegativeY, 0xff, 8),
    ] {
        let mut a = actor();
        a.position = Position {
            world_x: 0xff,
            x_fraction: 7,
            screen_y: 0xf8,
            y_fraction: 9,
        };
        snap(&mut a, s, direction);
        assert_eq!(
            a.position,
            Position {
                world_x: x,
                x_fraction: 7,
                screen_y: y,
                y_fraction: 9
            }
        );
    }
    let mut a = actor();
    a.position.world_x = 0x0101;
    let mut s = s;
    s.world_x = 0x010f;
    snap(&mut a, s, Snap::PositiveX);
    assert_eq!(a.position.world_x, 0x00f2);
}
#[test]
fn horizontal_solid_scan_requires_source_visible_and_candidate_active_visible_handler_eight() {
    for (source_render, state, render, handler, hit) in [
        (VISIBLE, ACTIVE, VISIBLE, SOLID_HANDLER, true),
        (0, ACTIVE, VISIBLE, SOLID_HANDLER, false),
        (VISIBLE, 0, VISIBLE, SOLID_HANDLER, false),
        (VISIBLE, ACTIVE, 0, SOLID_HANDLER, false),
        (VISIBLE, ACTIVE, VISIBLE, 9, false),
    ] {
        let mut all = actors();
        all[2] = actor();
        all[2].render = source_render;
        all[15] = actor();
        {
            let actor_value = state;
            all[15].set_state_code(actor_value);
        };
        all[15].render = render;
        {
            let actor_value = handler;
            all[15].set_handler(actor_value);
        };
        assert_eq!(
            solid_overlap(2, &all, &geometry(), ContactAxis::Horizontal).is_some(),
            hit
        );
    }
}
#[test]
fn solid_scans_exclude_self_and_player_and_choose_highest_qualifying_slot() {
    for axis in [ContactAxis::Horizontal, ContactAxis::Vertical] {
        let mut all = actors();
        all[2] = actor();
        {
            let actor_value = SOLID_HANDLER;
            all[2].set_handler(actor_value);
        };
        all[0] = all[2];
        assert_eq!(solid_overlap(2, &all, &geometry(), axis), None);
        all[1] = all[2];
        all[15] = all[2];
        assert_eq!(
            solid_overlap(2, &all, &geometry(), axis),
            Some(Contact {
                slot: 15,
                penetration: if axis == ContactAxis::Horizontal {
                    TEST_WIDTH
                } else {
                    TEST_HEIGHT
                }
            })
        );
    }
}
#[test]
fn horizontal_solid_overlap_has_strict_height_and_inclusive_width_with_zero_penetration() {
    for (dx, dy, expected) in [
        (TEST_WIDTH, 0, Some(0)),
        (TEST_WIDTH + 1, 0, None),
        (0, TEST_HEIGHT, None),
        (0, TEST_HEIGHT - 1, Some(TEST_WIDTH)),
    ] {
        let mut all = actors();
        all[2] = actor();
        all[15] = actor();
        {
            let actor_value = SOLID_HANDLER;
            all[15].set_handler(actor_value);
        };
        all[15].position.world_x += u16::from(dx);
        all[15].position.screen_y += dy;
        assert_eq!(
            solid_overlap(2, &all, &geometry(), ContactAxis::Horizontal).map(|c| c.penetration),
            expected
        );
    }
}
#[test]
fn vertical_solid_overlap_has_strict_width_and_inclusive_height_with_zero_penetration() {
    for (dx, dy, expected) in [
        (0, TEST_HEIGHT, Some(0)),
        (0, TEST_HEIGHT + 1, None),
        (TEST_WIDTH, 0, None),
        (TEST_WIDTH - 1, 0, Some(TEST_HEIGHT)),
    ] {
        let mut all = actors();
        all[2] = actor();
        all[15] = actor();
        {
            let actor_value = SOLID_HANDLER;
            all[15].set_handler(actor_value);
        };
        all[15].position.world_x += u16::from(dx);
        all[15].position.screen_y += dy;
        assert_eq!(
            solid_overlap(2, &all, &geometry(), ContactAxis::Vertical).map(|c| c.penetration),
            expected
        );
    }
}
#[test]
fn solid_overlap_uses_target_profile_not_source_and_does_not_require_profile_enable_bit() {
    let mut g = geometry();
    g.extents[1] = Extent {
        half_width: 2,
        half_height: 3,
    };
    let mut all = actors();
    all[2] = actor();
    all[2].profile = 1;
    all[15] = actor();
    all[15].profile = 0;
    {
        let actor_value = SOLID_HANDLER;
        all[15].set_handler(actor_value);
    };
    all[15].position.world_x += 5;
    assert_eq!(
        solid_overlap(2, &all, &g, ContactAxis::Horizontal),
        Some(Contact {
            slot: 15,
            penetration: 5
        })
    );
}
#[test]
fn world_x_contact_selects_absolute_low_byte_using_full_unsigned_borrow_without_high_distance_rejection()
 {
    let mut all = actors();
    all[2] = actor();
    all[15] = actor();
    {
        let actor_value = SOLID_HANDLER;
        all[15].set_handler(actor_value);
    };
    all[2].position.world_x = 0x0200;
    all[15].position.world_x = 0x0101;
    assert_eq!(
        solid_overlap(2, &all, &geometry(), ContactAxis::Horizontal),
        None
    ); // low differenceFF, not signed -1
    all[2].position.world_x = 0x0101;
    all[15].position.world_x = 0x0200;
    assert_eq!(
        solid_overlap(2, &all, &geometry(), ContactAxis::Horizontal),
        None
    ); // negated low01 ->FF
    all[15].position.world_x = 0x0301;
    assert_eq!(
        solid_overlap(2, &all, &geometry(), ContactAxis::Horizontal),
        Some(Contact {
            slot: 15,
            penetration: TEST_WIDTH
        })
    );
}
#[test]
fn player_horizontal_scan_uses_render_profile_enable_but_not_state_or_vertical_skip_bit() {
    let mut all = actors();
    all[0] = actor();
    all[15] = actor();
    {
        let actor_value = 0;
        all[15].set_state_code(actor_value);
    };
    all[15].render |= SKIP_VERTICAL_PLAYER_SCAN;
    let r = scan_player(
        ContactAxis::Horizontal,
        &mut all,
        &mut PlayerContact::default(),
        &geometry(),
    );
    assert!(r.any_contact);
    assert_eq!(
        r.last_contact,
        Some(Contact {
            slot: 15,
            penetration: TEST_WIDTH
        })
    );
    assert_eq!(r.bits, 1);
    all[15].profile = 0;
    assert!(
        !scan_player(
            ContactAxis::Horizontal,
            &mut all,
            &mut PlayerContact::default(),
            &geometry()
        )
        .any_contact
    );
}
#[test]
fn player_horizontal_protection_keeps_only_handlers_eight_and_fifteen_hex() {
    for (handler, hit) in [
        (SOLID_HANDLER, true),
        (PROTECTION_CONTACT_EXCEPTION, true),
        (0x42, false),
        (0, false),
    ] {
        let mut all = actors();
        all[0] = actor();
        all[15] = actor();
        {
            let actor_value = handler;
            all[15].set_handler(actor_value);
        };
        let mut p = PlayerContact {
            protection: 1,
            ..Default::default()
        };
        assert_eq!(
            scan_player(ContactAxis::Horizontal, &mut all, &mut p, &geometry()).any_contact,
            hit
        );
    }
}
#[test]
fn player_horizontal_scan_stops_highest_contact_and_flagged_contact_writes_immediate_damage() {
    let mut all = actors();
    all[0] = actor();
    all[15] = actor();
    all[15].profile |= 0x10;
    all[1] = all[15];
    let mut p = PlayerContact {
        queued_state: POSE_TERRAIN_STATE,
        ..Default::default()
    };
    let r = scan_player(ContactAxis::Horizontal, &mut all, &mut p, &geometry());
    assert_eq!(r.last_contact.unwrap().slot, 15);
    assert_eq!(p.immediate_state, DAMAGE_STATE);
    assert_eq!(p.queued_state, POSE_TERRAIN_STATE);
    assert_eq!(all[15].hit_marker, 0);
}
#[test]
fn player_horizontal_special_mode_increments_marker_with_wrap_instead_of_damage() {
    let mut all = actors();
    all[0] = actor();
    all[15] = actor();
    all[15].profile |= 0x10;
    all[15].hit_marker = 0xff;
    let mut p = PlayerContact {
        special_mode: 1,
        ..Default::default()
    };
    assert!(scan_player(ContactAxis::Horizontal, &mut all, &mut p, &geometry()).any_contact);
    assert_eq!(all[15].hit_marker, 0);
    assert_eq!(p.immediate_state, 0);
}
#[test]
fn player_pose_adjustment_adds_eight_due_to_compare_carry_and_only_flagged_scan_profiles() {
    for (flags, dy, hit) in [
        (0, TEST_HEIGHT - 1, true),
        (0x10, 11, true),
        (0x10, 12, false),
    ] {
        let mut all = actors();
        all[0] = actor();
        all[15] = actor();
        all[15].profile |= flags;
        all[15].position.screen_y += dy;
        let mut p = PlayerContact {
            animation: SPECIAL_POSE,
            ..Default::default()
        };
        assert_eq!(
            scan_player(ContactAxis::Horizontal, &mut all, &mut p, &geometry()).any_contact,
            hit
        );
    }
}
#[test]
fn player_vertical_scan_skips_render_bit_one_and_disabled_profiles_but_ignores_state_activity() {
    for (render, profile, state, hit) in [
        (VISIBLE, PROFILE_ENABLED, 0, true),
        (
            VISIBLE | SKIP_VERTICAL_PLAYER_SCAN,
            PROFILE_ENABLED,
            ACTIVE,
            false,
        ),
        (VISIBLE, 0, ACTIVE, false),
        (0, PROFILE_ENABLED, ACTIVE, false),
    ] {
        let mut all = actors();
        all[0] = actor();
        all[15] = actor();
        all[15].render = render;
        all[15].profile = profile;
        {
            let actor_value = state;
            all[15].set_state_code(actor_value);
        };
        assert_eq!(
            scan_player(
                ContactAxis::Vertical,
                &mut all,
                &mut PlayerContact::default(),
                &geometry()
            )
            .any_contact,
            hit
        );
    }
}
#[test]
fn player_vertical_scan_counts_every_contact_and_returns_lowest_slots_final_penetration() {
    let mut all = actors();
    all[0] = actor();
    all[15] = actor();
    all[1] = actor();
    all[1].position.screen_y += TEST_HEIGHT;
    let r = scan_player(
        ContactAxis::Vertical,
        &mut all,
        &mut PlayerContact::default(),
        &geometry(),
    );
    assert_eq!(r.contact_count, 2);
    assert_eq!(
        r.last_contact,
        Some(Contact {
            slot: 1,
            penetration: 0
        })
    );
    assert_eq!(r.bits, 1);
    assert!(r.any_contact);
}
#[test]
fn player_vertical_protection_rejects_reaction_profiles_but_preserves_ordinary_platform_contact() {
    let mut all = actors();
    all[0] = actor();
    all[15] = actor();
    all[15].profile |= 0x10;
    all[1] = actor();
    let mut p = PlayerContact {
        protection: 1,
        ..Default::default()
    };
    let r = scan_player(ContactAxis::Vertical, &mut all, &mut p, &geometry());
    assert_eq!(r.contact_count, 1);
    assert_eq!(r.last_contact.unwrap().slot, 1);
    assert_eq!(p.immediate_state, 0);
}
#[test]
fn player_vertical_pogo_marks_only_first_contact_and_requires_all_three_motion_gates() {
    for (velocity, animation, motion, marks) in [
        (-1, POGO_POSE, DESCENDING_MOTION, true),
        (0, POGO_POSE, DESCENDING_MOTION, false),
        (-1, 0, DESCENDING_MOTION, false),
        (-1, POGO_POSE, 0, false),
    ] {
        let mut all = actors();
        all[0] = actor();
        all[15] = actor();
        all[1] = actor();
        let mut p = PlayerContact {
            vertical_velocity: velocity,
            animation,
            motion,
            ..Default::default()
        };
        let r = scan_player(ContactAxis::Vertical, &mut all, &mut p, &geometry());
        assert_eq!(r.contact_count, 2);
        assert_eq!(all[15].hit_marker, u8::from(marks));
        assert_eq!(all[1].hit_marker, 0);
        assert_eq!(p.actor_contact_flags, u8::from(marks));
    }
}
#[test]
fn pogo_hit_marker_wrap_falls_through_to_flagged_reaction_in_the_same_contact() {
    let mut all = actors();
    all[0] = actor();
    all[15] = actor();
    all[15].profile |= 0x10;
    all[15].hit_marker = 0xff;
    let mut p = PlayerContact {
        vertical_velocity: -1,
        animation: POGO_POSE,
        motion: DESCENDING_MOTION,
        special_mode: 1,
        ..Default::default()
    };
    assert!(scan_player(ContactAxis::Vertical, &mut all, &mut p, &geometry()).any_contact);
    assert_eq!(all[15].hit_marker, 1);
    assert_eq!(p.actor_contact_flags, 1);
    assert_eq!(p.immediate_state, 0);
}
#[test]
fn vertical_flagged_contact_after_first_pogo_still_applies_immediate_damage_or_special_hit() {
    for special_mode in [0, 1] {
        let mut all = actors();
        all[0] = actor();
        all[15] = actor();
        all[1] = actor();
        all[1].profile |= 0x10;
        let mut p = PlayerContact {
            vertical_velocity: -1,
            animation: POGO_POSE,
            motion: DESCENDING_MOTION,
            special_mode,
            ..Default::default()
        };
        scan_player(ContactAxis::Vertical, &mut all, &mut p, &geometry());
        assert_eq!(all[15].hit_marker, 1);
        assert_eq!(all[1].hit_marker, special_mode);
        assert_eq!(
            p.immediate_state,
            if special_mode == 0 { DAMAGE_STATE } else { 0 }
        );
    }
}
#[test]
fn generic_player_predicate_has_strict_y_inclusive_x_and_does_not_require_enabled_profile_or_active_state()
 {
    for (dx, dy, hit) in [
        (TEST_WIDTH, 0, true),
        (TEST_WIDTH + 1, 0, false),
        (0, TEST_HEIGHT, false),
        (0, TEST_HEIGHT - 1, true),
    ] {
        let mut a = actor();
        {
            let actor_value = 0;
            a.set_state_code(actor_value);
        };
        a.profile = 0;
        let mut pos = a.position;
        pos.world_x += u16::from(dx);
        pos.screen_y += dy;
        assert_eq!(
            actor_player_contact(&mut a, pos, &mut PlayerContact::default(), &geometry()),
            hit
        );
    }
}
#[test]
fn generic_player_special_pose_adjusts_even_ordinary_profiles_by_eight_not_seven() {
    for (dy, hit) in [(11, true), (12, false)] {
        let mut a = actor();
        a.profile = 0;
        let mut pos = a.position;
        pos.screen_y += dy;
        let mut p = PlayerContact {
            animation: SPECIAL_POSE,
            ..Default::default()
        };
        assert_eq!(actor_player_contact(&mut a, pos, &mut p, &geometry()), hit);
    }
}
#[test]
fn generic_player_protection_is_only_a_damage_gate_and_unflagged_contact_still_returns_permission()
{
    for (flags, protection, permission, state) in [
        (0, 1, true, 0),
        (0x10, 1, false, 0),
        (0x10, 0, true, DAMAGE_STATE),
    ] {
        let mut a = actor();
        a.profile = flags;
        let pos = a.position;
        let mut p = PlayerContact {
            protection,
            ..Default::default()
        };
        assert_eq!(
            actor_player_contact(&mut a, pos, &mut p, &geometry()),
            permission
        );
        assert_eq!(p.immediate_state, state);
    }
}
#[test]
fn generic_player_special_mode_flagged_hit_returns_rejection_while_mutating_marker_even_when_protected()
 {
    let mut a = actor();
    a.profile |= 0x10;
    a.hit_marker = 0xff;
    let pos = a.position;
    let mut p = PlayerContact {
        special_mode: 1,
        protection: 1,
        ..Default::default()
    };
    assert!(!actor_player_contact(&mut a, pos, &mut p, &geometry()));
    assert_eq!(a.hit_marker, 0);
    assert_eq!(p.immediate_state, 0);
}
#[test]
fn generic_player_predicate_rejects_invisible_actor_before_any_hit_or_state_side_effect() {
    let mut a = actor();
    a.render = 0;
    a.profile |= 0x10;
    let old = a;
    let mut p = PlayerContact::default();
    assert!(!actor_player_contact(
        &mut a,
        old.position,
        &mut p,
        &geometry()
    ));
    assert_eq!(a, old);
    assert_eq!(p, PlayerContact::default());
}
#[test]
fn wrappers_release_full_horizontal_escape_immediately_with_matching_special_spawn_retry() {
    for (movement, x, speed, latch, expected_latch, expected_x, expected_fraction) in [
        (
            Movement::PositiveX,
            0xffff,
            0x0100,
            0x82,
            SPECIAL_SPAWN_RETRY,
            0,
            0,
        ),
        (
            Movement::NegativeX,
            0,
            1,
            0x82,
            SPECIAL_SPAWN_RETRY,
            0xffff,
            0xff,
        ),
        (Movement::PositiveX, 0xffff, 0x0100, 0x83, 0x83, 0, 0),
    ] {
        let mut all = actors();
        all[2] = actor();
        all[2].position.world_x = x;
        all[2].horizontal_magnitude = speed;
        let mut special = latch;
        let r = move_case(&mut all, movement, 3, 0, &mut special);
        assert_eq!(
            r.phases,
            vec![MovementPhase::Integrate, MovementPhase::ReleaseIdentity]
        );
        assert!(r.removed);
        assert!(r.probe.is_none());
        assert_eq!(
            (
                all[2].state_code(),
                all[2].render,
                all[2].spawn_identity,
                special
            ),
            (0, 0, None, expected_latch)
        );
        assert_eq!(
            (all[2].position.world_x, all[2].position.x_fraction),
            (expected_x, expected_fraction)
        );
    }
}
#[test]
fn horizontal_wrappers_apply_terrain_snap_then_facing_and_actor_penetration() {
    for (movement, expected_x, facing) in [
        (Movement::PositiveX, 0x0156, FACE_POSITIVE),
        (Movement::NegativeX, 0x017a, 0),
    ] {
        let mut all = actors();
        all[2] = actor();
        all[2].render |= FACE_POSITIVE;
        all[15] = actor();
        {
            let actor_value = SOLID_HANDLER;
            all[15].set_handler(actor_value);
        };
        // Start160, nibble0: +X snap stays160, actor penetration10 ->156.
        // -X snap becomes170, target170 gives penetration10 ->17A.
        if movement == Movement::NegativeX {
            all[15].position.world_x = 0x0170;
        }
        let r = move_case(&mut all, movement, 1, 0, &mut 0);
        assert_eq!(
            r.phases,
            vec![
                MovementPhase::Integrate,
                MovementPhase::Terrain,
                MovementPhase::SolidActors
            ]
        );
        assert_eq!(all[2].position.world_x, expected_x);
        assert_eq!(all[2].render & FACE_POSITIVE, facing);
        assert_eq!(r.contact.unwrap().slot, 15);
    }
}
#[test]
fn horizontal_wrapper_exemptions_are_directional_for_handler_twentyfour_hex() {
    for (handler, movement, scanned) in [
        (OVERLAP_EXEMPT_HANDLER, Movement::PositiveX, false),
        (OVERLAP_EXEMPT_HANDLER, Movement::NegativeX, false),
        (POSITIVE_ONLY_EXEMPT_HANDLER, Movement::PositiveX, false),
        (POSITIVE_ONLY_EXEMPT_HANDLER, Movement::NegativeX, true),
    ] {
        let mut all = actors();
        all[2] = actor();
        {
            let actor_value = handler;
            all[2].set_handler(actor_value);
        };
        all[15] = actor();
        {
            let actor_value = SOLID_HANDLER;
            all[15].set_handler(actor_value);
        };
        let r = move_case(&mut all, movement, 0, 0, &mut 0);
        assert_eq!(r.phases.contains(&MovementPhase::SolidActors), scanned);
        assert_eq!(r.contact.is_some(), scanned);
    }
}
#[test]
fn player_horizontal_wrapper_skips_actor_scan_but_still_sets_facing_and_terrain_contact() {
    let mut all = actors();
    all[0] = actor();
    all[15] = actor();
    {
        let actor_value = SOLID_HANDLER;
        all[15].set_handler(actor_value);
    };
    let r = move_with_collision(
        0,
        &mut all,
        &mut PlayerContact::default(),
        &cache(1),
        TEST_ROW,
        &geometry(),
        ONE_SAMPLE,
        Movement::PositiveX,
        0,
        &mut 0,
        &mut |_| 0,
    );
    assert!(!r.phases.contains(&MovementPhase::SolidActors));
    assert_eq!(all[0].render & FACE_POSITIVE, FACE_POSITIVE);
    assert_eq!(r.probe.unwrap().bits, 1);
}
#[test]
fn direct_y_wrappers_remove_on_unsigned_edge_and_otherwise_snap_without_gravity_or_actor_scan() {
    for (movement, y, velocity, removed, expected_y) in [
        (Movement::PositiveY, 0xff, 0x0100, true, 0),
        (Movement::NegativeY, 0, 0x0100, true, 0xff),
        (Movement::PositiveY, TEST_Y, 0, false, TEST_Y),
        (Movement::NegativeY, TEST_Y, 0, false, TEST_Y + 16),
    ] {
        let mut all = actors();
        all[2] = actor();
        all[2].position.screen_y = y;
        all[2].vertical_velocity = velocity;
        let r = move_case(&mut all, movement, 1, 0x40, &mut 0);
        assert_eq!(r.removed, removed);
        assert_eq!(all[2].position.screen_y, expected_y);
        assert_eq!(all[2].vertical_velocity, velocity);
        assert!(!r.phases.contains(&MovementPhase::Gravity));
        assert!(!r.phases.contains(&MovementPhase::SolidActors));
    }
}
#[test]
fn falling_applies_gravity_before_motion_and_preserves_probe_independent_borrow_boundary() {
    let mut all = actors();
    all[2] = actor();
    all[2].profile = 0;
    all[2].vertical_velocity = 0;
    let r = move_case(&mut all, Movement::Fall, 0, 0x40, &mut 0);
    assert_eq!(
        r.phases,
        vec![
            MovementPhase::Gravity,
            MovementPhase::Integrate,
            MovementPhase::Terrain
        ]
    );
    assert!(!r.removed);
    assert_eq!(
        (
            all[2].position.screen_y,
            all[2].position.y_fraction,
            all[2].vertical_velocity
        ),
        (TEST_Y, 0x40, -0x40)
    );
    all[2].position.screen_y = 0xff;
    all[2].position.y_fraction = 0;
    all[2].vertical_velocity = -0x0100;
    let r = move_case(&mut all, Movement::Fall, 3, 0, &mut 0);
    assert!(r.removed); // no borrow even though negative signed velocity
    assert_eq!(
        r.phases,
        vec![
            MovementPhase::Gravity,
            MovementPhase::Integrate,
            MovementPhase::Terrain,
            MovementPhase::ReleaseIdentity
        ]
    );
}
#[test]
fn falling_terrain_landing_snaps_integer_and_clears_velocity_without_clearing_position_fraction() {
    let mut all = actors();
    all[2] = actor();
    all[2].position.screen_y = 0x65;
    all[2].vertical_velocity = -0x0040;
    let r = move_case(&mut all, Movement::Fall, 1, 0, &mut 0);
    assert!(!r.removed);
    assert_eq!(
        (
            all[2].position.screen_y,
            all[2].position.y_fraction,
            all[2].vertical_velocity
        ),
        (0x60, 0x40, 0)
    );
    assert!(!r.phases.contains(&MovementPhase::SolidActors));
}
#[test]
fn falling_actor_landing_subtracts_penetration_and_zero_profile_or_handler_nine_bypass_it() {
    for (profile, handler, land) in [
        (PROFILE_ENABLED, 0, true),
        (0, 0, false),
        (PROFILE_ENABLED, OVERLAP_EXEMPT_HANDLER, false),
    ] {
        let mut all = actors();
        all[2] = actor();
        all[2].profile = profile;
        {
            let actor_value = handler;
            all[2].set_handler(actor_value);
        };
        all[2].vertical_velocity = -0x0100;
        all[15] = actor();
        {
            let actor_value = SOLID_HANDLER;
            all[15].set_handler(actor_value);
        };
        all[15].position.screen_y = TEST_Y + 1;
        let r = move_case(&mut all, Movement::Fall, 0, 0, &mut 0);
        assert_eq!(r.contact.is_some(), land);
        assert_eq!(
            all[2].position.screen_y,
            if land {
                TEST_Y + 1 - TEST_HEIGHT
            } else {
                TEST_Y + 1
            }
        );
        assert_eq!(all[2].vertical_velocity, if land { 0 } else { -0x0100 });
    }
}
#[test]
fn rising_integrates_before_gravity_ignores_unsigned_boundary_and_sets_downward_velocity_on_terrain()
 {
    let mut all = actors();
    all[2] = actor();
    all[2].vertical_velocity = 0x0100;
    all[2].position.screen_y = 0;
    let r = move_case(&mut all, Movement::Rise, 1, 0x40, &mut 0);
    assert_eq!(
        r.phases,
        vec![
            MovementPhase::Integrate,
            MovementPhase::Terrain,
            MovementPhase::Gravity
        ]
    );
    assert!(!r.removed);
    assert_eq!(
        (all[2].position.screen_y, all[2].vertical_velocity),
        (0, -0x0100)
    ); // FF snap +1 wraps0
    assert_eq!(all[2].state_code(), ACTIVE);
}
#[test]
fn rising_actor_contact_adds_penetration_plus_one_and_sets_ff00_not_zero_velocity() {
    let mut all = actors();
    all[2] = actor();
    all[2].vertical_velocity = 0x0100;
    all[15] = actor();
    {
        let actor_value = SOLID_HANDLER;
        all[15].set_handler(actor_value);
    };
    all[15].position.screen_y = TEST_Y - 1;
    let r = move_case(&mut all, Movement::Rise, 0, 0x40, &mut 0);
    assert_eq!(
        r.phases,
        vec![
            MovementPhase::Integrate,
            MovementPhase::Terrain,
            MovementPhase::Gravity,
            MovementPhase::SolidActors
        ]
    );
    assert_eq!(all[2].position.screen_y, TEST_Y + TEST_HEIGHT);
    assert_eq!(all[2].vertical_velocity, -0x0100);
    assert_eq!(
        r.contact,
        Some(Contact {
            slot: 15,
            penetration: TEST_HEIGHT
        })
    );
}
#[test]
fn rising_without_contact_keeps_postintegration_gravity_and_profile_zero_bypasses_only_actor_scan()
{
    let mut all = actors();
    all[2] = actor();
    all[2].profile = 0;
    all[2].vertical_velocity = 0x0100;
    let r = move_case(&mut all, Movement::Rise, 0, 0x40, &mut 0);
    assert_eq!(all[2].position.screen_y, TEST_Y - 1);
    assert_eq!(all[2].vertical_velocity, 0x00c0);
    assert!(!r.phases.contains(&MovementPhase::SolidActors));
}
#[test]
fn ordinary_transition_clear_preserves_slots_zero_one_and_clears_only_activity_render_identity_for_two_through_fifteen()
 {
    let mut all = [actor(); ACTOR_SLOTS];
    for a in &mut all {
        {
            let actor_value = 9;
            a.set_local_state_code(actor_value);
        };
        {
            let actor_value = 2;
            a.set_local_counter_code(actor_value);
        };
        a.hit_marker = 7;
    }
    let old = all;
    clear_transition_actors(&mut all, 0, 0);
    assert_eq!(all[0], old[0]);
    assert_eq!(all[1], old[1]);
    for actor in all.iter().skip(2) {
        assert_eq!(
            (actor.state_code(), actor.render, actor.spawn_identity),
            (0, 0, None)
        );
        assert_eq!(
            (
                actor.local_state_code(),
                actor.local_counter_code(),
                actor.hit_marker
            ),
            (9, 2, 7)
        );
    }
}
#[test]
fn stage_one_first_two_rows_preserve_every_actor_and_mark_only_fourteen_fifteen() {
    for row in [0, 1] {
        let mut all = [actor(); ACTOR_SLOTS];
        let old = all;
        clear_transition_actors(&mut all, 1, row);
        assert_eq!(&all[..14], &old[..14]);
        for slot in [14, 15] {
            let mut expected = old[slot];
            {
                let actor_value = expected.state_code() | (PRESERVED_ACTOR_MARK);
                expected.set_state_code(actor_value);
            };
            assert_eq!(all[slot], expected);
        }
    }
    let mut all = [actor(); ACTOR_SLOTS];
    clear_transition_actors(&mut all, 1, 2);
    assert_eq!(all[15].state_code(), 0);
}
#[test]
fn preserved_actor_timer_uses_literal_sixty_and_extra_sixteen_borrow_correction() {
    for (low, high, expected_low, expected_high) in [
        (0, 2, 0xb4, 1),
        (59, 2, 0xef, 1),
        (60, 2, 0, 2),
        (61, 2, 1, 2),
        (19, 0, 19, 0),
        (0, 1, 0xb4, 0),
    ] {
        let mut all = actors();
        all[14] = actor();
        all[15] = actor();
        {
            let actor_value = low;
            all[15].set_local_state_code(actor_value);
        };
        {
            let actor_value = high;
            all[15].set_local_counter_code(actor_value);
        };
        clear_transition_actors(&mut all, 1, 0);
        assert_eq!(
            (all[15].local_state_code(), all[15].local_counter_code()),
            (expected_low, expected_high)
        );
        assert_eq!(
            (all[14].local_state_code(), all[14].local_counter_code()),
            (0, 0)
        );
    }
}
#[test]
fn particle_batch_allocates_highest_inactive_slots_in_reverse_parameter_order_and_initializes_only_source_fields()
 {
    let mut all = [actor(); ACTOR_SLOTS];
    for slot in [15, 12, 7, 1] {
        {
            let actor_value = 0x7f;
            all[slot].set_state_code(actor_value);
        };
        all[slot].position.x_fraction = 9;
        all[slot].position.y_fraction = 11;
        all[slot].vertical_velocity = 0x1234;
        all[slot].hit_marker = 0x55;
    }
    let mut g = geometry();
    g.particle_y = [1, 2, 3, 4];
    g.particle_x = [5, 6, 7, 8];
    g.particle_ticks = [9, 10, 11, 12];
    let r = terrain_particles(&mut all, &g, 100, 200, 3);
    assert_eq!(r.slots, vec![15, 12, 7, 1]);
    assert_eq!(r.audio_commands, vec![TERRAIN_PARTICLE_SOUND]);
    for (slot, y, x, tick) in [
        (15, 104, 208, 12),
        (12, 103, 207, 11),
        (7, 102, 206, 10),
        (1, 101, 205, 9),
    ] {
        let a = all[slot];
        assert_eq!(
            (
                a.position.world_x,
                a.position.screen_y,
                a.playback.tick_code()
            ),
            (0x0300 | x, y, tick)
        );
        assert_eq!(
            (
                a.state_code(),
                a.render,
                a.animation,
                a.playback.frame_code(),
                a.profile,
                a.handler
            ),
            (
                TERRAIN_PARTICLE_STATE,
                TERRAIN_PARTICLE_RENDER,
                TERRAIN_PARTICLE_ANIMATION,
                0,
                0,
                0
            )
        );
        assert_eq!(
            (
                a.position.x_fraction,
                a.position.y_fraction,
                a.vertical_velocity,
                a.hit_marker,
                a.spawn_identity
            ),
            (9, 11, 0x1234, 0x55, Some(7))
        );
    }
}
#[test]
fn particles_carry_screen_y_overflow_into_x_but_never_carry_x_into_player_page() {
    let mut all = [actor(); ACTOR_SLOTS];
    {
        let actor_value = 0;
        all[15].set_state_code(actor_value);
    };
    let mut g = geometry();
    g.particle_y[3] = 16;
    g.particle_x[3] = 16;
    let r = terrain_particles(&mut all, &g, 0xf8, 0xf8, 4);
    assert_eq!(r.slots, vec![15]);
    assert_eq!(
        (all[15].position.screen_y, all[15].position.world_x),
        (8, 0x0409)
    );
}
#[test]
fn particle_allocation_excludes_player_stops_at_first_full_pool_and_always_queues_sound() {
    for free in [0, 1, 3] {
        let mut all = [actor(); ACTOR_SLOTS];
        {
            let actor_value = 0;
            all[0].set_state_code(actor_value);
        };
        for actor in all.iter_mut().take(free + 1).skip(1) {
            {
                let actor_value = 0;
                actor.set_state_code(actor_value);
            };
        }
        let old_player = all[0];
        let r = terrain_particles(&mut all, &geometry(), TEST_Y, 0x60, 1);
        assert_eq!(r.slots.len(), free);
        assert_eq!(r.audio_commands, vec![TERRAIN_PARTICLE_SOUND]);
        assert_eq!(all[0], old_player);
    }
}
#[test]
fn momentum_wrap_sample_itself_is_transparent_before_a_following_mark_only_sample() {
    let mut g = geometry();
    g.responses[0] = TerrainResponse::Momentum;
    g.responses[1] = TerrainResponse::Mark;
    let mut a = actor();
    let mut p = PlayerContact {
        momentum_control: 0xff,
        ..Default::default()
    };
    let mut descriptors = [0x00, 0x10].into_iter();
    let r = probe(
        0,
        &mut a,
        &mut p,
        &cache(3),
        TEST_ROW,
        &g,
        ProbeRange {
            start: 0,
            remaining: 1,
        },
        &mut |_| descriptors.next().unwrap(),
    );
    assert_eq!(r.bits, 0);
    assert_eq!(p.momentum_control, 0);
    assert_eq!(p.queued_state, 0);
    assert_eq!(a.render & TERRAIN_MARK, TERRAIN_MARK);
}
#[test]
fn player_vertical_pose_extent_is_inclusive_after_the_same_eight_pixel_carry_adjustment() {
    for (dy, expected) in [(12, true), (13, false)] {
        let mut all = actors();
        all[0] = actor();
        all[15] = actor();
        all[15].profile |= 0x10;
        all[15].position.screen_y += dy;
        let mut p = PlayerContact {
            animation: SPECIAL_POSE,
            ..Default::default()
        };
        assert_eq!(
            scan_player(ContactAxis::Vertical, &mut all, &mut p, &geometry()).any_contact,
            expected
        );
    }
}
#[test]
fn generic_pose_adjustment_wraps_eight_bit_separation_instead_of_saturating() {
    let mut a = actor();
    a.profile = 0;
    a.position.screen_y = 0xf8;
    let mut pos = a.position;
    pos.screen_y = 0;
    let mut p = PlayerContact {
        animation: SPECIAL_POSE,
        ..Default::default()
    };
    assert!(actor_player_contact(&mut a, pos, &mut p, &geometry())); // abs248+7+C1=256 ->0
}
#[test]
fn falling_positive_encoded_velocity_is_removed_on_no_borrow_even_without_physical_escape() {
    let mut all = actors();
    all[2] = actor();
    all[2].vertical_velocity = 0x0100;
    let r = move_case(&mut all, Movement::Fall, 0, 0, &mut 0);
    assert!(r.removed);
    assert_eq!(all[2].position.screen_y, TEST_Y - 1);
    assert_eq!(all[2].state_code(), 0);
}
#[test]
fn rising_no_contact_keeps_wrapped_y_instead_of_using_falling_lifetime_policy() {
    let mut all = actors();
    all[2] = actor();
    all[2].profile = 0;
    all[2].position.screen_y = 0;
    all[2].vertical_velocity = 0x0100;
    let r = move_case(&mut all, Movement::Rise, 0, 0, &mut 0);
    assert!(!r.removed);
    assert_eq!(all[2].position.screen_y, 0xff);
    assert_eq!(all[2].state_code(), ACTIVE);
}
#[test]
fn collision_recipe_rejects_truncation_before_any_required_table_or_preset_word() {
    let bytes = rom();
    for length in [
        0,
        PARTICLES_PHYSICAL_START + 11,
        SAMPLE_X_PHYSICAL_START + SAMPLE_GEOMETRIES - 1,
        PRESET_CONSUMER_PHYSICAL_START + PRESET_CONSUMER_BYTES - 1,
        WIDTHS_PHYSICAL_START + COLLISION_PROFILES - 1,
    ] {
        assert!(
            Geometry::original(&bytes[..length]).is_err(),
            "bounded loader accepted length{length}"
        );
    }
}

// B09 actual A04C repeats positive movement with A1 and this returned quantity.
// Ordinaryclass1 leaves initial count0 in Y; class3 Solid response leaves the
// high-nibble response selector5. F10D handler24 exemption then preserves it.
#[test]
fn external_positive_wrapper_returns_actual_repeat_count_after_ordinary_or_special_solid() {
    const SPECIAL_RESPONSE_SELECTOR: u8 = 5;
    for (class, expected) in [(1, 0), (3, SPECIAL_RESPONSE_SELECTOR)] {
        let mut all = actors();
        all[2] = actor();
        {
            let actor_value = POSITIVE_ONLY_EXEMPT_HANDLER;
            all[2].set_handler(actor_value);
        };
        let result = move_with_collision(
            2,
            &mut all,
            &mut PlayerContact::default(),
            &cache(class),
            TEST_ROW,
            &geometry(),
            ONE_SAMPLE,
            Movement::PositiveX,
            0,
            &mut 0,
            &mut |_| SPECIAL_RESPONSE_SELECTOR << 4,
        );
        assert_eq!(result.repeat_positive_count, Some(expected));
        assert!(!result.removed);
        assert_eq!(result.probe.unwrap().bits & SOLID_CONTACT, SOLID_CONTACT);
    }
}

#[test]
fn sample_result_one_authors_the_canonical_player_contact_alias() {
    for slot in [0, 2] {
        let mut a = actor();
        let mut p = PlayerContact {
            actor_contact_flags: 0x80,
            ..PlayerContact::default()
        };
        let report = probe(
            slot,
            &mut a,
            &mut p,
            &cache(2),
            TEST_ROW,
            &geometry(),
            ProbeRange {
                start: 0,
                remaining: 2,
            },
            &mut |_| 0,
        );
        assert!(report.results.contains(&(1, 2)));
        assert_eq!(p.actor_contact_flags, 2);
        probe(
            slot,
            &mut a,
            &mut p,
            &cache(0),
            TEST_ROW,
            &geometry(),
            ONE_SAMPLE,
            &mut |_| 0,
        );
        assert_eq!(
            p.actor_contact_flags, 2,
            "a range omitting result1 retains its canonical byte"
        );
    }
}

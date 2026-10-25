//! B04 source-backed contracts; see trace.md. No original payload is embedded.
use super::*;
use crate::{assets::formats::UploadCommit, config};
use romatlas_port::assets::Rom;
const FIXED_BANK_PHYSICAL: usize = 0x1c010;
const CPU_FIXED_ORIGIN: usize = 0xc000;
const ROOM_PAYLOAD_PHYSICAL: usize = 0x10010;
const TERRAIN_BANK_BYTES: usize = 0x4000;
const TERRAIN_BLOCK_CPU: usize = 0xb300;
const SWITCH_CPU_ORIGIN: usize = 0x8000;
const ROM_HEADER: usize = 16;
const FOURTH_STAGE: u8 = 3;
const FIFTH_STAGE: u8 = 4;
const SCENE_LAYOUT: u8 = 5;
const FORWARD_HELD: u8 = 1;
const BACKWARD_HELD: u8 = 2;
fn rom() -> Vec<u8> {
    let bytes =
        std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../input/game.rom"))
            .expect("local ROM fixture");
    Rom::from_bytes(bytes.clone(), config::ROM_SHA256).expect("validated ROM identity");
    bytes
}
fn fixed(rom: &[u8], address: usize, count: usize) -> &[u8] {
    let start = FIXED_BANK_PHYSICAL + address - CPU_FIXED_ORIGIN;
    &rom[start..start + count]
}
fn layout() -> Layout {
    Layout {
        rows: 3,
        columns: 8,
        rooms: vec![Some(0); 24],
    }
}
fn terrain() -> TerrainSet {
    let mut set = TerrainSet {
        blocks: vec![[1, 2, 3, 4]; BLOCK_DESCRIPTORS],
        tiles: [[0; 4]; TERRAIN_SELECTORS],
        properties: [0; TERRAIN_SELECTORS],
    };
    set.tiles[1] = [11, 12, 13, 14];
    set.tiles[2] = [21, 22, 23, 24];
    set.tiles[3] = [31, 32, 33, 34];
    set.tiles[4] = [41, 42, 43, 44];
    set.properties[1] = 0x10;
    set.properties[2] = 0x21;
    set.properties[3] = 0x32;
    set.properties[4] = 0x03;
    set.tiles[0x88] = [81, 82, 83, 84];
    set.tiles[0xd6] = [61, 62, 63, 64];
    set.tiles[0xd7] = [71, 72, 73, 74];
    set
}
fn expanded() -> ExpandedBlock {
    ExpandedBlock {
        tiles: [
            11, 12, 21, 22, 13, 14, 23, 24, 31, 32, 41, 42, 33, 34, 43, 44,
        ],
        palette: 0xe4,
        collision: 0x39,
        cache_index: 1,
    }
}
fn context() -> ExpansionContext {
    ExpansionContext {
        ring: 8,
        room_in_row: 1,
        ..ExpansionContext::default()
    }
}
fn player(x: u16) -> CameraPlayer {
    CameraPlayer { x, fraction: 0 }
}
fn camera_input() -> CameraInput {
    CameraInput {
        active: true,
        automatic: false,
        departure: 0,
        held: 0,
        late: 0,
    }
}
fn camera(x: u16, room: u8) -> Camera {
    Camera {
        x,
        room,
        previous_x: x as u8,
    }
}
#[test]
fn all_ten_layouts_have_source_dimensions_and_exact_independent_lists() {
    let bytes = rom();
    let catalog = WorldCatalog::original(&bytes).unwrap();
    let dimensions = [
        (7, 8),
        (8, 8),
        (6, 8),
        (6, 8),
        (6, 8),
        (1, 1),
        (1, 1),
        (1, 1),
        (1, 4),
        (1, 6),
    ];
    let addresses = [
        0xce6b, 0xcea3, 0xcee3, 0xcf13, 0xcf43, 0xcf73, 0xcf73, 0xcf74, 0xcf75, 0xcf79,
    ];
    assert_eq!(catalog.layouts.len(), ORIGINAL_LAYOUTS);
    for (selector, ((rows, columns), address)) in dimensions.into_iter().zip(addresses).enumerate()
    {
        let decoded = &catalog.layouts[selector];
        assert_eq!(
            (decoded.rows, decoded.columns),
            (rows, columns),
            "layout {selector}"
        );
        let raw = fixed(&bytes, address, usize::from(rows) * usize::from(columns));
        let expected: Vec<_> = raw
            .iter()
            .map(|&r| {
                if r == MISSING_ROOM {
                    None
                } else {
                    Some(usize::from(r))
                }
            })
            .collect();
        assert_eq!(decoded.rooms, expected, "entire source list {selector}");
    }
}
#[test]
fn scene_and_ending_layouts_do_not_require_nonexistent_tenth_stage_configuration() {
    let catalog = WorldCatalog::original(&rom()).unwrap();
    assert_eq!(catalog.layouts[5].rooms, vec![Some(0)]);
    assert_eq!(catalog.layouts[6].rooms, vec![Some(0)]);
    assert_eq!(catalog.layouts[7].rooms, vec![Some(195)]);
    assert_eq!(
        catalog.layouts[8].rooms,
        vec![Some(196), Some(197), Some(198), Some(199)]
    );
    assert_eq!(
        catalog.layouts[9].rooms,
        vec![
            Some(114),
            Some(115),
            Some(116),
            Some(43),
            Some(17),
            Some(19)
        ]
    );
}
#[test]
fn every_room_including_uniform_fragment_bytes_has_exact_72_byte_extent() {
    let bytes = rom();
    let catalog = WorldCatalog::original(&bytes).unwrap();
    assert_eq!(catalog.rooms.len(), ORIGINAL_ROOMS);
    for (index, room) in catalog.rooms.iter().enumerate() {
        let start = ROOM_PAYLOAD_PHYSICAL + index * ROOM_BYTES;
        assert_eq!(
            room.low.as_slice(),
            &bytes[start..start + ROOM_CELLS],
            "room {index} low selectors"
        );
        assert_eq!(
            room.ninth_bits.as_slice(),
            &bytes[start + ROOM_CELLS..start + ROOM_BYTES],
            "room {index} bitmap"
        );
    }
}
#[test]
fn all_four_terrain_banks_include_512_descriptors_and_five_selector_planes() {
    let bytes = rom();
    let catalog = WorldCatalog::original(&bytes).unwrap();
    assert_eq!(catalog.terrain.len(), 4);
    for (bank, set) in catalog.terrain.iter().enumerate() {
        assert_eq!(set.blocks.len(), BLOCK_DESCRIPTORS);
        let start = ROM_HEADER + bank * TERRAIN_BANK_BYTES + TERRAIN_BLOCK_CPU - SWITCH_CPU_ORIGIN;
        for (index, block) in set.blocks.iter().enumerate() {
            assert_eq!(
                block.as_slice(),
                &bytes[start + index * 4..start + index * 4 + 4],
                "bank {bank} descriptor {index}"
            );
        }
        let planes = start + BLOCK_DESCRIPTORS * 4;
        for selector in 0..TERRAIN_SELECTORS {
            assert_eq!(
                set.tiles[selector],
                [
                    bytes[planes + selector],
                    bytes[planes + TERRAIN_SELECTORS + selector],
                    bytes[planes + TERRAIN_SELECTORS * 2 + selector],
                    bytes[planes + TERRAIN_SELECTORS * 3 + selector]
                ]
            );
            assert_eq!(
                set.properties[selector],
                bytes[planes + TERRAIN_SELECTORS * 4 + selector]
            );
        }
    }
}
#[test]
fn room_decoder_rejects_short_or_surplus_payload_transactionally() {
    for size in [0, ROOM_CELLS, ROOM_BYTES - 1, ROOM_BYTES + 1] {
        assert!(Room::decode(&vec![0; size]).is_err(), "size {size}");
    }
}
#[test]
fn room_decoder_accepts_exact_length_and_distinguishes_bitmap_from_cells() {
    let mut bytes = [0; ROOM_BYTES];
    bytes[0] = 0x42;
    bytes[ROOM_CELLS] = 0x80;
    let room = Room::decode(&bytes).unwrap();
    assert_eq!(Room::block(0, &room), 0x142);
}
#[test]
fn ninth_bit_and_coordinate_transpose_cover_all_cells_and_both_descriptor_halves() {
    let mut room = Room {
        low: [0; ROOM_CELLS],
        ninth_bits: [0; 8],
    };
    for ring in 0..ROOM_CELLS {
        let column = ring / 8;
        let row = ring % 8;
        let coordinate = row * 8 + column;
        room.low[coordinate] = 0xff;
        room.ninth_bits[row] = 0x80 >> column;
        assert_eq!(Room::block(ring as u8, &room), 0x1ff, "ring {ring}");
        room.ninth_bits[row] = 0;
        assert_eq!(Room::block(ring as u8, &room), 0xff);
        room.low[coordinate] = 0;
    }
}
#[test]
fn room_coordinates_flatten_by_layout_stride_and_respect_ff_or_outside() {
    let mut l = layout();
    l.rooms[9] = None;
    assert_eq!(l.room_at(1, 2), Some(0));
    assert_eq!(l.room_at(1, 1), None);
    assert_eq!(l.room_at(3, 0), None);
    assert_eq!(l.room_at(0, 8), None);
}
#[test]
fn quadrant_expansion_interleaves_all_four_planes_and_packs_independent_properties() {
    let result = expand(&terrain(), 0, context());
    assert_eq!(
        result,
        ExpandedBlock {
            cache_index: 65,
            ..expanded()
        }
    );
}
#[test]
fn expansion_cache_uses_room_parity_and_transposed_cursor_for_all128_cells() {
    for parity in [0, 1] {
        for ring in 0..ROOM_CELLS {
            let result = expand(
                &terrain(),
                0,
                ExpansionContext {
                    room_in_row: parity,
                    ring: ring as u8,
                    ..context()
                },
            );
            assert_eq!(
                result.cache_index,
                usize::from(parity) * ROOM_CELLS + (ring % 8) * 8 + ring / 8
            );
        }
    }
}
#[test]
fn last_attribute_row_omits_bottom_quadrant_properties_and_upload_discards_stale_tiles() {
    let block = expand(
        &terrain(),
        0,
        ExpansionContext {
            ring: 7,
            ..context()
        },
    );
    assert_eq!((block.palette, block.collision), (0x04, 0x09));
    assert_eq!(&block.tiles[..8], &expanded().tiles[..8]);
}
#[test]
fn stage_three_override_clears_all_packed_properties_at_both_exact_coordinates() {
    for ring in [12, 13] {
        let block = expand(
            &terrain(),
            0,
            ExpansionContext {
                stage: FOURTH_STAGE,
                room: PROGRESSION_THREE_ROOM,
                stage_three_control: 1,
                ring,
                ..context()
            },
        );
        assert_eq!((block.palette, block.collision), (0, 0));
        assert_eq!(
            block.tiles,
            [
                81, 82, 81, 82, 83, 84, 83, 84, 81, 82, 81, 82, 83, 84, 83, 84
            ]
        );
    }
}
#[test]
fn stage_three_override_rejects_wrong_room_coordinate_zero_control_or_split_transition() {
    for c in [
        ExpansionContext {
            stage: FOURTH_STAGE,
            room: PROGRESSION_THREE_ROOM,
            stage_three_control: 0,
            ring: 12,
            ..context()
        },
        ExpansionContext {
            stage: FOURTH_STAGE,
            room: PROGRESSION_THREE_ROOM - 1,
            stage_three_control: 1,
            ring: 12,
            ..context()
        },
        ExpansionContext {
            stage: FOURTH_STAGE,
            room: PROGRESSION_THREE_ROOM,
            stage_three_control: 1,
            ring: 11,
            ..context()
        },
        ExpansionContext {
            stage: FOURTH_STAGE,
            room: PROGRESSION_THREE_ROOM,
            stage_three_control: 1,
            ring: 12,
            split: true,
            ..context()
        },
    ] {
        let b = expand(&terrain(), 0, c);
        assert_eq!((b.palette, b.collision), (0xe4, 0x39));
        assert_eq!(b.tiles, expanded().tiles);
    }
}
#[test]
fn stage_four_odd_quadrants_use_d6_and_even_quadrants_use_d7_without_changing_properties() {
    let left = expand(
        &terrain(),
        0,
        ExpansionContext {
            stage: FIFTH_STAGE,
            room: PROGRESSION_FOUR_ROOM,
            stage_four_control: 0x80,
            ring: 28,
            ..context()
        },
    );
    assert_eq!(
        left.tiles,
        [
            11, 12, 61, 62, 13, 14, 63, 64, 31, 32, 61, 62, 33, 34, 63, 64
        ]
    );
    assert_eq!((left.palette, left.collision), (0xe4, 0x39));
    let right = expand(
        &terrain(),
        0,
        ExpansionContext {
            stage: FIFTH_STAGE,
            room: PROGRESSION_FOUR_ROOM,
            stage_four_control: 0xff,
            ring: 36,
            ..context()
        },
    );
    assert_eq!(
        right.tiles,
        [
            71, 72, 21, 22, 73, 74, 23, 24, 71, 72, 41, 42, 73, 74, 43, 44
        ]
    );
    assert_eq!((right.palette, right.collision), (0xe4, 0x39));
}
#[test]
fn stage_four_override_requires_negative_control_exact_room_coordinate_and_no_split() {
    for c in [
        ExpansionContext {
            stage: FIFTH_STAGE,
            room: PROGRESSION_FOUR_ROOM,
            stage_four_control: 0x7f,
            ring: 28,
            ..context()
        },
        ExpansionContext {
            stage: FIFTH_STAGE,
            room: PROGRESSION_FOUR_ROOM - 1,
            stage_four_control: 0x80,
            ring: 28,
            ..context()
        },
        ExpansionContext {
            stage: FIFTH_STAGE,
            room: PROGRESSION_FOUR_ROOM,
            stage_four_control: 0x80,
            ring: 27,
            ..context()
        },
        ExpansionContext {
            stage: FIFTH_STAGE,
            room: PROGRESSION_FOUR_ROOM,
            stage_four_control: 0x80,
            ring: 44,
            ..context()
        },
        ExpansionContext {
            stage: FIFTH_STAGE,
            room: PROGRESSION_FOUR_ROOM,
            stage_four_control: 0x80,
            ring: 28,
            split: true,
            ..context()
        },
    ] {
        assert_eq!(expand(&terrain(), 0, c).tiles, expanded().tiles);
    }
}
#[test]
fn initial_stream_without_previous_room_visits96_columns_and_wraps_room() {
    let result = initial_columns(&layout(), 0, 0);
    assert_eq!(result.visits.len(), 96);
    assert_eq!(result.visits[0], StreamCursor::default());
    assert_eq!(
        result.visits[64],
        StreamCursor {
            ring: 0,
            room: 1,
            room_in_row: 1
        }
    );
    assert_eq!(result.direction, Direction::Forward);
    assert_eq!(result.final_cursor.ring, 31);
}
#[test]
fn initial_stream_with_both_neighbors_visits128_from_previous_half_room() {
    let result = initial_columns(&layout(), 9, 1);
    assert_eq!(result.visits.len(), 128);
    assert_eq!(
        result.visits[0],
        StreamCursor {
            ring: 32,
            room: 8,
            room_in_row: 0
        }
    );
    assert_eq!(
        result.visits[32],
        StreamCursor {
            ring: 0,
            room: 9,
            room_in_row: 1
        }
    );
    assert_eq!(
        result.visits[96],
        StreamCursor {
            ring: 0,
            room: 10,
            room_in_row: 2
        }
    );
}
#[test]
fn initial_stream_middle_count_has_literal_decrement_compare_and_missing_neighbor_paths() {
    let l = layout();
    let result = initial_columns(&l, 10, 2);
    assert_eq!(result.visits.len(), 96, "room-2 equals column count");
    assert_eq!(result.visits[0].room, 9);
    let mut l = layout();
    l.rooms[10] = None;
    let result = initial_columns(&l, 9, 1);
    assert_eq!(result.visits.len(), 96);
    assert_eq!(result.visits[0].ring, 32);
    l.rooms[8] = None;
    let result = initial_columns(&l, 9, 1);
    assert_eq!(result.visits.len(), 96);
    assert_eq!(result.visits[0].ring, 0);
    assert_eq!(result.visits[0].room, 9);
}
#[test]
fn whole_ring_visits64_positions_without_initial_builder_room_wrap_policy() {
    let visits = whole_ring(7);
    assert_eq!(visits.len(), ROOM_CELLS);
    for (ring, cursor) in visits.iter().enumerate() {
        assert_eq!(
            *cursor,
            StreamCursor {
                ring: ring as u8,
                room: 7,
                room_in_row: 7
            }
        );
    }
}
#[test]
fn camera_work_covers_forward_backward_alignment_zero_and_signed_lowbyte_wrap() {
    for (previous, current, direction, count) in [
        (0, 3, Direction::Forward, 0),
        (3, 4, Direction::Forward, 1),
        (1, 0, Direction::Backward, 0),
        (0, 255, Direction::Backward, 1),
        (255, 0, Direction::Forward, 1),
        (8, 0, Direction::Backward, 2),
        (0, 8, Direction::Forward, 2),
        (0, 128, Direction::Forward, 32),
    ] {
        assert_eq!(
            stream_work(previous, current, direction),
            count,
            "{previous}->{current} {direction:?}"
        );
    }
}
#[test]
fn stream_same_direction_wraps_ring_and_byte_room_while_reversal_only_shifts_room() {
    let cases = [
        (
            StreamCursor {
                ring: 63,
                room: 255,
                room_in_row: 7,
            },
            Direction::Forward,
            Direction::Forward,
            StreamCursor {
                ring: 0,
                room: 0,
                room_in_row: 8,
            },
        ),
        (
            StreamCursor::default(),
            Direction::Backward,
            Direction::Backward,
            StreamCursor {
                ring: 63,
                room: 255,
                room_in_row: 255,
            },
        ),
        (
            StreamCursor {
                ring: 12,
                room: 8,
                room_in_row: 0,
            },
            Direction::Backward,
            Direction::Forward,
            StreamCursor {
                ring: 12,
                room: 10,
                room_in_row: 2,
            },
        ),
        (
            StreamCursor {
                ring: 12,
                room: 0,
                room_in_row: 0,
            },
            Direction::Forward,
            Direction::Backward,
            StreamCursor {
                ring: 12,
                room: 254,
                room_in_row: 254,
            },
        ),
    ];
    for (before, previous, next, after) in cases {
        assert_eq!(advance_cursor(before, previous, next), after);
    }
}
#[test]
fn missing_or_outside_column_leaves_both_caches_and_upload_untouched() {
    let mut l = layout();
    l.rooms[0] = None;
    assert_eq!(
        column(
            &l,
            ExpansionContext {
                room: 0,
                ..context()
            },
            &expanded(),
            0
        ),
        ColumnResult {
            collision: None,
            attribute: None,
            upload: None
        }
    );
    assert_eq!(
        column(
            &layout(),
            ExpansionContext {
                room_in_row: 8,
                ..context()
            },
            &expanded(),
            0
        ),
        ColumnResult {
            collision: None,
            attribute: None,
            upload: None
        }
    );
}
#[test]
fn early_game_even_room_first_row_updates_collision_but_queues_no_tiles_or_attributes() {
    let result = column(
        &layout(),
        ExpansionContext {
            ring: 0,
            room_in_row: 0,
            ..context()
        },
        &ExpandedBlock {
            cache_index: 0,
            ..expanded()
        },
        0,
    );
    assert_eq!(result.collision, Some((0, 0x39)));
    assert_eq!(result.attribute, None);
    assert_eq!(result.upload, None);
}
#[test]
fn ordinary_column_emits_row_major_tiles_and_attribute_with_interrupt_commit() {
    let result = column(
        &layout(),
        context(),
        &ExpandedBlock {
            cache_index: 65,
            ..expanded()
        },
        0,
    );
    let upload = result.upload.unwrap();
    assert_eq!(upload.commit, UploadCommit::AtInterrupt);
    assert_eq!(upload.writes.len(), 17);
    assert_eq!(
        upload.writes[0],
        VideoWrite {
            address: 0x2404,
            value: 11
        }
    );
    assert_eq!(
        upload.writes[4],
        VideoWrite {
            address: 0x2424,
            value: 13
        }
    );
    assert_eq!(
        upload.writes[16],
        VideoWrite {
            address: 0x27c1,
            value: 0xe4
        }
    );
    assert_eq!(result.collision, Some((65, 0x39)));
    assert_eq!(result.attribute, Some((65, 0xe4)));
}
#[test]
fn even_room_row_one_copies_lower_half_and_masks_palette_to_high_nibble() {
    let result = column(
        &layout(),
        ExpansionContext {
            ring: 1,
            room_in_row: 0,
            ..context()
        },
        &ExpandedBlock {
            cache_index: 8,
            ..expanded()
        },
        0,
    );
    let upload = result.upload.unwrap();
    assert_eq!(upload.writes.len(), 9);
    assert_eq!(
        upload.writes[0],
        VideoWrite {
            address: 0x20c0,
            value: 31
        }
    );
    assert_eq!(
        upload.writes[8],
        VideoWrite {
            address: 0x23c8,
            value: 0xe0
        }
    );
}
#[test]
fn last_attribute_row_trims_bottom_half_for_all_layouts_and_masks_low_nibble() {
    for stage in [0, SCENE_LAYOUT] {
        let result = column(
            &layout(),
            ExpansionContext {
                stage,
                ring: 7,
                ..context()
            },
            &ExpandedBlock {
                cache_index: 120,
                ..expanded()
            },
            0,
        );
        let upload = result.upload.unwrap();
        assert_eq!(upload.writes.len(), 9);
        assert_eq!(
            upload.writes[0],
            VideoWrite {
                address: 0x2780,
                value: 11
            }
        );
        assert_eq!(
            upload.writes[8],
            VideoWrite {
                address: 0x27f8,
                value: 4
            }
        );
    }
}
#[test]
fn scene_layout_even_first_row_is_full_and_suspended_upload_is_immediate() {
    let result = column(
        &layout(),
        ExpansionContext {
            stage: SCENE_LAYOUT,
            ring: 0,
            room_in_row: 0,
            ..context()
        },
        &ExpandedBlock {
            cache_index: 0,
            ..expanded()
        },
        1,
    );
    let upload = result.upload.unwrap();
    assert_eq!(upload.writes.len(), 17);
    assert_eq!(upload.commit, UploadCommit::Immediate);
    assert_eq!(upload.writes[0].address, 0x2000);
}
#[test]
fn descriptor_probe_uses_room_row_stride_ninth_bit_and_requested_quadrant() {
    let mut catalog = WorldCatalog {
        rooms: vec![Room {
            low: [0; ROOM_CELLS],
            ninth_bits: [0; 8],
        }],
        layouts: vec![layout()],
        terrain: vec![terrain()],
    };
    catalog.rooms[0].low[9] = 0xff;
    catalog.rooms[0].ninth_bits[1] = 0x40;
    catalog.terrain[0].blocks[0x1ff] = [21, 22, 23, 24];
    for (quadrant, expected) in [21, 22, 23, 24].into_iter().enumerate() {
        assert_eq!(
            read_cell(
                &catalog,
                0,
                0,
                TerrainProbe {
                    room_row: 1,
                    world_page: 2,
                    block_coordinate: 9,
                    quadrant: quadrant as u8
                }
            ),
            expected
        );
    }
}
#[test]
fn inactive_or_centered_camera_does_not_stream_and_preserves_player() {
    for (active, x) in [(false, 0x190), (true, 0x180)] {
        let before = camera(0x100, 1);
        let result = camera_step(
            before,
            player(x),
            CameraInput {
                active,
                ..camera_input()
            },
            &layout(),
        );
        assert_eq!(result.camera, before);
        assert_eq!(result.player, player(x));
        assert_eq!(result.work, 0);
    }
}
#[test]
fn forward_camera_commits_lowbyte_carry_to_page_and_room_using_same_saved_carry() {
    let result = camera_step(camera(0x1f8, 9), player(0x282), camera_input(), &layout());
    assert_eq!(result.camera.x, 0x202);
    assert_eq!(result.camera.room, 10);
    assert_eq!(result.direction, Some(Direction::Forward));
    assert_eq!(result.work, 2);
}
#[test]
fn backward_camera_commits_borrow_to_page_and_room() {
    let result = camera_step(camera(0x208, 10), player(0x27e), camera_input(), &layout());
    assert_eq!(result.camera.x, 0x1fe);
    assert_eq!(result.camera.room, 9);
    assert_eq!(result.direction, Some(Direction::Backward));
    assert_eq!(result.work, 3);
}
#[test]
fn forward_camera_clamps_at_layout_last_page_missing_next_room_wrapped_room_or_late_mode() {
    for (before, mut l, input) in [
        (camera(0x6f8, 6), layout(), camera_input()),
        (camera(0x1f8, 9), layout(), camera_input()),
        (camera(0x1f8, 254), layout(), camera_input()),
        (
            camera(0x1f8, 9),
            layout(),
            CameraInput {
                late: 0x80,
                ..camera_input()
            },
        ),
    ] {
        if before.room == 9 && input.late == 0 {
            l.rooms[11] = None;
        }
        let result = camera_step(before, player((before.x & 0xff00) | 0xf7), input, &l);
        // Low-byte offset 7F moves proposed camera to next page. Positive clamp
        // commits proposed room/page, sets camera fraction0 and player <=F0.
        assert_eq!(result.camera.x & 0xff, 0);
        assert_eq!(result.player.x as u8, PLAYER_RIGHT_MARGIN);
    }
}
#[test]
fn backward_camera_clamp_preserves_current_room_page_and_enforces_left_margin() {
    for late in [0, 0x80] {
        let mut l = layout();
        l.rooms[9] = None;
        let result = camera_step(
            camera(0x198, 9),
            player(0x100),
            CameraInput {
                late,
                ..camera_input()
            },
            &l,
        );
        assert_eq!(result.camera.x, 0x100);
        assert_eq!(result.camera.room, 9);
        assert_eq!(result.player.x, 0x110);
    }
}
#[test]
fn backward_camera_missing_previous_room_preserves_page_even_when_proposal_borrows() {
    let mut l = layout();
    l.rooms[8] = None;
    let result = camera_step(camera(0x108, 9), player(0x17e), camera_input(), &l);
    assert_eq!(result.camera.x, 0x100);
    assert_eq!(result.camera.room, 9);
    assert_eq!(result.player.x, 0x17e);
}
#[test]
fn backward_camera_clamps_negative_proposed_page_even_without_ff_room() {
    let result = camera_step(camera(8, 0), player(0x7e), camera_input(), &layout());
    assert_eq!(result.camera.x, 0);
    assert_eq!(result.camera.room, 0);
    assert_eq!(result.player.x, 0x7e);
}
#[test]
fn automatic_camera_moves_only_for_negative_departure_and_nonzero_camera() {
    for (x, departure, expected) in [(0, 0x80, 0), (0x100, 0x80, 0xff), (0x100, 0x7f, 0x100)] {
        let result = camera_step(
            camera(x, 1),
            player(0x180),
            CameraInput {
                automatic: true,
                departure,
                ..camera_input()
            },
            &layout(),
        );
        assert_eq!(result.camera.x, expected);
        assert_eq!(result.player, player(0x180));
    }
}
#[test]
fn automatic_player_edge_correction_uses_24bit_fractional_steps_and_right_priority() {
    for (held, x, fraction, expected_x, expected_fraction) in [
        (BACKWARD_HELD, 0x108, 0, 0x109, 0x80),
        (FORWARD_HELD, 0x1f0, 0, 0x1ee, 0x80),
        (FORWARD_HELD | BACKWARD_HELD, 0x1f0, 0, 0x1ee, 0x80),
        (BACKWARD_HELD, 0x110, 0, 0x110, 0),
        (FORWARD_HELD, 0x1ef, 0, 0x1ef, 0),
        (0, 0x100, 0, 0x100, 0),
    ] {
        let result = camera_step(
            camera(0x100, 1),
            CameraPlayer { x, fraction },
            CameraInput {
                automatic: true,
                held,
                ..camera_input()
            },
            &layout(),
        );
        assert_eq!(
            result.player,
            CameraPlayer {
                x: expected_x,
                fraction: expected_fraction
            }
        );
    }
}
#[test]
fn animation_replacement_selector_preserves_subtract_carry_after_mask_and_ignores_highbit() {
    for (animation, expected) in [
        (0x19, 0x16),
        (0x1a, 0x14),
        (0x1b, 0x15),
        (0x1c, 0x16),
        (0x1d, 0x17),
        (0x9a, 0x14),
    ] {
        assert_eq!(
            replacement_index(Replacement::Animation {
                stage: StageId(0),
                animation
            }),
            expected
        );
    }
    assert_eq!(
        replacement_index(Replacement::Animation {
            stage: StageId(4),
            animation: TERRAIN_ACTOR_FIRST_ANIMATION
        }),
        0x24
    );
}
#[test]
fn actor_state_replacement_uses_stage_four_entry_stride_without_animation_offset() {
    for (stage, local, expected) in [(0, 0, 0), (1, 3, 7), (4, 11, 27), (8, 2, 34)] {
        assert_eq!(
            replacement_index(Replacement::ActorState {
                stage: StageId(stage),
                local
            }),
            expected
        );
    }
}
#[test]
fn mutation_merges_only_selected_quadrant_and_emits_matching_two_tile_pairs_and_attribute() {
    let mut cache = WorldCache {
        attributes: [0xff; CACHE_CELLS],
        collision: [0xff; CACHE_CELLS],
        changed_this_frame: 0,
    };
    let upload = cache.replace(
        &terrain(),
        Mutation {
            world_x: 0x30,
            spawn_y: 0x50,
            selector: 1,
            marks_changed: true,
        },
    );
    assert_eq!(cache.attributes[17], 0x3f);
    assert_eq!(cache.collision[17], 0x7f);
    assert_eq!(cache.attributes[16], 0xff);
    assert_eq!(cache.changed_this_frame, 1);
    assert_eq!(upload.commit, UploadCommit::AtInterrupt);
    assert_eq!(
        upload.writes,
        vec![
            VideoWrite {
                address: 0x2146,
                value: 11
            },
            VideoWrite {
                address: 0x2147,
                value: 12
            },
            VideoWrite {
                address: 0x2166,
                value: 13
            },
            VideoWrite {
                address: 0x2167,
                value: 14
            },
            VideoWrite {
                address: 0x23d1,
                value: 0x3f
            }
        ]
    );
}
#[test]
fn mutation_covers_all_quadrant_masks_both_room_parities_and_spawn_y_not_live_y() {
    for (world_x, spawn_y, index, collision, palette) in [
        (0x20, 0x40, 17, 0xfd, 0xfc),
        (0x30, 0x40, 17, 0xf7, 0xf3),
        (0x20, 0x50, 17, 0xdf, 0xcf),
        (0x30, 0x50, 17, 0x7f, 0x3f),
        (0x130, 0x50, 81, 0x7f, 0x3f),
    ] {
        let mut cache = WorldCache {
            attributes: [0xff; CACHE_CELLS],
            collision: [0xff; CACHE_CELLS],
            changed_this_frame: 0,
        };
        cache.replace(
            &terrain(),
            Mutation {
                world_x,
                spawn_y,
                selector: 1,
                marks_changed: false,
            },
        );
        assert_eq!(cache.attributes[index], palette);
        assert_eq!(cache.collision[index], collision);
        assert_eq!(cache.changed_this_frame, 0);
    }
}
#[test]
fn top_status_attribute_mask_changes_upload_only_and_keeps_full_logical_cache() {
    let mut cache = WorldCache {
        attributes: [0xff; CACHE_CELLS],
        collision: [0xff; CACHE_CELLS],
        changed_this_frame: 0,
    };
    let upload = cache.replace(
        &terrain(),
        Mutation {
            world_x: 0x10,
            spawn_y: 0x10,
            selector: 1,
            marks_changed: false,
        },
    );
    assert_eq!(cache.attributes[0], 0x3f);
    assert_eq!(cache.collision[0], 0x7f);
    assert_eq!(
        upload.writes.last(),
        Some(&VideoWrite {
            address: 0x23c0,
            value: 0x30
        })
    );
}
#[test]
fn dialogue_reveal_clears_only_two_collision_cells_and_preserves_attributes() {
    let mut cache = WorldCache {
        attributes: [0xaa; CACHE_CELLS],
        collision: [0xff; CACHE_CELLS],
        changed_this_frame: 7,
    };
    cache.dialogue_reveal();
    assert_eq!(cache.collision[0x21], 0);
    assert_eq!(cache.collision[0x29], 0);
    assert_eq!(cache.collision[0x20], 0xff);
    assert_eq!(cache.attributes, [0xaa; CACHE_CELLS]);
    assert_eq!(cache.changed_this_frame, 7);
}
#[test]
fn all_sixteen_vertical_dispatch_entries_match_actual_little_endian_words_and_repeat_modulo8() {
    let down = [
        VerticalPhase::BuildRoom,
        VerticalPhase::NextRoom,
        VerticalPhase::PreviousRoom,
        VerticalPhase::PollBackground,
        VerticalPhase::CopyStaged,
        VerticalPhase::PollBackground,
        VerticalPhase::PollBackground,
        VerticalPhase::PollBackground,
    ];
    let up = [
        VerticalPhase::NextRoom,
        VerticalPhase::PreviousRoom,
        VerticalPhase::BoundaryOrCopy,
        VerticalPhase::PollBackground,
        VerticalPhase::PollBackground,
        VerticalPhase::PollBackground,
        VerticalPhase::BuildRoom,
        VerticalPhase::PollBackground,
    ];
    for (direction, phases) in [(VerticalDirection::Down, down), (VerticalDirection::Up, up)] {
        for counter in 0..TRANSITION_TICKS {
            assert_eq!(
                vertical_phase(direction, counter),
                phases[usize::from(counter % 8)],
                "{direction:?} tick {counter}"
            );
        }
    }
}
#[test]
fn vertical_begin_saves_cursors_clears_actors_hides_sprites_and_yields_before_any_motion() {
    let cursor = StreamCursor {
        ring: 31,
        room: 9,
        room_in_row: 1,
    };
    for (direction, ring) in [(VerticalDirection::Up, 7), (VerticalDirection::Down, 0)] {
        let (transition, effects) = VerticalTransition::begin(direction, 0x80, 0x24c0, cursor);
        assert_eq!(transition.counter, 0);
        assert_eq!(transition.player_y, 0x80);
        assert_eq!(transition.split_control, 0x0c);
        assert_eq!(transition.cursor.ring, ring);
        assert_eq!(transition.saved_cursor, cursor);
        assert_eq!(
            effects,
            vec![
                VerticalEffect::ClearNonplayerActors,
                VerticalEffect::HideSprites,
                VerticalEffect::Yield
            ]
        );
    }
}
#[test]
fn vertical_even_tick_adjusts_player_split_then_dispatches_composes_and_yields() {
    for (direction, split, player_y, phase) in [
        (VerticalDirection::Up, 0x24a0, 0x87, VerticalPhase::NextRoom),
        (
            VerticalDirection::Down,
            0x24e0,
            0x79,
            VerticalPhase::BuildRoom,
        ),
    ] {
        let (mut transition, _) =
            VerticalTransition::begin(direction, 0x80, 0x24c0, StreamCursor::default());
        let effects = transition.resume();
        assert_eq!(transition.split_address, split);
        assert_eq!(transition.player_y, player_y);
        assert_eq!(transition.counter, 1);
        assert_eq!(
            effects,
            vec![
                VerticalEffect::Build(phase),
                VerticalEffect::HideSprites,
                VerticalEffect::ComposeActors,
                VerticalEffect::Yield
            ]
        );
    }
}
#[test]
fn vertical_odd_tick_changes_player_by_one_but_leaves_split_address_unchanged() {
    for (direction, expected) in [
        (VerticalDirection::Up, 0x7f),
        (VerticalDirection::Down, 0x81),
    ] {
        let (mut transition, _) =
            VerticalTransition::begin(direction, 0x80, 0x24c0, StreamCursor::default());
        transition.counter = 1;
        transition.resume();
        assert_eq!(transition.player_y, expected);
        assert_eq!(transition.split_address, 0x24c0);
    }
}
#[test]
fn vertical_special_wrap_ticks_replace_low_and_mask_high_without_even_tick_player_compensation() {
    for (direction, counter, split, player_y) in [
        (VerticalDirection::Up, 12, 0x27a0, 0x7f),
        (VerticalDirection::Down, 46, 0x2400, 0x81),
    ] {
        let (mut transition, _) =
            VerticalTransition::begin(direction, 0x80, 0x27e0, StreamCursor::default());
        transition.counter = counter;
        transition.resume();
        assert_eq!(transition.split_address, split);
        assert_eq!(transition.player_y, player_y);
    }
}
#[test]
fn sixty_vertical_frames_restore_ring_room_in_row_and_reset_spawns_before_palette_refresh() {
    let saved = StreamCursor {
        ring: 31,
        room: 9,
        room_in_row: 1,
    };
    for (direction, player_y) in [
        (VerticalDirection::Up, 0x2c),
        (VerticalDirection::Down, 0xd4),
    ] {
        let (mut transition, _) = VerticalTransition::begin(direction, 0x80, 0x24c0, saved);
        let mut last = Vec::new();
        for _ in 0..TRANSITION_TICKS {
            last = transition.resume();
        }
        assert_eq!(transition.counter, TRANSITION_TICKS);
        assert_eq!(transition.cursor.ring, saved.ring);
        assert_eq!(transition.cursor.room_in_row, saved.room_in_row);
        assert_eq!(transition.split_control, 0);
        assert_eq!(transition.split_address, 0x24c0);
        assert_eq!(transition.player_y, player_y);
        assert!(last.ends_with(&[
            VerticalEffect::ResetSpawnWindow,
            VerticalEffect::UpdateSpawnWindow,
            VerticalEffect::RefreshPalette
        ]));
    }
}
#[test]
fn vertical_rows_merge_directional_attribute_nibbles_and_keep_staged_other_half() {
    let blocks = vec![expanded(); 8];
    for (direction, first_tile, last_tile, attributes) in [
        (VerticalDirection::Up, 31, 11, 0xea),
        (VerticalDirection::Down, 11, 31, 0xa4),
    ] {
        let rows = vertical_rows(direction, 2, &blocks, [0xaa; 8]);
        assert_eq!(rows.tiles[0][0], first_tile);
        assert_eq!(rows.tiles[2][0], last_tile);
        assert_eq!(rows.attributes, [attributes; 8]);
        assert_eq!(rows.ring, 2);
        assert_eq!(rows.uploads.len(), 72);
    }
}
#[test]
fn vertical_neighbor_rejects_layout_edges_and_ff_entries_without_building() {
    let blocks = vec![expanded(); 4];
    let l = layout();
    assert_eq!(
        vertical_neighbor(VerticalDirection::Up, true, camera(0, 0), &l, 3, &blocks),
        None
    );
    assert_eq!(
        vertical_neighbor(
            VerticalDirection::Down,
            false,
            camera(0x700, 7),
            &l,
            3,
            &blocks
        ),
        None
    );
    let mut l = layout();
    l.rooms[8] = None;
    l.rooms[10] = None;
    for previous in [true, false] {
        assert_eq!(
            vertical_neighbor(
                VerticalDirection::Up,
                previous,
                camera(0x100, 9),
                &l,
                3,
                &blocks
            ),
            None
        );
    }
}
#[test]
fn vertical_neighbor_ordinary_rows_emit_four_planes_and_four_attributes_in_correct_half() {
    for (previous, base) in [(true, 0x2110), (false, 0x2100)] {
        let rows = vertical_neighbor(
            VerticalDirection::Down,
            previous,
            camera(0x300, 11),
            &layout(),
            2,
            &vec![expanded(); 4],
        )
        .unwrap();
        assert_eq!(rows.uploads.len(), 68);
        assert_eq!(rows.uploads[0].address, base);
        assert_eq!(rows.uploads[0].value, 11);
        assert_eq!(rows.ring, 2);
    }
}
#[test]
fn vertical_neighbor_row_one_and_seven_trim_half_tile_planes_and_attribute_nibbles() {
    for (ring, value, first) in [(1, 0xe0, 31), (7, 0x04, 11)] {
        let rows = vertical_neighbor(
            VerticalDirection::Up,
            false,
            camera(0x300, 11),
            &layout(),
            ring,
            &vec![expanded(); 4],
        )
        .unwrap();
        assert_eq!(rows.uploads.len(), 36);
        assert_eq!(rows.uploads[0].value, first);
        assert!(
            rows.uploads
                .iter()
                .rev()
                .take(4)
                .all(|write| write.value == value)
        );
    }
}
#[test]
fn changed_bursts_repeat_descriptor_rows_and_request_four_particles_at_directional_origin() {
    let mut set = terrain();
    set.tiles[0x88] = [11, 12, 13, 14];
    set.tiles[0xd6] = [21, 22, 23, 24];
    set.tiles[0xd7] = [31, 32, 33, 34];
    for (selector, origin, address, count) in
        [(0, (0x88, 0x28), 0x2204, 32), (6, (0x88, 0x78), 0x260e, 56)]
    {
        let burst = changed_burst(&set, selector);
        assert_eq!(burst.particle_origin, origin);
        assert_eq!(burst.particles, 4);
        assert_eq!(burst.upload.commit, UploadCommit::AtInterrupt);
        assert_eq!(burst.upload.writes.len(), count);
        assert_eq!(
            burst.upload.writes[0],
            VideoWrite {
                address,
                value: if selector == 0 { 11 } else { 21 }
            }
        );
        assert_eq!(
            burst.upload.writes.last().unwrap().address,
            address + if selector == 0 { 0xe3 } else { 0x1a3 }
        );
    }
}
#[test]
fn recovery_actor_gate_ignores_positive_counter_and_full_pool_but_excludes_player_slot() {
    assert_eq!(recovery_actor(0x7f, 0, 0x123, 0x45, &[false; 16]), None);
    assert_eq!(recovery_actor(0x80, 0, 0x123, 0x45, &[true; 16]), None);
    let mut active = [true; 16];
    active[0] = false;
    assert_eq!(recovery_actor(0xff, 0, 0x123, 0x45, &active), None);
}
#[test]
fn recovery_actor_uses_highest_free_slot_copies_position_and_clears_motion_collision_animation_clock()
 {
    let mut active = [true; 16];
    active[3] = false;
    active[12] = false;
    for (local, animation) in [(0, 0x24), (1, 0x28), (2, 0x25), (3, 0x26)] {
        assert_eq!(
            recovery_actor(0x80, local, 0x123, 0x45, &active),
            Some(RecoveryActor {
                slot: 12,
                state: 0xa1,
                handler: 0x13,
                animation,
                world_x: 0x123,
                screen_y: 0x45,
                animation_tick: 0,
                animation_frame: 0,
                collision_profile: 0,
                vertical_velocity: 0
            })
        );
    }
}
#[test]
fn vertical_wrap_boundary_builds_opposite_direction_only_at_row_seven() {
    for direction in [VerticalDirection::Up, VerticalDirection::Down] {
        let opposite = if direction == VerticalDirection::Up {
            VerticalDirection::Down
        } else {
            VerticalDirection::Up
        };
        assert_eq!(
            vertical_wrap_action(direction, 7),
            (opposite, 6, VerticalPhase::BuildRoom)
        );
        for row in 0..7 {
            assert_eq!(
                vertical_wrap_action(direction, row),
                (direction, row, VerticalPhase::CopyStaged)
            );
        }
    }
}
#[test]
fn staged_vertical_copy_uses_opposite_direction_destinations_and_wraps_next_row() {
    let staged = VerticalStaging {
        upper: [11; 32],
        lower: [13; 32],
        attributes: [0xe4; 8],
    };
    for (direction, row, next, address) in [
        (VerticalDirection::Up, 0, 7, 0x2400),
        (VerticalDirection::Down, 7, 0, 0x27c0),
    ] {
        let (next_row, upload) = copy_vertical_staging(direction, row, &staged);
        assert_eq!(next_row, next);
        assert_eq!(upload.commit, UploadCommit::AtInterrupt);
        assert_eq!(upload.writes.len(), 72);
        assert_eq!(upload.writes[0], VideoWrite { address, value: 11 });
        assert_eq!(
            upload.writes[32],
            VideoWrite {
                address: address | 0x20,
                value: 13
            }
        );
        assert_eq!(upload.writes[64].value, 0xe4);
    }
}
#[test]
fn vertical_neighbor_boundary_group_zero_leaves_no_upload_in_either_half() {
    for previous in [false, true] {
        assert_eq!(
            vertical_neighbor(
                VerticalDirection::Up,
                previous,
                camera(0x300, 11),
                &layout(),
                0,
                &vec![expanded(); 4]
            ),
            None
        );
    }
}
#[test]
fn packed_property_ignores_reserved_bits_and_replacement_counter_increments_with_byte_wrap() {
    let mut set = terrain();
    for property in &mut set.properties {
        *property |= 0xcc;
    }
    let block = expand(&set, 0, context());
    assert_eq!((block.palette, block.collision), (0xe4, 0x39));
    let mut cache = WorldCache {
        changed_this_frame: u8::MAX,
        ..WorldCache::default()
    };
    cache.replace(
        &set,
        Mutation {
            world_x: 0x30,
            spawn_y: 0x50,
            selector: 1,
            marks_changed: true,
        },
    );
    assert_eq!(cache.changed_this_frame, 0);
}
#[test]
fn raw_selector_twenty_one_retains_byte_cursor_layout_and_full_room_domain() {
    let rom = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap();
    let layout = raw_layout(&rom, 21).unwrap();
    assert_eq!(
        (layout.rows, layout.columns, layout.rooms.len()),
        (255, 36, 256)
    );
    assert_eq!(
        layout.rooms[128],
        (rom[0x1d106 + 128] != MISSING_ROOM).then_some(usize::from(rom[0x1d106 + 128]))
    );
    let rooms = raw_rooms(&rom).unwrap();
    assert_eq!(rooms.len(), 256);
    assert_eq!(rooms[228].low[0], rom[0x1c030]);
}

#[test]
fn changed_actor_center_selects_aligned_two_by_two_tile_quadrant_in_both_nametables() {
    // F574 maps center coordinates D8/B8 to quadrant origins D0/B0.
    // Source row22,column26 =>22DA, and X page1 selects the other nametable.
    for (world_x, expected) in [(0x00d8, 0x22da), (0x01d8, 0x26da)] {
        let mut cache = WorldCache::default();
        let upload = cache.replace(
            &terrain(),
            Mutation {
                world_x,
                spawn_y: 0xb8,
                selector: 1,
                marks_changed: false,
            },
        );
        assert_eq!(
            upload
                .writes
                .iter()
                .take(4)
                .map(|write| write.address)
                .collect::<Vec<_>>(),
            [expected, expected + 1, expected + 0x20, expected + 0x21]
        );
    }
}

//! B02 executable source contracts. See trace.md for instruction/consumer evidence.
//! Metadata tuples below name upload origins/counts/destinations; no original
//! graphics, audio or room payload is embedded. Synthetic bytes distinguish pages.
use super::{Assets, catalog::*, formats::*};
use crate::{
    config,
    identifiers::{AssetId, StageId},
};
use anyhow::{Context, ensure};
use romatlas_port::assets::{Asset, Collection, Origin, Rom};

fn rom() -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../input/game.rom");
    let bytes = std::fs::read(path).expect("matching local ROM fixture required");
    Rom::from_bytes(bytes.clone(), config::ROM_SHA256).expect("matching fixture identity");
    bytes
}
// Tuples: mapper bank, physical start, number of 256-byte pages, PPU destination.
type SpanExpectation = (u8, usize, u8, u16);
const CHR_EXPECTED: &[(&[SpanExpectation], usize)] = &[
    (
        &[
            (0, 0x10, 8, 0x0000),
            (0, 0x1710, 1, 0x0800),
            (1, 0x4010, 3, 0x1000),
            (1, 0x5010, 13, 0x1300),
        ],
        0x1c38a,
    ), // selector 0
    (
        &[
            (0, 0x10, 8, 0x0000),
            (0, 0x1710, 1, 0x0800),
            (1, 0x4010, 3, 0x1000),
            (1, 0x5d10, 13, 0x1300),
        ],
        0x1c397,
    ), // selector 1
    (
        &[
            (0, 0x10, 8, 0x0000),
            (0, 0x1710, 1, 0x0800),
            (1, 0x4010, 3, 0x1000),
            (2, 0x8010, 13, 0x1300),
        ],
        0x1c3a4,
    ), // selector 2
    (
        &[
            (0, 0x10, 8, 0x0000),
            (0, 0x2f10, 1, 0x0800),
            (1, 0x4010, 3, 0x1000),
            (2, 0x8d10, 13, 0x1300),
        ],
        0x1c3b1,
    ), // selector 3
    (
        &[
            (0, 0x10, 8, 0x0000),
            (0, 0x1710, 1, 0x0800),
            (1, 0x4010, 3, 0x1000),
            (2, 0x9a10, 13, 0x1300),
        ],
        0x1c3be,
    ), // selector 4
    (
        &[
            (0, 0x10, 8, 0x0000),
            (0, 0x1f10, 1, 0x0800),
            (0, 0x1810, 1, 0x0900),
            (0, 0x1d10, 2, 0x0a00),
            (3, 0xf110, 1, 0x0c00),
            (0, 0x3e10, 1, 0x0d00),
            (0, 0xf10, 1, 0x0e00),
            (0, 0xb10, 1, 0x0f00),
            (1, 0x4010, 16, 0x1000),
        ],
        0x1c3da,
    ), // selector 5
    (
        &[
            (3, 0xf210, 1, 0x0800),
            (0, 0x2010, 1, 0x0900),
            (3, 0xf110, 1, 0x0c00),
            (1, 0x4010, 3, 0x1000),
            (1, 0x5110, 1, 0x1300),
            (2, 0xac10, 1, 0x1400),
            (2, 0xa010, 1, 0x1500),
            (2, 0xad10, 1, 0x1600),
            (2, 0x8b10, 2, 0x1700),
            (2, 0xae10, 2, 0x1900),
            (2, 0x8a10, 1, 0x1b00),
            (2, 0x8e10, 1, 0x1c00),
        ],
        0x1c3ff,
    ), // selector 6
    (
        &[
            (0, 0x10, 8, 0x0000),
            (0, 0x1710, 1, 0x0800),
            (0, 0x1410, 1, 0x0900),
            (2, 0xb010, 1, 0x0a00),
            (2, 0xb210, 1, 0x0b00),
            (0, 0x1510, 2, 0x0e00),
            (1, 0x4010, 3, 0x1000),
            (2, 0x8010, 13, 0x1300),
        ],
        0x1c418,
    ), // selector 7
    (
        &[
            (0, 0x10, 8, 0x0000),
            (0, 0x1710, 1, 0x0800),
            (0, 0x1410, 1, 0x0900),
            (2, 0xb010, 1, 0x0a00),
            (2, 0xb210, 1, 0x0b00),
            (0, 0x1510, 2, 0x0e00),
            (1, 0x4010, 3, 0x1000),
            (2, 0x8010, 13, 0x1300),
        ],
        0x1c418,
    ), // selector 8
    (
        &[
            (0, 0x10, 8, 0x0000),
            (0, 0x2d10, 2, 0x0800),
            (0, 0x2910, 1, 0x0a00),
            (0, 0x2410, 1, 0x0b00),
            (0, 0x2610, 2, 0x0c00),
            (0, 0x3510, 1, 0x0e00),
            (1, 0x4010, 3, 0x1000),
            (1, 0x6a10, 6, 0x1300),
            (2, 0xa710, 7, 0x1900),
        ],
        0x1c434,
    ), // selector 9
    (&[(0, 0x1f10, 1, 0x0800), (3, 0xf110, 1, 0x0900)], 0x1c43b), // selector 10
];
const BACKGROUND_EXPECTED: &[(&[SpanExpectation], usize)] = &[
    (&[], 0x1c5b3), // selector 0
    (
        &[
            (0, 0x1410, 1, 0x0900),
            (0, 0x1010, 4, 0x0a00),
            (0, 0xa10, 2, 0x0e00),
        ],
        0x1c5bd,
    ), // selector 1
    (
        &[
            (0, 0x1410, 1, 0x0900),
            (0, 0xe10, 2, 0x0a00),
            (0, 0xc10, 2, 0x0c00),
            (0, 0x810, 2, 0x0e00),
        ],
        0x1c5ca,
    ), // selector 2
    (
        &[
            (0, 0xe10, 2, 0x0a00),
            (0, 0xc10, 2, 0x0c00),
            (0, 0x1510, 2, 0x0e00),
        ],
        0x1c5d4,
    ), // selector 3
    (&[(0, 0x1810, 1, 0x0900)], 0x1c5d8), // selector 4
    (
        &[
            (0, 0x1810, 1, 0x0900),
            (0, 0x1e10, 2, 0x0a00),
            (0, 0x1c10, 2, 0x0c00),
            (0, 0x3510, 1, 0x0e00),
            (0, 0x2a10, 1, 0x0f00),
        ],
        0x1c5e8,
    ), // selector 5
    (
        &[
            (0, 0x1810, 1, 0x0900),
            (0, 0x1e10, 2, 0x0a00),
            (0, 0x1910, 3, 0x0c00),
            (0, 0x2110, 1, 0x0f00),
        ],
        0x1c5f5,
    ), // selector 6
    (
        &[
            (0, 0x2210, 1, 0x0a00),
            (0, 0xf10, 1, 0x0b00),
            (0, 0xc10, 2, 0x0c00),
            (0, 0x2710, 1, 0x0e00),
            (0, 0x2a10, 1, 0x0f00),
        ],
        0x1c605,
    ), // selector 7
    (&[(0, 0x1810, 1, 0x0900), (0, 0x2210, 5, 0x0a00)], 0x1c60c), // selector 8
    (
        &[
            (0, 0x3b10, 1, 0x0900),
            (0, 0x2210, 5, 0x0a00),
            (0, 0x2a10, 1, 0x0f00),
        ],
        0x1c616,
    ), // selector 9
    (&[(0, 0x1410, 1, 0x0900), (0, 0x2210, 5, 0x0a00)], 0x1c61d), // selector 10
    (
        &[
            (0, 0x1d10, 1, 0x0a00),
            (0, 0x2810, 1, 0x0b00),
            (0, 0xc10, 2, 0x0c00),
            (3, 0xe910, 1, 0x0e00),
            (0, 0x2910, 1, 0x0f00),
        ],
        0x1c62d,
    ), // selector 11
    (
        &[
            (0, 0x1810, 1, 0x0900),
            (0, 0x2210, 1, 0x0a00),
            (0, 0x2810, 1, 0x0b00),
            (0, 0xc10, 2, 0x0c00),
            (0, 0x2710, 1, 0x0e00),
            (0, 0x2110, 1, 0x0f00),
        ],
        0x1c640,
    ), // selector 12
    (
        &[
            (0, 0x1810, 1, 0x0900),
            (0, 0x2b10, 3, 0x0a00),
            (0, 0x3a10, 1, 0x0d00),
            (0, 0xa10, 2, 0x0e00),
        ],
        0x1c64d,
    ), // selector 13
    (
        &[
            (0, 0x2c10, 2, 0x0b00),
            (0, 0x1310, 1, 0x0d00),
            (0, 0x2710, 1, 0x0e00),
            (0, 0x2110, 1, 0x0f00),
        ],
        0x1c65a,
    ), // selector 14
    (
        &[
            (0, 0x1410, 1, 0x0900),
            (0, 0x3010, 3, 0x0a00),
            (0, 0x2910, 1, 0x0d00),
            (0, 0xa10, 2, 0x0e00),
        ],
        0x1c667,
    ), // selector 15
    (&[(0, 0x2010, 1, 0x0900), (0, 0x2110, 1, 0x0f00)], 0x1c66e), // selector 16
    (&[(0, 0x1410, 1, 0x0900), (0, 0x3310, 5, 0x0a00)], 0x1c675), // selector 17
    (
        &[
            (0, 0x1810, 1, 0x0900),
            (0, 0x3310, 3, 0x0a00),
            (2, 0xb010, 1, 0x0d00),
            (0, 0x3710, 1, 0x0e00),
            (2, 0xb110, 1, 0x0f00),
        ],
        0x1c685,
    ), // selector 18
    (
        &[
            (0, 0x3b10, 1, 0x0900),
            (0, 0x3510, 3, 0x0c00),
            (0, 0x2a10, 1, 0x0f00),
        ],
        0x1c68f,
    ), // selector 19
    (
        &[
            (3, 0xee10, 2, 0x0a00),
            (3, 0xf010, 1, 0x0c00),
            (3, 0xed10, 1, 0x0d00),
            (0, 0xb10, 1, 0x0f00),
        ],
        0x1c69c,
    ), // selector 20
    (&[(0, 0x3c10, 2, 0x0a00), (0, 0xb10, 1, 0x0f00)], 0x1c6a3), // selector 21
    (
        &[
            (0, 0x3f10, 1, 0x0a00),
            (4, 0x13f10, 1, 0x0b00),
            (3, 0xe910, 1, 0x0c00),
            (0, 0xb10, 1, 0x0f00),
        ],
        0x1c6b0,
    ), // selector 22
    (
        &[
            (1, 0x7010, 3, 0x0a00),
            (0, 0x3a10, 1, 0x0d00),
            (0, 0xf10, 1, 0x0e00),
        ],
        0x1c6ba,
    ), // selector 23
    (
        &[
            (0, 0x3810, 3, 0x0a00),
            (3, 0xed10, 1, 0x0d00),
            (0, 0x3d10, 1, 0x0e00),
            (0, 0x3f10, 1, 0x0f00),
        ],
        0x1c6c7,
    ), // selector 24
    (
        &[
            (3, 0xea10, 3, 0x0a00),
            (0, 0x3e10, 1, 0x0d00),
            (3, 0xed10, 1, 0x0e00),
        ],
        0x1c6d1,
    ), // selector 25
    (
        &[
            (0, 0x1810, 1, 0x0900),
            (0, 0x3e10, 1, 0x0a00),
            (0, 0x2c10, 2, 0x0b00),
            (0, 0x3a10, 1, 0x0d00),
            (0, 0xa10, 2, 0x0e00),
        ],
        0x1c6e1,
    ), // selector 26
    (&[(3, 0xee10, 2, 0x0a00), (3, 0xf010, 1, 0x0c00)], 0x1c6e8), // selector 27
    (
        &[
            (0, 0x1810, 1, 0x0900),
            (0, 0x2210, 1, 0x0a00),
            (0, 0x2810, 1, 0x0b00),
            (0, 0xc10, 2, 0x0c00),
            (0, 0x2710, 1, 0x0e00),
            (0, 0x2110, 1, 0x0f00),
        ],
        0x1c640,
    ), // selector 28
    (
        &[
            (0, 0x1810, 1, 0x0900),
            (0, 0x2b10, 3, 0x0a00),
            (0, 0x3a10, 1, 0x0d00),
            (0, 0xa10, 2, 0x0e00),
        ],
        0x1c64d,
    ), // selector 29
    (
        &[
            (0, 0x2c10, 2, 0x0b00),
            (0, 0x1310, 1, 0x0d00),
            (0, 0x2710, 1, 0x0e00),
            (0, 0x2110, 1, 0x0f00),
        ],
        0x1c65a,
    ), // selector 30
    (
        &[
            (0, 0x1410, 1, 0x0900),
            (0, 0x3010, 3, 0x0a00),
            (0, 0x2910, 1, 0x0d00),
            (0, 0xa10, 2, 0x0e00),
        ],
        0x1c667,
    ), // selector 31
];

#[test]
fn all_eleven_chr_lists_match_every_record_endpoint_and_alias() {
    let bytes = rom();
    for (selector, (records, end)) in CHR_EXPECTED.iter().enumerate() {
        let actual = original_chr_list(&bytes, selector).unwrap();
        let expected: Vec<_> = records
            .iter()
            .map(
                |&(bank, source_offset, source_pages, destination)| UploadSpan {
                    bank,
                    source_offset,
                    source_pages,
                    destination,
                },
            )
            .collect();
        assert_eq!(actual.records, expected, "CHR selector {selector}");
        assert_eq!((actual.terminator_offset, actual.terminator), (*end, 0xff));
    }
    assert_eq!(
        original_chr_list(&bytes, 7).unwrap(),
        original_chr_list(&bytes, 8).unwrap()
    );
}
#[test]
fn all_background_lists_match_every_record_endpoint_and_repeated_origins() {
    let bytes = rom();
    for (selector, (records, end)) in BACKGROUND_EXPECTED.iter().enumerate() {
        let actual = original_background_list(&bytes, selector).unwrap();
        let expected: Vec<_> = records
            .iter()
            .map(
                |&(bank, source_offset, source_pages, destination)| UploadSpan {
                    bank,
                    source_offset,
                    source_pages,
                    destination,
                },
            )
            .collect();
        assert_eq!(actual.records, expected, "background selector {selector}");
        assert_eq!((actual.terminator_offset, actual.terminator), (*end, 0xff));
    }
    for (alias, original) in [(28, 12), (29, 13), (30, 14), (31, 15)] {
        assert_eq!(
            original_background_list(&bytes, alias).unwrap(),
            original_background_list(&bytes, original).unwrap()
        );
    }
}
#[test]
fn selectors_outside_original_pointer_domains_are_rejected() {
    let bytes = rom();
    for selector in [ORIGINAL_CHR_LISTS, usize::MAX] {
        assert!(original_chr_list(&bytes, selector).is_err());
    }
    for selector in [ORIGINAL_BACKGROUND_LISTS, usize::MAX] {
        assert!(original_background_list(&bytes, selector).is_err());
    }
}
// Synthetic descriptor location is outside source pages; no original data copied.
const SYNTHETIC_DESCRIPTOR: usize = 0x1c800;
fn synthetic(tags: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0; 0x20010];
    bytes[SYNTHETIC_DESCRIPTOR..SYNTHETIC_DESCRIPTOR + tags.len()].copy_from_slice(tags);
    bytes
}
#[test]
fn every_negative_tag_terminates_before_reading_following_bytes() {
    for tag in 0x80..=0xff {
        let bytes = synthetic(&[tag]);
        assert_eq!(
            decode_upload_list(&bytes, SYNTHETIC_DESCRIPTOR, SYNTHETIC_DESCRIPTOR + 1).unwrap(),
            UploadList {
                records: vec![],
                terminator_offset: SYNTHETIC_DESCRIPTOR,
                terminator: tag
            }
        );
    }
}
#[test]
fn all_counts_are_pages_minus_one_including_sixteen_page_record() {
    for count in 0..=15 {
        let bytes = synthetic(&[count, 0x80, 0x00, 0xff]);
        let list =
            decode_upload_list(&bytes, SYNTHETIC_DESCRIPTOR, SYNTHETIC_DESCRIPTOR + 4).unwrap();
        assert_eq!(
            list.records,
            vec![UploadSpan {
                bank: 0,
                source_offset: INES_HEADER_BYTES,
                source_pages: count + 1,
                destination: 0
            }]
        );
    }
}
#[test]
fn bank_context_and_fixed_window_are_distinct_from_cpu_addresses() {
    for bank in 0..=6 {
        let bytes = synthetic(&[bank << 4, 0x9f, 0x08, 0xff]);
        let list =
            decode_upload_list(&bytes, SYNTHETIC_DESCRIPTOR, SYNTHETIC_DESCRIPTOR + 4).unwrap();
        assert_eq!(
            list.records[0].source_offset,
            INES_HEADER_BYTES + usize::from(bank) * PRG_BANK_BYTES + 0x1f00
        );
    }
}
#[test]
fn truncated_missing_terminator_and_overflow_ranges_fail_without_panics() {
    let bytes = synthetic(&[0x00, 0x80, 0x00, 0xff]);
    for end in SYNTHETIC_DESCRIPTOR..SYNTHETIC_DESCRIPTOR + 4 {
        assert!(decode_upload_list(&bytes, SYNTHETIC_DESCRIPTOR, end).is_err());
    }
    for (start, end) in [
        (usize::MAX, usize::MAX),
        (bytes.len(), bytes.len() + 1),
        (5, 4),
    ] {
        assert!(decode_upload_list(&bytes, start, end).is_err());
    }
}
#[test]
fn source_pages_below_rom_and_switch_window_crossings_are_rejected() {
    // No source descriptor crosses BFFF. Native loader rejects malformed content
    // instead of accidentally resolving fixed-bank instructions as CHR payload.
    for record in [
        [0x00, 0x7f, 0x00, 0xff],
        [0x01, 0xbf, 0x00, 0xff],
        [0x70, 0x80, 0x00, 0xff],
    ] {
        let bytes = synthetic(&record);
        assert!(
            decode_upload_list(
                &bytes,
                SYNTHETIC_DESCRIPTOR,
                SYNTHETIC_DESCRIPTOR + record.len()
            )
            .is_err()
        );
    }
}
#[test]
fn page_carry_copies_distinct_successive_pages_and_preserves_record_order() {
    let mut bytes = synthetic(&[0x02, 0x8f, 0x09, 0x00, 0x80, 0x0c, 0xff]);
    for (start, fill, pages) in [
        (0xf10, 0x31, 1),
        (0x1010, 0x62, 1),
        (0x1110, 0x93, 1),
        (0x10, 0xc4, 1),
    ] {
        bytes[start..start + pages * SOURCE_PAGE_BYTES].fill(fill);
    }
    let list = decode_upload_list(&bytes, SYNTHETIC_DESCRIPTOR, SYNTHETIC_DESCRIPTOR + 7).unwrap();
    let actual = prepare_chr(&bytes, &list, 0x91).unwrap();
    assert_eq!(actual.control, 0x90);
    assert_eq!(actual.stride, 1);
    let expected = [
        vec![0x31; SOURCE_PAGE_BYTES],
        vec![0x62; SOURCE_PAGE_BYTES],
        vec![0x93; SOURCE_PAGE_BYTES],
    ]
    .concat();
    assert_eq!(
        actual.copies,
        vec![(0x900, expected), (0xc00, vec![0xc4; SOURCE_PAGE_BYTES])]
    );
    assert_eq!(actual.selected_bank, Some(0));
}
#[test]
fn chr_entry_clears_only_nametable_bit_zero_not_increment_or_shadow() {
    let bytes = synthetic(&[0xff]);
    let list = decode_upload_list(&bytes, SYNTHETIC_DESCRIPTOR, SYNTHETIC_DESCRIPTOR + 1).unwrap();
    for (shadow, control, stride) in [
        (0xff, 0xfe, 32),
        (0x95, 0x94, 32),
        (0x93, 0x92, 1),
        (0, 0, 1),
    ] {
        let actual = prepare_chr(&bytes, &list, shadow).unwrap();
        assert_eq!(
            (actual.control, actual.stride, actual.selected_bank),
            (control, stride, None)
        );
        assert!(actual.copies.is_empty());
    }
}
const STAGE_ROWS: &[&[usize]] = &[
    &[0, 4, 1, 3, 0, 2, 1],
    &[0, 0, 11, 5, 6, 6, 5, 5],
    &[8, 9, 7, 10, 11, 12],
    &[13, 0, 14, 14, 15, 26],
    &[19, 17, 17, 17, 18, 16],
    &[21, 20, 22, 23, 25, 24, 27],
    &[0],
    &[0],
    &[0],
];
fn context(stage: u8, row: u8, camera_page: u8, suspend: u8) -> BackgroundContext {
    BackgroundContext {
        stage: StageId(stage),
        room_row: row,
        camera_page,
        display_suspend_depth: suspend,
    }
}
#[test]
fn all_stage_row_selectors_and_both_tagged_camera_thresholds_are_exact() {
    let bytes = rom();
    for (stage, rows) in STAGE_ROWS.iter().enumerate() {
        for (row, &base) in rows.iter().enumerate() {
            for camera in [0, 2, 3, 4, 255] {
                let extra = usize::from(
                    (stage == 2 && row == 2 && camera >= 4)
                        || (stage == 4 && row == 1 && camera >= 3),
                );
                assert_eq!(
                    select_background(&bytes, context(stage as u8, row as u8, camera, 0))
                        .unwrap()
                        .list,
                    base + extra,
                    "stage{stage} row{row} camera{camera}"
                );
            }
        }
    }
}
#[test]
fn background_stage_and_row_bounds_do_not_bleed_into_adjacent_tables() {
    let bytes = rom();
    for (stage, rows) in STAGE_ROWS.iter().enumerate() {
        assert!(select_background(&bytes, context(stage as u8, rows.len() as u8, 0, 0)).is_err());
    }
    assert!(select_background(&bytes, context(9, 0, 0, 0)).is_err());
    assert!(select_background(&bytes, context(255, 255, 0, 0)).is_err());
}
#[test]
fn camera_variant_does_not_change_room_palette_and_last_quad_is_reachable() {
    let bytes = rom();
    for (stage, row, palette) in [
        (2, 2, [15, 15, 38, 23]),
        (4, 1, [15, 15, 32, 0]),
        (0, 1, [15, 15, 32, 17]),
    ] {
        for camera in [0, 3, 4, 255] {
            assert_eq!(
                select_background(&bytes, context(stage, row, camera, 0))
                    .unwrap()
                    .palette,
                palette
            );
        }
    }
}
fn single_page() -> (Vec<u8>, UploadList) {
    let mut bytes = synthetic(&[0x00, 0x80, 0x09, 0xff]);
    for (chunk, fill) in [0x11, 0x33, 0x55, 0x77].into_iter().enumerate() {
        let start = INES_HEADER_BYTES + chunk * BACKGROUND_CHUNK_BYTES;
        bytes[start..start + BACKGROUND_CHUNK_BYTES].fill(fill);
    }
    let list = decode_upload_list(&bytes, SYNTHETIC_DESCRIPTOR, SYNTHETIC_DESCRIPTOR + 4).unwrap();
    (bytes, list)
}
#[test]
fn background_one_page_requires_four_chunks_then_separate_terminal_poll() {
    let (bytes, list) = single_page();
    let mut upload = BackgroundUpload::default();
    for (chunk, fill) in [0x11, 0x33, 0x55, 0x77].into_iter().enumerate() {
        assert_eq!(
            upload
                .poll_prepared(&prepare_background(&bytes, &list), [1, 2, 3, 4], 0)
                .unwrap(),
            BackgroundEffect::Chunk {
                destination: 0x900 + (chunk * BACKGROUND_CHUNK_BYTES) as u16,
                bytes: vec![fill; BACKGROUND_CHUNK_BYTES],
                count_minus_one: 0x3f,
                commit: UploadCommit::AtInterrupt,
                selected_bank: 0,
                queue_terminator: 0xff
            }
        );
        assert!(
            !upload.completed(),
            "last page does not process negative tag on same call"
        );
    }
    assert_eq!(
        upload
            .poll_prepared(&prepare_background(&bytes, &list), [1, 2, 3, 4], 0)
            .unwrap(),
        BackgroundEffect::PaletteTail {
            base: [1, 2, 3, 4],
            active: [1, 2, 3, 4],
            terminal_tag: 0xff
        }
    );
    assert!(upload.completed());
    assert_eq!(
        upload
            .poll_prepared(&prepare_background(&bytes, &list), [9; 4], 0)
            .unwrap(),
        BackgroundEffect::None
    );
}
#[test]
fn suspended_depth_any_nonzero_flushes_each_chunk_immediately() {
    let (bytes, list) = single_page();
    for depth in [1, 2, 127, 128, 255] {
        let mut upload = BackgroundUpload::default();
        match upload
            .poll_prepared(&prepare_background(&bytes, &list), [0; 4], depth)
            .unwrap()
        {
            BackgroundEffect::Chunk {
                commit,
                count_minus_one,
                ..
            } => assert_eq!((commit, count_minus_one), (UploadCommit::Immediate, 0x3f)),
            other => panic!("unexpected {other:?}"),
        }
    }
}
#[test]
fn background_source_and_destination_carry_after_fourth_chunk() {
    let mut bytes = synthetic(&[0x01, 0x8f, 0x09, 0xff]);
    bytes[0xf10..0x1010].fill(0x25);
    bytes[0x1010..0x1110].fill(0x7a);
    let list = decode_upload_list(&bytes, SYNTHETIC_DESCRIPTOR, SYNTHETIC_DESCRIPTOR + 4).unwrap();
    let mut upload = BackgroundUpload::default();
    for chunk in 0..8 {
        let expected_fill = if chunk < 4 { 0x25 } else { 0x7a };
        match upload
            .poll_prepared(&prepare_background(&bytes, &list), [0; 4], 0)
            .unwrap()
        {
            BackgroundEffect::Chunk {
                destination, bytes, ..
            } => {
                assert_eq!(destination, 0x900 + (chunk * BACKGROUND_CHUNK_BYTES) as u16);
                assert_eq!(bytes, vec![expected_fill; BACKGROUND_CHUNK_BYTES]);
            }
            other => panic!("unexpected {other:?}"),
        }
    }
    assert!(matches!(
        upload
            .poll_prepared(&prepare_background(&bytes, &list), [0; 4], 0)
            .unwrap(),
        BackgroundEffect::PaletteTail { .. }
    ));
}
#[test]
fn empty_list_completes_without_bank_or_chunk_and_preserves_actual_negative_tag() {
    for tag in [0x80, 0x99, 0xff] {
        let bytes = synthetic(&[tag]);
        let list =
            decode_upload_list(&bytes, SYNTHETIC_DESCRIPTOR, SYNTHETIC_DESCRIPTOR + 1).unwrap();
        let mut upload = BackgroundUpload::default();
        assert_eq!(
            upload
                .poll_prepared(&prepare_background(&bytes, &list), [2, 4, 6, 8], 1)
                .unwrap(),
            BackgroundEffect::PaletteTail {
                base: [2, 4, 6, 8],
                active: [2, 4, 6, 8],
                terminal_tag: tag
            }
        );
        assert!(upload.completed());
    }
}
#[test]
fn record_boundary_restarts_source_and_destination_and_changes_bank_on_next_poll() {
    let mut bytes = synthetic(&[0x00, 0x80, 0x09, 0x10, 0x80, 0x0e, 0xff]);
    bytes[0x10..0x110].fill(0x24);
    bytes[0x4010..0x4110].fill(0x68);
    let list = decode_upload_list(&bytes, SYNTHETIC_DESCRIPTOR, SYNTHETIC_DESCRIPTOR + 7).unwrap();
    let mut upload = BackgroundUpload::default();
    for _ in 0..4 {
        upload
            .poll_prepared(&prepare_background(&bytes, &list), [0; 4], 0)
            .unwrap();
    }
    assert_eq!(
        upload
            .poll_prepared(&prepare_background(&bytes, &list), [0; 4], 0)
            .unwrap(),
        BackgroundEffect::Chunk {
            destination: 0xe00,
            bytes: vec![0x68; BACKGROUND_CHUNK_BYTES],
            count_minus_one: 0x3f,
            commit: UploadCommit::AtInterrupt,
            selected_bank: 1,
            queue_terminator: 0xff
        }
    );
}
#[test]
fn all_original_background_uploads_emit_exactly_every_source_byte_and_final_palette() {
    let bytes = rom();
    for selector in 0..ORIGINAL_BACKGROUND_LISTS {
        let list = original_background_list(&bytes, selector).unwrap();
        let mut upload = BackgroundUpload::default();
        for span in &list.records {
            let count = usize::from(span.source_pages) * SOURCE_PAGE_BYTES / BACKGROUND_CHUNK_BYTES;
            for chunk in 0..count {
                let start = span.source_offset + chunk * BACKGROUND_CHUNK_BYTES;
                assert_eq!(
                    upload
                        .poll_prepared(&prepare_background(&bytes, &list), [7, 8, 9, 10], 1)
                        .unwrap(),
                    BackgroundEffect::Chunk {
                        destination: span.destination + (chunk * BACKGROUND_CHUNK_BYTES) as u16,
                        bytes: bytes[start..start + BACKGROUND_CHUNK_BYTES].to_vec(),
                        count_minus_one: 0x3f,
                        commit: UploadCommit::Immediate,
                        selected_bank: span.bank,
                        queue_terminator: 0xff
                    }
                );
            }
        }
        assert_eq!(
            upload
                .poll_prepared(&prepare_background(&bytes, &list), [7, 8, 9, 10], 1)
                .unwrap(),
            BackgroundEffect::PaletteTail {
                base: [7, 8, 9, 10],
                active: [7, 8, 9, 10],
                terminal_tag: 0xff
            }
        );
    }
}

#[test]
fn prepared_backgrounds_preserve_every_original_poll_effect_and_cursor() {
    let bytes = rom();
    for selector in 0..ORIGINAL_BACKGROUND_LISTS {
        let list = original_background_list(&bytes, selector).unwrap();
        let prepared = LegacyPreparedBackground {
            spans: list
                .records
                .iter()
                .map(|span| LegacyPreparedBackgroundSpan {
                    bank: span.bank,
                    destination: span.destination,
                    bytes: bytes[span.source_offset
                        ..span.source_offset + usize::from(span.source_pages) * SOURCE_PAGE_BYTES]
                        .to_vec(),
                })
                .collect(),
            terminator: list.terminator,
        };
        for depth in [0, 1, 0x80, 0xff] {
            let mut original = LegacyBackgroundUpload::default();
            let prepared_plan = PreparedBackground::new(
                prepared
                    .spans
                    .iter()
                    .map(|span| PreparedBackgroundSpan {
                        bank: span.bank,
                        destination: span.destination,
                        bytes: span.bytes.clone(),
                    })
                    .collect(),
                prepared.terminator,
            )
            .unwrap();
            let mut decoded = BackgroundUpload::default();
            let bound = list
                .records
                .iter()
                .map(|span| usize::from(span.source_pages) * 4)
                .sum::<usize>()
                + 1;
            for _ in 0..bound {
                let expected = original.poll(&bytes, &list, [7, 8, 9, 10], depth).unwrap();
                let actual = decoded
                    .poll_prepared(&prepared_plan, [7, 8, 9, 10], depth)
                    .unwrap();
                assert_eq!(
                    actual, expected,
                    "selector {selector}, suspend depth {depth}"
                );
                assert_eq!(
                    decoded.position(&prepared_plan),
                    (original.record, original.chunk_in_record)
                );
                assert_eq!(decoded.completed(), original.completed);
            }
            assert!(original.completed);
            assert_eq!(
                decoded
                    .poll_prepared(&prepared_plan, [0; 4], depth)
                    .unwrap(),
                BackgroundEffect::None
            );
        }
    }
}
#[test]
fn prepared_background_validation_errors_preserve_continuation_before_advancing() {
    for (length, chunk) in [(63, 0), (65, 0), (0, 0), (64, 1), (256, 4)] {
        let prepared = LegacyPreparedBackground {
            spans: vec![LegacyPreparedBackgroundSpan {
                bank: 3,
                destination: 0xfff0,
                bytes: vec![7; length],
            }],
            terminator: 0x81,
        };
        let mut upload = LegacyBackgroundUpload {
            chunk_in_record: chunk,
            ..Default::default()
        };
        let before = upload.clone();
        assert!(upload.poll_prepared(&prepared, [1, 2, 3, 4], 0).is_err());
        assert_eq!(upload, before, "length {length}, chunk {chunk}");
    }
}

fn id(name: &str) -> AssetId {
    AssetId(name.to_owned())
}
fn catalog_fixture(extra: usize) -> (Assets, CatalogInput) {
    // Authored resource fixtures avoid interpreting another area's format. The
    // catalog joins typed IDs; their payload validators belong to their owners.
    let resource_specs = [
        ("fixture/rooms", ResourceKind::Rooms),
        ("fixture/terrain", ResourceKind::Terrain),
        ("fixture/spawns", ResourceKind::Spawns),
        ("fixture/chr", ResourceKind::Graphics),
        ("fixture/audio", ResourceKind::Audio),
    ];
    let assets = resource_specs
        .iter()
        .map(|&(name, _)| Asset {
            id: name.into(),
            bytes: vec![0x5a],
            origin: Origin::Rom {
                offset: 0,
                length: 1,
            },
        })
        .collect();
    let resources = resource_specs
        .iter()
        .map(|&(name, kind)| ResourceReference { id: id(name), kind })
        .collect();
    let levels = (0..10 + extra)
        .map(|index| LevelDefinition {
            id: id(&format!("fixture/level-{index}")),
            original_selector: if index < 10 {
                Some(StageId(index as u8))
            } else {
                None
            },
            rooms: id("fixture/rooms"),
            terrain: id("fixture/terrain"),
            spawns: if index == 9 {
                None
            } else {
                Some(id("fixture/spawns"))
            },
            graphics: vec![id("fixture/chr")],
            audio: vec![id("fixture/audio")],
        })
        .collect();
    (
        Assets {
            source: Collection { assets },
        },
        CatalogInput { levels, resources },
    )
}
#[test]
fn original_catalog_preserves_ten_layout_selectors_and_ending_has_no_spawn_selector() {
    let bytes = rom();
    let rom = Rom::from_bytes(bytes, config::ROM_SHA256).unwrap();
    let raw = Assets::load(&rom, &Default::default()).unwrap();
    let input = original_catalog(&raw).unwrap();
    assert_eq!(input.levels.len(), 10);
    for selector in 0..10 {
        let level = input
            .levels
            .iter()
            .find(|level| level.original_selector == Some(StageId(selector)))
            .expect("all source layout selectors");
        assert!(!level.id.0.is_empty());
        assert_eq!(
            level.spawns.is_none(),
            selector == 9,
            "ending never indexes nine-entry spawn table"
        );
    }
    let decoded = DecodedAssets::resolve(&raw, input).unwrap();
    for selector in 0..10 {
        assert!(decoded.original_level(StageId(selector)).is_some());
    }
    assert!(decoded.original_level(StageId(10)).is_none());
}
#[test]
fn added_named_levels_resolve_alongside_all_originals_without_fixed_ten_limit() {
    let (raw, input) = catalog_fixture(27);
    let originals = input.levels[..10].to_vec();
    let extras = input.levels[10..].to_vec();
    let decoded = DecodedAssets::resolve(&raw, input).unwrap();
    assert_eq!(decoded.levels.len(), 37);
    for original in originals {
        assert_eq!(
            decoded.original_level(original.original_selector.unwrap()),
            Some(&original)
        );
        assert_eq!(decoded.level(&original.id), Some(&original));
    }
    for extra in extras {
        assert_eq!(decoded.level(&extra.id), Some(&extra));
    }
    assert!(decoded.level(&id("missing")).is_none());
}
#[test]
fn duplicate_level_names_and_source_selector_aliases_are_rejected() {
    for duplicate_selector in [false, true] {
        let (raw, mut input) = catalog_fixture(1);
        if duplicate_selector {
            input.levels[10].original_selector = Some(StageId(0));
        } else {
            input.levels[10].id = input.levels[0].id.clone();
        }
        assert!(DecodedAssets::resolve(&raw, input).is_err());
    }
}
#[test]
fn all_resource_kinds_require_existing_named_payload_and_correct_kind() {
    for kind in [
        ResourceKind::Rooms,
        ResourceKind::Terrain,
        ResourceKind::Spawns,
        ResourceKind::Graphics,
        ResourceKind::Audio,
    ] {
        let (mut raw, input) = catalog_fixture(1);
        let missing = input
            .resources
            .iter()
            .find(|reference| reference.kind == kind)
            .unwrap()
            .id
            .clone();
        raw.source.assets.retain(|asset| asset.id != missing.0);
        assert!(
            DecodedAssets::resolve(&raw, input).is_err(),
            "missing {kind:?}"
        );
        let (raw, mut input) = catalog_fixture(1);
        let reference = input
            .resources
            .iter_mut()
            .find(|reference| reference.kind == kind)
            .unwrap();
        reference.kind = if kind == ResourceKind::Audio {
            ResourceKind::Graphics
        } else {
            ResourceKind::Audio
        };
        assert!(
            DecodedAssets::resolve(&raw, input).is_err(),
            "wrong kind {kind:?}"
        );
    }
}
#[test]
fn invalid_ids_duplicate_resources_and_empty_resource_payload_are_rejected() {
    for bad in ["", "bad?name", "bad name"] {
        let (raw, mut input) = catalog_fixture(1);
        input.levels[10].id = id(bad);
        assert!(DecodedAssets::resolve(&raw, input).is_err());
    }
    let (raw, mut input) = catalog_fixture(1);
    input.resources.push(input.resources[0].clone());
    assert!(DecodedAssets::resolve(&raw, input).is_err());
    let (mut raw, input) = catalog_fixture(1);
    raw.source.assets[0].bytes.clear();
    assert!(DecodedAssets::resolve(&raw, input).is_err());
}
#[test]
fn extensions_cannot_claim_nonexistent_source_selector_or_missing_graphics_audio() {
    let (raw, mut input) = catalog_fixture(1);
    input.levels[10].original_selector = Some(StageId(10));
    assert!(DecodedAssets::resolve(&raw, input).is_err());
    for graphics in [false, true] {
        let (raw, mut input) = catalog_fixture(1);
        if graphics {
            input.levels[10].graphics[0] = id("missing/graphics");
        } else {
            input.levels[10].audio[0] = id("missing/audio");
        }
        assert!(DecodedAssets::resolve(&raw, input).is_err());
    }
}
#[test]
fn catalog_validation_is_immutable_and_resolved_ids_do_not_borrow_raw_payloads() {
    let (raw, input) = catalog_fixture(1);
    let before: Vec<_> = raw
        .source
        .assets
        .iter()
        .map(|asset| (asset.id.clone(), asset.bytes.clone()))
        .collect();
    let decoded = DecodedAssets::resolve(&raw, input.clone()).unwrap();
    let extra = decoded.level(&input.levels[10].id).unwrap().clone();
    assert_eq!(
        raw.source
            .assets
            .iter()
            .map(|asset| (asset.id.clone(), asset.bytes.clone()))
            .collect::<Vec<_>>(),
        before
    );
    drop(raw);
    assert_eq!(decoded.level(&extra.id), Some(&extra));
}
#[test]
fn named_file_level_manifest_joins_resources_instead_of_treating_raw_slot_as_level() {
    let (raw, _) = catalog_fixture(0);
    let json=br#"{"version":1,"resources":[{"id":"fixture/rooms","kind":"rooms"},{"id":"fixture/terrain","kind":"terrain"},{"id":"fixture/spawns","kind":"spawns"},{"id":"fixture/chr","kind":"graphics"},{"id":"fixture/audio","kind":"audio"}],"levels":[{"id":"authored/cave","rooms":"fixture/rooms","terrain":"fixture/terrain","spawns":"fixture/spawns","graphics":["fixture/chr"],"audio":["fixture/audio"]}]}"#;
    let input = load_level_manifest(&raw.source, json).unwrap();
    assert_eq!(input.levels.len(), 1);
    let decoded = DecodedAssets::resolve(&raw, input).unwrap();
    let level = decoded.level(&id("authored/cave")).unwrap();
    assert_eq!(level.original_selector, None);
    assert_eq!(level.graphics, vec![id("fixture/chr")]);
    assert_eq!(level.audio, vec![id("fixture/audio")]);
}
#[test]
fn malformed_unknown_version_and_missing_manifest_references_are_rejected() {
    let (raw, _) = catalog_fixture(0);
    for json in [b"{".as_slice(),br#"{"version":2,"levels":[],"resources":[]}"#,br#"{"version":1,"levels":[{"id":"new","rooms":"missing","terrain":"missing","graphics":["missing"],"audio":["missing"]}],"resources":[]}"#] {
  assert!(load_level_manifest(&raw.source,json).is_err());
 }
}

#[test]
fn every_chr_and_background_source_span_resolves_through_raw_named_catalog() {
    let bytes = rom();
    let image = Rom::from_bytes(bytes.clone(), config::ROM_SHA256).unwrap();
    let raw = Assets::load(&image, &Default::default()).unwrap();
    for table in [CHR_EXPECTED, BACKGROUND_EXPECTED] {
        for (records, _) in table {
            for &(bank, source_offset, source_pages, destination) in *records {
                let span = UploadSpan {
                    bank,
                    source_offset,
                    source_pages,
                    destination,
                };
                assert_eq!(
                    resolve_upload_bytes(&raw, &span).unwrap(),
                    bytes[source_offset
                        ..source_offset + usize::from(source_pages) * SOURCE_PAGE_BYTES]
                );
            }
        }
    }
}
#[test]
fn named_graphics_overrides_apply_across_original_incbin_fragment_boundaries() {
    let bytes = rom();
    let image = Rom::from_bytes(bytes, config::ROM_SHA256).unwrap();
    let mut raw = Assets::load(&image, &Default::default()).unwrap();
    raw.source
        .assets
        .iter_mut()
        .find(|asset| asset.id == "ChrTiles_00F10")
        .unwrap()
        .bytes
        .fill(0x21);
    raw.source
        .assets
        .iter_mut()
        .find(|asset| asset.id == "AssetData_01010")
        .unwrap()
        .bytes
        .fill(0x43);
    let span = UploadSpan {
        bank: 0,
        source_offset: 0xf10,
        source_pages: 2,
        destination: 0xa00,
    };
    assert_eq!(
        resolve_upload_bytes(&raw, &span).unwrap(),
        [vec![0x21; SOURCE_PAGE_BYTES], vec![0x43; SOURCE_PAGE_BYTES]].concat()
    );
}
#[test]
fn missing_named_recipe_fragment_does_not_fall_back_to_rom_or_instruction_bytes() {
    let bytes = rom();
    let image = Rom::from_bytes(bytes, config::ROM_SHA256).unwrap();
    let mut raw = Assets::load(&image, &Default::default()).unwrap();
    raw.source
        .assets
        .retain(|asset| asset.id != "AssetData_01010");
    assert!(
        resolve_upload_bytes(
            &raw,
            &UploadSpan {
                bank: 0,
                source_offset: 0xf10,
                source_pages: 2,
                destination: 0xa00
            }
        )
        .is_err()
    );
    assert!(
        resolve_upload_bytes(
            &raw,
            &UploadSpan {
                bank: 3,
                source_offset: 0xc010,
                source_pages: 1,
                destination: 0
            }
        )
        .is_err()
    );
    assert!(
        resolve_upload_bytes(
            &raw,
            &UploadSpan {
                bank: 0,
                source_offset: usize::MAX,
                source_pages: 16,
                destination: 0
            }
        )
        .is_err()
    );
}
#[test]
fn all_four_terrain_banks_load_full_block_and_plane_sets_through_generic_records() {
    let bytes = rom();
    let image = Rom::from_bytes(bytes.clone(), config::ROM_SHA256).unwrap();
    let raw = Assets::load(&image, &Default::default()).unwrap();
    for bank in 0..4 {
        let start = INES_HEADER_BYTES + usize::from(bank) * PRG_BANK_BYTES + 0x3300;
        let actual = terrain_payload(&raw, bank).unwrap();
        assert_eq!(actual.block_records, bytes[start..start + 0x800]);
        for plane in 0..4 {
            let plane_start = start + 0x800 + plane * SOURCE_PAGE_BYTES;
            assert_eq!(
                actual.tile_planes[plane],
                bytes[plane_start..plane_start + SOURCE_PAGE_BYTES]
            );
        }
        assert_eq!(
            actual.packed_attributes,
            bytes[start + 0xc00..start + 0xd00]
        );
    }
    for bank in [4, 7, 255] {
        assert!(terrain_payload(&raw, bank).is_err());
    }
}
#[test]
fn genuinely_file_backed_level_graphics_audio_and_map_references_resolve_together() {
    let (mut raw, input) = catalog_fixture(1);
    for asset in &mut raw.source.assets {
        asset.origin = Origin::File(std::path::PathBuf::from(format!(
            "authored/{}.bin",
            asset.id.replace('/', "_")
        )));
    }
    let expected = input.levels[10].clone();
    let decoded = DecodedAssets::resolve(&raw, input).unwrap();
    assert_eq!(decoded.level(&expected.id), Some(&expected));
    assert!(decoded.original_level(StageId(0)).is_some());
}

#[test]
fn ending_scene_resources_borrow_layout_nine_but_load_explicit_terrain_and_chr_without_tenth_stage_config()
 {
    use crate::scenes::Scene;
    let bytes = rom();
    let source = Rom::from_bytes(bytes.clone(), config::ROM_SHA256).unwrap();
    let raw = Assets::load(&source, &Default::default()).unwrap();
    // Distinct synthetic layouts establish the actual indexed resource; there
    // are deliberately no gameplay terrain or spawn configuration arrays here.
    let world = crate::world::WorldCatalog {
        rooms: vec![],
        layouts: (0..10)
            .map(|index| crate::world::Layout {
                rows: 1,
                columns: 1,
                rooms: vec![Some(index)],
            })
            .collect(),
        terrain: vec![],
    };
    for (_scene, bank, chr) in [
        (Scene::OpeningFirst, 2u8, 9usize),
        (Scene::OpeningSecond, 2, 9),
        (Scene::OpeningThird, 2, 9),
        (Scene::Interior, 1, 9),
        (Scene::Value, 1, 10),
        (Scene::TerminalLead, 1, 10),
    ] {
        let context = SceneResourceContext {
            layout: crate::scenes::ENDING_LAYOUT,
            terrain: bank,
            chr: chr as u8,
            spawn_configuration: None,
        };
        let bound = original_scene_assets(&raw, &world, context).unwrap();
        assert!(std::ptr::eq(bound.layout, &world.layouts[9]));
        assert_eq!(bound.spawn_configuration, None);
        let terrain_start = INES_HEADER_BYTES + usize::from(bank) * PRG_BANK_BYTES + 0x3300;
        assert_eq!(
            bound.terrain.block_records,
            bytes[terrain_start..terrain_start + 0x800]
        );
        assert_eq!(
            bound.terrain.packed_attributes,
            bytes[terrain_start + 0xc00..terrain_start + 0xd00]
        );
        let (expected, end) = CHR_EXPECTED[chr];
        assert_eq!(bound.chr.terminator_offset, end);
        assert_eq!(
            bound
                .chr
                .records
                .iter()
                .map(|span| (
                    span.bank,
                    span.source_offset,
                    span.source_pages,
                    span.destination
                ))
                .collect::<Vec<_>>(),
            expected
        );
    }
}
#[test]
fn raw_selector_twenty_one_chr_is_banked_negative_terminator_and_background_is_list_sixteen() {
    let bytes = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap();
    let chr = prepare_raw_chr(&bytes, 21, 5, 0x91).unwrap();
    assert!(chr.copies.is_empty());
    assert_eq!((chr.control, chr.selected_bank), (0x90, None));
    let background = RawBackgroundCatalog::original(&bytes)
        .unwrap()
        .select(BackgroundContext {
            stage: crate::identifiers::StageId(21),
            room_row: 26,
            camera_page: 0,
            display_suspend_depth: 1,
        });
    assert_eq!(background.list, 16);
    assert_eq!(background.palette.as_slice(), &bytes[0x1c705..0x1c709]);
}

#[test]
fn background_plan_preserves_wrapping_destination_live_commit_policy_and_late_palette() {
    // C4B9..C4D5 advances the destination modulo 16 bits; C4DB reads the
    // current suspension depth for each chunk, and C4E9 reads the palette only
    // at the negative endpoint. These are runtime inputs, never frozen in a plan.
    let prepared = PreparedBackground::new(
        vec![PreparedBackgroundSpan {
            bank: 3,
            destination: 0xfff0,
            bytes: vec![7; SOURCE_PAGE_BYTES],
        }],
        0x99,
    )
    .unwrap();
    let mut upload = BackgroundUpload::default();
    for (index, depth) in [0, 1, 0, 255].into_iter().enumerate() {
        assert_eq!(
            upload
                .poll_prepared(&prepared, [index as u8; 4], depth)
                .unwrap(),
            BackgroundEffect::Chunk {
                destination: 0xfff0u16.wrapping_add((index * BACKGROUND_CHUNK_BYTES) as u16),
                bytes: vec![7; BACKGROUND_CHUNK_BYTES],
                count_minus_one: 0x3f,
                commit: if depth == 0 {
                    UploadCommit::AtInterrupt
                } else {
                    UploadCommit::Immediate
                },
                selected_bank: 3,
                queue_terminator: 0xff,
            }
        );
        assert!(!upload.completed());
    }
    assert_eq!(
        upload.poll_prepared(&prepared, [8; 4], 0).unwrap(),
        BackgroundEffect::PaletteTail {
            base: [8; 4],
            active: [8; 4],
            terminal_tag: 0x99
        }
    );
    assert_eq!(
        upload.poll_prepared(&prepared, [9; 4], 1).unwrap(),
        BackgroundEffect::None
    );
}

// Frozen iteration-007 continuation, retained only as a test oracle. Production
// gameplay uses the decoded plan below; source-backed expectations above remain
// independent of both implementations.
#[derive(Clone)]
struct LegacyPreparedBackgroundSpan {
    bank: u8,
    destination: u16,
    bytes: Vec<u8>,
}
#[derive(Clone)]
struct LegacyPreparedBackground {
    spans: Vec<LegacyPreparedBackgroundSpan>,
    terminator: u8,
}
/// Foreground upload continuation, not another PPU queue owner. B03 consumes its
/// request immediately or at B01's interrupt boundary as specified by the effect.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct LegacyBackgroundUpload {
    completed: bool,
    record: usize,
    chunk_in_record: usize,
}
impl LegacyBackgroundUpload {
    fn finish_palette(&mut self, palette: [u8; 4], terminal_tag: u8) -> BackgroundEffect {
        self.completed = true;
        BackgroundEffect::PaletteTail {
            base: palette,
            active: palette,
            terminal_tag,
        }
    }
    /// Both immutable source adapters validate/read before entering this one
    /// continuation transition. The palette endpoint is a later, separate poll.
    fn advance_chunk(
        &mut self,
        bytes: Vec<u8>,
        destination: u16,
        bank: u8,
        span_bytes: usize,
        suspend_depth: u8,
    ) -> BackgroundEffect {
        const QUEUE_TERMINATOR: u8 = 0xff;
        self.chunk_in_record += 1;
        if self.chunk_in_record * BACKGROUND_CHUNK_BYTES == span_bytes {
            self.record += 1;
            self.chunk_in_record = 0;
        }
        BackgroundEffect::Chunk {
            destination,
            bytes,
            count_minus_one: (BACKGROUND_CHUNK_BYTES - 1) as u8,
            commit: if suspend_depth == 0 {
                UploadCommit::AtInterrupt
            } else {
                UploadCommit::Immediate
            },
            selected_bank: bank,
            queue_terminator: QUEUE_TERMINATOR,
        }
    }
    fn poll(
        &mut self,
        rom: &[u8],
        list: &UploadList,
        palette: [u8; 4],
        suspend_depth: u8,
    ) -> anyhow::Result<BackgroundEffect> {
        if self.completed {
            return Ok(BackgroundEffect::None);
        }
        let Some(span) = list.records.get(self.record) else {
            return Ok(self.finish_palette(palette, list.terminator));
        };
        let offset = self.chunk_in_record * BACKGROUND_CHUNK_BYTES;
        let start = span
            .source_offset
            .checked_add(offset)
            .context("background source overflow")?;
        let bytes = rom
            .get(start..start + BACKGROUND_CHUNK_BYTES)
            .context("background source bounds")?
            .to_vec();
        let destination = span.destination.wrapping_add(offset as u16);
        Ok(self.advance_chunk(
            bytes,
            destination,
            span.bank,
            usize::from(span.source_pages) * SOURCE_PAGE_BYTES,
            suspend_depth,
        ))
    }
}
impl LegacyBackgroundUpload {
    fn poll_prepared(
        &mut self,
        prepared: &LegacyPreparedBackground,
        palette: [u8; 4],
        suspend_depth: u8,
    ) -> anyhow::Result<BackgroundEffect> {
        if self.completed {
            return Ok(BackgroundEffect::None);
        }
        let Some(span) = prepared.spans.get(self.record) else {
            return Ok(self.finish_palette(palette, prepared.terminator));
        };
        ensure!(
            span.bytes.len().is_multiple_of(BACKGROUND_CHUNK_BYTES),
            "prepared background chunk alignment"
        );
        let offset = self.chunk_in_record * BACKGROUND_CHUNK_BYTES;
        let bytes = span
            .bytes
            .get(offset..offset + BACKGROUND_CHUNK_BYTES)
            .context("prepared background chunk")?
            .to_vec();
        Ok(self.advance_chunk(
            bytes,
            span.destination.wrapping_add(offset as u16),
            span.bank,
            span.bytes.len(),
            suspend_depth,
        ))
    }
}

fn prepare_background(bytes: &[u8], list: &UploadList) -> PreparedBackground {
    PreparedBackground::new(
        list.records
            .iter()
            .map(|span| PreparedBackgroundSpan {
                bank: span.bank,
                destination: span.destination,
                bytes: bytes[span.source_offset
                    ..span.source_offset + usize::from(span.source_pages) * SOURCE_PAGE_BYTES]
                    .to_vec(),
            })
            .collect(),
        list.terminator,
    )
    .unwrap()
}
#[test]
fn background_plan_rejects_invalid_spans_at_asset_boundary_and_accepts_empty_list() {
    for length in [0, 63, 65] {
        assert!(
            PreparedBackground::new(
                vec![PreparedBackgroundSpan {
                    bank: 3,
                    destination: 0xfff0,
                    bytes: vec![7; length],
                }],
                0x81
            )
            .is_err()
        );
    }
    let empty = PreparedBackground::new(Vec::new(), 0x81).unwrap();
    let mut upload = BackgroundUpload::default();
    assert_eq!(upload.position(&empty), (0, 0));
    assert_eq!(
        upload.poll_prepared(&empty, [6; 4], 0).unwrap(),
        BackgroundEffect::PaletteTail {
            base: [6; 4],
            active: [6; 4],
            terminal_tag: 0x81
        }
    );
    assert!(upload.completed());
}

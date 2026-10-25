//! Terrain geometry to ordered video writes/source framing. The source-format
//! queue remains observable to audio, runtime composites and attribute aliases;
//! native world rules do not construct addresses, headers or field offsets.
use super::VideoWrite;
use crate::world::{ColumnUpdate, PaletteRun, TerrainUpdate, TileOrigin, TilePage};
const NAMETABLE_FIRST: u16 = 0x2000;
const NAMETABLE_SECOND: u16 = 0x2400;
const ATTRIBUTE_OFFSET: u16 = 0x3c0;
const TILE_ROW: u16 = 32;
pub(super) fn page_address(page: TilePage) -> u16 {
    match page {
        TilePage::First => NAMETABLE_FIRST,
        TilePage::Second => NAMETABLE_SECOND,
    }
}
pub(crate) fn tile_address(origin: TileOrigin) -> u16 {
    page_address(origin.page) + u16::from(origin.row) * TILE_ROW + u16::from(origin.column)
}
fn palette_writes(palette: &PaletteRun) -> impl Iterator<Item = VideoWrite> + '_ {
    let address = page_address(palette.page) + ATTRIBUTE_OFFSET + u16::from(palette.first_cell);
    palette
        .values
        .iter()
        .enumerate()
        .map(move |(index, &value)| VideoWrite {
            address: address.wrapping_add(index as u16),
            value,
        })
}
pub fn writes(update: &TerrainUpdate) -> Vec<VideoWrite> {
    let origin = tile_address(update.region.origin);
    let mut writes = Vec::new();
    for (row, tiles) in update.region.tiles.chunks(update.region.width).enumerate() {
        writes.extend(tiles.iter().enumerate().map(|(column, &value)| {
            VideoWrite {
                address: origin
                    .wrapping_add(row as u16 * TILE_ROW)
                    .wrapping_add(column as u16),
                value,
            }
        }));
    }
    if let Some(palette) = &update.palette {
        writes.extend(palette_writes(palette));
    }
    writes
}
/// Immutable observation for source contracts. Actual column submission also
/// preserves discarded-row queue writes and live cache/alias ordering in Video.
pub fn column_writes(update: &ColumnUpdate) -> Vec<VideoWrite> {
    let origin = tile_address(update.origin);
    let mut writes = Vec::new();
    for row in update.rows.range() {
        writes.extend(update.tiles[row * 4..row * 4 + 4].iter().enumerate().map(
            |(column, &value)| VideoWrite {
                address: origin + row as u16 * TILE_ROW + column as u16,
                value,
            },
        ));
    }
    writes.extend(palette_writes(&update.palette));
    writes
}
/// Source primary records use a last-index count and combine contiguous spans.
/// This translation is transient; the sole surviving state is the canonical
/// video queue, including any overflow into secondary/cache aliases.
pub fn encoded(update: &TerrainUpdate) -> Vec<u8> {
    let writes = writes(update);
    let mut bytes = Vec::new();
    let mut cursor = 0;
    while cursor < writes.len() {
        let start = cursor;
        let address = writes[start].address;
        cursor += 1;
        while cursor < writes.len()
            && cursor - start < 128
            && writes[cursor].address == address.wrapping_add((cursor - start) as u16)
        {
            cursor += 1;
        }
        bytes.extend([
            (address >> 8) as u8,
            address as u8,
            (cursor - start - 1) as u8,
        ]);
        bytes.extend(writes[start..cursor].iter().map(|write| write.value));
    }
    bytes.push(0xff);
    bytes
}
pub(super) fn changed_record(
    update: &TerrainUpdate,
) -> [u8; super::CHANGED_RECORD_BYTES as usize + 1] {
    assert_eq!((update.region.width, update.region.height()), (2, 2));
    let palette = update
        .palette
        .as_ref()
        .expect("changed block has a palette cell");
    assert_eq!(palette.values.len(), 1);
    let tile = tile_address(update.region.origin);
    let attribute = page_address(palette.page) + ATTRIBUTE_OFFSET + u16::from(palette.first_cell);
    let mut encoded = [0; super::CHANGED_RECORD_BYTES as usize + 1];
    for (at, address, count, values) in [
        (0, tile, 1, &update.region.tiles[..2]),
        (5, tile + TILE_ROW, 1, &update.region.tiles[2..]),
        (10, attribute, 0, &palette.values[..]),
    ] {
        encoded[at] = (address >> 8) as u8;
        encoded[at + 1] = address as u8;
        encoded[at + 2] = count;
        encoded[at + 3..at + 3 + values.len()].copy_from_slice(values);
    }
    encoded[super::CHANGED_RECORD_BYTES as usize] = 0xff;
    encoded
}

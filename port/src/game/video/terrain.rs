//! Alias-sensitive terrain submission boundary. World and scheduling code send
//! native geometry; source queue bytes are retained only because other owners
//! can observe and change them before the interrupt commits display writes.
use super::*;
use crate::world;
impl Video {
    pub fn finish_terrain_batch(
        &mut self,
        queue_offset: u8,
        attributes: &mut [u8; world::CACHE_CELLS],
    ) {
        self.write_primary_byte(usize::from(queue_offset), u8::MAX, attributes);
        if self.display.suspended == 0 {
            self.display.primary.pending = u8::MAX;
        } else {
            let bytes = self.primary_bytes(attributes);
            self.display.flush(
                presentation::Queue::Primary,
                &bytes,
                presentation::Stride::Across,
            );
            self.commit_writes();
        }
    }
    pub fn prepare_column(
        &mut self,
        update: &world::ColumnUpdate,
        cache_write: (usize, u8),
        queue_offset: u8,
        terminate: bool,
        attributes: &mut [u8; world::CACHE_CELLS],
    ) -> u8 {
        const TILE_RECORD_BYTES: usize = 7;
        const TILE_PAYLOAD: usize = 3;
        const NAMETABLE_ROW: u16 = 32;
        const ATTRIBUTE_RECORD_BYTES: u8 = 4;
        let base = presentation::terrain::tile_address(update.origin);
        // Header operands retain byte Y but add each absolute field offset
        // without wrapping. Payload Y is recomputed with byte addition for
        // each quadrant. These differ when a large camera batch crossesFF.
        for row in 0..4 {
            let at = usize::from(queue_offset) + row * TILE_RECORD_BYTES;
            let address = base + row as u16 * NAMETABLE_ROW;
            for (field, value) in [(0, (address >> 8) as u8), (1, address as u8), (2, 3)] {
                self.write_primary_byte(at + field, value, attributes);
            }
        }
        for quadrant in (0..4).rev() {
            let source = (quadrant / 2) * 8 + (quadrant % 2) * 2;
            let row_offset = (quadrant / 2) * 2 * TILE_RECORD_BYTES + (quadrant % 2) * 2;
            let at = usize::from(queue_offset.wrapping_add(row_offset as u8));
            for (field, source) in [
                (TILE_PAYLOAD, source),
                (TILE_PAYLOAD + 1, source + 1),
                (TILE_PAYLOAD + TILE_RECORD_BYTES, source + 4),
                (TILE_PAYLOAD + TILE_RECORD_BYTES + 1, source + 5),
            ] {
                self.write_primary_byte(at + field, update.tiles[source], attributes);
            }
        }
        let top_edge = update.rows == world::BlockRows::LowerHalf;
        if top_edge {
            for index in 0..2 * TILE_RECORD_BYTES {
                let at = usize::from(queue_offset.wrapping_add(index as u8));
                let value = self.primary_byte(at + 2 * TILE_RECORD_BYTES, attributes);
                self.write_primary_byte(at, value, attributes);
            }
        }
        let tile_bytes = if top_edge || update.rows == world::BlockRows::UpperHalf {
            2 * TILE_RECORD_BYTES
        } else {
            4 * TILE_RECORD_BYTES
        };
        let attribute_at = queue_offset.wrapping_add(tile_bytes as u8);
        let coordinate = update.palette.first_cell;
        let high = 0x23 | self.staging_queue_byte(9 + usize::from(attribute_at), attributes);
        let palette = update.palette.values[0];
        for (field, value) in [high, 0xc0 | coordinate, 0, palette]
            .into_iter()
            .enumerate()
        {
            self.write_primary_byte(usize::from(attribute_at) + field, value, attributes);
        }
        let (attribute_index, attribute_value) = cache_write;
        attributes[attribute_index] = attribute_value;
        // Source C94D terminates only after C92x's canonical cache store;
        // that final absolute +4 can itself alias attribute cell0.
        if terminate {
            self.write_primary_byte(
                usize::from(attribute_at) + usize::from(ATTRIBUTE_RECORD_BYTES),
                u8::MAX,
                attributes,
            );
        }
        if terminate && update.commit == crate::assets::formats::UploadCommit::AtInterrupt {
            self.display.primary.pending = u8::MAX;
        }
        if terminate && update.commit == crate::assets::formats::UploadCommit::Immediate {
            let shared = self.primary_bytes(attributes);
            self.display.flush(
                presentation::Queue::Primary,
                &shared,
                presentation::Stride::Across,
            );
            self.commit_writes();
        }
        attribute_at.wrapping_add(ATTRIBUTE_RECORD_BYTES)
    }
}

const ROOM_BLOCKS: u8 = 8;
const LAST_ROOM_ROW: u8 = 7;
const SECOND_CACHE_PAGE: usize = world::CACHE_CELLS / 2;
const FIRST_NAMETABLE: u16 = 0x2000;
const SECOND_NAMETABLE: u16 = 0x2400;
const NAMETABLE_ATTRIBUTE_OFFSET: u16 = 0x3c0;
const BLOCK_ROW_VIDEO_BYTES: u16 = 128;
const TILE_ROW_VIDEO_BYTES: u16 = 32;
const HALF_BLOCK_VIDEO_BYTES: u16 = 64;
const ROW_TILE_COUNT: usize = 32;
const ROW_COMMANDS: [usize; 3] = [0, 0x23, 0x46];
const ROW_PAYLOADS: [usize; 2] = [3, 0x26];
const RETAINED_ROW_PAYLOADS: [usize; 2] = [0x60, 0x80];
const ROW_ATTRIBUTE_PAYLOAD: usize = 0x49;
const ROW_TERMINATOR: usize = 0x51;
const NEIGHBOR_PAYLOADS: [usize; 4] = [3, 0x16, 0x29, 0x3c];
const NEIGHBOR_COMMAND_BYTES: usize = 19;
const NEIGHBOR_ATTRIBUTE_COMMAND: usize = 0x4c;
const NEIGHBOR_ATTRIBUTE_PAYLOAD: usize = 0x4f;
const NEIGHBOR_TERMINATOR: usize = 0x53;
const PALETTE_LOW_QUADRANTS: u8 = 0x0f;
const PALETTE_HIGH_QUADRANTS: u8 = 0xf0;
impl Video {
    fn terrain_header(
        &mut self,
        attributes: &mut [u8; world::CACHE_CELLS],
        offset: usize,
        address: u16,
        count: u8,
    ) {
        for (field, value) in [(0, (address >> 8) as u8), (1, address as u8), (2, count)] {
            self.write_primary_byte(offset + field, value, attributes);
        }
    }
    pub fn publish_vertical_row(
        &mut self,
        ring: u8,
        lower: bool,
        attributes: &mut [u8; world::CACHE_CELLS],
    ) {
        let address = SECOND_NAMETABLE
            + u16::from(ring & LAST_ROOM_ROW) * BLOCK_ROW_VIDEO_BYTES
            + if lower { HALF_BLOCK_VIDEO_BYTES } else { 0 };
        self.terrain_header(
            attributes,
            ROW_COMMANDS[0],
            address,
            (ROW_TILE_COUNT - 1) as u8,
        );
        self.terrain_header(
            attributes,
            ROW_COMMANDS[1],
            address | TILE_ROW_VIDEO_BYTES,
            (ROW_TILE_COUNT - 1) as u8,
        );
        self.terrain_header(
            attributes,
            ROW_COMMANDS[2],
            SECOND_NAMETABLE
                + NAMETABLE_ATTRIBUTE_OFFSET
                + u16::from(ring & LAST_ROOM_ROW) * u16::from(ROOM_BLOCKS),
            LAST_ROOM_ROW,
        );
        self.write_primary_byte(ROW_TERMINATOR, u8::MAX, attributes);
        self.display.primary.pending = 0xff;
    }

    pub fn stage_vertical_block(
        &mut self,
        update: &world::ColumnUpdate,
        cache_write: (usize, u8),
        attributes: &mut [u8; world::CACHE_CELLS],
    ) {
        let column = usize::from(update.origin.column) / 4;
        let first = if update.rows == world::BlockRows::LowerHalf {
            8
        } else {
            0
        };
        let retained = first ^ 8;
        for local in 0..4 {
            let at = column * 4 + local;
            self.write_primary_byte(
                ROW_PAYLOADS[0] + at,
                update.tiles[first + local],
                attributes,
            );
            self.write_primary_byte(
                ROW_PAYLOADS[1] + at,
                update.tiles[first + 4 + local],
                attributes,
            );
            self.write_primary_byte(
                RETAINED_ROW_PAYLOADS[0] + at,
                update.tiles[retained + local],
                attributes,
            );
            self.write_primary_byte(
                RETAINED_ROW_PAYLOADS[1] + at,
                update.tiles[retained + 4 + local],
                attributes,
            );
        }
        attributes[cache_write.0] = cache_write.1;
        self.write_primary_byte(
            ROW_ATTRIBUTE_PAYLOAD + column,
            update.palette.values[0],
            attributes,
        );
    }
    pub fn restore_vertical_row(&mut self, ring: u8, attributes: &mut [u8; world::CACHE_CELLS]) {
        for offset in (0..ROW_TILE_COUNT).rev() {
            let upper = self.primary_byte(RETAINED_ROW_PAYLOADS[0] + offset, attributes);
            let lower = self.primary_byte(RETAINED_ROW_PAYLOADS[1] + offset, attributes);
            self.write_primary_byte(ROW_PAYLOADS[0] + offset, upper, attributes);
            self.write_primary_byte(ROW_PAYLOADS[1] + offset, lower, attributes);
        }
        for column in 0..usize::from(ROOM_BLOCKS) {
            let value = attributes
                [SECOND_CACHE_PAGE + usize::from(ring) * usize::from(ROOM_BLOCKS) + column];
            self.write_primary_byte(ROW_ATTRIBUTE_PAYLOAD + column, value, attributes);
        }
    }
    pub fn stage_neighbor_block(
        &mut self,
        update: &world::ColumnUpdate,
        attributes: &mut [u8; world::CACHE_CELLS],
    ) {
        let column = usize::from(update.origin.column % 16) / 4;
        for (row, payload) in NEIGHBOR_PAYLOADS.iter().enumerate() {
            for tile in 0..4 {
                self.write_primary_byte(
                    payload + column * 4 + tile,
                    update.tiles[row * 4 + tile],
                    attributes,
                );
            }
        }
        attributes[usize::from(update.palette.first_cell)] = update.palette.values[0];
        self.write_primary_byte(
            NEIGHBOR_ATTRIBUTE_PAYLOAD + column,
            update.palette.values[0],
            attributes,
        );
    }

    pub fn publish_neighbor_row(
        &mut self,
        origin: world::TileOrigin,
        rows: world::BlockRows,
        attributes: &mut [u8; world::CACHE_CELLS],
    ) {
        let ring = origin.row / 4;
        let base = presentation::terrain::tile_address(origin);
        for row in 0..4 {
            self.terrain_header(
                attributes,
                row * NEIGHBOR_COMMAND_BYTES,
                base + row as u16 * TILE_ROW_VIDEO_BYTES,
                15,
            );
        }
        self.terrain_header(
            attributes,
            NEIGHBOR_ATTRIBUTE_COMMAND,
            FIRST_NAMETABLE
                + NAMETABLE_ATTRIBUTE_OFFSET
                + u16::from(ring) * u16::from(ROOM_BLOCKS)
                + u16::from(origin.column / 4),
            3,
        );
        self.write_primary_byte(NEIGHBOR_TERMINATOR, u8::MAX, attributes);
        if rows == world::BlockRows::UpperHalf {
            for at in (0..8).rev() {
                let value = self.primary_byte(NEIGHBOR_ATTRIBUTE_COMMAND + at, attributes);
                self.write_primary_byte(ROW_PAYLOADS[1] + at, value, attributes);
            }
        } else if rows == world::BlockRows::LowerHalf {
            for at in 0..0x2e {
                let value = self.primary_byte(ROW_PAYLOADS[1] + at, attributes);
                self.write_primary_byte(at, value, attributes);
            }
        }
        if rows != world::BlockRows::All {
            for at in 0..4 {
                let value = self.primary_byte(NEIGHBOR_PAYLOADS[2] + at, attributes)
                    & if rows == world::BlockRows::LowerHalf {
                        PALETTE_HIGH_QUADRANTS
                    } else {
                        PALETTE_LOW_QUADRANTS
                    };
                self.write_primary_byte(NEIGHBOR_PAYLOADS[2] + at, value, attributes);
            }
        }
        self.display.primary.pending = 0xff;
    }
}

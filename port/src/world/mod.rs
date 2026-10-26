//! Room decoding, streaming caches, camera and terrain edits.
//! Source derivations and native ownership are documented in trace.md.
use crate::assets::formats::UploadCommit;
use crate::identifiers::StageId;
mod display;
use anyhow::{Context, Result, ensure};
pub use display::{
    BlockRows, ColumnUpdate, PaletteRun, TerrainUpdate, TileOrigin, TilePage, TileRegion,
};
pub const ROOM_CELLS: usize = 64;
const ROOM_AXIS_CELLS: usize = 8;
const ROOM_AXIS_BITS: u8 = 3;
const ROOM_AXIS_MASK: u8 = 7;
pub const ROOM_BYTES: usize = 72;
pub const ORIGINAL_ROOMS: usize = 200;
pub const ORIGINAL_LAYOUTS: usize = 10;
pub const CACHE_CELLS: usize = 128;
pub const TERRAIN_SELECTORS: usize = 256;
pub const BLOCK_DESCRIPTORS: usize = 512;
pub const MISSING_ROOM: u8 = 0xff;
pub const TRANSITION_TICKS: u8 = 60;
pub const CAMERA_CENTER: u8 = 0x80;
pub const PLAYER_LEFT_MARGIN: u8 = 0x10;
pub const PLAYER_RIGHT_MARGIN: u8 = 0xf0;
pub const TERRAIN_ACTOR_FIRST_ANIMATION: u8 = 0x1a;
pub const PROGRESSION_THREE_ROOM: u8 = 0x28;
pub const PROGRESSION_FOUR_ROOM: u8 = 0x27;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Room {
    pub low: [u8; ROOM_CELLS],
    pub ninth_bits: [u8; 8],
}
impl Room {
    pub fn decode(payload: &[u8]) -> Result<Self> {
        ensure!(
            payload.len() == ROOM_BYTES,
            "room must contain exactly72bytes"
        );
        Ok(Self {
            low: payload[..ROOM_CELLS].try_into()?,
            ninth_bits: payload[ROOM_CELLS..].try_into()?,
        })
    }
    pub fn block(ring_cursor: u8, room: &Self) -> u16 {
        let row = usize::from(ring_cursor & ROOM_AXIS_MASK);
        let column = usize::from((ring_cursor >> ROOM_AXIS_BITS) & ROOM_AXIS_MASK);
        room.block_at(row, column)
    }
    /// Row-major room cell, independent of the streaming ring's transposed
    /// cursor. The ninth descriptor bit is stored MSB-first in each row bitmap.
    pub fn block_at(&self, row: usize, column: usize) -> u16 {
        let index = row * ROOM_AXIS_CELLS + column;
        let high = u16::from(self.ninth_bits[row] & (0x80 >> column) != 0) << 8;
        high | u16::from(self.low[index])
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Layout {
    pub rows: u8,
    pub columns: u8,
    pub rooms: Vec<Option<usize>>,
}
impl Layout {
    pub fn room_at(&self, row: u8, column: u8) -> Option<usize> {
        if row >= self.rows || column >= self.columns {
            return None;
        }
        self.rooms
            .get(usize::from(row) * usize::from(self.columns) + usize::from(column))
            .copied()
            .flatten()
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerrainSet {
    pub blocks: Vec<[u8; 4]>,
    /// Each selector's TL,TR,BL,BR tile planes and original packed descriptor.
    pub tiles: [[u8; 4]; TERRAIN_SELECTORS],
    pub properties: [u8; TERRAIN_SELECTORS],
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldCatalog {
    pub rooms: Vec<Room>,
    pub layouts: Vec<Layout>,
    pub terrain: Vec<TerrainSet>,
}
impl WorldCatalog {
    pub fn original(rom: &[u8]) -> Result<Self> {
        const LAYOUT_ROWS: usize = 0x1ce67;
        const LAYOUT_COLUMNS: usize = LAYOUT_ROWS + ORIGINAL_LAYOUTS;
        const LAYOUT_LOW: usize = 0x1cf8f;
        const LAYOUT_HIGH: usize = LAYOUT_LOW + ORIGINAL_LAYOUTS;
        const FIXED_ADDRESS_BIAS: usize = 0x10010;
        const ROOM_PAYLOADS: usize = 0x10010;
        const TERRAIN_BLOCK_BANK_OFFSET: usize = 0x3300;
        const BLOCK_BYTES: usize = BLOCK_DESCRIPTORS * 4;
        use crate::assets::formats::{INES_HEADER_BYTES, PRG_BANK_BYTES};
        let room_bytes = rom
            .get(ROOM_PAYLOADS..ROOM_PAYLOADS + ORIGINAL_ROOMS * ROOM_BYTES)
            .context("room payload bounds")?;
        let rooms = room_bytes
            .chunks_exact(ROOM_BYTES)
            .map(Room::decode)
            .collect::<Result<Vec<_>>>()?;
        let mut layouts = Vec::with_capacity(ORIGINAL_LAYOUTS);
        for selector in 0..ORIGINAL_LAYOUTS {
            let rows = *rom.get(LAYOUT_ROWS + selector).context("layout rows")?;
            let columns = *rom
                .get(LAYOUT_COLUMNS + selector)
                .context("layout columns")?;
            let address = usize::from(*rom.get(LAYOUT_LOW + selector).context("layout low")?)
                | (usize::from(*rom.get(LAYOUT_HIGH + selector).context("layout high")?) << 8);
            let start = address + FIXED_ADDRESS_BIAS;
            let count = usize::from(rows) * usize::from(columns);
            let source = rom.get(start..start + count).context("layout room list")?;
            let room_list = source
                .iter()
                .map(|&room| {
                    if room == MISSING_ROOM {
                        None
                    } else {
                        Some(usize::from(room))
                    }
                })
                .collect::<Vec<_>>();
            ensure!(
                room_list.iter().flatten().all(|&room| room < rooms.len()),
                "layout room selector"
            );
            layouts.push(Layout {
                rows,
                columns,
                rooms: room_list,
            });
        }
        let mut terrain = Vec::new();
        for bank in 0..4 {
            let start = INES_HEADER_BYTES + bank * PRG_BANK_BYTES + TERRAIN_BLOCK_BANK_OFFSET;
            let bytes = rom
                .get(start..start + BLOCK_BYTES + 5 * TERRAIN_SELECTORS)
                .context("terrain payload bounds")?;
            let blocks = bytes[..BLOCK_BYTES]
                .chunks_exact(4)
                .map(|chunk| chunk.try_into().expect("exact block record"))
                .collect();
            let tiles = std::array::from_fn(|selector| {
                std::array::from_fn(|plane| {
                    bytes[BLOCK_BYTES + plane * TERRAIN_SELECTORS + selector]
                })
            });
            let properties = bytes[BLOCK_BYTES + 4 * TERRAIN_SELECTORS..].try_into()?;
            terrain.push(TerrainSet {
                blocks,
                tiles,
                properties,
            });
        }
        Ok(Self {
            rooms,
            layouts,
            terrain,
        })
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ExpansionContext {
    pub stage: u8,
    pub room: u8,
    pub room_in_row: u8,
    pub ring: u8,
    pub split: bool,
    pub stage_three_control: u8,
    pub stage_four_control: u8,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpandedBlock {
    pub tiles: [u8; 16],
    pub palette: u8,
    pub collision: u8,
    pub cache_index: usize,
}
pub fn expand(set: &TerrainSet, block: u16, context: ExpansionContext) -> ExpandedBlock {
    const QUADRANT_POSITIONS: [usize; 4] = [0, 2, 8, 10];
    const BOTTOM_ATTRIBUTE_ROW: u8 = 0x38;
    const FOURTH_STAGE: u8 = 3;
    const FIFTH_STAGE: u8 = 4;
    const OPENED_PASSAGE_TILE: u8 = 0x88;
    const ALTERNATE_ODD_TILE: u8 = 0xd6;
    const ALTERNATE_EVEN_TILE: u8 = 0xd7;
    let coordinate = (context.ring & ROOM_AXIS_MASK) * ROOM_AXIS_CELLS as u8
        + ((context.ring >> ROOM_AXIS_BITS) & ROOM_AXIS_MASK);
    let mut result = ExpandedBlock {
        tiles: [0; 16],
        palette: 0,
        collision: 0,
        cache_index: usize::from(context.room_in_row & 1) * ROOM_CELLS + usize::from(coordinate),
    };
    for quadrant in (0..4).rev() {
        // The lower half of the final attribute row is never submitted. Only
        // valid top-quadrant data participates in cache and visible tile output.
        if coordinate >= BOTTOM_ATTRIBUTE_ROW && quadrant >= 2 {
            continue;
        }
        let mut selector = set.blocks[usize::from(block)][quadrant];
        let property = set.properties[usize::from(selector)];
        result.palette |= (property & 3) << (quadrant * 2);
        result.collision |= ((property >> 4) & 3) << (quadrant * 2);
        if !context.split
            && context.stage == FOURTH_STAGE
            && context.stage_three_control != 0
            && context.room == PROGRESSION_THREE_ROOM
            && matches!(coordinate, 0x21 | 0x29)
        {
            result.palette = 0;
            result.collision = 0;
            selector = OPENED_PASSAGE_TILE;
        }
        if !context.split
            && context.stage == FIFTH_STAGE
            && context.stage_four_control & 0x80 != 0
            && context.room == PROGRESSION_FOUR_ROOM
            && coordinate >= 0x20
        {
            match (context.ring & 0x38, quadrant & 1) {
                (0x18, 1) => selector = ALTERNATE_ODD_TILE,
                (0x20, 0) => selector = ALTERNATE_EVEN_TILE,
                _ => (),
            }
        }
        let at = QUADRANT_POSITIONS[quadrant];
        let [top_left, top_right, bottom_left, bottom_right] = set.tiles[usize::from(selector)];
        result.tiles[at] = top_left;
        result.tiles[at + 1] = top_right;
        result.tiles[at + 4] = bottom_left;
        result.tiles[at + 5] = bottom_right;
    }
    result
}
/// No independent CPU bank or register state exists in the native game.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Forward,
    Backward,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StreamCursor {
    pub ring: u8,
    pub room: u8,
    pub room_in_row: u8,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitialBuild {
    pub visits: Vec<StreamCursor>,
    pub final_cursor: StreamCursor,
    pub direction: Direction,
}
pub fn initial_columns(layout: &Layout, room: u8, room_in_row: u8) -> InitialBuild {
    const INITIAL_WORK: [usize; 3] = [96, 96, 128];
    const INITIAL_RING: [u8; 3] = [0, 32, 32];
    const FINAL_RING: u8 = 31;
    let present = |index: u8| {
        layout
            .rooms
            .get(usize::from(index))
            .copied()
            .flatten()
            .is_some()
    };
    let choice = if room_in_row == 0 || !present(room.wrapping_sub(1)) {
        0
    } else if room.wrapping_sub(2) == layout.columns || !present(room.wrapping_add(1)) {
        1
    } else {
        2
    };
    let mut current = StreamCursor {
        ring: INITIAL_RING[choice],
        room: room.wrapping_sub(u8::from(choice != 0)),
        room_in_row: 0,
    };
    current.room_in_row = current.room & ROOM_AXIS_MASK;
    let mut visits = Vec::with_capacity(INITIAL_WORK[choice]);
    for _ in 0..INITIAL_WORK[choice] {
        visits.push(current);
        current.ring = current.ring.wrapping_add(1) & (ROOM_CELLS - 1) as u8;
        if current.ring == 0 {
            current.room = current.room.wrapping_add(1);
            current.room_in_row = current.room & ROOM_AXIS_MASK;
        }
    }
    current.ring = FINAL_RING;
    InitialBuild {
        visits,
        final_cursor: current,
        direction: Direction::Forward,
    }
}
pub fn whole_ring(room: u8) -> Vec<StreamCursor> {
    (0..ROOM_CELLS)
        .map(|ring| StreamCursor {
            ring: ring as u8,
            room,
            room_in_row: room & ROOM_AXIS_MASK,
        })
        .collect()
}
pub fn stream_work(previous_x: u8, current_x: u8, direction: Direction) -> u8 {
    const STREAM_PIXELS: u8 = 4;
    let delta = current_x.wrapping_sub(previous_x) as i8;
    let distance = delta.unsigned_abs();
    let alignment = match direction {
        Direction::Forward => previous_x,
        Direction::Backward => !previous_x,
    } & (STREAM_PIXELS - 1);
    // Byte addition precedes division, including the source overflow behavior.
    alignment.wrapping_add(distance) / STREAM_PIXELS
}
pub fn advance_cursor(
    mut cursor: StreamCursor,
    previous: Direction,
    next: Direction,
) -> StreamCursor {
    let step = match next {
        Direction::Forward => 1_u8,
        Direction::Backward => u8::MAX,
    };
    let room_step = if previous != next {
        step.wrapping_mul(2)
    } else {
        let unmasked = cursor.ring.wrapping_add(step);
        cursor.ring = unmasked & (ROOM_CELLS - 1) as u8;
        if unmasked >= ROOM_CELLS as u8 {
            step
        } else {
            0
        }
    };
    cursor.room = cursor.room.wrapping_add(room_step);
    cursor.room_in_row = cursor.room_in_row.wrapping_add(room_step);
    cursor
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ColumnResult {
    pub collision: Option<(usize, u8)>,
    pub attribute: Option<(usize, u8)>,
    pub update: Option<ColumnUpdate>,
}
pub fn column(
    layout: &Layout,
    context: ExpansionContext,
    block: &ExpandedBlock,
    suspended: u8,
) -> ColumnResult {
    const FIRST_SCENE_LAYOUT: u8 = 5;
    let mut result = ColumnResult {
        collision: None,
        attribute: None,
        update: None,
    };
    if context.room_in_row >= layout.columns
        || layout
            .rooms
            .get(usize::from(context.room))
            .copied()
            .flatten()
            .is_none()
    {
        return result;
    }
    result.collision = Some((block.cache_index, block.collision));
    let row = context.ring & ROOM_AXIS_MASK;
    let early_even = context.stage < FIRST_SCENE_LAYOUT && context.room_in_row & 1 == 0;
    if early_even && row == 0 {
        return result;
    }
    let rows = if early_even && row == 1 {
        BlockRows::LowerHalf
    } else if row == ROOM_AXIS_MASK {
        BlockRows::UpperHalf
    } else {
        BlockRows::All
    };
    let palette = match rows {
        BlockRows::LowerHalf => block.palette & 0xf0,
        BlockRows::UpperHalf => block.palette & 0x0f,
        BlockRows::All => block.palette,
    };
    let page = TilePage::from_world_page(context.room_in_row);
    result.attribute = Some((block.cache_index, palette));
    result.update = Some(ColumnUpdate {
        origin: TileOrigin {
            page,
            column: (context.ring >> ROOM_AXIS_BITS) * 4,
            row: row * 4,
        },
        tiles: block.tiles,
        rows,
        palette: PaletteRun {
            page,
            first_cell: row * ROOM_AXIS_CELLS as u8 + (context.ring >> ROOM_AXIS_BITS),
            values: vec![palette],
        },
        commit: if suspended == 0 {
            UploadCommit::AtInterrupt
        } else {
            UploadCommit::Immediate
        },
    });
    result
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Camera {
    pub x: u16,
    pub room: u8,
    pub previous_x: u8,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CameraPlayer {
    pub x: u16,
    pub fraction: u8,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CameraInput {
    pub active: bool,
    pub automatic: bool,
    pub departure: u8,
    pub held: u8,
    pub late: u8,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CameraResult {
    pub camera: Camera,
    pub player: CameraPlayer,
    pub direction: Option<Direction>,
    pub work: u8,
}
pub fn camera_step(
    mut camera: Camera,
    mut player: CameraPlayer,
    input: CameraInput,
    layout: &Layout,
) -> CameraResult {
    let finish =
        |camera: Camera, player: CameraPlayer, direction: Option<Direction>| CameraResult {
            work: direction.map_or(0, |direction| {
                stream_work(camera.previous_x, camera.x as u8, direction)
            }),
            camera,
            player,
            direction,
        };
    if input.automatic {
        if input.departure & 0x80 != 0 && camera.x != 0 {
            camera.x = camera.x.wrapping_sub(1);
        }
        let screen_x = (player.x as u8).wrapping_sub(camera.x as u8);
        let right = input.held & crate::runtime::Buttons::RIGHT.0 != 0;
        let left = input.held & crate::runtime::Buttons::LEFT.0 != 0;
        const EDGE_CORRECTION: u32 = 0x180; // one and a half pixels in 8.8 units
        let position = (u32::from(player.x) << 8) | u32::from(player.fraction);
        let adjusted = if right && screen_x >= PLAYER_RIGHT_MARGIN {
            position.wrapping_sub(EDGE_CORRECTION)
        } else if !right && left && screen_x < PLAYER_LEFT_MARGIN {
            position.wrapping_add(EDGE_CORRECTION)
        } else {
            position
        };
        player.x = (adjusted >> 8) as u16;
        player.fraction = adjusted as u8;
        return finish(
            camera,
            player,
            (camera.x != 0).then_some(Direction::Backward),
        );
    }
    if !input.active {
        return finish(camera, player, None);
    }
    let delta = (player.x as u8)
        .wrapping_sub(camera.x as u8)
        .wrapping_sub(CAMERA_CENTER) as i8;
    if delta == 0 {
        return finish(camera, player, None);
    }
    let direction = if delta < 0 {
        Direction::Backward
    } else {
        Direction::Forward
    };
    let proposed_x = camera.x.wrapping_add_signed(i16::from(delta));
    // The same low-byte carry/borrow changes room and world page. A second
    // high-byte addition must not reuse the first high-byte addition's carry.
    let page_step = ((proposed_x >> 8) as u8).wrapping_sub((camera.x >> 8) as u8);
    let proposed_room = camera.room.wrapping_add(page_step);
    let proposed_page = (proposed_x >> 8) as u8;
    let exists = |room: u8| {
        layout
            .rooms
            .get(usize::from(room))
            .copied()
            .flatten()
            .is_some()
    };
    if direction == Direction::Forward {
        let next_room = proposed_room.wrapping_add(1);
        let blocked = input.late & 0x80 != 0
            || proposed_page.wrapping_add(1) == layout.columns
            || next_room == 0
            || !exists(next_room);
        camera.room = proposed_room;
        camera.x = if blocked {
            proposed_x & 0xff00
        } else {
            proposed_x
        };
        if blocked && player.x as u8 > PLAYER_RIGHT_MARGIN {
            player.x = (player.x & 0xff00) | u16::from(PLAYER_RIGHT_MARGIN);
        }
    } else {
        let blocked = input.late & 0x80 != 0 || proposed_page & 0x80 != 0 || !exists(proposed_room);
        if blocked {
            camera.x &= 0xff00;
            if player.x as u8 <= PLAYER_LEFT_MARGIN {
                player.x = (player.x & 0xff00) | u16::from(PLAYER_LEFT_MARGIN);
            }
        } else {
            camera.x = proposed_x;
            camera.room = proposed_room;
        }
    }
    finish(camera, player, Some(direction))
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TerrainProbe {
    pub room_row: u8,
    pub world_page: u8,
    pub block_coordinate: u8,
    pub quadrant: u8,
}
pub fn read_cell(catalog: &WorldCatalog, layout: usize, bank: usize, probe: TerrainProbe) -> u8 {
    let layout = &catalog.layouts[layout];
    let room_index = layout
        .room_at(probe.room_row, probe.world_page)
        .expect("validated terrain probe room");
    let room = &catalog.rooms[room_index];
    let coordinate = usize::from(probe.block_coordinate & (ROOM_CELLS - 1) as u8);
    let row = coordinate / ROOM_AXIS_CELLS;
    let column = coordinate % ROOM_AXIS_CELLS;
    let descriptor = usize::from(room.block_at(row, column));
    catalog.terrain[bank].blocks[descriptor][usize::from(probe.quadrant)]
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorldCache {
    #[serde(with = "crate::snapshot::array")]
    pub attributes: [u8; CACHE_CELLS],
    #[serde(with = "crate::snapshot::array")]
    pub collision: [u8; CACHE_CELLS],
    pub changed_this_frame: u8,
}
impl Default for WorldCache {
    fn default() -> Self {
        Self {
            attributes: [0; CACHE_CELLS],
            collision: [0; CACHE_CELLS],
            changed_this_frame: 0,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Replacement {
    ActorState { stage: StageId, local: u8 },
    Animation { stage: StageId, animation: u8 },
}
pub fn replacement_index(request: Replacement) -> u8 {
    const STAGE_REPLACEMENT_STRIDE: u8 = 4;
    const ANIMATION_REPLACEMENT_BASE: u8 = 0x13;
    match request {
        Replacement::ActorState { stage, local } => stage
            .0
            .wrapping_mul(STAGE_REPLACEMENT_STRIDE)
            .wrapping_add(local),
        Replacement::Animation { stage, animation } => {
            let animation = animation & 0x7f;
            let relative = animation.wrapping_sub(TERRAIN_ACTOR_FIRST_ANIMATION);
            // SBC's comparison survives AND03; ADC13 therefore adds one for
            // source animations at/above the first terrain animation.
            stage
                .0
                .wrapping_mul(STAGE_REPLACEMENT_STRIDE)
                .wrapping_add(relative & 3)
                .wrapping_add(ANIMATION_REPLACEMENT_BASE)
                .wrapping_add(u8::from(animation >= TERRAIN_ACTOR_FIRST_ANIMATION))
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Mutation {
    pub world_x: u16,
    pub spawn_y: u8,
    pub selector: u8,
    pub marks_changed: bool,
}
impl WorldCache {
    /// Cache changes occur now, display writes are committed at B01/B03 boundary.
    pub fn replace(&mut self, set: &TerrainSet, mutation: Mutation) -> TerrainUpdate {
        replace_cache(
            &mut self.attributes,
            &mut self.collision,
            &mut self.changed_this_frame,
            set,
            mutation,
        )
    }

    pub fn dialogue_reveal(&mut self) {
        const REVEAL_CELLS: [usize; 2] = [0x21, 0x29];
        for cell in REVEAL_CELLS {
            self.collision[cell] = 0;
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum VerticalDirection {
    Down,
    Up,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum VerticalPhase {
    BuildRoom,
    CopyStaged,
    BoundaryOrCopy,
    NextRoom,
    PreviousRoom,
    PollBackground,
}
pub fn vertical_phase(direction: VerticalDirection, counter: u8) -> VerticalPhase {
    use VerticalPhase::*;
    const DOWN: [VerticalPhase; 8] = [
        BuildRoom,
        NextRoom,
        PreviousRoom,
        PollBackground,
        CopyStaged,
        PollBackground,
        PollBackground,
        PollBackground,
    ];
    const UP: [VerticalPhase; 8] = [
        NextRoom,
        PreviousRoom,
        BoundaryOrCopy,
        PollBackground,
        PollBackground,
        PollBackground,
        BuildRoom,
        PollBackground,
    ];
    (match direction {
        VerticalDirection::Down => DOWN,
        VerticalDirection::Up => UP,
    })[usize::from(counter & ROOM_AXIS_MASK)]
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum VerticalEffect {
    ClearNonplayerActors,
    HideSprites,
    Yield,
    ComposeActors,
    Build(VerticalPhase),
    ResetSpawnWindow,
    UpdateSpawnWindow,
    RefreshPalette,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VerticalTransition {
    pub direction: VerticalDirection,
    pub counter: u8,
    pub player_y: u8,
    pub split_address: u16,
    pub split_control: u8,
    pub cursor: StreamCursor,
    pub saved_cursor: StreamCursor,
}
impl VerticalTransition {
    pub fn begin(
        direction: VerticalDirection,
        player_y: u8,
        split_address: u16,
        cursor: StreamCursor,
    ) -> (Self, Vec<VerticalEffect>) {
        const TRANSITION_SPLIT_CONTROL: u8 = 0x0c;
        let ring = if direction == VerticalDirection::Up {
            ROOM_AXIS_MASK
        } else {
            0
        };
        (
            Self {
                direction,
                counter: 0,
                player_y,
                split_address,
                split_control: TRANSITION_SPLIT_CONTROL,
                cursor: StreamCursor { ring, ..cursor },
                saved_cursor: cursor,
            },
            vec![
                VerticalEffect::ClearNonplayerActors,
                VerticalEffect::HideSprites,
                VerticalEffect::Yield,
            ],
        )
    }
    pub fn resume(&mut self) -> Vec<VerticalEffect> {
        const UP_WRAP_TICK: u8 = 12;
        const DOWN_WRAP_TICK: u8 = 46;
        const WRAP_HIGH_MASK: u8 = 0x24;
        const UP_WRAP_HIGH: u8 = 3;
        const UP_WRAP_LOW: u8 = 0xa0;
        const ADDRESS_ROW: u16 = 0x20;
        const PLAYER_ROW_COMPENSATION: u8 = 8;
        assert!(
            self.counter < TRANSITION_TICKS,
            "vertical continuation already complete"
        );
        let up = self.direction == VerticalDirection::Up;
        self.player_y = self.player_y.wrapping_add(if up { u8::MAX } else { 1 });
        if self.counter & 1 == 0 {
            if self.counter == if up { UP_WRAP_TICK } else { DOWN_WRAP_TICK } {
                let high = ((self.split_address >> 8) as u8 & WRAP_HIGH_MASK)
                    | if up { UP_WRAP_HIGH } else { 0 };
                self.split_address =
                    (u16::from(high) << 8) | u16::from(if up { UP_WRAP_LOW } else { 0 });
            } else {
                self.split_address = if up {
                    self.split_address.wrapping_sub(ADDRESS_ROW)
                } else {
                    self.split_address.wrapping_add(ADDRESS_ROW)
                };
                self.player_y = if up {
                    self.player_y.wrapping_add(PLAYER_ROW_COMPENSATION)
                } else {
                    self.player_y.wrapping_sub(PLAYER_ROW_COMPENSATION)
                };
            }
        }
        let phase = vertical_phase(self.direction, self.counter);
        let mut effects = vec![
            VerticalEffect::Build(phase),
            VerticalEffect::HideSprites,
            VerticalEffect::ComposeActors,
            VerticalEffect::Yield,
        ];
        self.counter += 1;
        if self.counter == TRANSITION_TICKS {
            self.cursor.ring = self.saved_cursor.ring;
            self.cursor.room_in_row = self.saved_cursor.room_in_row;
            self.split_control = 0;
            effects.extend([
                VerticalEffect::ResetSpawnWindow,
                VerticalEffect::UpdateSpawnWindow,
                VerticalEffect::RefreshPalette,
            ]);
        }
        effects
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerticalRows {
    pub tiles: [[u8; 32]; 4],
    pub attributes: [u8; 8],
    pub ring: u8,
    pub update: TerrainUpdate,
}
/// One live block in a vertical transition. Only the selected half is visible;
/// the video boundary retains the other half in its canonical staged rows.
pub fn vertical_block(
    direction: VerticalDirection,
    ring: u8,
    column: u8,
    block: &ExpandedBlock,
    old: u8,
) -> ColumnUpdate {
    let up = direction == VerticalDirection::Up;
    ColumnUpdate {
        origin: TileOrigin {
            page: TilePage::Second,
            column: column * 4,
            row: (ring & ROOM_AXIS_MASK) * 4,
        },
        tiles: block.tiles,
        rows: if up {
            BlockRows::LowerHalf
        } else {
            BlockRows::UpperHalf
        },
        palette: PaletteRun {
            page: TilePage::Second,
            first_cell: (ring & ROOM_AXIS_MASK) * 8 + column,
            values: vec![if up {
                (old & 0x0f) | (block.palette & 0xf0)
            } else {
                (old & 0xf0) | (block.palette & 0x0f)
            }],
        },
        commit: UploadCommit::AtInterrupt,
    }
}
pub fn neighbor_visible_rows(ring: u8) -> BlockRows {
    match ring & ROOM_AXIS_MASK {
        1 => BlockRows::LowerHalf,
        ROOM_AXIS_MASK => BlockRows::UpperHalf,
        _ => BlockRows::All,
    }
}
pub fn vertical_rows(
    direction: VerticalDirection,
    ring: u8,
    blocks: &[ExpandedBlock],
    old: [u8; 8],
) -> VerticalRows {
    assert_eq!(
        blocks.len(),
        ROOM_AXIS_CELLS,
        "full vertical row needs eight blocks"
    );
    let up = direction == VerticalDirection::Up;
    let mut tiles = [[0; 32]; 4];
    let mut attributes = [0; 8];
    for (index, block) in blocks.iter().enumerate() {
        for (local_row, tile_row) in tiles.iter_mut().enumerate() {
            let source_row = if up { (local_row + 2) % 4 } else { local_row };
            tile_row[index * 4..index * 4 + 4]
                .copy_from_slice(&block.tiles[source_row * 4..source_row * 4 + 4]);
        }
        attributes[index] = if up {
            (old[index] & 0x0f) | (block.palette & 0xf0)
        } else {
            (old[index] & 0xf0) | (block.palette & 0x0f)
        };
    }
    let ring = ring & ROOM_AXIS_MASK;
    let update = TerrainUpdate {
        region: TileRegion::new(
            TileOrigin {
                page: TilePage::Second,
                column: 0,
                row: ring * 4 + if up { 2 } else { 0 },
            },
            32,
            tiles[..2].concat(),
        ),
        palette: Some(PaletteRun {
            page: TilePage::Second,
            first_cell: ring * 8,
            values: attributes.to_vec(),
        }),
        commit: UploadCommit::AtInterrupt,
    };
    VerticalRows {
        tiles,
        attributes,
        ring,
        update,
    }
}

pub fn vertical_neighbor(
    _direction: VerticalDirection,
    previous: bool,
    camera: Camera,
    layout: &Layout,
    ring: u8,
    blocks: &[ExpandedBlock],
) -> Option<VerticalRows> {
    let page = (camera.x >> 8) as u8;
    if previous && page == 0 || !previous && page == layout.columns.wrapping_sub(1) {
        return None;
    }
    let neighbor = if previous {
        camera.room.wrapping_sub(1)
    } else {
        camera.room.wrapping_add(1)
    };
    layout.rooms.get(usize::from(neighbor)).copied().flatten()?;
    let ring = ring & ROOM_AXIS_MASK;
    if ring == 0 {
        return None;
    }
    assert_eq!(blocks.len(), 4, "neighbor half needs four blocks");
    let mut tiles = [[0; 32]; 4];
    let mut attributes = [0; 8];
    for (index, block) in blocks.iter().enumerate() {
        for (row, tile_row) in tiles.iter_mut().enumerate() {
            tile_row[index * 4..index * 4 + 4].copy_from_slice(&block.tiles[row * 4..row * 4 + 4]);
        }
        attributes[index] = block.palette;
    }
    let start_row = if ring == 1 { 2 } else { 0 };
    let end_row = if ring == ROOM_AXIS_MASK { 2 } else { 4 };
    let mask = if ring == 1 {
        0xf0
    } else if ring == ROOM_AXIS_MASK {
        0x0f
    } else {
        0xff
    };
    for attribute in &mut attributes[..4] {
        *attribute &= mask;
    }
    let update = TerrainUpdate {
        region: TileRegion::new(
            TileOrigin {
                page: TilePage::First,
                column: if previous { 16 } else { 0 },
                row: ring * 4 + start_row as u8,
            },
            16,
            tiles[start_row..end_row]
                .iter()
                .flat_map(|row| row[..16].iter().copied())
                .collect(),
        ),
        palette: Some(PaletteRun {
            page: TilePage::First,
            first_cell: ring * 8 + if previous { 4 } else { 0 },
            values: attributes[..4].to_vec(),
        }),
        commit: UploadCommit::AtInterrupt,
    };
    Some(VerticalRows {
        tiles,
        attributes,
        ring,
        update,
    })
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Burst {
    pub update: TerrainUpdate,
    pub particle_origin: (u8, u8),
    pub particles: usize,
}
pub fn changed_burst(set: &TerrainSet, selector: u8) -> Burst {
    const FIRST_BURST: u8 = 0;
    const SECOND_BURST: u8 = 6;
    const BURST_PARTICLES: usize = 4;
    let (pairs, page, column, seeds, origin) = match selector {
        FIRST_BURST => (4, TilePage::First, 4, [0x88_u8, 0x88], (0x88, 0x28)),
        SECOND_BURST => (7, TilePage::Second, 14, [0xd6_u8, 0xd7], (0x88, 0x78)),
        _ => panic!("validated burst descriptor selector"),
    };
    let mut tiles = Vec::new();
    for _ in 0..pairs {
        for row in 0..2 {
            for seed in seeds {
                tiles.extend_from_slice(&set.tiles[usize::from(seed)][row * 2..row * 2 + 2]);
            }
        }
    }
    Burst {
        update: TerrainUpdate {
            region: TileRegion::new(
                TileOrigin {
                    page,
                    column,
                    row: 16,
                },
                4,
                tiles,
            ),
            palette: None,
            commit: UploadCommit::AtInterrupt,
        },
        particle_origin: origin,
        particles: BURST_PARTICLES,
    }
}

#[cfg(test)]
mod contracts;
/// B08/B10 consume this one-shot recovery allocation before the same terrain
/// mutation. A full pool must not cancel the mutation; player slot0 is excluded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryActor {
    pub slot: usize,
    pub state: u8,
    pub handler: u8,
    pub animation: u8,
    pub world_x: u16,
    pub screen_y: u8,
    pub animation_tick: u8,
    pub animation_frame: u8,
    pub collision_profile: u8,
    pub vertical_velocity: i16,
}
pub fn recovery_actor(
    local_counter: u8,
    local_state: u8,
    world_x: u16,
    screen_y: u8,
    active: &[bool; 16],
) -> Option<RecoveryActor> {
    const RECOVERY_ANIMATIONS: [u8; 4] = [0x24, 0x28, 0x25, 0x26];
    const RECOVERY_STATE: u8 = 0xa1;
    const RECOVERY_HANDLER: u8 = 0x13;
    if local_counter & 0x80 == 0 {
        return None;
    }
    let slot = (1..active.len()).rev().find(|&slot| !active[slot])?;
    Some(RecoveryActor {
        slot,
        state: RECOVERY_STATE,
        handler: RECOVERY_HANDLER,
        animation: RECOVERY_ANIMATIONS[usize::from(local_state)],
        world_x,
        screen_y,
        animation_tick: 0,
        animation_frame: 0,
        collision_profile: 0,
        vertical_velocity: 0,
    })
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerticalStaging {
    pub upper: [u8; 32],
    pub lower: [u8; 32],
    pub attributes: [u8; 8],
}
/// Copy phase uses old row for destination, then advances the row; wrap phase at
/// row7 builds with opposite direction and decrements row instead of copying.
pub fn copy_vertical_staging(
    direction: VerticalDirection,
    ring: u8,
    staging: &VerticalStaging,
) -> (u8, TerrainUpdate) {
    let up = direction == VerticalDirection::Up;
    let update = TerrainUpdate {
        region: TileRegion::new(
            TileOrigin {
                page: TilePage::Second,
                column: 0,
                row: (ring & ROOM_AXIS_MASK) * 4 + if up { 0 } else { 2 },
            },
            32,
            [staging.upper, staging.lower].concat(),
        ),
        palette: Some(PaletteRun {
            page: TilePage::Second,
            first_cell: (ring & ROOM_AXIS_MASK) * 8,
            values: staging.attributes.to_vec(),
        }),
        commit: UploadCommit::AtInterrupt,
    };
    let next = ring.wrapping_add(if up { u8::MAX } else { 1 }) & ROOM_AXIS_MASK;
    (next, update)
}

pub fn vertical_wrap_action(
    direction: VerticalDirection,
    ring: u8,
) -> (VerticalDirection, u8, VerticalPhase) {
    if ring == ROOM_AXIS_MASK {
        let opposite = if direction == VerticalDirection::Up {
            VerticalDirection::Down
        } else {
            VerticalDirection::Up
        };
        (opposite, ring - 1, VerticalPhase::BuildRoom)
    } else {
        (direction, ring, VerticalPhase::CopyStaged)
    }
}

/// The raw selection continuation consumes the same dimension/pointer planes
/// even beyond the ten authored layouts. Room cursors remain bytes; do not
/// allocate rows×columns when adjacent data describes an exotic raw selector.
pub fn raw_layout(rom: &[u8], selector: u8) -> Result<Layout> {
    const ROWS: usize = 0x1ce67;
    const COLUMNS: usize = ROWS + ORIGINAL_LAYOUTS;
    const POINTER_LOW: usize = 0x1cf8f;
    const POINTER_HIGH: usize = POINTER_LOW + ORIGINAL_LAYOUTS;
    const FIXED_CPU_BASE: usize = 0xc000;
    const FIXED_PHYSICAL_BASE: usize = 0x1c010;
    let index = usize::from(selector);
    let read = |offset| rom.get(offset).copied().context("raw layout parameter");
    let address =
        usize::from(read(POINTER_LOW + index)?) | (usize::from(read(POINTER_HIGH + index)?) << 8);
    ensure!(
        address >= FIXED_CPU_BASE,
        "raw layout requires a non-fixed pointer consumer"
    );
    let start = FIXED_PHYSICAL_BASE + address - FIXED_CPU_BASE;
    let bytes = rom
        .get(start..start + 256)
        .context("raw layout byte cursors")?;
    Ok(Layout {
        rows: read(ROWS + index)?,
        columns: read(COLUMNS + index)?,
        rooms: bytes
            .iter()
            .map(|&room| (room != MISSING_ROOM).then_some(usize::from(room)))
            .collect(),
    })
}
/// Every byte room selector has a native packed-room consumer. Pointers beyond
/// BFFF naturally read the fixed bank, as the original CPU window does; this is
/// immutable format decoding rather than a runtime bus or memory map.
pub fn raw_rooms(rom: &[u8]) -> Result<Vec<Room>> {
    const ROOM_CPU_BASE: usize = 0x8000;
    const SWITCH_CPU_END: usize = 0xc000;
    const ROOM_BANK_PHYSICAL: usize = 0x10010;
    const FIXED_PHYSICAL: usize = 0x1c010;
    (0..=u8::MAX)
        .map(|selector| {
            let start = ROOM_CPU_BASE + usize::from(selector) * ROOM_BYTES;
            let mut bytes = [0; ROOM_BYTES];
            for (offset, byte) in bytes.iter_mut().enumerate() {
                let address = start + offset;
                let physical = if address < SWITCH_CPU_END {
                    ROOM_BANK_PHYSICAL + address - ROOM_CPU_BASE
                } else {
                    FIXED_PHYSICAL + address - SWITCH_CPU_END
                };
                *byte = *rom.get(physical).context("raw packed room")?;
            }
            Room::decode(&bytes)
        })
        .collect()
}

/// Immediate terrain replacement through disjoint borrows of the canonical
/// cache. Objects/platform contexts can hold the changed counter while their
/// boundary callback updates attributes and collision before the next live slot.
pub fn replace_cache(
    attributes: &mut [u8; CACHE_CELLS],
    collision: &mut [u8; CACHE_CELLS],
    changed: &mut u8,
    set: &TerrainSet,
    mutation: Mutation,
) -> TerrainUpdate {
    const STATUS_ATTRIBUTE_CELLS: usize = 0x10;
    let x = mutation.world_x as u8;
    let y = mutation.spawn_y;
    let parity = (mutation.world_x >> 8) as u8 & 1;
    let coordinate = ((y >> 2) & 0x38) | ((x >> 5) & 7);
    let index = usize::from(coordinate | (parity << 6));
    let quadrant = ((y >> 3) & 2) | ((x >> 4) & 1);
    let shift = quadrant * 2;
    let clear = !(3 << shift);
    let property = set.properties[usize::from(mutation.selector)];
    collision[index] = (collision[index] & clear) | (((property >> 4) & 3) << shift);
    attributes[index] = (attributes[index] & clear) | ((property & 3) << shift);
    if mutation.marks_changed {
        *changed = changed.wrapping_add(1);
    }
    let attribute = if index < STATUS_ATTRIBUTE_CELLS {
        attributes[index] & 0xf0
    } else {
        attributes[index]
    };
    let page = TilePage::from_world_page(parity);
    TerrainUpdate {
        region: TileRegion::new(
            TileOrigin {
                page,
                column: (x & 0xf0) >> 3,
                row: (y & 0xf0) >> 3,
            },
            2,
            set.tiles[usize::from(mutation.selector)].to_vec(),
        ),
        palette: Some(PaletteRun {
            page,
            first_cell: coordinate,
            values: vec![attribute],
        }),
        commit: UploadCommit::AtInterrupt,
    }
}

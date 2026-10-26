//! World-coordinate terrain supplements the source's two-page streaming cache.
//! Changes belong to a layout/row/tile, so revealing a third page cannot alias
//! a destroyed object or an actor's collision onto the opposite edge.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TerrainChange {
    pub row: u8,
    pub x: u16,
    pub y: u8,
    pub selector: u8,
}
#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NativeTerrain {
    pub layout: Option<u8>,
    pub columns: u8,
    pub row: u8,
    pub changes: Vec<TerrainChange>,
    pub collision: Vec<u8>,
}
impl NativeTerrain {
    pub fn probe(&self, probe: TerrainProbe) -> Option<u8> {
        self.layout?;
        if probe.world_page >= self.columns {
            return Some(0);
        }
        let room =
            usize::from(probe.room_row) * usize::from(self.columns) + usize::from(probe.world_page);
        Some(
            self.collision
                .get(room * ROOM_CELLS + usize::from(probe.block_coordinate & 63))
                .copied()
                .unwrap_or(0),
        )
    }
    pub fn replace(&mut self, set: &TerrainSet, mutation: Mutation) {
        if self.layout.is_none() {
            return;
        }
        let change = TerrainChange {
            row: self.row,
            x: mutation.world_x & !15,
            y: mutation.spawn_y & !15,
            selector: mutation.selector,
        };
        self.changes
            .retain(|old| (old.row, old.x, old.y) != (change.row, change.x, change.y));
        self.apply_collision(set, &change);
        self.changes.push(change);
    }
    pub fn apply_collision(&mut self, set: &TerrainSet, change: &TerrainChange) {
        let room = usize::from(change.row) * usize::from(self.columns) + usize::from(change.x >> 8);
        let cell = usize::from(change.y / 32) * 8 + usize::from((change.x & 255) / 32);
        if let Some(value) = self.collision.get_mut(room * ROOM_CELLS + cell) {
            let shift = ((change.y >> 3) & 2 | ((change.x >> 4) as u8 & 1)) * 2;
            *value = (*value & !(3 << shift))
                | ((set.properties[usize::from(change.selector)] >> 4 & 3) << shift);
        }
    }
}
pub struct NativeCollision<'a> {
    cells: &'a [u8; CACHE_CELLS],
    native: &'a NativeTerrain,
}
pub fn native_collision<'a>(
    cells: &'a [u8; CACHE_CELLS],
    native: &'a NativeTerrain,
) -> NativeCollision<'a> {
    NativeCollision { cells, native }
}
impl crate::collision::CollisionCells for NativeCollision<'_> {
    fn collision_cell(&self, index: usize) -> u8 {
        self.cells[index]
    }
    fn collision_probe(&self, index: usize, probe: TerrainProbe) -> u8 {
        self.native.probe(probe).unwrap_or(self.cells[index])
    }
}

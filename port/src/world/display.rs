//! Native terrain display changes. Coordinates are logical tiles/palette cells;
//! presentation alone maps them to video addresses and source queue records.
use crate::assets::formats::UploadCommit;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TilePage {
    First,
    Second,
}
impl TilePage {
    pub fn from_world_page(page: u8) -> Self {
        if page & 1 == 0 {
            Self::First
        } else {
            Self::Second
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TileOrigin {
    pub page: TilePage,
    pub column: u8,
    pub row: u8,
}
/// A row-major rectangular change. Rows30/31 are intentionally representable:
/// unusual source callers can write into the palette portion of a nametable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TileRegion {
    pub origin: TileOrigin,
    pub width: usize,
    pub tiles: Vec<u8>,
}
impl TileRegion {
    pub fn new(origin: TileOrigin, width: usize, tiles: Vec<u8>) -> Self {
        assert!(width != 0 && tiles.len().is_multiple_of(width));
        Self {
            origin,
            width,
            tiles,
        }
    }
    pub fn height(&self) -> usize {
        self.tiles.len() / self.width
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaletteRun {
    pub page: TilePage,
    pub first_cell: u8,
    pub values: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerrainUpdate {
    pub region: TileRegion,
    pub palette: Option<PaletteRun>,
    pub commit: UploadCommit,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockRows {
    All,
    UpperHalf,
    LowerHalf,
}
impl BlockRows {
    pub fn range(self) -> std::ops::Range<usize> {
        match self {
            Self::All => 0..4,
            Self::UpperHalf => 0..2,
            Self::LowerHalf => 2..4,
        }
    }
}
/// Column output keeps the complete expanded block and its visible slice. The
/// video boundary needs the clipped-away rows for observable staging/queue writes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ColumnUpdate {
    pub origin: TileOrigin,
    pub tiles: [u8; 16],
    pub rows: BlockRows,
    pub palette: PaletteRun,
    pub commit: UploadCommit,
}

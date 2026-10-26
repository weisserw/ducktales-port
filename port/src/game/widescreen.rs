//! Native gameplay viewport. Maps and sprites retain world coordinates beyond
//! the cartridge's nametable/OAM limits. The interrupt freezes prepared geometry;
//! render only reads that snapshot and the committed palette/patterns.
use super::{
    simulation::{Core, Services},
    video::Video,
};
use crate::{
    config,
    presentation::{self, ActorVisual, Composite},
    world,
};
use romatlas_port::{graphics::Frame, nes::ppu};

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct WideSprite {
    pub x: i16,
    pub y: u8,
    pub tile: u8,
    pub attributes: u8,
}
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct TerrainView {
    pub camera: u16,
    #[serde(default)]
    pub layout: u8,
    #[serde(default)]
    pub left: i32,
    pub row: u8,
    #[serde(default)]
    pub vertical_shift: Option<i16>,
    pub rooms: Vec<RoomView>,
}
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct VerticalView {
    pub from_row: u8,
    pub shift: i16,
    pub left: i32,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RoomView {
    pub page: u8,
    pub row: u8,
    pub tiles: Vec<u8>,
    pub attributes: Vec<u8>,
}

/// Extend symmetrically while scrolling; at a connected room boundary, use
/// the available map instead of exposing empty space outside the level.
pub fn viewport_left(layout: &world::Layout, camera: u16, row: u8) -> i32 {
    let page = (camera >> 8) as u8;
    let desired = i32::from(camera) - i32::from(config::SIDE_PIXELS);
    if layout.room_at(row, page).is_none() {
        return desired;
    }
    let mut first = page;
    let mut last = page;
    while first > 0 && layout.room_at(row, first - 1).is_some() {
        first -= 1;
    }
    while last + 1 < layout.columns && layout.room_at(row, last + 1).is_some() {
        last += 1;
    }
    let start = i32::from(first) * 256;
    let end = (i32::from(last) + 1) * 256;
    let width = config::SIZE.0 as i32;
    if end - start < width {
        (start + end - width) / 2
    } else {
        desired.clamp(start, end - width)
    }
}
pub fn project(actor: &mut ActorVisual, left: i32, clock: u8) -> bool {
    use presentation::*;
    let x = if actor.state & WORLD_POSITIONED != 0 {
        i32::from(actor.world_x) - left
    } else {
        i32::from(actor.world_x as u8) + i32::from(config::SIDE_PIXELS)
    };
    actor.screen_x = x.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
    let right = config::SIZE.0 as i32;
    if actor.state & WORLD_POSITIONED != 0 && !(0..right).contains(&x) {
        if actor.animation != OFFPAGE_PERSISTENT_ANIMATION
            && (x < -i32::from(NEAR_CAMERA_MARGIN) || x >= right + i32::from(NEAR_CAMERA_MARGIN))
        {
            actor.state = 0;
            actor.render = 0;
            actor.spawn_identity = None;
        } else {
            actor.render &= !VISIBLE;
        }
        return false;
    }
    actor.render |= VISIBLE;
    actor.state & PROJECTION_ONLY == 0 && (actor.render & FLASH == 0 || clock & FLASH != 0)
}

pub fn emit(
    sprites: &mut Vec<WideSprite>,
    slot: usize,
    actor: &ActorVisual,
    composite: &Composite,
) {
    for piece in &composite.pieces {
        let x = actor.screen_x + i16::from(piece.dx);
        let y = i16::from(actor.screen_y) + i16::from(piece.dy);
        if x <= -8
            || x >= config::SIZE.0 as i16
            || !(0..=255).contains(&y)
            || slot != 0
                && actor.render & presentation::ABOVE_STATUS == 0
                && y < i16::from(presentation::STATUS_TOP)
        {
            continue;
        }
        let mut attributes = piece.attributes ^ (actor.render & presentation::FLIP_HORIZONTAL);
        if actor.render & presentation::PRIORITY != 0 {
            attributes |= 0x20;
        }
        sprites.push(WideSprite {
            x,
            y: y as u8,
            tile: piece.tile,
            attributes,
        });
    }
}
impl Services {
    pub fn prepare_wide_terrain(&mut self, core: &mut Core) {
        self.video.wide_terrain = self.refresh_native_terrain(core);
        if self.video.wide_terrain.rooms.is_empty() {
            self.video.wide_gameplay = false;
        }
    }
    pub fn refresh_native_terrain(&mut self, core: &mut Core) -> TerrainView {
        if self.video.gameplay_display == 0
            || core.progression.ending_latch != 0
            || core.layout >= 5
        {
            return TerrainView::default();
        }
        let layout = &self.content.world.layouts[usize::from(core.layout)];
        let set = &self.content.world.terrain[usize::from(core.terrain_bank)];
        if core.cache.native.layout != Some(core.layout) {
            core.cache.native = world::NativeTerrain {
                layout: Some(core.layout),
                columns: layout.columns,
                ..Default::default()
            };
        }
        core.cache.native.row = core.row;
        core.cache
            .native
            .collision
            .resize(layout.rooms.len() * world::ROOM_CELLS, 0);
        let mut view = TerrainView {
            camera: core.camera.x,
            layout: core.layout,
            left: viewport_left(layout, core.camera.x, core.row),
            row: self.video.vertical_view.map_or(core.row, |v| v.from_row),
            vertical_shift: self.video.vertical_view.map(|v| v.shift),
            rooms: Vec::new(),
        };
        if let Some(vertical) = self.video.vertical_view {
            view.left = vertical.left;
        }
        let left = view.left.div_euclid(256);
        let right = (view.left + config::SIZE.0 as i32 - 1).div_euclid(256);
        for (index, payload) in layout.rooms.iter().enumerate() {
            let page = index as u8 % layout.columns;
            let row = index as u8 / layout.columns;
            let visible = (left..=right).contains(&i32::from(page)) && row.abs_diff(view.row) <= 1;
            let mut room_view = RoomView {
                page,
                row,
                tiles: vec![0; 32 * 30],
                attributes: vec![0; 64],
            };
            for cell in 0..world::ROOM_CELLS {
                let Some(payload) = payload else {
                    core.cache.native.collision[index * world::ROOM_CELLS + cell] = 0;
                    continue;
                };
                let x = cell % 8;
                let y = cell / 8;
                let context = world::ExpansionContext {
                    stage: core.progression.stage.0,
                    room: index as u8,
                    room_in_row: page,
                    ring: (x * 8 + y) as u8,
                    split: false,
                    stage_three_control: core.progression.hit_dialogue_count,
                    stage_four_control: core.progression.escort_prerequisite,
                };
                let block = self.content.raw_rooms[*payload].block_at(y, x);
                let expanded = world::expand(set, block, context);
                core.cache.native.collision[index * world::ROOM_CELLS + cell] = expanded.collision;
                if visible {
                    room_view.attributes[cell] = expanded.palette;
                    for dy in 0..4 {
                        for dx in 0..4 {
                            let ty = y * 4 + dy;
                            if ty < 30 {
                                room_view.tiles[ty * 32 + x * 4 + dx] = expanded.tiles[dy * 4 + dx];
                            }
                        }
                    }
                }
            }
            if visible && payload.is_some() {
                view.rooms.push(room_view);
            }
        }
        for change in core.cache.native.changes.clone() {
            core.cache.native.apply_collision(set, &change);
            if let Some(room) = view
                .rooms
                .iter_mut()
                .find(|room| room.page == (change.x >> 8) as u8 && room.row == change.row)
            {
                let tx = usize::from((change.x & 255) / 8);
                let ty = usize::from(change.y / 8);
                let tiles = set.tiles[usize::from(change.selector)];
                for dy in 0..2 {
                    for dx in 0..2 {
                        if ty + dy < 30 {
                            room.tiles[(ty + dy) * 32 + tx + dx] = tiles[dy * 2 + dx];
                        }
                    }
                }
                let cell = ty / 4 * 8 + tx / 4;
                let shift = ((ty & 2) * 2 + (tx & 2)) as u8;
                room.attributes[cell] = (room.attributes[cell] & !(3 << shift))
                    | ((set.properties[usize::from(change.selector)] & 3) << shift);
            }
        }
        view
    }
}
fn pattern(memory: &presentation::VideoMemory, tile: u8, table: usize, x: usize, y: usize) -> u8 {
    let at = table + usize::from(tile) * 16 + y;
    ((memory.chr[at] >> (7 - x)) & 1) | (((memory.chr[at + 8] >> (7 - x)) & 1) << 1)
}
impl TerrainView {
    fn pixel(&self, memory: &presentation::VideoMemory, x: i32, y: i32, table: usize) -> u8 {
        let page = x.div_euclid(256);
        let row = i32::from(self.row) + y.div_euclid(240);
        let Some(room) = self
            .rooms
            .iter()
            .find(|room| i32::from(room.page) == page && i32::from(room.row) == row)
        else {
            return 0;
        };
        let x = x.rem_euclid(256) as usize;
        let y = y.rem_euclid(240) as usize;
        let tx = x / 8;
        let ty = y / 8;
        let tile = room.tiles[ty * 32 + tx];
        let group = (room.attributes[ty / 4 * 8 + tx / 4] >> ((ty & 2) * 2 + (tx & 2))) & 3;
        let pixel = pattern(memory, tile, table, x & 7, y & 7);
        if pixel == 0 { 0 } else { group * 4 + pixel }
    }
}
impl Video {
    /// Native actor pixels use their gameplay patterns on both sides of the
    /// HUD split. Eligibility, flashing and top clipping were applied at emit.
    fn native_sprite_line(
        &self,
        y: u32,
        control: u8,
    ) -> [Option<ppu::SpritePixel>; config::SIZE.0 as usize] {
        let mut sprites = [None; config::SIZE.0 as usize];
        for sprite in &self.committed_wide_sprites {
            let height = if control & 0x20 != 0 { 16 } else { 8 };
            let dy = y as i32 - i32::from(sprite.y) - 1;
            if !(0..height).contains(&dy) {
                continue;
            }
            let row = if sprite.attributes & 0x80 != 0 {
                height - 1 - dy
            } else {
                dy
            } as usize;
            let (tile, table) = if height == 16 {
                (
                    (sprite.tile & 0xfe) + (row / 8) as u8,
                    usize::from(sprite.tile & 1) * 4096,
                )
            } else {
                (sprite.tile, if control & 8 != 0 { 4096 } else { 0 })
            };
            for dx in 0..8 {
                let x = i32::from(sprite.x) + dx;
                if !(0..config::SIZE.0 as i32).contains(&x) || sprites[x as usize].is_some() {
                    continue;
                }
                let px = if sprite.attributes & 0x40 != 0 {
                    7 - dx
                } else {
                    dx
                } as usize;
                let index = pattern(&self.memory, tile, table, px, row % 8);
                if index != 0 {
                    sprites[x as usize] = Some(ppu::SpritePixel {
                        palette: 16 + (sprite.attributes & 3) * 4 + index,
                        behind_background: sprite.attributes & 0x20 != 0,
                        sprite_zero: false,
                    });
                }
            }
        }
        sprites
    }
    pub fn hide_sprites(&mut self) {
        self.composition.hide(self.gameplay_display != 0);
        self.wide_sprites.clear();
    }
    pub fn render(&self, frame: &mut Frame) {
        frame.clear([0, 0, 0, 255]);
        let mut source =
            Frame::new(config::SOURCE_SIZE.0, config::SOURCE_SIZE.1).expect("source frame");
        let split = self.render_source(
            &mut source,
            self.committed_wide_gameplay && !self.wide_terrain.rooms.is_empty(),
        );
        let wide = split.is_some()
            && self.committed_wide_gameplay
            && !self.wide_terrain.rooms.is_empty()
            && self.raster.playfield_scroll.is_some();
        // Gameplay and UI now share the original display's pixel proportions.
        // Keep UI pixels unchanged instead of resampling the 256-column image.
        let ui_width = config::SOURCE_SIZE.0;
        let ui_left = (config::SIZE.0 - ui_width) / 2;
        // Keep the actual committed sprite-zero split, including transparent
        // pattern rows and stalled source raster polling.
        let split = split.unwrap_or(config::SOURCE_SIZE.1 as u16);
        for y in 0..config::SIZE.1 {
            // The hit scanline already contains scrolled playfield pixels.
            // Widen the whole row so that its source-camera tail cannot leak
            // into the differently scaled HUD band.
            if !wide || y < u32::from(split) {
                for dx in 0..ui_width {
                    let sx = dx;
                    let at = ((y * config::SOURCE_SIZE.0 + sx) * 4) as usize;
                    frame.pixel(
                        (ui_left + dx) as i32,
                        y as i32,
                        source.pixels[at..at + 4].try_into().unwrap(),
                    );
                }
                if wide {
                    // HUD suppression is still owned by Composition::begin.
                    // Eligible gameplay pieces, especially Scrooge, draw in
                    // front of this UI band even when marked behind terrain.
                    let control = self
                        .raster
                        .playfield_control
                        .unwrap_or(self.raster.ui_control);
                    let mask = self.raster.playfield_mask;
                    for (x, sprite) in self.native_sprite_line(y, control).into_iter().enumerate() {
                        let pixel = ppu::compose(0, sprite, x as u16, mask);
                        if pixel.palette != 0 {
                            let color = ppu::color(
                                &config::NES_RGB,
                                self.memory.read(0x3f00 + u16::from(pixel.palette)),
                                mask,
                            );
                            frame.pixel(x as i32, y as i32, color);
                        }
                    }
                }
                continue;
            }
            let control = self
                .raster
                .playfield_control
                .unwrap_or(self.raster.ui_control);
            let mask = self.raster.playfield_mask;
            let scroll_y = if let Some(shift) = self.wide_terrain.vertical_shift {
                y as i32 + i32::from(shift)
            } else if let Some(address) = self.raster.split_address {
                let origin = ((address >> 5) & 31) * 8
                    + ((address >> 12) & 7)
                    + if address & 0x800 != 0 { 240 } else { 0 };
                i32::from(origin) + y as i32 - i32::from(split)
            } else {
                i32::from(self.raster.playfield_scroll.unwrap().1) + y as i32
            };
            let sprites = self.native_sprite_line(y, control);
            for x in 0..config::SIZE.0 {
                let wx = self.wide_terrain.left + x as i32;
                let bg = self.wide_terrain.pixel(
                    &self.memory,
                    wx,
                    scroll_y,
                    if control & 0x10 != 0 { 4096 } else { 0 },
                );
                let pixel = ppu::compose(bg, sprites[x as usize], x as u16, mask);
                let color = ppu::color(
                    &config::NES_RGB,
                    self.memory.read(0x3f00 + u16::from(pixel.palette)),
                    mask,
                );
                frame.background_pixel(x as i32, y as i32, bg, color);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{actors, collision, identifiers::StageId};
    use romatlas_port::{
        Game,
        assets::{MemoryAssets, Rom},
    };
    fn services() -> Services {
        let rom = Rom::open(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../input/game.rom")
                .as_path(),
            config::ROM_SHA256,
        )
        .unwrap();
        let memory = MemoryAssets::default();
        let assets = crate::assets::Assets::load(&rom, &memory).unwrap();
        Services::new(super::super::content::Content::load(&rom, &assets, &memory).unwrap())
    }
    #[test]
    fn gameplay_and_hud_keep_the_original_display_pixel_proportions() {
        let source_pixel_aspect = (f64::from(config::SOURCE_DISPLAY_ASPECT.0)
            / f64::from(config::SOURCE_DISPLAY_ASPECT.1))
            / (f64::from(config::SOURCE_SIZE.0) / f64::from(config::SOURCE_SIZE.1));
        let gameplay_pixel_aspect = (f64::from(config::DISPLAY_ASPECT.0)
            / f64::from(config::DISPLAY_ASPECT.1))
            / (f64::from(config::SIZE.0) / f64::from(config::SIZE.1));
        assert!((source_pixel_aspect - gameplay_pixel_aspect).abs() < 0.002);
        // Every source UI column maps directly to one native column; gameplay
        // uses that same horizontal pixel scale below the HUD boundary.
        let mut video = Video::reset();
        video.memory.chr[16..24].fill(0xaa);
        video.memory.nametables.fill(1);
        video.memory.palette[1] = 1;
        video.raster.ui_mask = 0x0a;
        let mut source = Frame::new(config::SOURCE_SIZE.0, config::SOURCE_SIZE.1).unwrap();
        video.render_source(&mut source, false);
        let mut frame = Frame::new(config::SIZE.0, config::SIZE.1).unwrap();
        video.render(&mut frame);
        let left = ((config::SIZE.0 - config::SOURCE_SIZE.0) / 2) as usize;
        for y in 0..config::SIZE.1 as usize {
            let actual = (y * config::SIZE.0 as usize + left) * 4;
            let expected = y * config::SOURCE_SIZE.0 as usize * 4;
            assert_eq!(
                &frame.pixels[actual..actual + 256 * 4],
                &source.pixels[expected..expected + 256 * 4]
            );
        }
    }
    #[test]
    fn camera_fills_connected_rooms_and_centers_short_arenas() {
        let layout = world::Layout {
            rows: 1,
            columns: 5,
            rooms: vec![Some(0); 5],
        };
        assert_eq!(viewport_left(&layout, 0, 0), 0);
        assert_eq!(
            viewport_left(&layout, 512, 0),
            512 - i32::from(config::SIDE_PIXELS)
        );
        assert_eq!(
            viewport_left(&layout, 1024, 0),
            1280 - config::SIZE.0 as i32
        );
        let arena = world::Layout {
            rows: 1,
            columns: 5,
            rooms: vec![None, None, Some(0), None, None],
        };
        assert_eq!(
            viewport_left(&arena, 512, 0),
            (512 + 768 - config::SIZE.0 as i32) / 2
        );
    }
    #[test]
    fn native_sprite_line_retains_more_than_eight_pieces_and_clips_without_wrapping() {
        let mut sprites = Vec::new();
        let visual = ActorVisual {
            screen_x: 300,
            screen_y: 100,
            ..Default::default()
        };
        let composite = Composite {
            pieces: vec![
                presentation::Piece {
                    tile: 1,
                    attributes: 0,
                    dy: 0,
                    dx: 0
                };
                12
            ],
        };
        emit(&mut sprites, 1, &visual, &composite);
        assert_eq!(sprites.len(), 12);
        assert!(sprites.iter().all(|p| p.x == 300));
        let visual = ActorVisual {
            screen_x: -4,
            ..visual
        };
        emit(&mut sprites, 1, &visual, &composite);
        assert!(sprites.iter().skip(12).all(|p| p.x == -4));
    }
    #[test]
    fn vertical_scroll_selects_the_source_and_destination_room_rows() {
        let mut services = services();
        let mut core = Core::reset();
        let content = std::sync::Arc::make_mut(&mut services.content);
        content.world.layouts[0] = world::Layout {
            rows: 3,
            columns: 3,
            rooms: vec![Some(0); 9],
        };
        services.video.gameplay_display = 0x40;
        core.row = 1;
        core.camera.x = 256;
        services.video.vertical_view = Some(VerticalView {
            from_row: 0,
            shift: 120,
            left: 0,
        });
        services.prepare_wide_terrain(&mut core);
        assert_eq!(services.video.wide_terrain.row, 0);
        assert_eq!(services.video.wide_terrain.vertical_shift, Some(120));
        assert!(services.video.wide_terrain.rooms.iter().any(|r| r.row == 1));
        services.video.vertical_view = Some(VerticalView {
            from_row: 2,
            shift: -120,
            left: 0,
        });
        services.prepare_wide_terrain(&mut core);
        assert_eq!(services.video.wide_terrain.row, 2);
        assert_eq!(services.video.wide_terrain.vertical_shift, Some(-120));
        assert!(services.video.wide_terrain.rooms.iter().any(|r| r.row == 1));
    }
    #[test]
    fn native_pixels_cover_the_full_view_and_keep_unlimited_sprite_priority() {
        let mut video = Video::reset();
        // Opaque synthetic patterns, independently colored background/sprites.
        video.memory.chr[16..24].fill(255);
        video.memory.chr[40..48].fill(255);
        video.memory.nametables.fill(1);
        video.memory.palette[1] = 1;
        video.memory.palette[18] = 2;
        video.raster.ui_mask = 0x1e;
        video.raster.playfield_mask = 0x1e;
        video.raster.playfield_scroll = Some((0, 0));
        video.raster.playfield_control = Some(0);
        video.committed_oam[0] = presentation::Sprite {
            x: 8,
            y: 0,
            tile: 1,
            attributes: 0,
        };
        video.wide_gameplay = true;
        video.committed_wide_gameplay = true;
        video.wide_terrain.rooms = (0..4)
            .map(|page| RoomView {
                page,
                row: 0,
                tiles: vec![1; 32 * 30],
                attributes: vec![0; 64],
            })
            .collect();
        video.committed_wide_sprites = (0..12)
            .map(|i| WideSprite {
                x: 16 * i,
                y: 100,
                tile: 2,
                attributes: 0,
            })
            .collect();
        video.committed_wide_sprites.push(WideSprite {
            x: 300,
            y: 100,
            tile: 2,
            attributes: 0x20,
        });
        video.committed_wide_sprites.push(WideSprite {
            x: config::SIZE.0 as i16 - 1,
            y: 100,
            tile: 2,
            attributes: 0,
        });
        let mut frame = Frame::new(config::SIZE.0, config::SIZE.1).unwrap();
        video.render(&mut frame);
        let width = config::SIZE.0 as usize;
        let pixel =
            |x: usize, y: usize| &frame.pixels[(y * width + x) * 4..(y * width + x + 1) * 4];
        let bg = ppu::color(&config::NES_RGB, 1, 0x1e);
        let sprite = ppu::color(&config::NES_RGB, 2, 0x1e);
        assert_eq!(pixel(0, 99), bg);
        assert_eq!(pixel(width - 1, 99), bg);
        for i in 0..12 {
            assert_eq!(pixel(i * 16, 101), sprite, "sprite {i}");
        }
        assert_eq!(pixel(300, 101), bg);
        assert_eq!(pixel(width - 1, 101), sprite);
        assert_eq!(pixel(0, 0), [0, 0, 0, 255]);
        // The sprite-zero hit is on row 1, which must use the full native
        // viewport even as source scroll crosses tile/nametable boundaries.
        for scroll in [0_u16, 1, 7, 8, 255, 256] {
            video.wide_terrain.left = i32::from(scroll);
            video.raster.playfield_scroll = Some((scroll as u8, 0));
            video.raster.playfield_control = Some((scroll >> 8) as u8 & 1);
            video.render(&mut frame);
            assert_eq!(frame.background_index(0, 1), Some(1), "left at {scroll}");
            assert_eq!(
                frame.background_index(config::SIZE.0 as i32 - 1, 1),
                Some(1),
                "right at {scroll}"
            );
            assert_eq!(&frame.pixels[..4], &[0, 0, 0, 255]);
        }
    }
    #[test]
    fn player_crosses_the_hud_in_front_without_changing_terrain_priority() {
        let mut video = Video::reset();
        // HUD patterns live in a different table from gameplay sprites.
        video.memory.chr[0x1000 + 16..0x1000 + 24].fill(255);
        video.memory.chr[16..24].fill(255);
        video.memory.chr[32..40].fill(0xf0); // Half-opaque player piece.
        video.memory.chr[48..56].fill(255); // Split marker.
        video.memory.nametables.fill(1);
        video.memory.palette[1] = 1;
        video.memory.palette[17] = 2;
        video.raster.ui_control = 0x10;
        video.raster.ui_mask = 0x1e;
        video.raster.playfield_mask = 0x1e;
        video.raster.playfield_scroll = Some((0, 0));
        video.raster.playfield_control = Some(0);
        video.committed_oam[0] = presentation::Sprite {
            x: 8,
            y: 44,
            tile: 3,
            attributes: 0x20,
        };
        video.committed_wide_gameplay = true;
        video.wide_terrain.rooms = (0..2)
            .map(|page| RoomView {
                page,
                row: 0,
                tiles: vec![1; 32 * 30],
                attributes: vec![0; 64],
            })
            .collect();
        let mut background = Frame::new(config::SIZE.0, config::SIZE.1).unwrap();
        video.render(&mut background);
        let player = ActorVisual {
            screen_x: 80,
            screen_y: 42,
            ..Default::default()
        };
        let fg = ppu::color(&config::NES_RGB, 2, 0x1e);
        let width = config::SIZE.0 as usize;
        for attributes in [0, 0x20, 0x40, 0x60] {
            let piece = Composite {
                pieces: vec![presentation::Piece {
                    tile: 2,
                    attributes,
                    dx: 0,
                    dy: 0,
                }],
            };
            video.committed_wide_sprites.clear();
            emit(&mut video.committed_wide_sprites, 0, &player, &piece);
            let mut frame = background.clone();
            video.render(&mut frame);
            let opaque_x = if attributes & 0x40 == 0 { 80 } else { 84 };
            let transparent_x = if attributes & 0x40 == 0 { 84 } else { 80 };
            for y in 43..=50 {
                let at = (y * width + opaque_x) * 4;
                if y < 45 || attributes & 0x20 == 0 {
                    assert_eq!(
                        &frame.pixels[at..at + 4],
                        &fg,
                        "row {y}, attributes {attributes}"
                    );
                } else {
                    assert_eq!(&frame.pixels[at..at + 4], &background.pixels[at..at + 4]);
                }
                let at = (y * width + transparent_x) * 4;
                assert_eq!(&frame.pixels[at..at + 4], &background.pixels[at..at + 4]);
            }
            // A hidden sprite mask still suppresses the foreground.
            video.raster.playfield_mask = 0x0e;
            video.render(&mut frame);
            let at = (43 * width + opaque_x) * 4;
            assert_eq!(&frame.pixels[at..at + 4], &background.pixels[at..at + 4]);
            video.raster.playfield_mask = 0x1e;
            // Other actors retain their original top-of-screen eligibility.
            let mut ordinary = Vec::new();
            emit(&mut ordinary, 1, &player, &piece);
            assert!(ordinary.is_empty());
        }
        video.committed_wide_gameplay = false;
        let mut source = Frame::new(config::SOURCE_SIZE.0, config::SOURCE_SIZE.1).unwrap();
        video.render_source(&mut source, false);
        let mut frame = background.clone();
        video.render(&mut frame);
        let at = (43 * width + 80) * 4;
        let source_at =
            (43 * config::SOURCE_SIZE.0 as usize + 80 - config::SIDE_PIXELS as usize) * 4;
        assert_eq!(
            &frame.pixels[at..at + 4],
            &source.pixels[source_at..source_at + 4]
        );
    }
    #[test]
    fn wide_projection_keeps_signed_coordinates_and_culls_at_the_new_edges() {
        let left = -i32::from(config::SIDE_PIXELS);
        let right = config::SIZE.0 as i32 + left;
        for (relative, visible, alive) in [
            (left - 1, false, true),
            (left, true, true),
            (256, true, true),
            (right - 1, true, true),
            (right, false, true),
            (right + 64, false, false),
            (left - 65, false, false),
        ] {
            let mut visual = ActorVisual {
                state: 0xa0,
                world_x: (1024_i32 + relative) as u16,
                ..Default::default()
            };
            assert_eq!(
                project(&mut visual, 1024 - i32::from(config::SIDE_PIXELS), 0),
                visible,
                "{relative}"
            );
            assert_eq!(visual.state != 0, alive, "{relative}");
            if visible {
                assert_eq!(
                    i32::from(visual.screen_x),
                    relative + i32::from(config::SIDE_PIXELS)
                );
            }
        }
    }
    #[test]
    fn a_letterboxed_dialogue_does_not_despawn_actors_in_the_wings() {
        let mut services = services();
        let mut core = Core::reset();
        std::sync::Arc::make_mut(&mut services.content)
            .world
            .layouts[0] = world::Layout {
            rows: 1,
            columns: 5,
            rooms: vec![Some(0); 5],
        };
        services.video.gameplay_display = 0x40;
        core.camera.x = 512;
        services.prepare_wide_terrain(&mut core);
        core.actors[15].position.world_x = 784;
        core.actors[15].position.screen_y = 100;
        core.actors[15].set_state_code(0xa0);
        core.actors[15].render = 0xa0;
        core.player.action = crate::player::Action::Dialogue;
        services.compose(&mut core);
        assert!(!services.video.wide_gameplay);
        assert_eq!(
            core.actors[15].state_code() & actors::ACTIVE,
            actors::ACTIVE
        );
        assert_eq!(core.actors[15].screen_x, 784 - 512 + config::SIDE_PIXELS);
    }
    #[test]
    fn spawn_preparation_does_not_move_the_committed_background() {
        let mut services = services();
        let mut core = Core::reset();
        let content = std::sync::Arc::make_mut(&mut services.content);
        content.world.layouts[0] = world::Layout {
            rows: 1,
            columns: 5,
            rooms: vec![Some(0); 5],
        };
        content.spawns.levels.clear();
        content.spawns.source_levels.fill(None);
        content.spawn_planes.get_mut(&0).unwrap().x_high.fill(0xff);
        services.video.gameplay_display = 0x40;
        core.camera.x = 512;
        services.prepare_wide_terrain(&mut core);
        let before = serde_json::to_string(&services.video.wide_terrain).unwrap();
        core.camera.x += 32;
        services.spawn(&mut core);
        assert_eq!(
            before,
            serde_json::to_string(&services.video.wide_terrain).unwrap()
        );
        services.prepare_wide_terrain(&mut core);
        assert_eq!(
            services.video.wide_terrain.left,
            544 - i32::from(config::SIDE_PIXELS)
        );
    }
    #[test]
    fn native_collision_and_edits_do_not_alias_rooms_two_pages_apart() {
        let mut services = services();
        let content = std::sync::Arc::make_mut(&mut services.content);
        content.world.layouts[0] = world::Layout {
            rows: 1,
            columns: 4,
            rooms: vec![Some(0); 4],
        };
        content.raw_rooms[0].low.fill(0);
        content.raw_rooms[0].ninth_bits.fill(0);
        let set = &mut content.world.terrain[0];
        set.blocks[0] = [0; 4];
        set.properties[0] = 0x10;
        set.properties[1] = 0x20;
        let mut core = Core::reset();
        core.camera.x = 256;
        core.layout = 0;
        services.video.gameplay_display = 0x40;
        services.prepare_wide_terrain(&mut core);
        core.cache.replace(
            &services.content.world.terrain[0],
            world::Mutation {
                world_x: 0x230,
                spawn_y: 0x50,
                selector: 1,
                marks_changed: true,
            },
        );
        for (x, class) in [(0x30, 1), (0x230, 2)] {
            let sample = collision::sample(
                &core.cache,
                collision::Position {
                    world_x: x,
                    screen_y: 0x50,
                    ..Default::default()
                },
                0,
                collision::SampleOffset::default(),
            );
            assert_eq!(sample.class, class);
        }
        services.prepare_wide_terrain(&mut core);
        let view = &services.video.wide_terrain;
        assert!(view.rooms.iter().any(|room| room.page == 0));
        assert!(view.rooms.iter().any(|room| room.page == 2));
        assert_eq!(core.cache.native.changes.len(), 1);
        let snapshot = serde_json::to_string(&core).unwrap();
        let restored: Core = serde_json::from_str(&snapshot).unwrap();
        assert_eq!(core.cache.native, restored.cache.native);
    }
    #[test]
    fn widened_spawn_window_activates_both_wings_and_tracks_direction_reversal() {
        let mut services = services();
        let mut core = Core::reset();
        let content = std::sync::Arc::make_mut(&mut services.content);
        content.world.layouts[0] = world::Layout {
            rows: 1,
            columns: 4,
            rooms: vec![Some(0); 4],
        };
        content.spawns.levels.clear();
        content.spawns.source_levels.fill(None);
        let planes = content.spawn_planes.get_mut(&0).unwrap();
        planes.x_high.fill(0xff);
        planes.room_starts.fill(0);
        let template = content
            .spawns
            .templates
            .iter()
            .position(|t| t.flags_and_local & 0xa0 == 0xa0 && !matches!(t.handler, 8 | 10))
            .unwrap();
        let left = 0x200 - config::SIDE_PIXELS as u16;
        let right = left + config::SIZE.0 as u16 - 1;
        for (index, x) in [left, 0x1ff, 0x300, right, right + 1]
            .into_iter()
            .enumerate()
        {
            planes.x_high[index] = (x >> 8) as u8;
            planes.x_low[index] = x as u8;
            planes.templates[index] = template as u8;
            planes.y[index] = 100;
        }
        core.camera.x = 0x200;
        core.spawn_window.direction = 1;
        core.special_spawn = actors::SpecialControl::Cooldown(1);
        services.video.gameplay_display = 0x40;
        services.spawn(&mut core);
        let identities: Vec<_> = core
            .actors
            .iter()
            .filter_map(|a| a.spawn_identity)
            .collect();
        assert_eq!(identities.len(), 4);
        for id in 0..4 {
            assert!(identities.contains(&id));
        }
        assert!(!identities.contains(&4));
        core.camera.x += 1;
        services.spawn(&mut core);
        assert!(core.actors.iter().any(|a| a.spawn_identity == Some(4)));
        // Move past the left record, remove it, then turn back across the edge.
        core.camera.x = 0x201;
        services.spawn(&mut core);
        for actor in &mut core.actors {
            if actor.spawn_identity == Some(0) {
                *actor = Default::default();
            }
        }
        core.camera.x = 0x200;
        core.spawn_window.direction = 2;
        services.spawn(&mut core);
        assert!(core.actors.iter().any(|a| a.spawn_identity == Some(0)));
    }
    #[test]
    fn a_full_actor_pool_defers_new_visible_spawns_instead_of_losing_them() {
        let mut pool = [actors::ActorContact::default(); actors::ACTOR_SLOTS];
        for actor in pool.iter_mut().skip(1) {
            actor.set_state_code(0xa0);
        }
        let records = [actors::SpawnRecord {
            identity: 0,
            world_x: config::SIZE.0 as u16 - 1,
            screen_y: 100,
            template: 0,
        }];
        let templates = [actors::Template {
            flags_and_local: 0xa0,
            ..Default::default()
        }];
        let mut context = actors::SpawnContext::default();
        let mut window = actors::Window {
            direction: 1,
            ..Default::default()
        };
        actors::scan_window_extent_with(
            &mut pool,
            &[0; 32],
            &records,
            &templates,
            &mut window,
            &mut context,
            config::SIZE.0 as u16 - 1,
            &mut |_, _, _| {},
        );
        assert_eq!(window.upper, 0);
        pool[15] = Default::default();
        let report = actors::scan_window_extent_with(
            &mut pool,
            &[0; 32],
            &records,
            &templates,
            &mut window,
            &mut context,
            config::SIZE.0 as u16 - 1,
            &mut |_, _, _| {},
        );
        assert_eq!(report.ordinary, [(0, 15)]);
        assert_eq!(window.upper, 1);
    }
    #[test]
    fn distant_visible_actors_cannot_hit_each_other_through_low_byte_wrap() {
        let mut pool = [actors::ActorContact::default(); actors::ACTOR_SLOTS];
        for (slot, x) in [(1, 256), (2, 512)] {
            pool[slot].position.world_x = x;
            pool[slot].position.screen_y = 100;
            pool[slot].render = 0x80;
            pool[slot].set_state_code(0xa0);
        }
        let extents = [collision::Extent {
            half_width: 16,
            half_height: 16,
        }; 32];
        assert_eq!(actors::actor_overlap(&mut pool, 1, &extents), None);
        pool[2].position.world_x = 264;
        assert_eq!(actors::actor_overlap(&mut pool, 1, &extents), Some(2));
    }
    #[test]
    fn every_stage_renders_full_width_without_mutating_rules() {
        for stage in 0..5 {
            let mut services = services();
            let mut core = Core::reset();
            core.progression.stage = StageId(stage);
            services.video.display.suspended = 1;
            services.load_stage_presentation(&mut core, stage);
            services.build_terrain(&mut core, false);
            services.video.gameplay_display = 0x40;
            services.video.wide_gameplay = true;
            services.video.display.suspended = 0;
            services.video.raster.playfield_scroll = Some((0, 0));
            services.video.raster.playfield_mask = 0x1e;
            services.video.raster.split_sprite.y = 46;
            services.prepare_wide_terrain(&mut core);
            assert!(
                !services.video.wide_terrain.rooms.is_empty(),
                "stage {stage}"
            );
            let before = serde_json::to_string(&core).unwrap();
            let mut frame = Frame::new(config::SIZE.0, config::SIZE.1).unwrap();
            services.video.render(&mut frame);
            assert_eq!(before, serde_json::to_string(&core).unwrap());
            assert!(frame.pixels.chunks_exact(4).all(|p| p[3] == 255));
            assert_eq!(&frame.pixels[..4], &[0, 0, 0, 255]);
        }
    }
    #[test]
    fn natural_gameplay_commits_wide_sprites_and_snapshots() {
        let rom = Rom::open(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../input/game.rom")
                .as_path(),
            config::ROM_SHA256,
        )
        .unwrap();
        let mut port = super::super::Port::load_assets(&rom, &MemoryAssets::default()).unwrap();
        for _ in 0..180 {
            port.tick(&Default::default());
        }
        let button = |name: &str| romatlas_port::input::InputState {
            held: [name.to_string()].into(),
            ..Default::default()
        };
        port.tick(&button("start"));
        for _ in 0..180 {
            port.tick(&Default::default());
        }
        port.tick(&button("a"));
        for _ in 0..360 {
            port.tick(&Default::default());
        }
        assert_eq!(port.mode(), crate::identifiers::Mode::Gameplay);
        assert!(!port.machine.services.video.wide_terrain.rooms.is_empty());
        assert!(
            !port
                .machine
                .services
                .video
                .committed_wide_sprites
                .is_empty()
        );
        let snapshot = port.snapshot_json().unwrap();
        let mut restored = port.clone();
        restored.restore_snapshot_json(&snapshot).unwrap();
        let mut frame = Frame::new(config::SIZE.0, config::SIZE.1).unwrap();
        let mut other = frame.clone();
        port.render(&mut frame);
        restored.render(&mut other);
        assert_eq!(frame.pixels, other.pixels);
        if let Ok(path) = std::env::var("DUCKTALES_CAPTURE_RGBA") {
            std::fs::write(path, &frame.pixels).unwrap();
        }
    }
}

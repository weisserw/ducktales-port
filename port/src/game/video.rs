//! Prepared display data and immutable committed pixels have separate lifetimes.
//! The world owns the overlapping0500 attribute bytes; only disjoint secondary
//! queue bytes are retained here. Rendering never advances source clocks.
mod terrain;
use crate::presentation::{
    self, Composition, Display, RasterSnapshot, Sprite, VideoMemory, VideoWrite,
};
use romatlas_port::{graphics::Frame, nes::ppu};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TimerLayout {
    #[default]
    Countdown,
    GameOver,
}
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Video {
    pub display: Display,
    pub composition: Composition,
    pub memory: VideoMemory,
    pub raster: RasterSnapshot,
    #[serde(with = "crate::snapshot::array")]
    pub committed_oam: [Sprite; presentation::OAM_ENTRIES],
    pub gameplay_display: u8,
    pub split_y: u8,
    pub ui_x: u8,
    pub ui_y: u8,
    pub playfield_y: u8,
    pub split_address: u16,
    pub overlay_gate: u8,
    pub timer_layout: TimerLayout,
    pub oam_guard: u8,
    #[serde(with = "crate::snapshot::array")]
    pub terrain_staging: [u8; 16],
    /// Reset clears these two unwritten bytes between preset base and staging.
    #[serde(with = "crate::snapshot::array")]
    pub preset_padding: [u8; 2],
}
impl Video {
    pub fn reset() -> Self {
        let mut display = Display {
            control: crate::runtime::RESET_CONTROL,
            secondary: presentation::PreparedSecondary::source_shared(),
            ..Display::default()
        };
        display.primary.encoded.resize(64, 0);
        for page in [0x20, 0x24] {
            display.fill(
                page,
                crate::runtime::RESET_BACKGROUND_TILE,
                0,
                presentation::Stride::Across,
            );
        }
        let composition = Composition::default();
        let mut video = Self {
            display,
            committed_oam: composition.snapshot(),
            composition,
            memory: VideoMemory::blank(),
            raster: RasterSnapshot::default(),
            gameplay_display: 0,
            split_y: 0,
            ui_x: 0,
            ui_y: 0,
            playfield_y: 0,
            split_address: 0,
            overlay_gate: 0,
            timer_layout: TimerLayout::Countdown,
            oam_guard: 0,
            terrain_staging: [0; 16],
            preset_padding: [0; 2],
        };
        video.commit_writes();
        video
    }
    pub fn commit_writes(&mut self) {
        for write in self.display.writes.drain(..) {
            self.memory.write(write);
        }
    }
    /// The audio low-F consumer receives only the concrete named domains its
    /// byte selector can reach. No addressable CPU image or duplicate queue.
    pub fn audio_alias(&self, address: u16) -> u8 {
        match address {
            0x03ee..=0x03ef => self.preset_padding[usize::from(address - 0x03ee)],
            0x03f0..=0x03ff => self.terrain_staging[usize::from(address - 0x03f0)],
            0x0400..=0x043f => self.display.primary.encoded[usize::from(address - 0x0400)],
            0x0440..=0x04cc => self.display.secondary.encoded[usize::from(address - 0x0440)],
            _ => unreachable!("indexed audio selector exceeds its concrete borrowed domains"),
        }
    }
    /// Original status messages can exceed the64-byte primary prefix. Their
    /// overflow immediately authors the same secondary bytes consumed by audio,
    /// terrain and runtime composites; no copied full-primary buffer is kept.
    pub fn prepare_primary(
        &mut self,
        bytes: &[u8],
        pending: u8,
        attributes: &mut [u8; crate::world::CACHE_CELLS],
    ) {
        prepare_primary_owners(
            &mut self.display.primary,
            &mut self.display.secondary,
            attributes,
            bytes,
            pending,
        );
    }
    /// Reads the canonical primary/secondary/cache aliases without copying storage.
    pub fn primary_byte(&self, offset: usize, attributes: &[u8; crate::world::CACHE_CELLS]) -> u8 {
        const PREFIX: usize = 64;
        if offset < PREFIX {
            self.display.primary.encoded[offset]
        } else if offset < PREFIX + presentation::SECONDARY_PREFIX_BYTES {
            self.display.secondary.encoded[offset - PREFIX]
        } else {
            attributes[offset - PREFIX - presentation::SECONDARY_PREFIX_BYTES]
        }
    }
    pub fn write_primary_byte(
        &mut self,
        offset: usize,
        value: u8,
        attributes: &mut [u8; crate::world::CACHE_CELLS],
    ) {
        const PREFIX: usize = 64;
        if offset < PREFIX {
            self.display.primary.encoded[offset] = value;
        } else if offset < PREFIX + presentation::SECONDARY_PREFIX_BYTES {
            self.display.secondary.encoded[offset - PREFIX] = value;
        } else {
            attributes[offset - PREFIX - presentation::SECONDARY_PREFIX_BYTES] = value;
        }
    }
    /// QueueTerrainAttributes reads the nametable high byte via staging+9,Y.
    /// This consumer spans retained staging and the canonical primary prefixes.
    pub fn staging_queue_byte(
        &self,
        index: usize,
        attributes: &[u8; crate::world::CACHE_CELLS],
    ) -> u8 {
        if index < self.terrain_staging.len() {
            self.terrain_staging[index]
        } else {
            self.primary_byte(index - self.terrain_staging.len(), attributes)
        }
    }
    /// ExpandTerrainBlock skips the lower quadrants at the bottom ring row.
    /// Their staging bytes survive from the previous expansion, including for
    /// the audio indexed reader. Only written tiles update this canonical owner.
    pub fn retain_terrain_staging(&mut self, tiles: &[u8; 16], bottom_row: bool) {
        let written = if bottom_row {
            8
        } else {
            self.terrain_staging.len()
        };
        self.terrain_staging[..written].copy_from_slice(&tiles[..written]);
    }
    pub fn primary_bytes(&self, attributes: &[u8; crate::world::CACHE_CELLS]) -> Vec<u8> {
        self.display
            .primary
            .encoded
            .iter()
            .chain(self.display.secondary.encoded.iter())
            .chain(attributes)
            .copied()
            .collect()
    }
    /// Predict the source polling condition from the committed UI snapshot.
    /// A missing hit stalls that native owner while interrupts/audio continue;
    /// this query neither renders nor mutates the simulation or task controls.
    pub fn ui_sprite_zero_hit(&self) -> bool {
        const CHR_BANKS: [usize; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
        let mut oam = [0; 256];
        for (bytes, sprite) in oam.chunks_exact_mut(4).zip(self.committed_oam) {
            bytes.copy_from_slice(&[sprite.y, sprite.tile, sprite.attributes, sprite.x]);
        }
        let control = self.raster.ui_control;
        let background = ppu::Background {
            chr: &self.memory.chr,
            chr_banks: CHR_BANKS,
            nametables: &self.memory.nametables,
            mirroring: ppu::Mirroring::Vertical,
            pattern_table: if control & 0x10 != 0 { 0x1000 } else { 0 },
            scroll: ppu::Scroll {
                x: u16::from(self.raster.ui_scroll.0),
                y: u16::from(self.raster.ui_scroll.1),
                nametable: control & 3,
            },
        };
        for y in 0..crate::config::SIZE.1 as u16 {
            let sprites = ppu::sprites(
                &self.memory.chr,
                CHR_BANKS,
                &oam,
                y,
                control & 0x20 != 0,
                if control & 8 != 0 { 0x1000 } else { 0 },
            )
            .expect("validated UI sprite view");
            for x in 0..crate::config::SIZE.0 as u16 {
                if ppu::compose(
                    background
                        .pixel(x, y)
                        .expect("validated UI background view"),
                    sprites.pixels[usize::from(x)],
                    x,
                    self.raster.ui_mask,
                )
                .sprite_zero_hit
                {
                    return true;
                }
            }
        }
        false
    }
    pub fn render(&self, frame: &mut Frame) {
        const CHR_BANKS: [usize; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
        let mut oam = [0; 256];
        for (record, sprite) in oam.chunks_exact_mut(4).zip(self.committed_oam) {
            record.copy_from_slice(&[sprite.y, sprite.tile, sprite.attributes, sprite.x]);
        }
        let mut playfield = false;
        let mut split_line = 0;
        for y in 0..crate::config::SIZE.1 as u16 {
            let ui_control = self.raster.ui_control;
            let sprite_line = ppu::sprites(
                &self.memory.chr,
                CHR_BANKS,
                &oam,
                y,
                ui_control & 0x20 != 0,
                if ui_control & 8 != 0 { 0x1000 } else { 0 },
            )
            .expect("validated sprite view");
            let playfield_control = self.raster.playfield_control.unwrap_or(ui_control);
            let playfield_sprites = ppu::sprites(
                &self.memory.chr,
                CHR_BANKS,
                &oam,
                y,
                playfield_control & 0x20 != 0,
                if playfield_control & 8 != 0 {
                    0x1000
                } else {
                    0
                },
            )
            .expect("validated playfield sprite view");
            for x in 0..crate::config::SIZE.0 as u16 {
                let control = if playfield {
                    self.raster.playfield_control.unwrap_or(ui_control)
                } else {
                    ui_control
                };
                let scroll = if playfield {
                    self.raster
                        .playfield_scroll
                        .unwrap_or(self.raster.ui_scroll)
                } else {
                    self.raster.ui_scroll
                };
                let mut vertical_scroll = u16::from(scroll.1);
                if playfield && let Some(address) = self.raster.split_address {
                    // PPUADDR replaces the active vertical tile/fine position;
                    // the following PPUSCROLL supplies horizontal fine scroll.
                    // Retain that explicit origin through the remaining rows.
                    let origin = ((address >> 5) & 31) * 8
                        + ((address >> 12) & 7)
                        + if address & 0x800 != 0 { 240 } else { 0 };
                    vertical_scroll = (origin + 480 - split_line) % 480;
                }
                let background = ppu::Background {
                    chr: &self.memory.chr,
                    chr_banks: CHR_BANKS,
                    nametables: &self.memory.nametables,
                    mirroring: ppu::Mirroring::Vertical,
                    pattern_table: if control & 0x10 != 0 { 0x1000 } else { 0 },
                    scroll: ppu::Scroll {
                        x: u16::from(scroll.0),
                        y: vertical_scroll,
                        nametable: control & 3,
                    },
                }
                .pixel(x, y)
                .expect("validated background view");
                let mask = if playfield {
                    self.raster.playfield_mask
                } else {
                    self.raster.ui_mask
                };
                let sprites = if playfield {
                    &playfield_sprites
                } else {
                    &sprite_line
                };
                let pixel = ppu::compose(background, sprites.pixels[usize::from(x)], x, mask);
                let color_index = self.memory.read(0x3f00 + u16::from(pixel.palette));
                frame.background_pixel(
                    i32::from(x),
                    i32::from(y),
                    background,
                    ppu::color(&crate::config::NES_RGB, color_index, mask),
                );
                // Native raster policy changes the remaining scanline after the
                // first actual sprite-zero/background overlap. CPU polling and
                // register-write instruction delays are deliberately not emulated.
                if !playfield && pixel.sprite_zero_hit && self.raster.playfield_scroll.is_some() {
                    playfield = true;
                    split_line = y;
                }
            }
        }
    }
    pub fn write_direct(&mut self, address: u16, value: u8) {
        self.memory.write(VideoWrite { address, value });
    }
}

/// Write the primary prefix and its physically shared neighboring fields through
/// disjoint canonical borrows, including while dialogue lends its Panel owner.
pub fn prepare_primary_owners(
    primary: &mut presentation::PreparedPrimary,
    secondary: &mut presentation::PreparedSecondary,
    attributes: &mut [u8; crate::world::CACHE_CELLS],
    bytes: &[u8],
    pending: u8,
) {
    prepare_primary_at_owners(primary, secondary, attributes, 0, bytes, pending);
}
pub fn prepare_primary_at_owners(
    primary: &mut presentation::PreparedPrimary,
    secondary: &mut presentation::PreparedSecondary,
    attributes: &mut [u8; crate::world::CACHE_CELLS],
    offset: usize,
    bytes: &[u8],
    pending: u8,
) {
    const PRIMARY_PREFIX_BYTES: usize = 64;
    primary
        .encoded
        .resize(primary.encoded.len().max(PRIMARY_PREFIX_BYTES), 0);
    for (index, &value) in bytes.iter().enumerate() {
        let offset = offset + index;
        if offset < PRIMARY_PREFIX_BYTES {
            primary.encoded[offset] = value;
        } else if offset < PRIMARY_PREFIX_BYTES + presentation::SECONDARY_PREFIX_BYTES {
            secondary.encoded[offset - PRIMARY_PREFIX_BYTES] = value;
        } else {
            attributes[offset - PRIMARY_PREFIX_BYTES - presentation::SECONDARY_PREFIX_BYTES] =
                value;
        }
    }
    primary.pending = pending;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bottom_terrain_row_keeps_unwritten_staging_visible_to_audio() {
        let mut video = Video::reset();
        video.retain_terrain_staging(&[0x79; 16], false);
        video.retain_terrain_staging(&[0x35; 16], true);
        assert_eq!(&video.terrain_staging[..8], &[0x35; 8]);
        assert_eq!(&video.terrain_staging[8..], &[0x79; 8]);
        assert_eq!(video.audio_alias(0x03f7), 0x35);
        assert_eq!(video.audio_alias(0x03f8), 0x79);
    }
    #[test]
    fn primary_message_overflow_and_short_rewrite_share_retained_secondary_bytes() {
        let mut video = Video::reset();
        let mut attributes = [0; crate::world::CACHE_CELLS];
        let mut message = (0..104).map(|index| index as u8).collect::<Vec<_>>();
        message.push(0xff);
        video.prepare_primary(&message, 0x67, &mut attributes);
        assert_eq!(video.display.primary.encoded.len(), 64);
        assert_eq!(&video.display.secondary.encoded[..41], &message[64..]);
        assert_eq!(&video.primary_bytes(&attributes)[..105], message);
        video.prepare_primary(&[0xff], 1, &mut attributes);
        assert_eq!(video.audio_alias(0x0449), 73);
        assert_eq!(video.audio_alias(0x043f), 63);
        video.display.secondary.encoded[9] = 0xb7;
        assert_eq!(video.primary_bytes(&attributes)[73], 0xb7);
    }
}

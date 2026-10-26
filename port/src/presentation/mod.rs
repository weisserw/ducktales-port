//! Source-owned sprite composition and committed display state.
//! Adjacent contracts/trace derive the rules; rendering only reads snapshots.
use crate::runtime::HIDDEN_SPRITE_Y;
pub const ACTOR_SLOTS: usize = 16;
pub const OAM_ENTRIES: usize = 64;
pub const PALETTE_COLORS: usize = 32;
pub const WORLD_POSITIONED: u8 = 0x20;
pub const ACTIVE: u8 = 0x80;
pub const PROJECTION_ONLY: u8 = 0x10;
pub const VISIBLE: u8 = 0x80;
pub const FLASH: u8 = 0x02;
pub const PRIORITY: u8 = 0x04;
pub const DESTROY_AT_END: u8 = 0x08;
pub const ABOVE_STATUS: u8 = 0x10;
pub const FLIP_HORIZONTAL: u8 = 0x40;
pub const HIDDEN_RENDER: u8 = 0xff;
pub const OFFPAGE_PERSISTENT_ANIMATION: u8 = 0x5a;
pub const STATUS_TOP: u8 = 0x30;
pub const OVERLAY_PLAYER_MIN_Y: u8 = 0x44;
mod playback;
pub mod terrain;
pub use playback::AnimationPlayback;
pub const NEAR_CAMERA_MARGIN: u8 = 0x40;
pub const DIALOGUE_PLAYER_STATE: u8 = 0x0f;
pub const BLACK: u8 = 0x0f;
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Sprite {
    pub y: u8,
    pub tile: u8,
    pub attributes: u8,
    pub x: u8,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ActorVisual {
    pub state: u8,
    pub render: u8,
    pub world_x: u16,
    pub screen_x: i16,
    pub screen_y: u8,
    pub animation: u8,
    pub playback: AnimationPlayback,
    pub spawn_identity: Option<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Animation {
    pub last_frame: u8,
    pub last_tick: u8,
    pub frames: Vec<u8>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Piece {
    pub tile: u8,
    pub attributes: u8,
    pub dy: i8,
    pub dx: i8,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Composite {
    pub pieces: Vec<Piece>,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OverlayGate {
    pub gameplay_latch: u8,
    pub dialogue_cursor: u8,
    pub ending: u8,
    pub status_stream: u8,
    pub player_state: u8,
    pub player_y: u8,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Composition {
    pub clock: u8,
    pub cursor: u8,
    #[serde(with = "crate::snapshot::array")]
    pub oam: [Sprite; OAM_ENTRIES],
}
impl Default for Composition {
    fn default() -> Self {
        Self {
            clock: 0,
            cursor: 4,
            oam: [Sprite {
                y: HIDDEN_SPRITE_Y,
                ..Sprite::default()
            }; OAM_ENTRIES],
        }
    }
}
impl Composition {
    pub fn player_flags(actor: &mut ActorVisual, protection: u8, special: u8) {
        const TRANSIENT_RENDER_BITS: u8 = 3;
        actor.render &= !TRANSIENT_RENDER_BITS;
        if protection | special != 0 {
            actor.render |= FLASH;
        }
    }
    pub fn begin(&mut self, gate: OverlayGate, overlays: [&[u8]; 3]) -> Vec<usize> {
        const SPRITE_BYTES: u8 = 4;
        self.cursor = SPRITE_BYTES;
        if gate.gameplay_latch & ACTIVE != 0
            && gate.dialogue_cursor == 0
            && gate.ending == 0
            && gate.status_stream == 0
            && gate.player_state != DIALOGUE_PLAYER_STATE
            && gate.player_y >= OVERLAY_PLAYER_MIN_Y
        {
            for overlay in overlays {
                for &byte in overlay
                    .iter()
                    .take(256)
                    .take_while(|&&byte| byte != u8::MAX)
                {
                    let sprite = &mut self.oam[usize::from(self.cursor / SPRITE_BYTES)];
                    match self.cursor % SPRITE_BYTES {
                        0 => sprite.y = byte,
                        1 => sprite.tile = byte,
                        2 => sprite.attributes = byte,
                        _ => sprite.x = byte,
                    }
                    self.cursor = self.cursor.wrapping_add(1);
                }
            }
        }
        self.clock = self.clock.wrapping_add(1);
        if self.clock & 1 != 0 {
            (0..ACTOR_SLOTS).rev().collect()
        } else {
            (0..ACTOR_SLOTS).collect()
        }
    }
    pub fn project(&mut self, actor: &mut ActorVisual, camera: u16) -> bool {
        if actor.state & WORLD_POSITIONED != 0 {
            let relative = actor.world_x.wrapping_sub(camera);
            actor.screen_x = i16::from(relative as u8);
            if relative >> 8 != 0 {
                let low_distance = (relative as u8 as i8).unsigned_abs();
                if actor.animation != OFFPAGE_PERSISTENT_ANIMATION
                    && low_distance >= NEAR_CAMERA_MARGIN
                {
                    actor.state = 0;
                    actor.render = 0;
                    actor.spawn_identity = None;
                } else {
                    actor.screen_x = 0;
                    actor.render &= !VISIBLE;
                }
                return false;
            }
            actor.render |= VISIBLE;
            if actor.state & PROJECTION_ONLY != 0 {
                return false;
            }
        } else {
            actor.screen_x = i16::from(actor.world_x as u8);
            actor.render |= VISIBLE;
        }
        actor.render & FLASH == 0 || self.clock & FLASH != 0
    }
    pub fn advance(actor: &mut ActorVisual, sequence: &Animation, player_state: u8) -> Option<u8> {
        if player_state != DIALOGUE_PLAYER_STATE
            && actor.playback.advance(
                sequence.last_tick,
                sequence.last_frame,
                actor.render & DESTROY_AT_END != 0,
            )
        {
            actor.state = 0;
            actor.render = 0;
            actor.spawn_identity = None;
            return None;
        }
        Some(sequence.frames[actor.playback.frame_index()])
    }

    pub fn emit(&mut self, slot: usize, actor: &ActorVisual, frame: &Composite) {
        const SPRITE_BYTES: u8 = 4;
        const OAM_PRIORITY_ATTRIBUTE: u8 = 0x20;
        if self.cursor == 0 {
            return;
        }
        for piece in &frame.pieces {
            let entry = &mut self.oam[usize::from(self.cursor / SPRITE_BYTES)];
            entry.tile = piece.tile;
            let y = i16::from(actor.screen_y) + i16::from(piece.dy);
            if !(0..=255).contains(&y)
                || slot != 0 && actor.render & ABOVE_STATUS == 0 && y < i16::from(STATUS_TOP)
            {
                entry.y = HIDDEN_SPRITE_Y;
                continue;
            }
            entry.y = y as u8;
            entry.attributes = piece.attributes ^ (actor.render & FLIP_HORIZONTAL);
            if actor.render & PRIORITY != 0 {
                entry.attributes |= OAM_PRIORITY_ATTRIBUTE;
            }
            let x = actor.screen_x + i16::from(piece.dx);
            entry.x = x as u8;
            if !(0..=255).contains(&x) {
                entry.y = HIDDEN_SPRITE_Y;
                continue;
            }
            self.cursor = self.cursor.wrapping_add(SPRITE_BYTES);
            if self.cursor == 0 {
                break;
            }
        }
    }
    pub fn hide(&mut self, gameplay: bool) {
        for sprite in self.oam.iter_mut().skip(usize::from(gameplay)) {
            sprite.y = HIDDEN_SPRITE_Y;
        }
    }
    pub fn snapshot(&self) -> [Sprite; OAM_ENTRIES] {
        self.oam
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stride {
    Across,
    Down,
}
impl Stride {
    fn increment(self) -> u16 {
        const TILE_ROW_BYTES: u16 = 32;
        match self {
            Self::Across => 1,
            Self::Down => TILE_ROW_BYTES,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VideoWrite {
    pub address: u16,
    pub value: u8,
}
/// Canonical prepared primary upload. Exact pending byte survives owner yields;
/// interrupt commit consumes this queue. B15 borrows this same object.
#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PreparedPrimary {
    pub encoded: Vec<u8>,
    pub pending: u8,
}
impl PreparedPrimary {
    /// Read-only decoded text-row observation of the prepared byte stream.
    /// No second retained row store is permitted.
    pub fn rows(&self) -> [Vec<u8>; 3] {
        const ENCODED_ROW_BYTES: usize = 20;
        std::array::from_fn(|row| {
            let offset = row * ENCODED_ROW_BYTES;
            let Some(&high) = self.encoded.get(offset) else {
                return Vec::new();
            };
            if high & 0x80 != 0 {
                return Vec::new();
            }
            let Some(&count) = self.encoded.get(offset + 2) else {
                return Vec::new();
            };
            if count & 0x80 != 0 {
                return Vec::new();
            }
            self.encoded
                .get(offset + 3..offset + 4 + usize::from(count))
                .unwrap_or(&[])
                .to_vec()
        })
    }
}
#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Display {
    pub suspended: u8,
    pub mask: u8,
    pub control: u8,
    pub primary: PreparedPrimary,
    pub secondary_pending: bool,
    pub secondary: PreparedSecondary,
    pub palette_pending: bool,
    #[serde(with = "crate::snapshot::array")]
    pub base: [u8; PALETTE_COLORS],
    #[serde(with = "crate::snapshot::array")]
    pub active: [u8; PALETTE_COLORS],
    pub writes: Vec<VideoWrite>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Queue {
    Primary,
    Secondary,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FadeDirection {
    In,
    Out,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Fade {
    pub direction: FadeDirection,
    pub subtraction: u8,
    pub waits_remaining: u8,
    pub complete: bool,
}
impl Fade {
    pub fn start(direction: FadeDirection) -> Self {
        const FADE_IN_FIRST_SUBTRACTION: u8 = 0x30;
        const FADE_OUT_FIRST_SUBTRACTION: u8 = 0x10;
        const FADE_STEP_WAITS: u8 = 4;
        Self {
            direction,
            subtraction: if direction == FadeDirection::In {
                FADE_IN_FIRST_SUBTRACTION
            } else {
                FADE_OUT_FIRST_SUBTRACTION
            },
            waits_remaining: FADE_STEP_WAITS,
            complete: false,
        }
    }
    /// First step is immediate; later steps follow four resumed yields each.
    pub fn publish(&self, display: &mut Display) {
        for (active, &base) in display.active.iter_mut().zip(&display.base) {
            let color = base.wrapping_sub(self.subtraction);
            *active = if color & 0x80 != 0 { BLACK } else { color };
        }
        display.palette_pending = true;
    }
    pub fn resumed(&mut self) -> bool {
        const FADE_INTENSITY_STEP: u8 = 0x10;
        const FADE_IN_COMPLETE: u8 = 0xf0;
        const FADE_OUT_COMPLETE: u8 = 0x50;
        const STEP_WAITS: u8 = 4;
        if self.complete {
            return true;
        }
        self.waits_remaining -= 1;
        if self.waits_remaining != 0 {
            return false;
        }
        self.subtraction = if self.direction == FadeDirection::In {
            self.subtraction.wrapping_sub(FADE_INTENSITY_STEP)
        } else {
            self.subtraction.wrapping_add(FADE_INTENSITY_STEP)
        };
        self.complete = self.subtraction
            == if self.direction == FadeDirection::In {
                FADE_IN_COMPLETE
            } else {
                FADE_OUT_COMPLETE
            };
        self.waits_remaining = STEP_WAITS;
        true
    }
}
impl Display {
    /// Append one logical write and advance its call-local destination. Source
    /// parsers keep their own byte cursors/counts; all writes share this address
    /// wrapping rule, independent of when Video later commits the ordered list.
    fn append_write(&mut self, address: &mut u16, value: u8, stride: Stride) {
        const VIDEO_ADDRESS_MASK: u16 = 0x3fff;
        self.writes.push(VideoWrite {
            address: *address & VIDEO_ADDRESS_MASK,
            value,
        });
        *address = address.wrapping_add(stride.increment());
    }
    fn append_sequence(
        &mut self,
        mut address: u16,
        values: impl IntoIterator<Item = u8>,
        stride: Stride,
    ) {
        for value in values {
            self.append_write(&mut address, value, stride);
        }
    }
    pub fn disable(&mut self) {
        self.suspended = self.suspended.wrapping_add(1);
        self.mask = 0;
    }
    pub fn enable(&mut self) {
        const ENABLE_BACKGROUND_SPRITES: u8 = 0x1e;
        self.suspended = self.suspended.wrapping_sub(1);
        self.mask = ENABLE_BACKGROUND_SPRITES;
    }
    pub fn disable_nmi(&mut self) {
        const BASE_TABLE_SELECTION: u8 = 0x11;
        self.control &= BASE_TABLE_SELECTION;
    }
    pub fn enable_nmi(&mut self) {
        const NMI_ENABLE: u8 = 0x80;
        self.control |= NMI_ENABLE;
    }
    /// B01 primary-pending gate commits the canonical prepared queue. Zero leaves
    /// bytes and visible writes untouched; any nonzero source byte submits once.
    pub fn commit_primary(&mut self, stride: Stride) {
        if self.primary.pending != 0 {
            let bytes = self.primary.encoded.clone();
            self.flush(Queue::Primary, &bytes, stride);
        }
    }
    pub fn flush(&mut self, queue: Queue, encoded: &[u8], stride: Stride) {
        match queue {
            Queue::Primary => self.primary.pending = 0,
            Queue::Secondary => self.secondary_pending = false,
        }
        let mut cursor = 0_u8;
        loop {
            let high = encoded[usize::from(cursor)];
            if high & 0x80 != 0 {
                break;
            }
            let mut address = u16::from(high) << 8 | u16::from(encoded[usize::from(cursor) + 1]);
            let mut count = encoded[usize::from(cursor) + 2];
            loop {
                self.append_write(&mut address, encoded[usize::from(cursor) + 3], stride);
                cursor = cursor.wrapping_add(1);
                count = count.wrapping_sub(1);
                if count & 0x80 != 0 {
                    break;
                }
            }
            cursor = cursor.wrapping_add(3);
            if cursor == 0 {
                break;
            }
        }
    }
    pub fn script(&mut self, selector: u8, encoded: &[u8], status_tail: &[u8; 8], stride: Stride) {
        const SCRIPT_END: u8 = 0;
        const PAYLOAD_END: u8 = 0xfe;
        const VALUE_SCRIPT: u8 = 11;
        let mut cursor = 0;
        let mut address = 0_u16;
        while encoded[cursor] != SCRIPT_END {
            address = u16::from(encoded[cursor]) << 8 | u16::from(encoded[cursor + 1]);
            cursor += 2;
            while encoded[cursor] != PAYLOAD_END {
                self.append_write(&mut address, encoded[cursor], stride);
                cursor += 1;
            }
            cursor += 1;
        }
        if selector == VALUE_SCRIPT {
            self.append_sequence(address, status_tail.iter().copied(), stride);
        }
    }
    pub fn fill(&mut self, page: u8, tile: u8, attribute_or_pages: u8, stride: Stride) {
        const NAMETABLE_FIRST_PAGE: u8 = 0x20;
        const NAMETABLE_PAGES: usize = 4;
        const PAGE_BYTES: usize = 256;
        const ATTRIBUTE_BYTES: usize = 64;
        self.control &= !1;
        let pages = if page >= NAMETABLE_FIRST_PAGE {
            NAMETABLE_PAGES
        } else if attribute_or_pages == 0 {
            PAGE_BYTES
        } else {
            usize::from(attribute_or_pages)
        };
        let start = u16::from(page) << 8;
        self.append_sequence(start, std::iter::repeat_n(tile, pages * PAGE_BYTES), stride);
        if page >= NAMETABLE_FIRST_PAGE {
            let start = (u16::from(page.wrapping_add(3)) << 8) | 0xc0;
            self.append_sequence(
                start,
                std::iter::repeat_n(attribute_or_pages, ATTRIBUTE_BYTES),
                stride,
            );
        }
    }
    pub fn scene_palette(
        &mut self,
        colors: &[u8],
        background_end: u8,
        sprite_end: u8,
        option: usize,
        options: &[[u8; 4]],
    ) {
        const PALETTE_HALF: usize = PALETTE_COLORS / 2;
        for offset in 0..PALETTE_HALF {
            let destination = PALETTE_HALF - 1 - offset;
            self.base[destination] = colors[usize::from(background_end.wrapping_sub(offset as u8))];
            self.base[destination + PALETTE_HALF] =
                colors[usize::from((sprite_end & 0x7f).wrapping_sub(offset as u8))];
        }
        self.base[PALETTE_HALF] = self.base[0];
        if sprite_end & 0x80 != 0 {
            self.base[PALETTE_COLORS - 4..].copy_from_slice(&options[option]);
        }
        self.active = self.base;
    }
    /// LoadStagePalette replaces both background/sprite halves in both shadows.
    /// Loading does not publish colors or change the pending upload request;
    /// callers retain their title edits, fades and interrupt/scene boundaries.
    pub fn load_palette(
        &mut self,
        background: &[u8; PALETTE_COLORS / 2],
        sprites: &[u8; PALETTE_COLORS / 2],
    ) {
        const PALETTE_HALF: usize = PALETTE_COLORS / 2;
        self.base[..PALETTE_HALF].copy_from_slice(background);
        self.base[PALETTE_HALF..].copy_from_slice(sprites);
        self.active = self.base;
    }
    pub fn commit_palette(&mut self) {
        self.palette_pending = false;
        const PALETTE_ADDRESS: u16 = 0x3f00;
        self.writes.extend(
            self.active
                .iter()
                .enumerate()
                .map(|(offset, &value)| VideoWrite {
                    address: PALETTE_ADDRESS + offset as u16,
                    value,
                }),
        );
    }
}
/// Decoded metadata must include dynamically authored RAM frames alongside ROM frames.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentationCatalog {
    pub animations: Vec<Animation>,
    pub composites: Vec<Composite>,
    pub coordinate_variants: usize,
    pub coordinate_streams: Vec<Vec<u8>>,
    pub flipped_composites: Vec<Composite>,
    pub scripts: Vec<Vec<VideoWrite>>,
}
impl PresentationCatalog {
    pub fn from_original(rom: &[u8], ram_frame: &[u8]) -> Self {
        Self::try_from_original(rom, ram_frame).expect("valid original presentation records")
    }
    pub fn try_from_original(rom: &[u8], ram_frame: &[u8]) -> anyhow::Result<Self> {
        use anyhow::{Context as _, ensure};
        const BANK_PHYSICAL: usize = 0x14010;
        const CPU_WINDOW: usize = 0x8000;
        const ANIMATION_LOW: usize = 0xb1c7;
        const ANIMATION_HIGH: usize = 0xb257;
        const FRAME_LOW: usize = 0x9681;
        const FRAME_HIGH: usize = 0x9779;
        const COORDINATE_LOW: usize = 0xade1;
        const COORDINATE_HIGH: usize = 0xaec1;
        const SCRIPT_LOW: usize = 0xbafa;
        const SCRIPT_HIGH: usize = 0xbb06;
        const ANIMATION_COUNT: usize = 144;
        const FRAME_COUNT: usize = 248;
        const COORDINATE_COUNT: usize = 224;
        const SCRIPT_COUNT: usize = 12;
        const RUNTIME_FRAME_COUNT: usize = 4;
        const SCRIPT_RECORD_END: u8 = 0xfe;
        let pointer = |low: usize, high: usize, selector: usize| -> anyhow::Result<usize> {
            let low = usize::from(
                *rom.get(BANK_PHYSICAL + low - CPU_WINDOW + selector)
                    .context("presentation pointer low")?,
            );
            let high = usize::from(
                *rom.get(BANK_PHYSICAL + high - CPU_WINDOW + selector)
                    .context("presentation pointer high")?,
            );
            let address = low | high << 8;
            ensure!(
                (CPU_WINDOW..CPU_WINDOW + 0x4000).contains(&address),
                "presentation pointer leaves source bank"
            );
            Ok(BANK_PHYSICAL + address - CPU_WINDOW)
        };
        let animations = (0..ANIMATION_COUNT)
            .map(|selector| -> anyhow::Result<Animation> {
                let start = pointer(ANIMATION_LOW, ANIMATION_HIGH, selector)?;
                let count = usize::from(*rom.get(start).context("animation header")?);
                ensure!(
                    start + count + 3 <= BANK_PHYSICAL + 0x4000,
                    "animation crosses source bank"
                );
                let bytes = rom
                    .get(start..start + count + 3)
                    .context("animation record")?;
                let animation = decode_animation(bytes);
                ensure!(
                    animation
                        .frames
                        .iter()
                        .all(|&frame| usize::from(frame) < FRAME_COUNT),
                    "animation frame selector"
                );
                Ok(animation)
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        let mut records = Vec::with_capacity(FRAME_COUNT);
        let mut coordinate_lengths = vec![0; COORDINATE_COUNT];
        for selector in 0..FRAME_COUNT {
            let record = if selector < RUNTIME_FRAME_COUNT {
                ram_frame
            } else {
                rom.get(pointer(FRAME_LOW, FRAME_HIGH, selector)?..BANK_PHYSICAL + 0x4000)
                    .context("composite pointer")?
            };
            let count = usize::from(*record.first().context("composite header")?) + 1;
            let record = record
                .get(..2 + count * 2)
                .context("composite record")?
                .to_vec();
            for flip in [0_u8, 1] {
                let coordinate = usize::from(record[1].wrapping_add(flip));
                let length = coordinate_lengths
                    .get_mut(coordinate)
                    .context("composite coordinate selector")?;
                *length = (*length).max(count * 2);
            }
            records.push(record);
        }
        let coordinate_streams = (0..COORDINATE_COUNT)
            .map(|selector| {
                let start = pointer(COORDINATE_LOW, COORDINATE_HIGH, selector)?;
                ensure!(
                    start + coordinate_lengths[selector] <= BANK_PHYSICAL + 0x4000,
                    "coordinates cross source bank"
                );
                Ok(rom
                    .get(start..start + coordinate_lengths[selector])
                    .context("coordinate stream")?
                    .to_vec())
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        let composites = records
            .iter()
            .map(|record| decode_composite(record, &coordinate_streams, false))
            .collect();
        let flipped_composites = records
            .iter()
            .map(|record| decode_composite(record, &coordinate_streams, true))
            .collect();
        let scripts = (0..SCRIPT_COUNT)
            .map(|selector| -> anyhow::Result<Vec<VideoWrite>> {
                let mut cursor = pointer(SCRIPT_LOW, SCRIPT_HIGH, selector)?;
                let read = |at: usize| -> anyhow::Result<u8> {
                    ensure!(
                        at < BANK_PHYSICAL + 0x4000,
                        "screen script crosses source bank"
                    );
                    rom.get(at).copied().context("screen script record")
                };
                let mut writes = Vec::new();
                while read(cursor)? != 0 {
                    let mut address = u16::from(read(cursor)?) << 8 | u16::from(read(cursor + 1)?);
                    cursor += 2;
                    while read(cursor)? != SCRIPT_RECORD_END {
                        writes.push(VideoWrite {
                            address: address & 0x3fff,
                            value: read(cursor)?,
                        });
                        address = address.wrapping_add(1);
                        cursor += 1;
                    }
                    cursor += 1;
                }
                Ok(writes)
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(Self {
            animations,
            composites,
            coordinate_variants: COORDINATE_COUNT,
            coordinate_streams,
            flipped_composites,
            scripts,
        })
    }
}
#[cfg(test)]
mod contracts;
/// Values read by B01's ordered display effects. No second queue or scheduler.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RasterInputs {
    pub split_y: u8,
    pub ui_x: u8,
    pub ui_y: u8,
    pub camera_x: u16,
    pub playfield_y: u8,
    pub control: u8,
    pub mask: u8,
    pub split_address: u16,
}
#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RasterSnapshot {
    pub split_sprite: Sprite,
    pub ui_scroll: (u8, u8),
    pub ui_control: u8,
    pub ui_mask: u8,
    pub playfield_scroll: Option<(u8, u8)>,
    pub playfield_control: Option<u8>,
    pub split_address: Option<u16>,
    pub playfield_mask: u8,
}
impl RasterSnapshot {
    pub fn apply(&mut self, effects: &[crate::runtime::FrameEffect], inputs: RasterInputs) {
        use crate::runtime::{FrameEffect as E, UiControl};
        const SPLIT_SPRITE_TILE: u8 = 0x21;
        const SPLIT_SPRITE_ATTRIBUTES: u8 = 0x20;
        const SPLIT_SPRITE_X: u8 = 8;
        const GAMEPLAY_PATTERN_SELECTION: u8 = 0x18;
        const SPRITES_ONLY_MASK: u8 = 0x10;
        const PLAYFIELD_CONTROL: u8 = 0x90;
        for effect in effects {
            match effect {
                E::PrimeSplitSprite => {
                    self.split_sprite = Sprite {
                        y: inputs.split_y,
                        tile: SPLIT_SPRITE_TILE,
                        attributes: SPLIT_SPRITE_ATTRIBUTES,
                        x: SPLIT_SPRITE_X,
                    }
                }
                E::InstallUiScroll => {
                    // Each committed frame starts before the split; previous
                    // optional split requests must not leak into a later frame.
                    self.playfield_scroll = None;
                    self.playfield_control = None;
                    self.split_address = None;
                    self.ui_scroll = (inputs.ui_x, inputs.ui_y);
                    self.ui_mask = inputs.mask;
                    self.playfield_mask = inputs.mask;
                }
                E::InstallUiControl(control) => {
                    self.ui_control = inputs.control
                        | if *control == UiControl::ForceGameplayPatterns {
                            GAMEPLAY_PATTERN_SELECTION
                        } else {
                            0
                        }
                }
                E::InstallSplitAddress => self.split_address = Some(inputs.split_address),
                E::SpriteOnlyMask => self.playfield_mask = SPRITES_ONLY_MASK,
                E::InstallPlayfieldScroll => {
                    self.playfield_scroll = Some((inputs.camera_x as u8, inputs.playfield_y));
                    self.playfield_control =
                        Some(PLAYFIELD_CONTROL | ((inputs.camera_x >> 8) as u8 & 1));
                }
                _ => (),
            }
        }
    }
}
impl Composition {
    /// Cleanup precedes culling. Hidden render FF skips even projection/animation;
    /// inactive slots release stale identity. Returns eligible slots in source order.
    pub fn eligible(
        &mut self,
        actors: &mut [ActorVisual; ACTOR_SLOTS],
        order: &[usize],
    ) -> Vec<usize> {
        order
            .iter()
            .copied()
            .filter(|&slot| {
                let actor = &mut actors[slot];
                if actor.state & ACTIVE == 0 {
                    actor.render = 0;
                    actor.spawn_identity = None;
                    false
                } else {
                    actor.render != HIDDEN_RENDER
                }
            })
            .collect()
    }
}
/// Runtime frame bytes use the same header as ROM records. Horizontal flip
/// chooses the adjacent coordinate stream; it does not negate a base offset.
pub fn decode_composite(record: &[u8], coordinate_streams: &[Vec<u8>], flipped: bool) -> Composite {
    let count = usize::from(record[0]) + 1;
    let coordinate = record[1].wrapping_add(u8::from(flipped));
    let offsets = &coordinate_streams[usize::from(coordinate)];
    let pieces = (0..count)
        .map(|index| Piece {
            tile: record[2 + index * 2],
            attributes: record[3 + index * 2],
            dy: offsets[index * 2] as i8,
            dx: offsets[index * 2 + 1] as i8,
        })
        .collect();
    Composite { pieces }
}
/// Raw last-index counts are retained separately from ordinary length types.
pub fn decode_animation(record: &[u8]) -> Animation {
    Animation {
        last_frame: record[0],
        last_tick: record[1],
        frames: record[2..3 + usize::from(record[0])].to_vec(),
    }
}
impl Display {
    pub fn begin_fade(
        &mut self,
        direction: FadeDirection,
        composition: &mut Composition,
        gameplay: bool,
    ) -> Fade {
        if direction == FadeDirection::In {
            composition.hide(gameplay);
        }
        Fade::start(direction)
    }
    pub fn apply_chr(&mut self, prepared: &crate::assets::formats::ChrPreparation) {
        // Temporary hardware control does not overwrite the foreground shadow.
        for (destination, bytes) in &prepared.copies {
            for (offset, &value) in bytes.iter().enumerate() {
                self.writes.push(VideoWrite {
                    address: destination
                        .wrapping_add((offset as u16).wrapping_mul(prepared.stride))
                        & 0x3fff,
                    value,
                });
            }
        }
    }
}
/// Cartridge vertical mirroring, CHR RAM and palette aliasing. This is logical
/// display data, never an emulated CPU bus. Independent toolkit pixel helpers
/// consume these snapshots; foreground prepares changes and commit publishes them.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VideoMemory {
    pub chr: Vec<u8>,
    pub nametables: Vec<u8>,
    #[serde(with = "crate::snapshot::array")]
    pub palette: [u8; PALETTE_COLORS],
}
impl VideoMemory {
    pub fn blank() -> Self {
        Self {
            chr: vec![0; 0x2000],
            nametables: vec![0; 0x800],
            palette: [0; PALETTE_COLORS],
        }
    }
    pub fn write(&mut self, write: VideoWrite) {
        let address = usize::from(write.address & 0x3fff);
        match address {
            0..0x2000 => self.chr[address] = write.value,
            0x2000..0x3f00 => self.nametables[(address - 0x2000) % 0x800] = write.value,
            _ => self.palette[palette_index(address)] = write.value,
        }
    }
    pub fn read(&self, address: u16) -> u8 {
        let address = usize::from(address & 0x3fff);
        match address {
            0..0x2000 => self.chr[address],
            0x2000..0x3f00 => self.nametables[(address - 0x2000) % 0x800],
            _ => self.palette[palette_index(address)],
        }
    }
}

/// The encoded secondary queue and four runtime composite selectors share these
/// same bytes. This is a source-format buffer, not a CPU-addressable RAM image.
/// No independent runtime-frame copy or invented status author is permitted.
#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PreparedSecondary {
    pub encoded: Vec<u8>,
}
pub const RUNTIME_COMPOSITE_QUEUE_OFFSET: usize = 0x80;
pub const SECONDARY_PREFIX_BYTES: usize = 0xc0;
pub const CHANGED_RECORD_BYTES: u8 = 14;
impl PreparedSecondary {
    /// B01 reset clears the queue including the alias tail before any writer.
    pub fn source_zeroed() -> Self {
        const SOURCE_SECONDARY_CAPACITY: usize = 256 + CHANGED_RECORD_BYTES as usize;
        Self {
            encoded: vec![0; SOURCE_SECONDARY_CAPACITY],
        }
    }
    /// B04 returns five logical writes; preserve the original two counted tile
    /// pairs plus one attribute encoding and overlapping terminator. Cursor is
    /// incremented by fourteen modulo256 after writing; no duplicate queue.
    pub fn append_changed(&mut self, cursor: &mut u8, upload: &crate::world::TerrainUpdate) {
        let record = terrain::changed_record(upload);
        let at = usize::from(*cursor);
        let end = at + record.len();
        if self.encoded.len() < end {
            self.encoded.resize(end, 0);
        }
        self.encoded[at..end].copy_from_slice(&record);
        *cursor = cursor.wrapping_add(CHANGED_RECORD_BYTES);
    }
    /// In the composed game, the source address0500 belongs to the world
    /// attribute cache. Keep only the disjoint queue prefix here; overflow
    /// writes immediately change that canonical cache before the next actor.
    pub fn source_shared() -> Self {
        Self {
            encoded: vec![0; SECONDARY_PREFIX_BYTES],
        }
    }
    pub fn append_changed_shared(
        &mut self,
        cursor: &mut u8,
        upload: &crate::world::TerrainUpdate,
        attributes: &mut [u8; crate::world::CACHE_CELLS],
    ) {
        assert_eq!(self.encoded.len(), SECONDARY_PREFIX_BYTES);
        for (field, value) in terrain::changed_record(upload).into_iter().enumerate() {
            let offset = usize::from(*cursor) + field;
            if offset < SECONDARY_PREFIX_BYTES {
                self.encoded[offset] = value;
            } else {
                attributes[offset - SECONDARY_PREFIX_BYTES] = value;
            }
        }
        *cursor = cursor.wrapping_add(CHANGED_RECORD_BYTES);
    }
    /// A transient consumer view joins the disjoint canonical owners. It is
    /// never retained as another queue or runtime composite record.
    pub fn shared_bytes(&self, attributes: &[u8; crate::world::CACHE_CELLS]) -> Vec<u8> {
        assert_eq!(self.encoded.len(), SECONDARY_PREFIX_BYTES);
        self.encoded.iter().chain(attributes).copied().collect()
    }
    /// Selectors0..3 borrow the identical backing tail. Reading is immutable and
    /// cannot commit an upload, clear bytes, advance animation or allocate data.
    pub fn runtime_composite(&self, selector: u8) -> &[u8] {
        assert!(selector < 4, "only four source runtime composite aliases");
        &self.encoded[RUNTIME_COMPOSITE_QUEUE_OFFSET..]
    }
}

// Sprite universal-color entries mirror their corresponding background entries.
fn palette_index(address: usize) -> usize {
    let index = address & (PALETTE_COLORS - 1);
    if index >= 16 && index.is_multiple_of(4) {
        index - 16
    } else {
        index
    }
}

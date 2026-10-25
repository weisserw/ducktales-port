//! Immutable decoding boundaries for CHR/background formats. Source derivation is
//! in trace.md. B03 owns committed display state.
use crate::identifiers::StageId;
use anyhow::{Context, Result, ensure};

pub const INES_HEADER_BYTES: usize = 16;
pub const PRG_BANK_BYTES: usize = 0x4000;
pub const SOURCE_PAGE_BYTES: usize = 0x100;
pub const BACKGROUND_CHUNK_BYTES: usize = 0x40;
pub const ORIGINAL_CHR_LISTS: usize = 11;
pub const ORIGINAL_BACKGROUND_LISTS: usize = 32;
pub const CHR_POINTERS_OFFSET: usize = 0x1c368;
pub const CHR_RECORDS_OFFSET: usize = 0x1c37e;
pub const CHR_RECORDS_END: usize = 0x1c43c;
pub const BACKGROUND_CONFIG_OFFSET: usize = 0x1c518;
pub const BACKGROUND_CONFIG_END: usize = 0x1c72d;

/// An already validated upload slice; CPU bank context is resolved at load time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UploadSpan {
    pub bank: u8,
    pub source_offset: usize,
    pub source_pages: u8,
    pub destination: u16,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UploadList {
    pub records: Vec<UploadSpan>,
    pub terminator_offset: usize,
    pub terminator: u8,
}
/// Decode exactly through the first negative tag. Original list aliases are legal.
/// Invalid selectors, truncated records and out-of-cartridge reads are errors.
// All table addresses here are physical, including the iNES header. Pointer
// values themselves use the fixed bank's CPU window and are decoded only here.
const FIXED_CPU_BASE: usize = 0xc000;
const FIXED_PHYSICAL_BASE: usize = INES_HEADER_BYTES + 7 * PRG_BANK_BYTES;
const SWITCH_CPU_BASE: usize = 0x8000;
const SWITCH_CPU_END: usize = 0xc000;
const BACKGROUND_LIST_LOW: usize = 0x1c573;
const BACKGROUND_LIST_HIGH: usize = BACKGROUND_LIST_LOW + ORIGINAL_BACKGROUND_LISTS;
fn fixed_pointer(rom: &[u8], low: usize, high: usize) -> Result<usize> {
    let pointer = usize::from(*rom.get(low).context("missing pointer low byte")?)
        | (usize::from(*rom.get(high).context("missing pointer high byte")?) << 8);
    ensure!(pointer >= FIXED_CPU_BASE, "pointer outside fixed bank");
    Ok(FIXED_PHYSICAL_BASE + pointer - FIXED_CPU_BASE)
}
pub fn original_chr_list(rom: &[u8], selector: usize) -> Result<UploadList> {
    ensure!(selector < ORIGINAL_CHR_LISTS, "invalid CHR selector");
    let start = fixed_pointer(
        rom,
        CHR_POINTERS_OFFSET + selector,
        CHR_POINTERS_OFFSET + ORIGINAL_CHR_LISTS + selector,
    )?;
    ensure!(
        (CHR_RECORDS_OFFSET..CHR_RECORDS_END).contains(&start),
        "CHR list origin"
    );
    decode_upload_list(rom, start, CHR_RECORDS_END)
}
pub fn decode_upload_list(
    rom: &[u8],
    descriptor_offset: usize,
    descriptor_end: usize,
) -> Result<UploadList> {
    ensure!(
        descriptor_offset < descriptor_end && descriptor_end <= rom.len(),
        "descriptor bounds"
    );
    let mut cursor = descriptor_offset;
    let mut records = Vec::new();
    while cursor < descriptor_end {
        let tag = rom[cursor];
        if tag & 0x80 != 0 {
            return Ok(UploadList {
                records,
                terminator_offset: cursor,
                terminator: tag,
            });
        }
        ensure!(cursor + 3 <= descriptor_end, "truncated upload record");
        let bank = tag >> 4;
        let source_pages = (tag & 0x0f) + 1;
        let source_cpu = usize::from(rom[cursor + 1]) * SOURCE_PAGE_BYTES;
        let length = usize::from(source_pages) * SOURCE_PAGE_BYTES;
        ensure!(
            bank < 7 && source_cpu >= SWITCH_CPU_BASE && source_cpu + length <= SWITCH_CPU_END,
            "upload leaves switch bank"
        );
        let source_offset =
            INES_HEADER_BYTES + usize::from(bank) * PRG_BANK_BYTES + source_cpu - SWITCH_CPU_BASE;
        ensure!(
            source_offset + length <= rom.len(),
            "upload leaves cartridge"
        );
        records.push(UploadSpan {
            bank,
            source_offset,
            source_pages,
            destination: u16::from(rom[cursor + 2]) << 8,
        });
        cursor += 3;
    }
    anyhow::bail!("upload list missing negative terminator")
}
pub fn original_background_list(rom: &[u8], selector: usize) -> Result<UploadList> {
    ensure!(
        selector < ORIGINAL_BACKGROUND_LISTS,
        "invalid background selector"
    );
    let start = fixed_pointer(
        rom,
        BACKGROUND_LIST_LOW + selector,
        BACKGROUND_LIST_HIGH + selector,
    )?;
    ensure!(
        (BACKGROUND_CONFIG_OFFSET..BACKGROUND_CONFIG_END).contains(&start),
        "background list origin"
    );
    decode_upload_list(rom, start, BACKGROUND_CONFIG_END)
}
/// Returns hardware control written on entry and ordered copies; shadow is input
/// only. AND FE clears bit0, while bit2 continues to choose the hardware stride.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChrPreparation {
    pub control: u8,
    pub stride: u16,
    pub copies: Vec<(u16, Vec<u8>)>,
    pub selected_bank: Option<u8>,
}
pub fn prepare_chr(rom: &[u8], list: &UploadList, control_shadow: u8) -> Result<ChrPreparation> {
    const NAMETABLE_LOW_BIT: u8 = 0x01;
    const VERTICAL_INCREMENT_BIT: u8 = 0x04;
    const VERTICAL_INCREMENT: u16 = 32;
    let control = control_shadow & !NAMETABLE_LOW_BIT;
    let copies = list
        .records
        .iter()
        .map(|span| {
            let end = span
                .source_offset
                .checked_add(usize::from(span.source_pages) * SOURCE_PAGE_BYTES)
                .context("upload size overflow")?;
            Ok((
                span.destination,
                rom.get(span.source_offset..end)
                    .context("CHR source bounds")?
                    .to_vec(),
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(ChrPreparation {
        control,
        stride: if control & VERTICAL_INCREMENT_BIT != 0 {
            VERTICAL_INCREMENT
        } else {
            1
        },
        copies,
        selected_bank: list.records.last().map(|span| span.bank),
    })
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackgroundContext {
    pub stage: StageId,
    pub room_row: u8,
    pub camera_page: u8,
    pub display_suspend_depth: u8,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackgroundSelection {
    pub list: usize,
    pub palette: [u8; 4],
}
pub fn select_background(rom: &[u8], context: BackgroundContext) -> Result<BackgroundSelection> {
    const STAGE_ROWS: [u8; 9] = [7, 8, 6, 6, 6, 7, 1, 1, 1];
    const ROOM_SELECTORS: usize = BACKGROUND_CONFIG_OFFSET + STAGE_ROWS.len();
    const PALETTE_SELECTORS: usize = ROOM_SELECTORS + 41;
    const PALETTE_QUADS: usize = 0x1c6e9;
    const MOON_SELECTOR: u8 = 4;
    ensure!(
        usize::from(context.stage.0) < STAGE_ROWS.len(),
        "background stage has no configuration"
    );
    ensure!(
        context.room_row < STAGE_ROWS[usize::from(context.stage.0)],
        "background room row"
    );
    let start = *rom
        .get(BACKGROUND_CONFIG_OFFSET + usize::from(context.stage.0))
        .context("background stage offset")?;
    let row = usize::from(start) + usize::from(context.room_row);
    let tag = *rom
        .get(ROOM_SELECTORS + row)
        .context("background row selector")?;
    let variant_threshold = if context.stage.0 == MOON_SELECTOR {
        3
    } else {
        4
    };
    let list = usize::from(tag & 0x7f)
        + usize::from(tag & 0x80 != 0 && context.camera_page >= variant_threshold);
    ensure!(
        list < ORIGINAL_BACKGROUND_LISTS,
        "background variant bounds"
    );
    let palette_index = usize::from(
        *rom.get(PALETTE_SELECTORS + row)
            .context("palette selector")?,
    );
    let palette_start = PALETTE_QUADS + palette_index * 4;
    let palette = rom
        .get(palette_start..palette_start + 4)
        .context("palette quad")?
        .try_into()?;
    Ok(BackgroundSelection { list, palette })
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UploadCommit {
    AtInterrupt,
    Immediate,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BackgroundEffect {
    Chunk {
        destination: u16,
        bytes: Vec<u8>,
        count_minus_one: u8,
        commit: UploadCommit,
        selected_bank: u8,
        queue_terminator: u8,
    },
    PaletteTail {
        base: [u8; 4],
        active: [u8; 4],
        terminal_tag: u8,
    },
    None,
}
/// Foreground upload continuation, not another PPU queue owner. B03 consumes its
/// request immediately or at B01's interrupt boundary as specified by the effect.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BackgroundUpload {
    pub completed: bool,
    pub record: usize,
    pub chunk_in_record: usize,
}
impl BackgroundUpload {
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
    pub fn poll(
        &mut self,
        rom: &[u8],
        list: &UploadList,
        palette: [u8; 4],
        suspend_depth: u8,
    ) -> Result<BackgroundEffect> {
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

/// Resolve a validated physical recipe through named raw overrides, combining
/// adjacent maintained fragments when one original consumer spans their boundary.
/// Never read instruction bytes to fill holes or silently fall back to the ROM.
pub fn resolve_upload_bytes(raw: &super::Assets, span: &UploadSpan) -> Result<Vec<u8>> {
    let length = usize::from(span.source_pages) * SOURCE_PAGE_BYTES;
    let end = span
        .source_offset
        .checked_add(length)
        .context("upload range overflow")?;
    let bank_start = INES_HEADER_BYTES + usize::from(span.bank) * PRG_BANK_BYTES;
    ensure!(
        span.bank < 7 && span.source_offset >= bank_start && end <= bank_start + PRG_BANK_BYTES,
        "upload bank bounds"
    );
    resolve_named_range(raw, span.source_offset, length)
}
/// Resolve physical immutable data through original named records, even when a
/// replacement has File origin. Origins never become a second source of bytes.
pub fn resolve_named_range(raw: &super::Assets, start: usize, length: usize) -> Result<Vec<u8>> {
    let end = start.checked_add(length).context("named range overflow")?;
    let recipes = crate::recipes::original_assets();
    let mut cursor = start;
    let mut output = Vec::with_capacity(length);
    while cursor < end {
        let recipe = recipes
            .iter()
            .find(|recipe| cursor >= recipe.offset && cursor < recipe.offset + recipe.length)
            .context("range has no named data consumer recipe")?;
        let asset = raw
            .source
            .assets
            .iter()
            .find(|asset| asset.id == recipe.id)
            .context("missing named data fragment")?;
        ensure!(
            asset.bytes.len() == recipe.length,
            "named data fragment length"
        );
        let fragment_end = end.min(recipe.offset + recipe.length);
        output
            .extend_from_slice(&asset.bytes[cursor - recipe.offset..fragment_end - recipe.offset]);
        cursor = fragment_end;
    }
    Ok(output)
}
/// B04 owns the block/plane decoding. B02 supplies all bytes of the source set,
/// including slices whose provisional source label only says AssetData.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerrainPayload {
    pub block_records: Vec<u8>,
    pub tile_planes: [Vec<u8>; 4],
    pub packed_attributes: Vec<u8>,
}
pub fn terrain_payload(raw: &super::Assets, bank: u8) -> Result<TerrainPayload> {
    const TERRAIN_BANKS: u8 = 4;
    const BLOCK_RECORDS_OFFSET: usize = 0x3300;
    const BLOCK_RECORDS_BYTES: usize = 0x800;
    const PLANE_COUNT: usize = 4;
    ensure!(bank < TERRAIN_BANKS, "terrain bank");
    let start = INES_HEADER_BYTES + usize::from(bank) * PRG_BANK_BYTES + BLOCK_RECORDS_OFFSET;
    let block_records = resolve_named_range(raw, start, BLOCK_RECORDS_BYTES)?;
    let mut tile_planes = std::array::from_fn(|_| Vec::new());
    for (index, plane) in tile_planes.iter_mut().enumerate() {
        *plane = resolve_named_range(
            raw,
            start + BLOCK_RECORDS_BYTES + index * SOURCE_PAGE_BYTES,
            SOURCE_PAGE_BYTES,
        )?;
    }
    let packed_attributes = resolve_named_range(
        raw,
        start + BLOCK_RECORDS_BYTES + PLANE_COUNT * SOURCE_PAGE_BYTES,
        SOURCE_PAGE_BYTES,
    )?;
    Ok(TerrainPayload {
        block_records,
        tile_planes,
        packed_attributes,
    })
}

/// Raw source selectors read adjacent pointer-plane bytes. Unlike an authored
/// replacement descriptor, this can point into the currently selected immutable
/// graphics bank. Interpret the original data list at startup, retaining bank
/// selection between records; no runtime CPU/bus is introduced.
pub fn prepare_raw_chr(
    rom: &[u8],
    selector: u8,
    entry_bank: u8,
    control: u8,
) -> Result<ChrPreparation> {
    let read = |bank: u8, address: u16| -> Result<u8> {
        let address = usize::from(address);
        ensure!(
            address >= SWITCH_CPU_BASE,
            "raw CHR list requires a non-ROM consumer"
        );
        let physical = if address >= FIXED_CPU_BASE {
            FIXED_PHYSICAL_BASE + address - FIXED_CPU_BASE
        } else {
            INES_HEADER_BYTES + usize::from(bank & 7) * PRG_BANK_BYTES + address - SWITCH_CPU_BASE
        };
        rom.get(physical).copied().context("raw CHR data bounds")
    };
    let index = usize::from(selector);
    let low = *rom
        .get(CHR_POINTERS_OFFSET + index)
        .context("raw CHR pointer low")?;
    let high = *rom
        .get(CHR_POINTERS_OFFSET + ORIGINAL_CHR_LISTS + index)
        .context("raw CHR pointer high")?;
    let pointer = u16::from_le_bytes([low, high]);
    let mut bank = entry_bank;
    let mut cursor = 0_u8;
    let mut visited = std::collections::BTreeSet::new();
    let mut copies = Vec::new();
    let mut selected_bank = None;
    loop {
        ensure!(
            visited.insert((bank, cursor)),
            "raw CHR list repeats without a terminator"
        );
        let tag = read(bank, pointer.wrapping_add(u16::from(cursor)))?;
        if tag & 0x80 != 0 {
            break;
        }
        let source = read(
            bank,
            pointer.wrapping_add(u16::from(cursor.wrapping_add(1))),
        )?;
        let destination = read(
            bank,
            pointer.wrapping_add(u16::from(cursor.wrapping_add(2))),
        )?;
        cursor = cursor.wrapping_add(3);
        bank = tag >> 4;
        selected_bank = Some(bank);
        let count = (usize::from(tag & 15) + 1) * SOURCE_PAGE_BYTES;
        let bytes = (0..count)
            .map(|offset| read(bank, (u16::from(source) << 8).wrapping_add(offset as u16)))
            .collect::<Result<Vec<_>>>()?;
        copies.push((u16::from(destination) << 8, bytes));
    }
    Ok(ChrPreparation {
        control: control & !1,
        stride: if control & 4 != 0 { 32 } else { 1 },
        copies,
        selected_bank,
    })
}

/// Contiguous overlapping parameters consumed by raw-stage background entry.
/// Byte addition and palette-index multiplication deliberately wrap like source.
#[derive(Clone)]
pub struct RawBackgroundCatalog {
    stage_offsets: [u8; 256],
    room_tags: [u8; 256],
    room_palettes: [u8; 256],
    palette_bytes: [u8; 256],
}
impl RawBackgroundCatalog {
    pub fn original(rom: &[u8]) -> Result<Self> {
        const ROOM_TAGS: usize = BACKGROUND_CONFIG_OFFSET + 9;
        const ROOM_PALETTES: usize = ROOM_TAGS + 41;
        const PALETTES: usize = 0x1c6e9;
        let plane = |start| -> Result<[u8; 256]> {
            Ok(rom
                .get(start..start + 256)
                .context("raw background parameter extent")?
                .try_into()?)
        };
        Ok(Self {
            stage_offsets: plane(BACKGROUND_CONFIG_OFFSET)?,
            room_tags: plane(ROOM_TAGS)?,
            room_palettes: plane(ROOM_PALETTES)?,
            palette_bytes: plane(PALETTES)?,
        })
    }
    pub fn select(&self, context: BackgroundContext) -> BackgroundSelection {
        let row = self.stage_offsets[usize::from(context.stage.0)].wrapping_add(context.room_row);
        let tag = self.room_tags[usize::from(row)];
        let threshold = if context.stage.0 == 4 { 3 } else { 4 };
        let list = (tag & 0x7f).wrapping_add(u8::from(
            tag & 0x80 != 0 && context.camera_page >= threshold,
        ));
        let palette = self.room_palettes[usize::from(row)].wrapping_mul(4);
        BackgroundSelection {
            list: usize::from(list),
            palette: std::array::from_fn(|offset| {
                self.palette_bytes[usize::from(palette.wrapping_add(offset as u8))]
            }),
        }
    }
}

/// Immutable decoded source copies. The foreground upload continuation retains
/// cursors only; prepared data and the canonical PPU queue have separate owners.
#[derive(Clone)]
pub struct PreparedBackgroundSpan {
    pub bank: u8,
    pub destination: u16,
    pub bytes: Vec<u8>,
}
#[derive(Clone)]
pub struct PreparedBackground {
    pub spans: Vec<PreparedBackgroundSpan>,
    pub terminator: u8,
}
impl BackgroundUpload {
    pub fn poll_prepared(
        &mut self,
        prepared: &PreparedBackground,
        palette: [u8; 4],
        suspend_depth: u8,
    ) -> Result<BackgroundEffect> {
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

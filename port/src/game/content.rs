//! Startup decodes immutable source resources once. Runtime rules consume these
//! catalogs; no cartridge image, banked reads or executable source remain here.
use crate::{
    actors, assets, audio, dialogue, finale, player, presentation, progression, scenes, selection,
    status, world,
};
use anyhow::{Context, Result, ensure};
use romatlas_port::assets::{MemoryAssets, Rom};

const CARTRIDGE_BYTES: usize = 0x20010;

#[derive(Clone)]
pub struct Content {
    pub additions: std::sync::Arc<super::levels::Registry>,
    pub active_file: Option<std::sync::Arc<super::levels::AddedLevel>>,
    pub levels: assets::catalog::DecodedAssets,
    pub world: world::WorldCatalog,
    pub changed_terrain_selectors: [u8; 256],
    pub spawns: actors::SpawnCatalog,
    pub spawn_planes: std::collections::BTreeMap<u8, actors::SpawnPlanes>,
    pub player: player::PlayerData,
    pub geometry: crate::collision::Geometry,
    pub enemies: actors::enemies::Catalog,
    pub objects: actors::objects::Catalog,
    pub platforms: actors::platforms::Catalog,
    pub bosses: actors::bosses::Catalog,
    pub selection: selection::Catalog,
    pub raw_lands: selection::RawLandCatalog,
    pub progression: progression::Catalog,
    pub raw_stage_presets: Vec<progression::StagePreset>,
    pub raw_layouts: Vec<std::result::Result<world::Layout, String>>,
    pub raw_rooms: Vec<world::Room>,
    pub dialogue: dialogue::Catalog,
    pub scenes: scenes::Catalog,
    pub scene_palette_colors: Vec<u8>,
    pub scene_option_palettes: [[u8; 4]; 3],
    pub finale: finale::Catalog,
    pub audio: audio::Catalog,
    pub presentation: presentation::PresentationCatalog,
    pub runtime_coordinates: Vec<Vec<u8>>,
    pub status: status::StatusCatalog,
    pub status_template: Vec<u8>,
    pub stage_ui_template: Vec<u8>,
    pub health_template: Vec<u8>,
    pub title_overlay: Vec<u8>,
    pub timer_overlay: Vec<u8>,
    pub chr: Vec<assets::formats::ChrPreparation>,
    pub raw_chr: Vec<std::result::Result<assets::formats::ChrPreparation, String>>,
    pub background_parameters: assets::formats::RawBackgroundCatalog,
    pub backgrounds: Vec<assets::formats::PreparedBackground>,
}
impl Content {
    pub fn load(rom: &Rom, raw: &assets::Assets, overrides: &MemoryAssets) -> Result<Self> {
        // Some established consumers read adjacent instruction bytes as data.
        // Start from the validated complete cartridge only at this loader
        // boundary, then patch every named resource. Zero-filled recipe holes
        // would change those original consumers. The image is dropped below.
        let mut image = rom.read(0, CARTRIDGE_BYTES)?.to_vec();
        for recipe in crate::recipes::original_assets() {
            let asset = raw
                .source
                .get(&recipe.id)
                .context("missing original source record")?;
            ensure!(
                asset.bytes.len() == recipe.length,
                "source replacement length"
            );
            image[recipe.offset..recipe.offset + recipe.length].copy_from_slice(&asset.bytes);
        }
        let mut catalog = assets::catalog::original_catalog(raw)?;
        if let Some(document) = overrides.files.get(assets::catalog::LEVEL_MANIFEST) {
            let additions = assets::catalog::load_level_manifest(&raw.source, document)?;
            catalog.levels.extend(additions.levels);
            catalog.resources.extend(additions.resources);
        }
        let levels = assets::catalog::DecodedAssets::resolve(raw, catalog)?;
        let chr = (0..assets::formats::ORIGINAL_CHR_LISTS)
            .map(|selector| {
                let list = assets::formats::original_chr_list(&image, selector)?;
                assets::formats::prepare_chr(&image, &list, crate::runtime::RESET_CONTROL)
            })
            .collect::<Result<Vec<_>>>()?;
        let backgrounds = (0..assets::formats::ORIGINAL_BACKGROUND_LISTS)
            .map(|selector| {
                let list = assets::formats::original_background_list(&image, selector)?;
                let spans = list
                    .records
                    .into_iter()
                    .map(|record| {
                        let length =
                            usize::from(record.source_pages) * assets::formats::SOURCE_PAGE_BYTES;
                        Ok(assets::formats::PreparedBackgroundSpan {
                            bank: record.bank,
                            destination: record.destination,
                            bytes: image[record.source_offset..record.source_offset + length]
                                .to_vec(),
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(assets::formats::PreparedBackground {
                    spans,
                    terminator: list.terminator,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        // Runtime composites can request more coordinate pairs than any static
        // frame. Their byte count permits256 pieces, so preserve that actual
        // immutable data-consumer extent instead of truncating to static use.
        const COORDINATE_LOW: usize = 0x16df1;
        const COORDINATE_HIGH: usize = 0x16ed1;
        const GRAPHICS_SWITCH_PHYSICAL: usize = 0x14010;
        const SWITCH_CPU_BASE: usize = 0x8000;
        const FIXED_CPU_BASE: usize = 0xc000;
        const FIXED_PHYSICAL: usize = 0x1c010;
        let runtime_coordinates = (0..224)
            .map(|selector| -> Result<Vec<u8>> {
                let pointer = usize::from(image[COORDINATE_LOW + selector])
                    | usize::from(image[COORDINATE_HIGH + selector]) << 8;
                ensure!(pointer >= SWITCH_CPU_BASE, "runtime coordinate pointer");
                (0..512)
                    .map(|offset| {
                        let address = pointer + offset;
                        let physical = if address < FIXED_CPU_BASE {
                            GRAPHICS_SWITCH_PHYSICAL + address - SWITCH_CPU_BASE
                        } else {
                            FIXED_PHYSICAL + address - FIXED_CPU_BASE
                        };
                        image
                            .get(physical)
                            .copied()
                            .context("runtime coordinate consumer extent")
                    })
                    .collect()
            })
            .collect::<Result<Vec<_>>>()?;
        let zeroed_secondary = presentation::PreparedSecondary::source_zeroed();
        let additions = super::levels::Registry::decode(&levels, raw)?;
        Ok(Self {
            additions: std::sync::Arc::new(additions),
            active_file: None,
            levels,
            world: world::WorldCatalog::original(&image)?,
            // F600 indexes the immutable fixed-bank F653 plane with a byte.
            changed_terrain_selectors: image[0x1f663..0x1f763].try_into()?,
            spawns: actors::SpawnCatalog::original(&image)?,
            spawn_planes: (0..9)
                .chain(std::iter::once(21))
                .map(|selector| {
                    Ok((
                        selector,
                        actors::SpawnPlanes::original(
                            &image,
                            crate::identifiers::StageId(selector),
                        )?,
                    ))
                })
                .collect::<Result<_>>()?,
            player: player::PlayerData::original(&image)?,
            geometry: crate::collision::Geometry::original(&image)?,
            enemies: actors::enemies::Catalog::original(&image)?,
            objects: actors::objects::Catalog::original(&image)?,
            platforms: actors::platforms::Catalog::original(&image)?,
            bosses: actors::bosses::Catalog::original(&image)?,
            selection: selection::Catalog::decode(
                &image[selection::TITLE_PARAMETER_OFFSET
                    ..selection::TITLE_PARAMETER_OFFSET + selection::TITLE_PARAMETER_BYTES],
                &image[selection::LAND_PARAMETER_OFFSET
                    ..selection::LAND_PARAMETER_OFFSET + selection::LAND_PARAMETER_BYTES],
            )?,
            raw_lands: selection::RawLandCatalog::original(&image)?,
            progression: progression::Catalog::original(&image)?,
            raw_stage_presets: (0..=u8::MAX)
                .map(|stage| {
                    progression::raw_stage_preset(&image, crate::identifiers::StageId(stage))
                })
                .collect::<Result<Vec<_>>>()?,
            // Known raw5/6/7/21 pointers are fixed-bank consumers. Other byte
            // selectors retain their actual unsupported pointer classification;
            // they must not silently become an ordinary authored layout.
            raw_layouts: (0..=u8::MAX)
                .map(|stage| world::raw_layout(&image, stage).map_err(|error| format!("{error:#}")))
                .collect(),
            raw_rooms: world::raw_rooms(&image)?,
            dialogue: dialogue::Catalog::original(&image)?,
            scenes: scenes::Catalog::original(&image)?,
            // The complete named ScenePaletteAndParameterData recipe backs
            // both tables; offsets are physical fixed-bank source identities.
            scene_palette_colors: image[0x1e153..0x1e1b3].to_vec(),
            scene_option_palettes: std::array::from_fn(|option| {
                image[0x1e1b3 + option * 4..0x1e1b7 + option * 4]
                    .try_into()
                    .expect("scene option palette extent")
            }),
            finale: finale::Catalog::original(&image)?,
            audio: audio::Catalog::from_assets(raw)?,
            presentation: presentation::PresentationCatalog::try_from_original(
                &image,
                zeroed_secondary.runtime_composite(0),
            )?,
            runtime_coordinates,
            status: status::StatusCatalog::decode_original(&image)?,
            status_template: image[0x1dae4..0x1daf0].to_vec(),
            stage_ui_template: image[0x1daf0..0x1dafd].to_vec(),
            health_template: image[0x1dafd..0x1db12].to_vec(),
            title_overlay: image[0x1db12..0x1db1f].to_vec(),
            timer_overlay: image[0x1db1f..0x1db3c].to_vec(),
            raw_chr: (0..=u8::MAX)
                .map(|selector| {
                    assets::formats::prepare_raw_chr(
                        &image,
                        selector,
                        5,
                        crate::runtime::RESET_CONTROL,
                    )
                    .map_err(|error| format!("{error:#}"))
                })
                .collect(),
            background_parameters: assets::formats::RawBackgroundCatalog::original(&image)?,
            chr,
            backgrounds,
        })
    }
}

//! Named immutable catalog boundary. Format owners B04/B10/B15/B16/B18 decode
//! payloads; this owner validates joins and resolves ROM/default plus file levels.
use crate::{
    assets::Assets,
    identifiers::{AssetId, StageId},
};
use anyhow::{Context, Result, ensure};
use romatlas_port::assets::Collection;
use std::collections::{BTreeMap, BTreeSet};

pub const LEVEL_MANIFEST: &str = "levels.json";
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LevelDefinition {
    pub id: AssetId,
    pub original_selector: Option<StageId>,
    pub rooms: AssetId,
    pub terrain: AssetId,
    pub spawns: Option<AssetId>,
    pub graphics: Vec<AssetId>,
    pub audio: Vec<AssetId>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceReference {
    pub id: AssetId,
    pub kind: ResourceKind,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceKind {
    Rooms,
    Terrain,
    Spawns,
    Graphics,
    Audio,
}
#[derive(Clone, Debug)]
pub struct CatalogInput {
    pub levels: Vec<LevelDefinition>,
    pub resources: Vec<ResourceReference>,
}
#[derive(Clone, Debug)]
pub struct DecodedAssets {
    pub levels: Vec<LevelDefinition>,
}
impl DecodedAssets {
    /// Immutable validation is transactional: no partial catalog escapes on error.
    /// Source selector9 is ending layout; absence of spawn selector9 is intentional.
    pub fn resolve(raw: &Assets, input: CatalogInput) -> Result<Self> {
        validate_catalog(&raw.source, &input)?;
        Ok(Self {
            levels: input.levels,
        })
    }
    pub fn level(&self, id: &AssetId) -> Option<&LevelDefinition> {
        self.levels.iter().find(|level| level.id == *id)
    }
    pub fn original_level(&self, selector: StageId) -> Option<&LevelDefinition> {
        self.levels
            .iter()
            .find(|level| level.original_selector == Some(selector))
    }
}
/// Added file definitions share reference validation with original definitions;
/// raw Collection extensions alone are not playable content.
fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_./".contains(&byte))
}
fn validate_catalog(raw: &Collection, input: &CatalogInput) -> Result<()> {
    let mut resources = BTreeMap::new();
    for resource in &input.resources {
        ensure!(valid_id(&resource.id.0), "invalid resource ID");
        ensure!(
            resources
                .insert(resource.id.0.as_str(), resource.kind)
                .is_none(),
            "duplicate resource ID"
        );
        let payload = raw
            .assets
            .iter()
            .find(|asset| asset.id == resource.id.0)
            .context("missing resource payload")?;
        ensure!(!payload.bytes.is_empty(), "empty resource payload");
    }
    let mut names = BTreeSet::new();
    let mut selectors = BTreeSet::new();
    for level in &input.levels {
        ensure!(
            valid_id(&level.id.0) && names.insert(&level.id.0),
            "invalid or duplicate level ID"
        );
        if let Some(selector) = level.original_selector {
            ensure!(
                usize::from(selector.0) < crate::world::ORIGINAL_LAYOUTS
                    && selectors.insert(selector.0),
                "invalid or duplicate original selector"
            );
        }
        let check = |id: &AssetId, kind| -> Result<()> {
            ensure!(
                resources.get(id.0.as_str()) == Some(&kind),
                "missing or wrong-kind resource {}",
                id.0
            );
            Ok(())
        };
        check(&level.rooms, ResourceKind::Rooms)?;
        check(&level.terrain, ResourceKind::Terrain)?;
        if let Some(spawns) = &level.spawns {
            check(spawns, ResourceKind::Spawns)?;
        }
        ensure!(
            !level.graphics.is_empty() && !level.audio.is_empty(),
            "level needs graphics and audio"
        );
        for graphics in &level.graphics {
            check(graphics, ResourceKind::Graphics)?;
        }
        for audio in &level.audio {
            check(audio, ResourceKind::Audio)?;
        }
    }
    Ok(())
}
pub fn load_level_manifest(raw: &Collection, json: &[u8]) -> Result<CatalogInput> {
    let document: serde_json::Value = serde_json::from_slice(json)?;
    ensure!(
        document["version"].as_u64() == Some(1),
        "unsupported level manifest version"
    );
    let id = |object: &serde_json::Value, key: &str| -> Result<AssetId> {
        Ok(AssetId(
            object[key]
                .as_str()
                .context("missing resource ID")?
                .to_owned(),
        ))
    };
    let mut resources = Vec::new();
    for value in document["resources"]
        .as_array()
        .context("missing manifest resources")?
    {
        let kind = match value["kind"].as_str() {
            Some("rooms") => ResourceKind::Rooms,
            Some("terrain") => ResourceKind::Terrain,
            Some("spawns") => ResourceKind::Spawns,
            Some("graphics") => ResourceKind::Graphics,
            Some("audio") => ResourceKind::Audio,
            _ => anyhow::bail!("unknown resource kind"),
        };
        resources.push(ResourceReference {
            id: id(value, "id")?,
            kind,
        });
    }
    let mut levels = Vec::new();
    for value in document["levels"]
        .as_array()
        .context("missing manifest levels")?
    {
        ensure!(
            value.get("original_selector").is_none(),
            "extensions cannot replace source selector identities"
        );
        let ids = |key: &str| -> Result<Vec<AssetId>> {
            value[key]
                .as_array()
                .context("missing resource list")?
                .iter()
                .map(|value| {
                    Ok(AssetId(
                        value.as_str().context("resource list ID")?.to_owned(),
                    ))
                })
                .collect()
        };
        levels.push(LevelDefinition {
            id: id(value, "id")?,
            original_selector: None,
            rooms: id(value, "rooms")?,
            terrain: id(value, "terrain")?,
            spawns: value
                .get("spawns")
                .filter(|value| !value.is_null())
                .map(|_| id(value, "spawns"))
                .transpose()?,
            graphics: ids("graphics")?,
            audio: ids("audio")?,
        });
    }
    let input = CatalogInput { levels, resources };
    validate_catalog(raw, &input)?;
    Ok(input)
}
/// Source-aware catalog recipes; B04 owns layout/terrain formats and B10 owns
/// source spawn decoding. Metadata preserves selector9 ending-only separation.
pub fn original_catalog(raw: &Assets) -> Result<CatalogInput> {
    // Original resources identify recipe groups, not extracted file formats.
    // Their selector chooses the bounded original decoder. File levels instead
    // carry explicit payloads consumed by their corresponding format owner.
    const LAYOUT_RECIPES: &str = "StageRoomLayoutLists_1CE67Bytes";
    const TERRAIN_RECIPES: &str = "StagePaletteAndOverlayData";
    const SPAWN_RECIPES: &str = "SpecialAndMainSpawnRecords_1B174Bytes";
    const GRAPHICS_RECIPES: &str = "ChrUploadLists";
    const AUDIO_RECIPES: &str = "AudioCommandHeaderPointers_0CA50Bytes";
    const LEVEL_NAMES: [&str; crate::world::ORIGINAL_LAYOUTS] = [
        "original/amazon",
        "original/transylvania",
        "original/african-mines",
        "original/himalayas",
        "original/moon",
        "original/selection-layout",
        "original/title-layout",
        "original/special-room-first",
        "original/special-room-second",
        "original/ending-layout",
    ];
    const ENDING_SELECTOR: usize = 9;
    let resources = [
        (LAYOUT_RECIPES, ResourceKind::Rooms),
        (TERRAIN_RECIPES, ResourceKind::Terrain),
        (SPAWN_RECIPES, ResourceKind::Spawns),
        (GRAPHICS_RECIPES, ResourceKind::Graphics),
        (AUDIO_RECIPES, ResourceKind::Audio),
    ]
    .into_iter()
    .map(|(name, kind)| ResourceReference {
        id: AssetId(name.to_owned()),
        kind,
    })
    .collect();
    let levels = LEVEL_NAMES
        .into_iter()
        .enumerate()
        .map(|(selector, name)| LevelDefinition {
            id: AssetId(name.to_owned()),
            original_selector: Some(StageId(selector as u8)),
            rooms: AssetId(LAYOUT_RECIPES.to_owned()),
            terrain: AssetId(TERRAIN_RECIPES.to_owned()),
            spawns: (selector != ENDING_SELECTOR).then(|| AssetId(SPAWN_RECIPES.to_owned())),
            graphics: vec![AssetId(GRAPHICS_RECIPES.to_owned())],
            audio: vec![AssetId(AUDIO_RECIPES.to_owned())],
        })
        .collect();
    let input = CatalogInput { levels, resources };
    validate_catalog(&raw.source, &input)?;
    Ok(input)
}

/// B17 loads layout9 once; B16 supplies explicit per-scene terrain and CHR.
/// This is separate from nine original gameplay initialization/spawn configs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneResourceContext {
    pub layout: u8,
    pub terrain: u8,
    pub chr: u8,
    pub spawn_configuration: Option<u8>,
}
#[derive(Debug)]
pub struct SceneAssets<'a> {
    pub layout: &'a crate::world::Layout,
    pub terrain: super::formats::TerrainPayload,
    pub chr: super::formats::UploadList,
    pub spawn_configuration: Option<u8>,
}
pub fn original_scene_assets<'a>(
    raw: &Assets,
    world: &'a crate::world::WorldCatalog,
    context: SceneResourceContext,
) -> Result<SceneAssets<'a>> {
    let layout = world
        .layouts
        .get(usize::from(context.layout))
        .context("scene layout selector")?;
    let terrain = super::formats::terrain_payload(raw, context.terrain)?;
    // Descriptor bytes come from the named immutable override, never a second
    // cartridge read. The decoder takes a physical image view for table joins.
    let bytes = source_data_image(raw)?;
    let chr = super::formats::original_chr_list(&bytes, usize::from(context.chr))?;
    Ok(SceneAssets {
        layout,
        terrain,
        chr,
        spawn_configuration: context.spawn_configuration,
    })
}

/// Loader-only physical projection of named data recipes. Instruction holes stay
/// unavailable to consumers: format owners must bound their named table ranges.
/// This is immutable asset preparation, never native game state or CPU execution.
pub fn source_data_image(raw: &Assets) -> Result<Vec<u8>> {
    const CARTRIDGE_BYTES: usize = 0x20010;
    let mut bytes = vec![0; CARTRIDGE_BYTES];
    for recipe in crate::recipes::original_assets() {
        let asset = raw
            .source
            .assets
            .iter()
            .find(|asset| asset.id == recipe.id)
            .context("missing original data recipe")?;
        ensure!(
            asset.bytes.len() == recipe.length,
            "original data recipe length"
        );
        bytes[recipe.offset..recipe.offset + recipe.length].copy_from_slice(&asset.bytes);
    }
    Ok(bytes)
}

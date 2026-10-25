//! Immutable authored-level views. Runtime actors, caches and queues remain in
//! Core/Services; switching content never installs a second mutable game state.
use super::content::Content;
use crate::{
    assets::{
        self,
        catalog::DecodedAssets,
        file_resources::{
            AudioResource, GraphicsResource, RoomResource, SpawnsResource, TerrainResource,
        },
    },
    identifiers::AssetId,
};
use anyhow::{Context, Result, ensure};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
#[derive(Clone, Debug)]
pub struct AddedLevel {
    pub id: AssetId,
    pub rooms: RoomResource,
    pub terrain: TerrainResource,
    pub graphics: Vec<GraphicsResource>,
    pub spawns: Option<SpawnsResource>,
    pub audio: Option<AudioResource>,
}
#[derive(Clone, Debug, Default)]
pub struct Registry {
    pub levels: BTreeMap<String, Arc<AddedLevel>>,
    pub successors: BTreeMap<String, String>,
}
impl Registry {
    pub fn decode(catalog: &DecodedAssets, raw: &assets::Assets) -> Result<Self> {
        let mut registry = Self::default();
        for definition in catalog
            .levels
            .iter()
            .filter(|level| level.original_selector.is_none())
        {
            let payload = |id: &AssetId| {
                raw.source
                    .get(&id.0)
                    .with_context(|| format!("missing authored payload {}", id.0))
            };
            let rooms = RoomResource::decode(&payload(&definition.rooms)?.bytes)?;
            let terrain = TerrainResource::decode(&payload(&definition.terrain)?.bytes)?;
            let spawns = definition
                .spawns
                .as_ref()
                .map(|id| SpawnsResource::decode(&payload(id)?.bytes, rooms.layout.rows))
                .transpose()?;
            ensure!(
                definition.audio.len() <= 1,
                "one authored stage song per level"
            );
            let audio = definition
                .audio
                .first()
                .map(|id| AudioResource::decode(&payload(id)?.bytes))
                .transpose()?;
            let graphics = definition
                .graphics
                .iter()
                .map(|id| GraphicsResource::decode(&payload(id)?.bytes))
                .collect::<Result<Vec<_>>>()?;
            ensure!(
                registry
                    .successors
                    .insert(rooms.after.0.clone(), definition.id.0.clone())
                    .is_none(),
                "two authored successors for one level"
            );
            registry.levels.insert(
                definition.id.0.clone(),
                Arc::new(AddedLevel {
                    id: definition.id.clone(),
                    rooms,
                    terrain,
                    graphics,
                    spawns,
                    audio,
                }),
            );
        }
        // Validate the entire graph before exposing any playable view. A missing
        // predecessor or cycle is a startup error, never a partial installation.
        for level in registry.levels.values() {
            let mut current = level.id.0.as_str();
            let mut seen = BTreeSet::new();
            loop {
                ensure!(seen.insert(current), "authored level predecessor cycle");
                if let Some(parent) = registry.levels.get(current) {
                    current = &parent.rooms.after.0;
                } else {
                    let original = catalog
                        .level(&AssetId(current.into()))
                        .context("missing authored level predecessor")?;
                    ensure!(
                        original.original_selector.is_some_and(|stage| stage.0 < 5),
                        "authored predecessor is not a playable original level"
                    );
                    break;
                }
            }
        }
        Ok(registry)
    }
}
impl Content {
    pub fn authored_view(&self, id: &str) -> Result<Self> {
        let level = self
            .additions
            .levels
            .get(id)
            .context("unknown authored level")?
            .clone();
        let mut view = self.clone();
        let stage = usize::from(level.rooms.rule_stage);
        let first_room = view.raw_rooms.len();
        view.raw_rooms.extend(level.rooms.rooms.iter().cloned());
        let mut layout = level.rooms.layout.clone();
        for room in layout.rooms.iter_mut().flatten() {
            *room += first_room;
        }
        view.world.layouts[stage] = layout.clone();
        view.raw_layouts[stage] = Ok(layout);
        let bank = usize::from(view.raw_stage_presets[stage].terrain_bank);
        view.world.terrain[bank] = level.terrain.terrain.clone();
        let preset = &mut view.raw_stage_presets[stage];
        preset.row = level.rooms.entry_room / level.rooms.layout.columns;
        preset.camera_room = level.rooms.entry_room;
        preset.player_x = level.rooms.player_x;
        preset.background_palette = level.terrain.palette;
        preset.countdown_pair = level.rooms.countdown;
        let mut chr = view.raw_chr[stage]
            .as_ref()
            .map_err(|error| anyhow::anyhow!(error.clone()))?
            .clone();
        for resource in &level.graphics {
            chr.copies.extend(resource.copies.iter().cloned());
        }
        view.raw_chr[stage] = Ok(chr);
        if let Some(spawns) = &level.spawns {
            let first_template = view.spawns.templates.len();
            let templates = spawns
                .templates
                .iter()
                .map(|template| match template {
                    assets::file_resources::FileTemplate::Original(index) => {
                        self.spawns.templates[*index]
                    }
                    assets::file_resources::FileTemplate::Authored(template) => *template,
                })
                .collect::<Vec<_>>();
            view.spawns.templates.extend(templates);
            let mut rows = spawns.rows.clone();
            let mut planes = crate::actors::SpawnPlanes {
                x_high: [0xff; 256],
                x_low: [0; 256],
                room_starts: [0; 256],
                y: [0; 256],
                templates: [0; 256],
            };
            let mut first = 0;
            for (row, records) in rows.iter_mut().enumerate() {
                planes.room_starts[row] = first;
                for record in records.iter_mut() {
                    record.template += first_template;
                    let index = usize::from(record.identity);
                    planes.x_high[index] = (record.world_x >> 8) as u8;
                    planes.x_low[index] = record.world_x as u8;
                    planes.y[index] = record.screen_y;
                    planes.templates[index] = record.template as u8;
                }
                first = records.last().map_or(first.wrapping_add(1), |record| {
                    record.identity.wrapping_add(2)
                });
            }
            view.spawn_planes.insert(stage as u8, planes);
            let id = AssetId(format!("{}/spawns", level.id.0));
            view.spawns.source_levels[stage] = Some(id.clone());
            view.spawns.levels.push(crate::actors::SpawnDefinition {
                id,
                special_rows: vec![Vec::new(); rows.len()],
                rows,
            });
        }
        if let Some(audio) = &level.audio {
            audio.install(&mut view.audio, stage as u8);
        }
        view.active_file = Some(level);
        Ok(view)
    }
    /// Source room-list users compare only against FF. Authored room payload
    /// identities are usize indices, so they must not truncate into that sentinel.
    pub fn room_presence(&self, room: Option<usize>) -> u8 {
        room.map_or(crate::world::MISSING_ROOM, |index| {
            if self.active_file.is_some() {
                0
            } else {
                index as u8
            }
        })
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use serde_json::json;
    fn memory() -> romatlas_port::assets::MemoryAssets {
        let mut memory = romatlas_port::assets::MemoryAssets {
            enabled: true,
            ..Default::default()
        };
        let mut entries = vec![json!({"id":"extension/levels","file":"levels.json"})];
        let payloads = [
            (
                "file/rooms",
                json!({"version":1,"format":"ducktales.rooms","after":"original/amazon","rule_stage":0,"rows":1,"columns":1,"cells":[0],"rooms":[{"low":vec![1;64],"ninth_bits":vec![0;8]}],"player_y":96}),
            ),
            (
                "file/terrain",
                json!({"version":1,"format":"ducktales.terrain","palette":vec![15;16],"tiles":[{"id":1,"corners":[1,2,3,4],"palette":2,"collision":1}],"blocks":[{"id":1,"quadrants":[1,1,1,1]}]}),
            ),
            (
                "file/spawns",
                json!({"version":1,"format":"ducktales.spawns","templates":[{"handler":8,"animation":1}],"rows":[[{"x":192,"y":96,"template":0}]]}),
            ),
            (
                "file/graphics",
                json!({"version":1,"format":"ducktales.graphics","copies":[{"destination":16,"bytes":vec![0x55;16]}]}),
            ),
            (
                "file/audio",
                json!({"version":1,"format":"ducktales.audio","periods":vec![321;32],"voices":[[{"key":1,"ticks":2,"volume":9}],[],[],[]]}),
            ),
        ];
        for (id, payload) in payloads {
            let file = format!("{}.json", id.replace('/', "-"));
            entries.push(json!({"id":id,"file":file}));
            memory
                .files
                .insert(file, serde_json::to_vec(&payload).unwrap());
        }
        memory.files.insert(assets::catalog::LEVEL_MANIFEST.into(),serde_json::to_vec(&json!({"version":1,"resources":[{"id":"file/rooms","kind":"rooms"},{"id":"file/terrain","kind":"terrain"},{"id":"file/spawns","kind":"spawns"},{"id":"file/graphics","kind":"graphics"},{"id":"file/audio","kind":"audio"}],"levels":[{"id":"file/level","rooms":"file/rooms","terrain":"file/terrain","spawns":"file/spawns","graphics":["file/graphics"],"audio":["file/audio"]}]})).unwrap());
        memory.manifests.insert(
            assets::MANIFEST.into(),
            serde_json::from_value(json!({"version":1,"entries":entries,"order":[]})).unwrap(),
        );
        memory
    }
    pub fn content() -> Content {
        let rom = romatlas_port::assets::Rom::from_bytes(
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap(),
            crate::config::ROM_SHA256,
        )
        .unwrap();
        let memory = memory();
        let raw = assets::Assets::load(&rom, &memory).unwrap();
        Content::load(&rom, &raw, &memory).unwrap()
    }
    #[test]
    fn authored_payload_view_feeds_real_shared_terrain_chr_spawn_and_audio_consumers() {
        let base = content();
        let rooms_before = base.raw_rooms.len();
        let view = base.authored_view("file/level").unwrap();
        let mut services = super::super::simulation::Services::new(view);
        let mut core = super::super::simulation::Core::reset();
        services.load_stage_presentation(&mut core, 0);
        services.build_column(&mut core, crate::world::StreamCursor::default());
        assert!(core.cache.collision.contains(&0x55));
        assert_eq!(services.content.raw_rooms.len(), rooms_before + 1);
        assert_eq!(services.content.room_presence(Some(255)), 0);
        let chr = services.content.raw_chr[0].as_ref().unwrap();
        services.video.display.apply_chr(chr);
        services.video.commit_writes();
        assert_eq!(services.video.memory.read(16), 0x55);
        core.spawn_window =
            crate::actors::reset_window(0, &services.content.spawn_planes[&0].x_high, 0);
        core.spawn_window.direction = 1;
        services.spawn(&mut core);
        assert!(
            core.actors
                .iter()
                .any(|actor| actor.handler == 8 && actor.spawn_identity == Some(0))
        );
        services
            .audio
            .command(&services.content.audio, 0, 0, &mut Vec::new());
        services.audio.frame(&services.content.audio, 0);
        assert_eq!(services.audio.channels[0].primary.period, 321);
        assert_eq!(base.raw_rooms.len(), rooms_before);
        assert_eq!(
            base.authored_view("file/level").unwrap().raw_rooms.len(),
            rooms_before + 1,
            "fresh immutable base prevents repeated activation accumulation"
        );
    }
    #[test]
    fn authored_period_fragment_preserves_all_original_primary_stream_and_period_consumers() {
        let base = content();
        let view = base.authored_view("file/level").unwrap();
        let mut pending = Vec::new();
        let mut seen = BTreeSet::new();
        let mut selectors = BTreeSet::new();
        for header in &base.audio.headers {
            if let crate::audio::Header::Primary {
                cursors,
                preset_base,
                ..
            } = header
            {
                pending.extend(cursors.iter().copied().filter(|cursor| *cursor != 0));
                for offset in 0..128 {
                    let address = preset_base.wrapping_add(offset);
                    assert_eq!(
                        base.audio.read(address).unwrap(),
                        view.audio.read(address).unwrap()
                    );
                }
            }
        }
        while let Some(mut cursor) = pending.pop() {
            loop {
                if !seen.insert(cursor) {
                    break;
                }
                let read = |address| {
                    let original = base.audio.read(address).unwrap();
                    assert_eq!(
                        original,
                        view.audio.read(address).unwrap(),
                        "original encoded consumer {address:04x}"
                    );
                    original
                };
                let command = read(cursor);
                cursor = cursor.wrapping_add(1);
                if command >= 0x20 {
                    continue;
                }
                assert!(command <= 9, "source primary opcode");
                let count = [1, 1, 1, 1, 3, 1, 1, 2, 1, 0][usize::from(command)];
                let mut args = Vec::new();
                for _ in 0..count {
                    args.push(read(cursor));
                    cursor = cursor.wrapping_add(1);
                }
                if command == 5 {
                    selectors.insert(args[0]);
                    let table = 0x897b_u16.wrapping_add(u16::from(args[0].wrapping_mul(2)));
                    for offset in 0..64 {
                        read(table.wrapping_add(offset));
                    }
                }
                if command == 4 {
                    pending.push(u16::from_le_bytes([args[1], args[2]]));
                    if args[0] == 0 {
                        break;
                    }
                }
                if command == 9 {
                    break;
                }
            }
        }
        assert!(!selectors.contains(&0x63));
        assert!(!seen.is_empty());
    }

    #[test]
    fn file_directory_delivery_reaches_shared_port_with_metadata_and_every_payload() {
        let memory = memory();
        let root =
            std::env::temp_dir().join(format!("ducktales-file-resources-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        for (name, bytes) in &memory.files {
            std::fs::write(root.join(name), bytes).unwrap();
        }
        std::fs::write(
            root.join(assets::MANIFEST),
            serde_json::to_vec(&memory.manifests[assets::MANIFEST]).unwrap(),
        )
        .unwrap();
        // A reusable, original-code-free example is emitted from this exact
        // fixture format; frontend delivery is tested independently below.
        let loaded =
            romatlas_port::assets::MemoryAssets::from_directory(&root, &[assets::MANIFEST])
                .unwrap();
        assert!(loaded.files.contains_key("levels.json"));
        let rom = romatlas_port::assets::Rom::from_bytes(
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap(),
            crate::config::ROM_SHA256,
        )
        .unwrap();
        let port = super::super::Port::load_assets(&rom, &loaded).unwrap();
        assert!(
            port.machine
                .services
                .content
                .additions
                .levels
                .contains_key("file/level")
        );
        assert_eq!(
            port.machine.services.content.additions.successors["original/amazon"],
            "file/level"
        );
        // Browser MemoryAssets receives these same manifest entries/file keys.
        assert_eq!(loaded.files, memory.files);
    }

    #[test]
    fn missing_cyclic_and_forked_predecessors_fail_transactionally() {
        let rom = romatlas_port::assets::Rom::from_bytes(
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap(),
            crate::config::ROM_SHA256,
        )
        .unwrap();
        for predecessor in ["missing/level", "file/level", "original/ending-layout"] {
            let mut memory = memory();
            let mut room: serde_json::Value =
                serde_json::from_slice(&memory.files["file-rooms.json"]).unwrap();
            room["after"] = json!(predecessor);
            memory
                .files
                .insert("file-rooms.json".into(), serde_json::to_vec(&room).unwrap());
            let raw = assets::Assets::load(&rom, &memory).unwrap();
            assert!(Content::load(&rom, &raw, &memory).is_err(), "{predecessor}");
        }
        let mut memory = memory();
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&memory.files["levels.json"]).unwrap();
        let mut second = manifest["levels"][0].clone();
        second["id"] = json!("file/second");
        manifest["levels"].as_array_mut().unwrap().push(second);
        memory
            .files
            .insert("levels.json".into(), serde_json::to_vec(&manifest).unwrap());
        let raw = assets::Assets::load(&rom, &memory).unwrap();
        assert!(Content::load(&rom, &raw, &memory).is_err());
    }
    #[test]
    fn delivered_manifests_load_original_only_and_authored_source_template_without_embedding_assets()
     {
        let rom = romatlas_port::assets::Rom::from_bytes(
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap(),
            crate::config::ROM_SHA256,
        )
        .unwrap();
        for directory in ["original-only", "authored-level"] {
            let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../examples"))
                .join(directory);
            let port = super::super::Port::load(&rom, Some(&root)).unwrap();
            if directory == "original-only" {
                assert!(port.machine.services.content.additions.levels.is_empty());
            } else {
                let base = &port.machine.services.content;
                let view = base.authored_view("example/room").unwrap();
                assert_eq!(
                    view.spawns.templates[base.spawns.templates.len()],
                    base.spawns.templates[80]
                );
                assert_eq!(
                    view.world.terrain[usize::from(view.raw_stage_presets[0].terrain_bank)]
                        .properties[2],
                    0x10
                );
            }
        }
    }
}

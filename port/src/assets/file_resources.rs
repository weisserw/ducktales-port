//! Authored file payloads, distinct from the original recipe-group anchors.
//! These decode into the SAME room/block/tile structures consumed by B04.
//! Source rule profiles retain their selectors; named level identity is separate.
use crate::{identifiers::AssetId, world};
use anyhow::{Context, Result, ensure};
use serde_json::Value;
const PLAYABLE_RULE_PROFILES: u8 = 5;
const SHARED_ANIMATION_COUNT: usize = 144;
const DEFAULT_ENTRY_PIXEL: u8 = 0x80;
const SOURCE_ROW_ROOMS: u8 = 8;
const COLLISION_PROPERTY_SHIFT: u8 = 4;
#[derive(Clone, Debug)]
pub struct RoomResource {
    pub after: AssetId,
    pub rule_stage: u8,
    pub layout: world::Layout,
    pub rooms: Vec<world::Room>,
    pub entry_room: u8,
    pub player_x: u8,
    pub player_y: u8,
    pub countdown: [u8; 2],
}
#[derive(Clone, Debug)]
pub struct TerrainResource {
    pub terrain: world::TerrainSet,
    pub palette: [u8; 16],
}
fn document(bytes: &[u8], format: &str) -> Result<Value> {
    let value: Value = serde_json::from_slice(bytes).context("file resource JSON")?;
    ensure!(
        value["version"] == 1 && value["format"] == format,
        "unsupported file resource format/version"
    );
    Ok(value)
}
fn byte(value: &Value, field: &str) -> Result<u8> {
    u8::try_from(
        value[field]
            .as_u64()
            .with_context(|| format!("missing unsigned byte {field}"))?,
    )
    .with_context(|| format!("byte range {field}"))
}
fn optional_byte(value: &Value, field: &str, default: u8) -> Result<u8> {
    if value.get(field).is_some() {
        byte(value, field)
    } else {
        Ok(default)
    }
}
fn bytes<const N: usize>(value: &Value) -> Result<[u8; N]> {
    let values = value.as_array().context("byte array")?;
    ensure!(values.len() == N, "byte array extent must be{N}");
    let mut result = [0; N];
    for (out, value) in result.iter_mut().zip(values) {
        *out = u8::try_from(value.as_u64().context("unsigned byte array value")?)
            .context("byte array value range")?;
    }
    Ok(result)
}
fn array<'a>(value: &'a Value, field: &str) -> Result<&'a [Value]> {
    Ok(value[field]
        .as_array()
        .with_context(|| format!("missing array {field}"))?)
}
impl RoomResource {
    pub fn decode(payload: &[u8]) -> Result<Self> {
        let value = document(payload, "ducktales.rooms")?;
        let after = value["after"]
            .as_str()
            .context("level needs a named predecessor")?;
        ensure!(
            !after.is_empty()
                && after.len() <= 128
                && after
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"-_./".contains(&byte)),
            "invalid predecessor ID"
        );
        let rule_stage = byte(&value, "rule_stage")?;
        ensure!(
            rule_stage < PLAYABLE_RULE_PROFILES,
            "unknown shared source rule profile"
        );
        let rows = byte(&value, "rows")?;
        let columns = byte(&value, "columns")?;
        ensure!(
            rows != 0 && columns != 0 && usize::from(rows) * usize::from(columns) <= 256,
            "room-grid coordinate extent"
        );
        // Multirow source gameplay uses the eight-room ring-row convention.
        // Single rows also support narrower maps. This bounds room arithmetic,
        // not the number of named levels in the extension registry.
        ensure!(
            columns == SOURCE_ROW_ROOMS || rows == 1 && columns < SOURCE_ROW_ROOMS,
            "multirow maps require the source eight-room row format"
        );
        let rooms = array(&value, "rooms")?
            .iter()
            .map(|room| {
                let mut encoded = bytes::<{ world::ROOM_CELLS }>(&room["low"])?.to_vec();
                encoded.extend(bytes::<8>(&room["ninth_bits"])?);
                world::Room::decode(&encoded)
            })
            .collect::<Result<Vec<_>>>()?;
        ensure!(!rooms.is_empty(), "room payload list is empty");
        let cells = array(&value, "cells")?;
        ensure!(
            cells.len() == usize::from(rows) * usize::from(columns),
            "room-grid cell extent"
        );
        let cells = cells
            .iter()
            .map(|cell| {
                if cell.is_null() {
                    Ok(None)
                } else {
                    let index = usize::try_from(cell.as_u64().context("room-grid payload index")?)?;
                    ensure!(index < rooms.len(), "missing room payload");
                    Ok(Some(index))
                }
            })
            .collect::<Result<Vec<_>>>()?;
        let entry_room = optional_byte(&value, "entry_room", 0)?;
        ensure!(
            cells
                .get(usize::from(entry_room))
                .copied()
                .flatten()
                .is_some(),
            "entry room is missing"
        );
        let countdown = if value.get("countdown").is_some() {
            bytes::<2>(&value["countdown"])?
        } else {
            [5, 0]
        };
        ensure!(
            countdown.iter().all(|digit| *digit <= 9),
            "countdown decimal digit"
        );
        Ok(Self {
            after: AssetId(after.into()),
            rule_stage,
            layout: world::Layout {
                rows,
                columns,
                rooms: cells,
            },
            rooms,
            entry_room,
            player_x: optional_byte(&value, "player_x", DEFAULT_ENTRY_PIXEL)?,
            player_y: optional_byte(&value, "player_y", DEFAULT_ENTRY_PIXEL)?,
            countdown,
        })
    }
}
impl TerrainResource {
    pub fn decode(payload: &[u8]) -> Result<Self> {
        let value = document(payload, "ducktales.terrain")?;
        let palette = bytes::<16>(&value["palette"])?;
        ensure!(palette.iter().all(|color| *color < 64), "NES color index");
        let mut terrain = world::TerrainSet {
            blocks: vec![[0; 4]; world::BLOCK_DESCRIPTORS],
            tiles: [[0; 4]; world::TERRAIN_SELECTORS],
            properties: [0; world::TERRAIN_SELECTORS],
        };
        let mut seen = std::collections::BTreeSet::new();
        for tile in array(&value, "tiles")? {
            let id = byte(tile, "id")?;
            ensure!(seen.insert(id), "duplicate terrain descriptor");
            let palette = byte(tile, "palette")?;
            let collision = byte(tile, "collision")?;
            ensure!(
                palette < 4 && collision < 4,
                "terrain descriptor palette/collision class"
            );
            terrain.tiles[usize::from(id)] = bytes::<4>(&tile["corners"])?;
            terrain.properties[usize::from(id)] = palette | collision << COLLISION_PROPERTY_SHIFT;
        }
        let mut seen = std::collections::BTreeSet::new();
        for block in array(&value, "blocks")? {
            let id = usize::try_from(block["id"].as_u64().context("block ID")?)?;
            ensure!(
                id < world::BLOCK_DESCRIPTORS && seen.insert(id),
                "duplicate/out-of-range block"
            );
            terrain.blocks[id] = bytes::<4>(&block["quadrants"])?;
        }
        Ok(Self { terrain, palette })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn authored_room_and_terrain_payloads_feed_the_shared_expansion_rule() {
        let room = json!({"version":1,"format":"ducktales.rooms","after":"original/amazon","rule_stage":0,"rows":1,"columns":1,"cells":[0],"rooms":[{"low":vec![1;64],"ninth_bits":vec![0;8]}]});
        let rooms = RoomResource::decode(&serde_json::to_vec(&room).unwrap()).unwrap();
        let terrain = json!({"version":1,"format":"ducktales.terrain","palette":vec![15;16],"tiles":[{"id":1,"corners":[1,2,3,4],"palette":2,"collision":1}],"blocks":[{"id":1,"quadrants":[1,1,1,1]}]});
        let terrain = TerrainResource::decode(&serde_json::to_vec(&terrain).unwrap()).unwrap();
        let block = world::Room::block(0, &rooms.rooms[rooms.layout.rooms[0].unwrap()]);
        let expanded = world::expand(&terrain.terrain, block, world::ExpansionContext::default());
        assert_eq!(expanded.collision, 0x55);
        assert_eq!(expanded.palette, 0xaa);
        assert_eq!(&expanded.tiles[..4], &[1, 2, 1, 2]);
    }
    #[test]
    fn malformed_authored_grid_and_descriptor_fail_before_a_level_escapes() {
        let terrain = json!({"version":1,"format":"ducktales.terrain","palette":vec![15;16],"tiles":[{"id":1,"corners":[0,0,0,0],"palette":0,"collision":4}],"blocks":[]});
        assert!(TerrainResource::decode(&serde_json::to_vec(&terrain).unwrap()).is_err());
        let room = json!({"version":1,"format":"ducktales.rooms","after":"original/amazon","rule_stage":0,"rows":1,"columns":1,"cells":[1],"rooms":[{"low":vec![0;64],"ninth_bits":vec![0;8]}]});
        assert!(RoomResource::decode(&serde_json::to_vec(&room).unwrap()).is_err());
    }
}

/// Authored pattern bytes use the same CHR upload owner as original content.
/// Destinations and extents are checked before any display state is changed.
#[derive(Clone, Debug)]
pub struct GraphicsResource {
    pub copies: Vec<(u16, Vec<u8>)>,
}
impl GraphicsResource {
    pub fn decode(payload: &[u8]) -> Result<Self> {
        let value = document(payload, "ducktales.graphics")?;
        let copies = array(&value, "copies")?
            .iter()
            .map(|copy| {
                let destination = u16::try_from(
                    copy["destination"]
                        .as_u64()
                        .context("graphics destination")?,
                )?;
                let data = array(copy, "bytes")?
                    .iter()
                    .map(|value| {
                        u8::try_from(value.as_u64().context("graphics byte")?)
                            .context("graphics byte range")
                    })
                    .collect::<Result<Vec<_>>>()?;
                ensure!(
                    destination & 15 == 0
                        && !data.is_empty()
                        && data.len() % 16 == 0
                        && usize::from(destination) + data.len() <= 0x2000,
                    "graphics tile extent"
                );
                Ok((destination, data))
            })
            .collect::<Result<Vec<_>>>()?;
        ensure!(!copies.is_empty(), "empty authored graphics");
        Ok(Self { copies })
    }
}

#[derive(Clone, Debug)]
pub enum FileTemplate {
    Original(usize),
    Authored(crate::actors::Template),
}
#[derive(Clone, Debug)]
pub struct SpawnsResource {
    pub templates: Vec<FileTemplate>,
    pub rows: Vec<Vec<crate::actors::SpawnRecord>>,
}
impl SpawnsResource {
    pub fn decode(payload: &[u8], room_rows: u8) -> Result<Self> {
        let value = document(payload, "ducktales.spawns")?;
        let templates = array(&value, "templates")?
            .iter()
            .map(|template| {
                if let Some(original) = template.get("original_template") {
                    let original =
                        usize::try_from(original.as_u64().context("original template selector")?)?;
                    ensure!(
                        original < crate::actors::TEMPLATE_COUNT,
                        "original template selector"
                    );
                    return Ok(FileTemplate::Original(original));
                }
                let handler = byte(template, "handler")?;
                ensure!(
                    usize::from(handler) < crate::actors::HANDLER_COUNT,
                    "unknown shared actor handler"
                );
                let animation = byte(template, "animation")?;
                ensure!(
                    usize::from(animation) < SHARED_ANIMATION_COUNT,
                    "actor animation selector"
                );
                let profile = optional_byte(template, "profile", 0)?;
                ensure!(profile & 0x7f < 32, "actor collision profile selector");
                Ok(FileTemplate::Authored(crate::actors::Template {
                    handler,
                    animation,
                    flags_and_local: optional_byte(template, "flags_and_local", 0)?,
                    profile,
                }))
            })
            .collect::<Result<Vec<_>>>()?;
        ensure!(
            templates.len() + crate::actors::TEMPLATE_COUNT <= 256,
            "authored template byte extent"
        );
        let rows = array(&value, "rows")?;
        ensure!(
            rows.len() == usize::from(room_rows),
            "spawn row count differs from room grid"
        );
        let mut identity = 0usize;
        let rows = rows
            .iter()
            .map(|row| {
                let mut previous = None;
                let result = row
                    .as_array()
                    .context("spawn row array")?
                    .iter()
                    .map(|record| {
                        let world_x = u16::try_from(record["x"].as_u64().context("spawn x")?)?;
                        ensure!(
                            world_x < 0x8000 && previous.is_none_or(|x| x <= world_x),
                            "spawn x order/source sentinel"
                        );
                        previous = Some(world_x);
                        let template = usize::from(byte(record, "template")?);
                        ensure!(
                            template < templates.len() && identity < 255,
                            "spawn template/identity extent"
                        );
                        let result = crate::actors::SpawnRecord {
                            identity: identity as u8,
                            world_x,
                            screen_y: byte(record, "y")?,
                            template,
                        };
                        identity += 1;
                        Ok(result)
                    })
                    .collect::<Result<Vec<_>>>()?;
                identity += 1;
                ensure!(identity <= 256, "spawn row terminator extent");
                Ok(result)
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { templates, rows })
    }
}

/// A finite note language compiles to safe B18 primary commands; file content
/// cannot introduce arbitrary stream jumps or unbounded inline-opcode loops.
#[derive(Clone, Debug)]
pub struct AudioResource {
    pub periods: Vec<u8>,
    pub streams: Vec<Vec<u8>>,
    pub looping: bool,
}
impl AudioResource {
    pub fn decode(payload: &[u8]) -> Result<Self> {
        let value = document(payload, "ducktales.audio")?;
        let periods = array(&value, "periods")?;
        ensure!(periods.len() == 32, "audio period table extent");
        let periods = periods
            .iter()
            .map(|period| {
                let period = u16::try_from(period.as_u64().context("audio period")?)?;
                ensure!(period <= 0x7ff, "audio period range");
                Ok(period.to_le_bytes())
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect();
        let voices = array(&value, "voices")?;
        ensure!(voices.len() == crate::audio::CHANNELS, "audio voice count");
        let mut total = 0;
        let streams = voices
            .iter()
            .map(|voice| {
                let mut stream = vec![5, 0x63, 7, 0, 0];
                for note in voice.as_array().context("audio voice notes")? {
                    total += 1;
                    ensure!(total <= 1024, "audio note extent");
                    let key = byte(note, "key")?;
                    let ticks = byte(note, "ticks")?;
                    let volume = byte(note, "volume")?;
                    let duty = optional_byte(note, "duty", 0)?;
                    ensure!(
                        key < 32 && ticks != 0 && volume < 16 && duty < 4,
                        "audio note fields"
                    );
                    stream.extend([2, duty << 6, 3, 0x30 | volume, 0, ticks, 0x60 | key]);
                }
                if stream.len() == 5 {
                    stream.clear();
                }
                Ok(stream)
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            periods,
            streams,
            looping: value.get("loop").and_then(Value::as_bool).unwrap_or(true),
        })
    }
    pub fn install(&self, catalog: &mut crate::audio::Catalog, stage: u8) {
        const SCRIPT_START: u16 = 0x1000;
        // Header pointers are fully decoded at startup; no runtime stream reads them.
        const PERIOD_TABLE: u16 = 0x8a41;
        let mut origin = SCRIPT_START;
        let mut cursors = [0; crate::audio::CHANNELS];
        for (channel, stream) in self
            .streams
            .iter()
            .enumerate()
            .filter(|(_, stream)| !stream.is_empty())
        {
            cursors[channel] = origin;
            let mut bytes = stream.clone();
            if self.looping {
                bytes.extend([4, 0, origin as u8, (origin >> 8) as u8]);
            } else {
                bytes.push(9);
            }
            origin += bytes.len() as u16;
            catalog.fragments.insert(
                0,
                crate::audio::Fragment {
                    origin: cursors[channel],
                    bytes,
                },
            );
        }
        catalog.fragments.insert(
            0,
            crate::audio::Fragment {
                origin: PERIOD_TABLE,
                bytes: self.periods.clone(),
            },
        );
        catalog.headers[usize::from(stage)] = crate::audio::Header::Primary {
            priority: 1,
            cursors,
            preset_base: 0,
        };
    }
}

#[cfg(test)]
mod audio_tests {
    use super::*;
    #[test]
    fn compiled_notes_keep_original_duration_table_and_loop_on_actual_driver_frames() {
        let value = serde_json::json!({"version":1,"format":"ducktales.audio","periods":vec![321;32],"voices":[[{"key":1,"ticks":2,"volume":9,"duty":2}],[],[],[]]});
        let resource = AudioResource::decode(&serde_json::to_vec(&value).unwrap()).unwrap();
        let mut catalog = crate::audio::Catalog {
            headers: vec![crate::audio::Header::Primary {
                priority: 1,
                cursors: [0; 4],
                preset_base: 0,
            }],
            duration: [0, 0, 2, 4, 8, 16, 32, 64],
            ..Default::default()
        };
        resource.install(&mut catalog, 0);
        let mut driver = crate::audio::Driver::default();
        driver.command(&catalog, 0, 0, &mut Vec::new());
        driver.frame(&catalog, 0);
        assert_eq!(
            (
                driver.channels[0].primary.period,
                driver.channels[0].primary.duration
            ),
            (321, 8)
        );
        let after_note = driver.channels[0].primary.cursor;
        driver.frame(&catalog, 0);
        assert_eq!(
            (
                driver.channels[0].primary.cursor,
                driver.channels[0].primary.duration
            ),
            (after_note, 4)
        );
        driver.frame(&catalog, 0);
        assert_eq!(
            (
                driver.channels[0].primary.cursor,
                driver.channels[0].primary.duration
            ),
            (after_note, 8)
        );
        assert_eq!(catalog.duration, [0, 0, 2, 4, 8, 16, 32, 64]);
    }
}

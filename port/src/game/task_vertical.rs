//! The source's sixty-tick vertical call. Tile staging remains in the same
//! secondary queue bytes used by audio/composites; this continuation owns only
//! the phase/cursor lifetime and the currently entered wait.
use super::*;
// Offsets describe the original upload format, projected onto Video's disjoint
// owners. They are not another retained address space.
const ROOM_BLOCKS: u8 = 8;
const LAST_ROOM_ROW: u8 = ROOM_BLOCKS - 1;
const FIRST_NAMETABLE: u16 = 0x2000;
const SECOND_NAMETABLE: u16 = 0x2400;
const NAMETABLE_ATTRIBUTE_OFFSET: u16 = 0x3c0;
const BLOCK_ROW_VIDEO_BYTES: u16 = 128;
const TILE_ROW_VIDEO_BYTES: u16 = 32;
const HALF_BLOCK_VIDEO_BYTES: u16 = 64;
const ROW_TILE_COUNT: usize = 32;
const ROW_COMMANDS: [usize; 3] = [0, 0x23, 0x46];
const ROW_PAYLOADS: [usize; 2] = [3, 0x26];
const RETAINED_ROW_PAYLOADS: [usize; 2] = [0x60, 0x80];
const ROW_ATTRIBUTE_PAYLOAD: usize = 0x49;
const ROW_TERMINATOR: usize = 0x51;
const NEIGHBOR_PAYLOADS: [usize; 4] = [3, 0x16, 0x29, 0x3c];
const NEIGHBOR_COMMAND_BYTES: usize = 19;
const NEIGHBOR_ATTRIBUTE_COMMAND: usize = 0x4c;
const NEIGHBOR_ATTRIBUTE_PAYLOAD: usize = 0x4f;
const NEIGHBOR_TERMINATOR: usize = 0x53;
const SECOND_CACHE_PAGE: usize = world::CACHE_CELLS / 2;
const VERTICAL_DIRECTION_DOWN: u8 = 4;
const PALETTE_LOW_QUADRANTS: u8 = 0x0f;
const PALETTE_HIGH_QUADRANTS: u8 = 0xf0;
#[derive(Clone)]
pub(super) struct VerticalCall {
    transition: world::VerticalTransition,
    work: VecDeque<world::VerticalEffect>,
}
impl VerticalCall {
    pub(super) fn observe(&self) -> serde_json::Value {
        let transition = &self.transition;
        serde_json::json!({"direction":format!("{:?}",transition.direction),"counter":transition.counter,"player_y":transition.player_y,"split_address":transition.split_address,"split_control":transition.split_control,
            "cursor":{"ring":transition.cursor.ring,"room":transition.cursor.room,"room_in_row":transition.cursor.room_in_row},
            "saved_cursor":{"ring":transition.saved_cursor.ring,"room":transition.saved_cursor.room,"room_in_row":transition.saved_cursor.room_in_row},"pending":self.work.front().map(|work|format!("{work:?}"))})
    }
}
impl Tasks {
    pub fn vertical_transition(
        &mut self,
        core: &mut Core,
        services: &mut Services,
    ) -> OwnerStep<()> {
        if self.vertical.is_none() {
            let direction =
                if services.runtime.controllers.ports[0].held.0 & VERTICAL_DIRECTION_DOWN != 0 {
                    world::VerticalDirection::Down
                } else {
                    world::VerticalDirection::Up
                };
            let (transition, effects) = world::VerticalTransition::begin(
                direction,
                core.actors[0].position.screen_y,
                services.video.split_address,
                core.stream,
            );
            core.stream = transition.cursor;
            core.split_transition = transition.split_control;
            self.background = None;
            self.vertical = Some(VerticalCall {
                transition,
                work: effects.into(),
            });
        }
        loop {
            if self.vertical.as_ref().unwrap().work.is_empty() {
                if self.vertical.as_ref().unwrap().transition.counter == world::TRANSITION_TICKS {
                    self.vertical = None;
                    return OwnerStep::Complete(());
                }
                let call = self.vertical.as_mut().unwrap();
                call.transition.player_y = core.actors[0].position.screen_y;
                call.transition.split_address = services.video.split_address;
                call.transition.cursor = core.stream;
                call.work = call.transition.resume().into();
                core.actors[0].position.screen_y = call.transition.player_y;
                services.video.split_address = call.transition.split_address;
            }
            let effect = self
                .vertical
                .as_ref()
                .unwrap()
                .work
                .front()
                .unwrap()
                .clone();
            match effect {
                world::VerticalEffect::Yield => {
                    if self.wait(WaitOwner::VerticalYield, 1) == OwnerStep::Suspended {
                        return OwnerStep::Suspended;
                    }
                }
                world::VerticalEffect::ClearNonplayerActors => clear_nonplayer(core),
                world::VerticalEffect::HideSprites => services
                    .video
                    .composition
                    .hide(services.video.gameplay_display != 0),
                world::VerticalEffect::ComposeActors => {
                    select_bank(services, 5);
                    services.compose(core);
                }
                world::VerticalEffect::Build(phase) => {
                    let direction = self.vertical.as_ref().unwrap().transition.direction;
                    match phase {
                        world::VerticalPhase::BuildRoom => build_row(core, services, direction),
                        world::VerticalPhase::CopyStaged => copy_staged(core, services, direction),
                        world::VerticalPhase::BoundaryOrCopy => {
                            if core.stream.ring == LAST_ROOM_ROW {
                                build_row(
                                    core,
                                    services,
                                    if direction == world::VerticalDirection::Up {
                                        world::VerticalDirection::Down
                                    } else {
                                        world::VerticalDirection::Up
                                    },
                                );
                                core.stream.ring = core.stream.ring.wrapping_sub(1);
                            } else {
                                copy_staged(core, services, direction);
                            }
                        }
                        world::VerticalPhase::NextRoom => {
                            neighbor(core, services, direction, false)
                        }
                        world::VerticalPhase::PreviousRoom => {
                            neighbor(core, services, direction, true)
                        }
                        world::VerticalPhase::PollBackground => {
                            self.poll_background(core, services);
                        }
                    }
                }
                world::VerticalEffect::ResetSpawnWindow => {
                    // Source restores only the two pushed cursor bytes after
                    // the final actual yield, not when its last row is queued.
                    let saved = self.vertical.as_ref().unwrap().transition.saved_cursor;
                    core.stream.ring = saved.ring;
                    core.stream.room_in_row = saved.room_in_row;
                    let planes = &services.content.spawn_planes[&core.spawn_configuration];
                    core.spawn_window = actors::reset_window(
                        planes.room_starts[usize::from(core.row)],
                        &planes.x_high,
                        core.camera.x,
                    );
                    core.spawn_window.direction = 1;
                    select_bank(services, 6);
                }
                world::VerticalEffect::UpdateSpawnWindow => services.spawn(core),
                world::VerticalEffect::RefreshPalette => {
                    core.split_transition = 0;
                    services.video.display.palette_pending = true;
                }
            }
            self.vertical.as_mut().unwrap().work.pop_front();
        }
    }
}
fn expand(
    core: &mut Core,
    services: &mut Services,
    room: u8,
    ring: u8,
    parity: u8,
) -> world::ExpandedBlock {
    let content = std::sync::Arc::clone(&services.content);
    let layout = if usize::from(core.layout) < content.world.layouts.len() {
        &content.world.layouts[usize::from(core.layout)]
    } else {
        content.raw_layouts[usize::from(core.layout)]
            .as_ref()
            .expect("validated vertical layout")
    };
    let room_id =
        layout.rooms[usize::from(room)].expect("vertical room caller checked its room-list entry");
    core.stream.room_in_row = parity;
    let mut block = world::expand(
        &content.world.terrain[usize::from(core.terrain_bank)],
        world::Room::block(ring, &content.raw_rooms[room_id]),
        world::ExpansionContext {
            stage: core.progression.stage.0,
            room: core.stream.room,
            room_in_row: parity,
            ring,
            split: true,
            stage_three_control: core.progression.hit_dialogue_count,
            stage_four_control: core.progression.escort_prerequisite,
        },
    );
    select_bank(services, core.terrain_bank);
    services
        .video
        .retain_terrain_staging(&block.tiles, ring & LAST_ROOM_ROW == LAST_ROOM_ROW);
    block.tiles = services.video.terrain_staging;
    let coordinate =
        usize::from((ring & LAST_ROOM_ROW) * ROOM_BLOCKS + ((ring / ROOM_BLOCKS) & LAST_ROOM_ROW))
            | (usize::from(parity & 1) * SECOND_CACHE_PAGE);
    core.cache.collision[coordinate] = block.collision;
    block
}
fn byte(core: &mut Core, services: &mut Services, offset: usize, value: u8) {
    services
        .video
        .write_primary_byte(offset, value, &mut core.cache.attributes);
}
fn header(core: &mut Core, services: &mut Services, offset: usize, address: u16, count: u8) {
    for (field, value) in [(0, (address >> 8) as u8), (1, address as u8), (2, count)] {
        byte(core, services, offset + field, value);
    }
}
fn row_headers(core: &mut Core, services: &mut Services, ring: u8, lower: bool) {
    let address = SECOND_NAMETABLE
        + u16::from(ring & LAST_ROOM_ROW) * BLOCK_ROW_VIDEO_BYTES
        + if lower { HALF_BLOCK_VIDEO_BYTES } else { 0 };
    header(
        core,
        services,
        ROW_COMMANDS[0],
        address,
        (ROW_TILE_COUNT - 1) as u8,
    );
    header(
        core,
        services,
        ROW_COMMANDS[1],
        address | TILE_ROW_VIDEO_BYTES,
        (ROW_TILE_COUNT - 1) as u8,
    );
    header(
        core,
        services,
        ROW_COMMANDS[2],
        SECOND_NAMETABLE
            + NAMETABLE_ATTRIBUTE_OFFSET
            + u16::from(ring & LAST_ROOM_ROW) * u16::from(ROOM_BLOCKS),
        LAST_ROOM_ROW,
    );
    byte(core, services, ROW_TERMINATOR, u8::MAX);
    services.video.display.primary.pending = 0xff;
}
fn build_row(core: &mut Core, services: &mut Services, direction: world::VerticalDirection) {
    let ring = core.stream.ring & LAST_ROOM_ROW;
    let up = direction == world::VerticalDirection::Up;
    for column in 0..ROOM_BLOCKS {
        let block = expand(
            core,
            services,
            core.camera.room,
            ring + column * ROOM_BLOCKS,
            1,
        );
        let attribute_index =
            SECOND_CACHE_PAGE + usize::from(ring) * usize::from(ROOM_BLOCKS) + usize::from(column);
        let old = core.cache.attributes[attribute_index];
        // Source writes two upload planes and two retained secondary planes
        // for each block before replacing the cache's complete palette byte.
        for local in 0..4usize {
            let first = if up { 8 } else { 0 };
            let staged = first ^ 8;
            let at = usize::from(column) * 4 + local;
            byte(
                core,
                services,
                ROW_PAYLOADS[0] + at,
                block.tiles[first + local],
            );
            byte(
                core,
                services,
                ROW_PAYLOADS[1] + at,
                block.tiles[first + 4 + local],
            );
            byte(
                core,
                services,
                RETAINED_ROW_PAYLOADS[0] + at,
                block.tiles[staged + local],
            );
            byte(
                core,
                services,
                RETAINED_ROW_PAYLOADS[1] + at,
                block.tiles[staged + 4 + local],
            );
        }
        core.cache.attributes[attribute_index] = block.palette;
        let merged = if up {
            old & PALETTE_LOW_QUADRANTS | block.palette & PALETTE_HIGH_QUADRANTS
        } else {
            old & PALETTE_HIGH_QUADRANTS | block.palette & PALETTE_LOW_QUADRANTS
        };
        byte(
            core,
            services,
            ROW_ATTRIBUTE_PAYLOAD + usize::from(column),
            merged,
        );
    }
    core.stream.ring = ring;
    row_headers(core, services, ring, up);
}
fn copy_staged(core: &mut Core, services: &mut Services, direction: world::VerticalDirection) {
    for offset in (0..ROW_TILE_COUNT).rev() {
        let upper = services
            .video
            .primary_byte(RETAINED_ROW_PAYLOADS[0] + offset, &core.cache.attributes);
        let lower = services
            .video
            .primary_byte(RETAINED_ROW_PAYLOADS[1] + offset, &core.cache.attributes);
        byte(core, services, ROW_PAYLOADS[0] + offset, upper);
        byte(core, services, ROW_PAYLOADS[1] + offset, lower);
    }
    let ring = core.stream.ring & LAST_ROOM_ROW;
    for column in 0..usize::from(ROOM_BLOCKS) {
        byte(
            core,
            services,
            ROW_ATTRIBUTE_PAYLOAD + column,
            core.cache.attributes
                [SECOND_CACHE_PAGE + usize::from(ring) * usize::from(ROOM_BLOCKS) + column],
        );
    }
    core.stream.ring = ring.wrapping_add(if direction == world::VerticalDirection::Up {
        0xff
    } else {
        1
    }) & LAST_ROOM_ROW;
    row_headers(
        core,
        services,
        ring,
        direction != world::VerticalDirection::Up,
    );
}
fn neighbor(
    core: &mut Core,
    services: &mut Services,
    _direction: world::VerticalDirection,
    previous: bool,
) {
    let content = std::sync::Arc::clone(&services.content);
    let layout = if usize::from(core.layout) < content.world.layouts.len() {
        &content.world.layouts[usize::from(core.layout)]
    } else {
        content.raw_layouts[usize::from(core.layout)]
            .as_ref()
            .unwrap()
    };
    let page = (core.camera.x >> 8) as u8;
    let room = core
        .camera
        .room
        .wrapping_add(if previous { 0xff } else { 1 });
    let ring = core.stream.ring & LAST_ROOM_ROW;
    if (previous && page == 0)
        || (!previous && page == layout.columns.wrapping_sub(1))
        || layout
            .rooms
            .get(usize::from(room))
            .copied()
            .flatten()
            .is_none()
    {
        core.stream.ring = ring;
        return;
    }
    for column in 0..4u8 {
        let cursor = ring + if previous { 32 } else { 0 } + column * 8;
        let block = expand(core, services, room, cursor, 0);
        if ring == 0 {
            continue;
        }
        for (row, payload) in NEIGHBOR_PAYLOADS.iter().enumerate() {
            for tile in 0..4usize {
                byte(
                    core,
                    services,
                    payload + usize::from(column) * 4 + tile,
                    block.tiles[row * 4 + tile],
                );
            }
        }
        core.cache.attributes
            [usize::from(ring) * 8 + usize::from(column) + if previous { 4 } else { 0 }] =
            block.palette;
        byte(
            core,
            services,
            NEIGHBOR_ATTRIBUTE_PAYLOAD + usize::from(column),
            block.palette,
        );
    }
    core.stream.ring = ring;
    if ring == 0 {
        return;
    }
    let base =
        FIRST_NAMETABLE + u16::from(ring) * BLOCK_ROW_VIDEO_BYTES + if previous { 16 } else { 0 };
    for row in 0..4 {
        header(
            core,
            services,
            row * NEIGHBOR_COMMAND_BYTES,
            base + row as u16 * TILE_ROW_VIDEO_BYTES,
            15,
        );
    }
    header(
        core,
        services,
        NEIGHBOR_ATTRIBUTE_COMMAND,
        FIRST_NAMETABLE
            + NAMETABLE_ATTRIBUTE_OFFSET
            + u16::from(ring) * u16::from(ROOM_BLOCKS)
            + if previous { 4 } else { 0 },
        3,
    );
    byte(core, services, NEIGHBOR_TERMINATOR, u8::MAX);
    if ring == LAST_ROOM_ROW {
        for at in (0..8).rev() {
            let value = services
                .video
                .primary_byte(NEIGHBOR_ATTRIBUTE_COMMAND + at, &core.cache.attributes);
            byte(core, services, ROW_PAYLOADS[1] + at, value);
        }
    } else if ring == 1 {
        for at in 0..0x2e {
            let value = services
                .video
                .primary_byte(ROW_PAYLOADS[1] + at, &core.cache.attributes);
            byte(core, services, at, value);
        }
    }
    if ring == 1 || ring == LAST_ROOM_ROW {
        for at in 0..4 {
            let value = services
                .video
                .primary_byte(NEIGHBOR_PAYLOADS[2] + at, &core.cache.attributes)
                & if ring == 1 {
                    PALETTE_HIGH_QUADRANTS
                } else {
                    PALETTE_LOW_QUADRANTS
                };
            byte(core, services, NEIGHBOR_PAYLOADS[2] + at, value);
        }
    }
    services.video.display.primary.pending = 0xff;
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Core, Services) {
        use romatlas_port::assets::{MemoryAssets, Rom};
        let bytes =
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap();
        let rom = Rom::from_bytes(bytes, crate::config::ROM_SHA256).unwrap();
        let overrides = MemoryAssets::default();
        let assets = crate::assets::Assets::load(&rom, &overrides).unwrap();
        (
            Core::reset(),
            Services::new(
                super::super::super::content::Content::load(&rom, &assets, &overrides).unwrap(),
            ),
        )
    }
    #[test]
    fn copy_phase_uses_retained_secondary_rows_and_current_full_cache_attributes() {
        let (mut core, mut services) = fixture();
        core.stream.ring = 3;
        for offset in 0..32 {
            byte(&mut core, &mut services, 0x60 + offset, offset as u8);
            byte(&mut core, &mut services, 0x80 + offset, 0x80 | offset as u8);
        }
        core.cache.attributes[88..96].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        copy_staged(&mut core, &mut services, world::VerticalDirection::Down);
        assert_eq!(core.stream.ring, 4);
        for offset in 0..32 {
            assert_eq!(
                services
                    .video
                    .primary_byte(3 + offset, &core.cache.attributes),
                offset as u8
            );
            assert_eq!(
                services
                    .video
                    .primary_byte(0x26 + offset, &core.cache.attributes),
                0x80 | offset as u8
            );
        }
        assert_eq!(
            &services.video.display.secondary.encoded[9..17],
            &[1, 2, 3, 4, 5, 6, 7, 8]
        );
        assert_eq!(
            &services.video.display.primary.encoded[..3],
            &[0x25, 0xc0, 31]
        );
        assert_eq!(services.video.display.secondary.encoded[17], 0xff);
    }
    #[test]
    fn vertical_call_restores_saved_cursors_only_after_its_last_actual_wait() {
        let (mut core, mut services) = fixture();
        // A committed transition has already validated its target room. Reset
        // layout state can point at FF; choose an actual original target entry.
        let layout = &services.content.world.layouts[usize::from(core.layout)];
        let target = layout.rooms.iter().position(Option::is_some).unwrap() as u8;
        core.camera.room = target;
        core.row = target / layout.columns;
        core.camera.x = u16::from(target % layout.columns) << 8;
        core.stream = world::StreamCursor {
            ring: 0x2a,
            room: target,
            room_in_row: 2,
        };
        services.runtime.controllers.ports[0].held = crate::runtime::Buttons::DOWN;
        let mut tasks = Tasks::default();
        assert_eq!(
            tasks.vertical_transition(&mut core, &mut services),
            OwnerStep::Suspended
        );
        assert_eq!((core.stream.ring, core.split_transition), (0, 12));
        for _ in 0..world::TRANSITION_TICKS {
            assert_eq!(
                tasks.vertical_transition(&mut core, &mut services),
                OwnerStep::Suspended
            );
        }
        assert_ne!(
            core.split_transition, 0,
            "final queued row still awaits its original yield"
        );
        assert_eq!(
            tasks.vertical_transition(&mut core, &mut services),
            OwnerStep::Complete(())
        );
        assert_eq!(
            (
                core.stream.ring,
                core.stream.room_in_row,
                core.split_transition
            ),
            (0x2a, 2, 0)
        );
        assert!(services.video.display.palette_pending);
        assert!(tasks.vertical.is_none());
    }
}

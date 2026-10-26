//! The source's sixty-tick vertical call. Tile staging remains in the same
//! secondary queue bytes used by audio/composites; this continuation owns only
//! the phase/cursor lifetime and the currently entered wait.
use super::*;
// Room geometry stays in the transition owner; source upload framing and
// retained byte-plane access belong to Video.
const ROOM_BLOCKS: u8 = 8;
const LAST_ROOM_ROW: u8 = ROOM_BLOCKS - 1;
const SECOND_CACHE_PAGE: usize = world::CACHE_CELLS / 2;
const VERTICAL_DIRECTION_DOWN: u8 = 4;
#[derive(Clone, serde::Serialize, serde::Deserialize)]
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
            services.video.vertical_view = Some(super::super::widescreen::VerticalView {
                from_row: if direction == world::VerticalDirection::Down {
                    core.row.saturating_sub(1)
                } else {
                    core.row.saturating_add(1)
                },
                shift: 0,
                left: services.video.wide_terrain.left,
            });
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
                    services.video.vertical_view = None;
                    return OwnerStep::Complete(());
                }
                let call = self.vertical.as_mut().unwrap();
                call.transition.player_y = core.actors[0].position.screen_y;
                call.transition.split_address = services.video.split_address;
                call.transition.cursor = core.stream;
                call.work = call.transition.resume().into();
                if let Some(view) = services.video.vertical_view.as_mut() {
                    let distance = i16::from(call.transition.counter.div_ceil(2)) * 8;
                    view.shift = if call.transition.direction == world::VerticalDirection::Down {
                        distance
                    } else {
                        -distance
                    };
                }
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
                world::VerticalEffect::HideSprites => services.video.hide_sprites(),
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
#[cfg(test)]
fn byte(core: &mut Core, services: &mut Services, offset: usize, value: u8) {
    services
        .video
        .write_primary_byte(offset, value, &mut core.cache.attributes);
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
        let update = world::vertical_block(direction, ring, column, &block, old);
        services.video.stage_vertical_block(
            &update,
            (attribute_index, block.palette),
            &mut core.cache.attributes,
        );
    }
    core.stream.ring = ring;
    services
        .video
        .publish_vertical_row(ring, up, &mut core.cache.attributes);
}
fn copy_staged(core: &mut Core, services: &mut Services, direction: world::VerticalDirection) {
    let ring = core.stream.ring & LAST_ROOM_ROW;
    services
        .video
        .restore_vertical_row(ring, &mut core.cache.attributes);
    core.stream.ring = ring.wrapping_add(if direction == world::VerticalDirection::Up {
        0xff
    } else {
        1
    }) & LAST_ROOM_ROW;
    services.video.publish_vertical_row(
        ring,
        direction != world::VerticalDirection::Up,
        &mut core.cache.attributes,
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
        let page = world::TilePage::First;
        let update = world::ColumnUpdate {
            origin: world::TileOrigin {
                page,
                column: (if previous { 16 } else { 0 }) + column * 4,
                row: ring * 4,
            },
            tiles: block.tiles,
            rows: world::BlockRows::All,
            palette: world::PaletteRun {
                page,
                first_cell: ring * 8 + column + if previous { 4 } else { 0 },
                values: vec![block.palette],
            },
            commit: crate::assets::formats::UploadCommit::AtInterrupt,
        };
        services
            .video
            .stage_neighbor_block(&update, &mut core.cache.attributes);
    }
    core.stream.ring = ring;
    if ring == 0 {
        return;
    }
    services.video.publish_neighbor_row(
        world::TileOrigin {
            page: world::TilePage::First,
            column: if previous { 16 } else { 0 },
            row: ring * 4,
        },
        world::neighbor_visible_rows(ring),
        &mut core.cache.attributes,
    );
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
    fn patterned_fixture() -> (Core, Services) {
        let (mut core, mut services) = fixture();
        let content = std::sync::Arc::make_mut(&mut services.content);
        content.world.layouts[0].columns = 3;
        content.world.layouts[0].rooms.fill(Some(0));
        content.raw_rooms[0].low.fill(0);
        content.raw_rooms[0].ninth_bits.fill(0);
        let set = &mut content.world.terrain[0];
        set.blocks[0] = [1, 2, 3, 4];
        set.tiles[1] = [11, 12, 13, 14];
        set.tiles[2] = [21, 22, 23, 24];
        set.tiles[3] = [31, 32, 33, 34];
        set.tiles[4] = [41, 42, 43, 44];
        set.properties[..5].copy_from_slice(&[0, 0x10, 0x21, 0x32, 0x03]);
        core.cache.attributes.fill(0xa5);
        (core, services)
    }
    #[test]
    fn vertical_native_blocks_publish_selected_rows_and_retain_complete_cache_palettes() {
        // CAA0..CB5A: selected two planes, opposite retained planes, full cache
        // palette and a direction-specific outgoing nibble merge.
        for (direction, low, visible, retained, palette) in [
            (
                world::VerticalDirection::Up,
                0xc0,
                [31, 32, 41, 42, 33, 34, 43, 44],
                [11, 12, 21, 22, 13, 14, 23, 24],
                0xe5,
            ),
            (
                world::VerticalDirection::Down,
                0x80,
                [11, 12, 21, 22, 13, 14, 23, 24],
                [31, 32, 41, 42, 33, 34, 43, 44],
                0xa4,
            ),
        ] {
            let (mut core, mut services) = patterned_fixture();
            core.stream.ring = 3;
            build_row(&mut core, &mut services, direction);
            assert_eq!(
                &services.video.display.primary.encoded[..3],
                &[0x25, low, 31]
            );
            for column in 0..8 {
                for local in 0..4 {
                    for (offset, value) in [
                        (3, visible[local]),
                        (0x26, visible[4 + local]),
                        (0x60, retained[local]),
                        (0x80, retained[4 + local]),
                    ] {
                        assert_eq!(
                            services
                                .video
                                .primary_byte(offset + column * 4 + local, &core.cache.attributes),
                            value
                        );
                    }
                }
                assert_eq!(
                    services
                        .video
                        .primary_byte(0x49 + column, &core.cache.attributes),
                    palette
                );
            }
            assert_eq!(&core.cache.attributes[88..96], &[0xe4; 8]);
            assert_eq!(&core.cache.collision[88..96], &[0x39; 8]);
            assert_eq!(
                services.video.primary_byte(0x51, &core.cache.attributes),
                0xff
            );
            assert_eq!(services.video.display.primary.pending, 0xff);
        }
    }
    #[test]
    fn native_neighbor_edges_keep_full_cache_but_publish_the_source_trimmed_queue() {
        // CB8E..CC82: row1 copies the lower planes upward; row7 moves the
        // attribute command over the unused planes. Both keep full cache data.
        for (ring, first_low, palette, cache_palette) in
            [(1, 0xc0, 0xe0, 0xe4), (7, 0x80, 0x04, 0x04)]
        {
            for previous in [false, true] {
                let (mut core, mut services) = patterned_fixture();
                core.stream.ring = ring;
                core.camera.x = 256;
                core.camera.room = 1;
                neighbor(
                    &mut core,
                    &mut services,
                    world::VerticalDirection::Up,
                    previous,
                );
                assert_eq!(
                    &services.video.display.primary.encoded[..3],
                    &[
                        if ring == 1 { 0x20 } else { 0x23 },
                        first_low + if previous { 16 } else { 0 },
                        15
                    ]
                );
                assert_eq!(
                    &services.video.display.primary.encoded[0x26..0x29],
                    &[0x23, 0xc0 + ring * 8 + if previous { 4 } else { 0 }, 3]
                );
                for column in 0..4 {
                    assert_eq!(
                        services
                            .video
                            .primary_byte(0x29 + column, &core.cache.attributes),
                        palette
                    );
                    assert_eq!(
                        core.cache.attributes
                            [usize::from(ring) * 8 + column + if previous { 4 } else { 0 }],
                        cache_palette
                    );
                }
                assert_eq!(
                    services.video.primary_byte(0x2d, &core.cache.attributes),
                    0xff
                );
                assert_eq!(services.video.display.primary.pending, 0xff);
            }
        }
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

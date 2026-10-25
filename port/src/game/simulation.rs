//! Canonical simulation owners and source interrupt/foreground joins. Temporary
//! views borrow these fields; adapters never keep a second actor pool or queue.
use super::{content::Content, video::Video};
use crate::{actors, audio, player, presentation, progression, runtime, selection, status, world};

#[derive(Clone)]
pub struct Core {
    pub progression: progression::Progression,
    pub player: player::Player,
    pub actors: actors::ActorPool,
    pub camera: world::Camera,
    pub cache: world::WorldCache,
    pub status: status::Status,
    pub row: u8,
    pub terrain_bank: u8,
    pub layout: u8,
    pub stream: world::StreamCursor,
    pub scroll_direction: u8,
    /// Previous terrain-stream direction29 and spawn direction4B have distinct
    /// lifetimes from player/camera scroll direction38.
    pub previous_scroll_direction: u8,
    pub spawn_configuration: u8,
    pub camera_displacement: u8,
    pub secondary_cursor: u8,
    pub split_transition: u8,
    pub spawn_window: actors::Window,
    pub suppression: [u8; actors::SUPPRESSION_BYTES],
    pub special_spawn: actors::SpecialControl,
}
impl Core {
    pub fn reset() -> Self {
        Self {
            progression: progression::Progression::default(),
            player: player::Player::default(),
            actors: [actors::ActorContact::default(); actors::ACTOR_SLOTS],
            camera: world::Camera::default(),
            cache: world::WorldCache::default(),
            status: status::Status {
                reward: [0; 6],
                total: [0; 7],
                lives: 0,
                health: status::Health {
                    segments: [status::HealthSegment {
                        y: 0,
                        tile: 0,
                        depleted: 0,
                        x: 0,
                    }; status::OVERLAY_SEGMENTS],
                    end: 0,
                    alternating: 0,
                },
                countdown: [0; 3],
                divider: 0,
                dirty: 0,
                ui_pending: 0,
            },
            row: 0,
            terrain_bank: 0,
            layout: 0,
            stream: world::StreamCursor::default(),
            scroll_direction: 0,
            previous_scroll_direction: 0,
            spawn_configuration: 0,
            camera_displacement: 0,
            secondary_cursor: 0,
            split_transition: 0,
            spawn_window: actors::Window::default(),
            suppression: [0; actors::SUPPRESSION_BYTES],
            special_spawn: actors::SpecialControl::Ready,
        }
    }
}
pub struct Services {
    pub content: std::sync::Arc<Content>,
    pub events: crate::events::Events,
    pub runtime: runtime::Runtime,
    pub video: Video,
    pub commands: audio::CommandQueue,
    pub collision_outcome: crate::collision::CollisionOutcome,
    pub audio: audio::Driver,
    pub synthesizer: audio::Synthesizer,
    pub option: selection::TitleOption,
    pub dialogue: crate::dialogue::Dialogue,
    pub departure_signal: u8,
    pub scene_cursor: u8,
}
// A snapshot must fork the shared journal and rebind the command callback.
// Ordinary Events::clone is deliberately a shared handle within ONE simulation.
impl Clone for Services {
    fn clone(&self) -> Self {
        let events = self.events.fork();
        let mut commands = self.commands.clone();
        commands.observe_with(events.clone());
        Self {
            content: self.content.clone(),
            events,
            commands,
            runtime: self.runtime.clone(),
            video: self.video.clone(),
            collision_outcome: self.collision_outcome,
            audio: self.audio.clone(),
            synthesizer: self.synthesizer.clone(),
            option: self.option,
            dialogue: self.dialogue.clone(),
            departure_signal: self.departure_signal,
            scene_cursor: self.scene_cursor,
        }
    }
}
impl Services {
    #[cfg(test)]
    pub fn new(content: Content) -> Self {
        Self::with_content(std::sync::Arc::new(content))
    }
    pub fn reset(&mut self) {
        let events = self.events.clone();
        *self = Self::with_content(std::sync::Arc::clone(&self.content));
        self.events = events;
        self.commands.observe_with(self.events.clone());
        self.events.record("SourceReset", serde_json::json!({}));
    }
    pub(super) fn with_content(content: std::sync::Arc<Content>) -> Self {
        let events = crate::events::Events::new(crate::config::SEMANTIC_EVENT_CAPACITY);
        let mut commands = audio::CommandQueue::default();
        commands.observe_with(events.clone());
        Self {
            content,
            events,
            runtime: runtime::Runtime::reset().0,
            video: Video::reset(),
            commands,
            collision_outcome: crate::collision::CollisionOutcome::default(),
            audio: audio::Driver::default(),
            synthesizer: audio::Synthesizer::default(),
            option: selection::TitleOption::Easy,
            dialogue: crate::dialogue::Dialogue::default(),
            departure_signal: 0,
            scene_cursor: 0,
        }
    }
    /// Exactly one source interrupt precedes foreground work at a host boundary.
    /// Display suspension suppresses commits, not task wake or audio sequencing.
    pub fn interrupt(&mut self, core: &mut Core) {
        use runtime::FrameEffect as E;
        let effects = self.runtime.interrupt(
            runtime::InterruptSource::Nmi,
            runtime::InterruptContext {
                display_suspend_depth: self.video.display.suspended,
                gameplay_display: self.video.gameplay_display != 0,
                oam_guard: self.video.oam_guard != 0,
                primary_pending: self.video.display.primary.pending != 0,
                secondary_pending: self.video.display.secondary_pending,
                palette_pending: self.video.display.palette_pending,
                player_screen_y: core.actors[0].position.screen_y,
                ending_layout: core.layout == 9,
                alternate_split_address: core.split_transition != 0,
                race_intro: core.progression.race_intro != 0,
            },
        );
        self.video.raster.apply(
            &effects,
            presentation::RasterInputs {
                split_y: self.video.split_y,
                ui_x: self.video.ui_x,
                ui_y: self.video.ui_y,
                camera_x: core.camera.x,
                playfield_y: self.video.playfield_y,
                control: self.video.display.control,
                mask: self.video.display.mask,
                split_address: self.video.split_address,
            },
        );
        for effect in effects {
            match effect {
                E::PrimeSplitSprite => {
                    self.video.composition.oam[0] = self.video.raster.split_sprite
                }
                E::CopyOam => self.video.committed_oam = self.video.composition.snapshot(),
                E::FlushPrimary => {
                    let bytes = self.video.primary_bytes(&core.cache.attributes);
                    self.video.display.flush(
                        presentation::Queue::Primary,
                        &bytes,
                        presentation::Stride::Across,
                    );
                }
                E::FlushSecondary => {
                    let bytes = self
                        .video
                        .display
                        .secondary
                        .shared_bytes(&core.cache.attributes);
                    self.video.display.flush(
                        presentation::Queue::Secondary,
                        &bytes,
                        presentation::Stride::Across,
                    );
                }
                E::UploadPalette => self.video.display.commit_palette(),
                E::AdvanceAudioFrame => {
                    let video = &self.video;
                    let writes = self.audio.interrupt_with_alias(
                        &mut self.commands,
                        &self.content.audio,
                        core.progression.status_selector,
                        &mut |address| video.audio_alias(address),
                    );
                    self.synthesizer.apply(&writes);
                }
                _ => {}
            }
        }
        self.video.commit_writes();
        self.events.record("InterruptCommitted", serde_json::json!({"interrupt_tick":self.runtime.interrupt_tick,"audio_frame":self.audio.frames,"display_suspended":self.video.display.suspended,"oam_guard":self.video.oam_guard,"primary_pending":self.video.display.primary.pending,"secondary_pending":self.video.display.secondary_pending,"palette_pending":self.video.display.palette_pending}));
    }
    pub fn copy_base_overlays(&mut self, core: &mut Core) {
        core.status
            .total
            .copy_from_slice(&self.content.stage_ui_template[4..11]);
        for (segment, bytes) in core
            .status
            .health
            .segments
            .iter_mut()
            .zip(self.content.health_template.chunks_exact(4))
        {
            *segment = status::HealthSegment {
                y: bytes[0],
                tile: bytes[1],
                depleted: bytes[2],
                x: bytes[3],
            };
        }
        core.status.health.end = *self
            .content
            .health_template
            .last()
            .expect("health terminator");
    }
    pub fn compose(&mut self, core: &mut Core) {
        compose_owners(
            &self.content,
            &mut self.video,
            &mut core.actors,
            &core.player,
            &core.status,
            &core.progression,
            &core.camera,
            &core.cache.attributes,
        );
    }
}

/// Compose directly from canonical borrowed owners. Dialogue/lifecycle contexts
/// can use this entry without assembling a temporary replacement Core.
// Separate borrows keep canonical owners visible across immediate callbacks.
#[allow(clippy::too_many_arguments)]
pub fn compose_owners(
    content: &Content,
    video: &mut Video,
    actors: &mut actors::ActorPool,
    player: &player::Player,
    status: &status::Status,
    progression: &progression::Progression,
    camera: &world::Camera,
    attributes: &[u8; world::CACHE_CELLS],
) {
    let mut health = Vec::new();
    for segment in status.health.segments {
        if segment.y == u8::MAX {
            break;
        }
        health.extend([segment.y, segment.tile, segment.depleted, segment.x]);
    }
    health.push(u8::MAX);
    let mut title = content.title_overlay.clone();
    title[9] = status.lives;
    let mut timer = match video.timer_layout {
        super::video::TimerLayout::Countdown => content.timer_overlay.clone(),
        super::video::TimerLayout::GameOver => content.progression.game_over_overlay.clone(),
    };
    for (index, digit) in status.countdown.into_iter().enumerate() {
        timer[17 + index * 4] = digit;
    }
    let order = video.composition.begin(
        presentation::OverlayGate {
            gameplay_latch: video.overlay_gate,
            dialogue_cursor: progression.dialogue_cursor,
            ending: progression.ending_latch,
            status_stream: progression.status_selector,
            player_state: player.action as u8,
            player_y: actors[0].position.screen_y,
        },
        [&health, &title, &timer],
    );
    for slot in order {
        let actor = &mut actors[slot];
        // Apply eligibility directly to canonical actors before borrowing
        // each visual view. No second visual actor array is retained.
        if actor.state & actors::ACTIVE == 0 {
            actor.render = 0;
            actor.spawn_identity = None;
            continue;
        }
        if actor.render == presentation::HIDDEN_RENDER {
            continue;
        }
        let mut visual = presentation::ActorVisual {
            state: actor.state,
            render: actor.render,
            world_x: actor.position.world_x,
            screen_x: actor.screen_x,
            screen_y: actor.position.screen_y,
            animation: actor.animation,
            tick: actor.animation_tick,
            frame: actor.animation_frame,
            restart: actor.animation_restart,
            spawn_identity: actor.spawn_identity,
        };
        if slot == 0 {
            presentation::Composition::player_flags(
                &mut visual,
                player.protection,
                player.special_mode,
            );
        }
        if video.composition.project(&mut visual, camera.x) {
            let sequence = &content.presentation.animations[usize::from(visual.animation)];
            if let Some(frame) =
                presentation::Composition::advance(&mut visual, sequence, player.action as u8)
            {
                if frame < 4 {
                    let shared = video.display.secondary.shared_bytes(attributes);
                    let record = &shared[presentation::RUNTIME_COMPOSITE_QUEUE_OFFSET..];
                    let composite = presentation::decode_composite(
                        record,
                        &content.runtime_coordinates,
                        visual.render & presentation::FLIP_HORIZONTAL != 0,
                    );
                    video.composition.emit(slot, &visual, &composite);
                } else {
                    let frames = if visual.render & presentation::FLIP_HORIZONTAL != 0 {
                        &content.presentation.flipped_composites
                    } else {
                        &content.presentation.composites
                    };
                    video
                        .composition
                        .emit(slot, &visual, &frames[usize::from(frame)]);
                }
            }
        }
        actor.state = visual.state;
        actor.render = visual.render;
        actor.screen_x = visual.screen_x;
        actor.animation_tick = visual.tick;
        actor.animation_frame = visual.frame;
        actor.spawn_identity = visual.spawn_identity;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{assets::Assets, config};
    use romatlas_port::assets::{MemoryAssets, Rom};
    // QA E: D9CA writes TerrainRoomInRow, while C710 consults its retained
    // value before resetting its own cursors. D79B and D7D1 call C710 twice.
    #[test]
    fn stage_presentation_restores_in_row_from_preset_without_resetting_other_stream_fields() {
        let rom = Rom::from_bytes(
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap(),
            config::ROM_SHA256,
        )
        .unwrap();
        let overrides = MemoryAssets::default();
        let raw = Assets::load(&rom, &overrides).unwrap();
        let mut services = Services::new(Content::load(&rom, &raw, &overrides).unwrap());
        let mut core = Core::reset();
        core.stream = world::StreamCursor {
            room: 19,
            ring: 27,
            room_in_row: 7,
        };
        services.load_stage_presentation(&mut core, 0);
        assert_eq!(
            core.stream.room_in_row, 0,
            "D9CA installs preset page into TerrainRoomInRow"
        );
        assert_eq!(
            (core.stream.room, core.stream.ring),
            (19, 27),
            "D9CA does not reset the other cursors"
        );
    }

    #[test]
    fn repeated_initial_build_reads_retained_in_row_and_preserves_preceding_room_far_cache() {
        const AMAZON_STAGE: u8 = 0;
        const PRECEDING_ROOM: usize = 39;
        const CAMERA_ROOM: usize = 40;
        const NEXT_ROOM: usize = 41;
        const FAR_CACHE_CELL: usize = 64 + 5;
        const PRECEDING_BLOCK: u8 = 1;
        const CAMERA_BLOCK: u8 = 2;
        const NEXT_BLOCK: u8 = 3;
        const PRECEDING_COLLISION: u8 = 0x20;
        const CAMERA_COLLISION: u8 = 0x10;
        const NEXT_COLLISION: u8 = 0x30;
        let rom = Rom::from_bytes(
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap(),
            config::ROM_SHA256,
        )
        .unwrap();
        let overrides = MemoryAssets::default();
        let raw = Assets::load(&rom, &overrides).unwrap();
        let mut content = Content::load(&rom, &raw, &overrides).unwrap();
        // Source Amazon's first build leaves room_in_row1. The second starts
        // from camera room40 but sees neighbors39/41, choosing the128 visits
        // that retain preceding-room columns in the unused opposite half.
        for (room, payload, block) in [
            (PRECEDING_ROOM, 0, PRECEDING_BLOCK),
            (CAMERA_ROOM, 1, CAMERA_BLOCK),
            (NEXT_ROOM, 2, NEXT_BLOCK),
        ] {
            content.world.layouts[usize::from(AMAZON_STAGE)].rooms[room] = Some(payload);
            content.raw_rooms[payload].low.fill(block);
            content.raw_rooms[payload].ninth_bits.fill(0);
        }
        let terrain_bank =
            usize::from(content.raw_stage_presets[usize::from(AMAZON_STAGE)].terrain_bank);
        for (block, property) in [
            (PRECEDING_BLOCK, PRECEDING_COLLISION),
            (CAMERA_BLOCK, CAMERA_COLLISION),
            (NEXT_BLOCK, NEXT_COLLISION),
        ] {
            content.world.terrain[terrain_bank].blocks[usize::from(block)] = [block; 4];
            content.world.terrain[terrain_bank].properties[usize::from(block)] = property;
        }
        let mut services = Services::new(content);
        let mut core = Core::reset();
        core.progression.stage.0 = AMAZON_STAGE;
        services.video.display.suspended = 1;
        services.load_stage_presentation(&mut core, AMAZON_STAGE);
        services.build_terrain(&mut core, false);
        assert_eq!(core.stream.room_in_row, 1);
        assert_eq!(core.cache.collision[FAR_CACHE_CELL], 0);
        services.build_terrain(&mut core, false);
        assert_eq!(
            core.cache.collision[FAR_CACHE_CELL], 0xaa,
            "second source initialization must preserve preceding-room class2 quadrants"
        );
    }

    #[test]
    fn moon_initial_attribute_cells_match_source_palette_bytes() {
        let rom = Rom::from_bytes(
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap(),
            config::ROM_SHA256,
        )
        .unwrap();
        let overrides = MemoryAssets::default();
        let raw = Assets::load(&rom, &overrides).unwrap();
        let mut services = Services::new(Content::load(&rom, &raw, &overrides).unwrap());
        let mut core = Core::reset();
        core.progression.stage.0 = 4;
        services.video.display.suspended = 1;
        core.cache.attributes.fill(0xaa);
        services.load_stage_presentation(&mut core, 4);
        services.build_terrain(&mut core, false);
        services.build_terrain(&mut core, false);
        core.spawn_configuration = 4;
        let planes = &services.content.spawn_planes[&4];
        core.spawn_window =
            actors::reset_window(planes.room_starts[4], &planes.x_high, core.camera.x);
        core.spawn_window.direction = 1;
        core.special_spawn = actors::SpecialControl::Cooldown(1);
        services.spawn(&mut core);
        assert!(
            core.actors
                .iter()
                .any(|actor| actor.spawn_identity == Some(121))
        );
        assert_eq!(
            [core.cache.attributes[46], core.cache.attributes[47]],
            [0, 0]
        );
    }

    #[test]
    fn transposed_bottom_ring_seven_retains_lower_staging_but_ring_thirty_eight_rewrites_it() {
        let bytes =
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap();
        let rom = Rom::from_bytes(bytes, config::ROM_SHA256).unwrap();
        let overrides = MemoryAssets::default();
        let raw = Assets::load(&rom, &overrides).unwrap();
        let mut services = Services::new(Content::load(&rom, &raw, &overrides).unwrap());
        let mut core = Core::reset();
        core.layout = 5;
        services.video.display.suspended = 1;
        services.video.terrain_staging = [0x79; 16];
        services.build_column(
            &mut core,
            world::StreamCursor {
                ring: 7,
                room: 0,
                room_in_row: 0,
            },
        );
        assert_eq!(&services.video.terrain_staging[8..], &[0x79; 8]);
        let expected = world::expand(
            &services.content.world.terrain[0],
            world::Room::block(
                0x38,
                &services.content.raw_rooms[services.content.world.layouts[5].rooms[0].unwrap()],
            ),
            world::ExpansionContext {
                stage: 5,
                room: 0,
                room_in_row: 0,
                ring: 0x38,
                ..world::ExpansionContext::default()
            },
        );
        services.build_column(
            &mut core,
            world::StreamCursor {
                ring: 0x38,
                room: 0,
                room_in_row: 0,
            },
        );
        assert_eq!(services.video.terrain_staging, expected.tiles);
        services.video.terrain_staging = [0x79; 16];
        services.build_column(
            &mut core,
            world::StreamCursor {
                ring: 0,
                room: 0,
                room_in_row: 1,
            },
        );
        assert_eq!(services.video.terrain_staging, [0x79; 16]);
    }
    #[test]
    fn stage_four_terrain_override_reads_escort_progression_instead_of_reward_flags() {
        const STAGE: usize = 4;
        const SPECIAL_ROOM: u8 = 0x27;
        const OVERRIDE_RING: u8 = 0x1c;
        let bytes =
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap();
        let rom = Rom::from_bytes(bytes, config::ROM_SHA256).unwrap();
        let overrides = MemoryAssets::default();
        let raw = Assets::load(&rom, &overrides).unwrap();
        let mut content = Content::load(&rom, &raw, &overrides).unwrap();
        content.world.layouts[STAGE].columns = 8;
        content.world.layouts[STAGE].rooms[usize::from(SPECIAL_ROOM)] = Some(0);
        content.raw_rooms[0].low.fill(0);
        content.raw_rooms[0].ninth_bits.fill(0);
        content.world.terrain[0].blocks[0] = [1; 4];
        content.world.terrain[0].tiles[1] = [5; 4];
        content.world.terrain[0].tiles[0xd6] = [9; 4];
        let mut services = Services::new(content);
        let mut core = Core::reset();
        core.layout = STAGE as u8;
        core.progression.stage.0 = STAGE as u8;
        core.progression.escort_prerequisite = 0x80;
        services.video.display.suspended = 1;
        services.build_column(
            &mut core,
            world::StreamCursor {
                ring: OVERRIDE_RING,
                room: SPECIAL_ROOM,
                room_in_row: 7,
            },
        );
        assert_eq!(services.video.terrain_staging[2], 9);
        core.progression.escort_prerequisite = 0;
        core.progression.reward_progress = 0x80;
        services.build_column(
            &mut core,
            world::StreamCursor {
                ring: OVERRIDE_RING,
                room: SPECIAL_ROOM,
                room_in_row: 7,
            },
        );
        assert_eq!(services.video.terrain_staging[2], 5);
    }

    #[test]
    fn camera_batches_two_columns_before_any_interrupt_and_preserves_spawn_direction() {
        let rom = Rom::from_bytes(
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap(),
            config::ROM_SHA256,
        )
        .unwrap();
        let overrides = MemoryAssets::default();
        let raw = Assets::load(&rom, &overrides).unwrap();
        let mut content = Content::load(&rom, &raw, &overrides).unwrap();
        content.world.layouts[0].columns = 8;
        content.world.layouts[0].rooms.fill(Some(0));
        let mut services = Services::new(content);
        let mut core = Core::reset();
        core.camera = world::Camera {
            x: 0x100,
            room: 1,
            previous_x: 0,
        };
        core.actors[0].position.world_x = 0x188;
        core.scroll_direction = 1;
        core.previous_scroll_direction = 1;
        core.spawn_window.direction = 2;
        core.stream = world::StreamCursor {
            ring: 0x10,
            room: 1,
            room_in_row: 1,
        };
        services.camera(&mut core);
        assert_eq!(
            (core.camera.x, core.camera_displacement, core.stream.ring),
            (0x108, 8, 0x12)
        );
        assert_eq!(
            &services.video.display.primary.encoded[..3],
            &[0x24, 0x88, 3]
        );
        assert_eq!(
            &services.video.display.primary.encoded[32..35],
            &[0x25, 0x08, 3]
        );
        assert_eq!(
            services.video.display.secondary.encoded[0],
            u8::MAX,
            "two complete32-byte columns terminate in the canonical secondary prefix"
        );
        assert_eq!(
            (
                services.video.display.primary.pending,
                core.spawn_window.direction,
                services.runtime.interrupt_tick
            ),
            (u8::MAX, 2, 0)
        );
    }

    #[test]
    fn column_byte_indices_wrap_quadrant_and_attribute_cursors_but_not_absolute_fields() {
        let rom = Rom::from_bytes(
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap(),
            config::ROM_SHA256,
        )
        .unwrap();
        let overrides = MemoryAssets::default();
        let raw = Assets::load(&rom, &overrides).unwrap();
        let mut content = Content::load(&rom, &raw, &overrides).unwrap();
        content.world.layouts[0].columns = 8;
        content.world.layouts[0].rooms.fill(Some(0));
        content.raw_rooms[0].low.fill(0);
        content.raw_rooms[0].ninth_bits.fill(0);
        content.world.terrain[0].blocks[0] = [1; 4];
        content.world.terrain[0].tiles[1] = [1, 2, 3, 4];
        content.world.terrain[0].properties[1] = 0x11;
        let mut services = Services::new(content);
        let mut core = Core::reset();
        services.video.display.primary.encoded[5] = 0x40;
        let next = services.build_column_at(
            &mut core,
            world::StreamCursor {
                ring: 0x12,
                room: 1,
                room_in_row: 1,
            },
            0xf0,
        );
        assert_eq!(next, Some(0x10));
        assert_eq!(
            &services.video.display.primary.encoded[0x0c..0x0f],
            &[0x63, 0xd2, 0],
            "wrapped attributeY reads its actual retained staging/primary alias"
        );
        assert_eq!(
            (
                services.video.display.primary.encoded[3],
                services.video.display.primary.encoded[10]
            ),
            (1, 3),
            "quadrant payloadY wraps before its absolute payload fields"
        );
        assert_eq!(
            (
                core.cache.attributes[5],
                core.cache.attributes[6],
                core.cache.attributes[7]
            ),
            (0x25, 0x68, 3),
            "row headers add absolute operand offsets after the byte cursor"
        );
        assert_eq!(services.video.display.primary.pending, u8::MAX);
        core.progression.stage.0 = 5;
        services.build_column_at(
            &mut core,
            world::StreamCursor {
                ring: 0x28,
                room: 0,
                room_in_row: 0,
            },
            0xf0,
        );
        assert_eq!(
            core.cache.attributes[5], 0x55,
            "canonical cache store follows an aliased row header"
        );
        core.cache.attributes[0] = 0x77;
        let next = services.build_column_record(
            &mut core,
            world::StreamCursor {
                ring: 0x12,
                room: 1,
                room_in_row: 1,
            },
            0xe0,
            false,
        );
        assert_eq!(
            (next, core.cache.attributes[0]),
            (Some(0), 0x77),
            "intermediate record cannot append a native-only terminator into cachecell0"
        );
        services.build_column_at(
            &mut core,
            world::StreamCursor {
                ring: 0,
                room: 0,
                room_in_row: 0,
            },
            0xe0,
        );
        assert_eq!(
            core.cache.attributes[0],
            u8::MAX,
            "final terminator follows canonical cache store and aliases cell0"
        );
    }

    #[test]
    fn composition_commits_culling_and_end_destruction_to_canonical_slots() {
        let bytes =
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap();
        let rom = Rom::from_bytes(bytes, config::ROM_SHA256).unwrap();
        let overrides = MemoryAssets::default();
        let raw = Assets::load(&rom, &overrides).unwrap();
        let mut services = Services::new(Content::load(&rom, &raw, &overrides).unwrap());
        let mut core = Core::reset();
        core.actors[1].state = actors::ACTIVE | presentation::WORLD_POSITIONED;
        core.actors[1].render = presentation::VISIBLE;
        core.actors[1].position.world_x = 0x0250;
        core.actors[1].spawn_identity = Some(7);
        core.actors[2].state = actors::ACTIVE;
        core.actors[2].render = presentation::VISIBLE | presentation::DESTROY_AT_END;
        std::sync::Arc::get_mut(&mut services.content)
            .unwrap()
            .presentation
            .animations[0] = presentation::Animation {
            last_frame: 0,
            last_tick: 0,
            frames: vec![0],
        };
        services.compose(&mut core);
        assert_eq!(
            (
                core.actors[1].state,
                core.actors[1].render,
                core.actors[1].spawn_identity
            ),
            (0, 0, None)
        );
        assert_eq!(core.actors[2].state, 0);
    }
}

impl Services {
    pub fn load_stage_presentation(&mut self, core: &mut Core, selector: u8) {
        let preset = &self.content.raw_stage_presets[usize::from(selector)];
        self.commands.submit_stage(selector);
        core.terrain_bank = preset.terrain_bank;
        core.row = preset.row;
        core.camera.room = preset.camera_room;
        let page = preset.camera_room & 7;
        core.camera.x = (core.camera.x & 0xff) | u16::from(page) << 8;
        // D9CA writes this source cursor independently of CameraXHi. The first
        // C710 build advances it, and a repeated build must read that result.
        core.stream.room_in_row = page;
        core.actors[0].position.world_x = u16::from(page) << 8 | u16::from(preset.player_x);
        self.video.display.base[..16].copy_from_slice(&preset.background_palette);
        self.video.display.active[..16].copy_from_slice(&preset.background_palette);
        self.video.display.base[16..].copy_from_slice(&self.content.progression.sprite_palette);
        self.video.display.active[16..].copy_from_slice(&self.content.progression.sprite_palette);
        core.layout = selector;
    }
    pub fn build_column(&mut self, core: &mut Core, cursor: world::StreamCursor) {
        self.build_column_at(core, cursor, 0);
    }
    /// Multiple same-call camera columns share Scratch05's byte record offset.
    /// A missing column contributes no queue bytes and does not clear pending.
    pub fn build_column_at(
        &mut self,
        core: &mut Core,
        cursor: world::StreamCursor,
        queue_offset: u8,
    ) -> Option<u8> {
        self.build_column_record(core, cursor, queue_offset, true)
    }
    pub fn build_column_record(
        &mut self,
        core: &mut Core,
        cursor: world::StreamCursor,
        queue_offset: u8,
        terminate: bool,
    ) -> Option<u8> {
        let layout = if usize::from(core.layout) < self.content.world.layouts.len() {
            &self.content.world.layouts[usize::from(core.layout)]
        } else {
            self.content.raw_layouts[usize::from(core.layout)]
                .as_ref()
                .expect("raw layout pointer consumer assessed at entry")
        };
        if cursor.room_in_row >= layout.columns {
            return None;
        }
        let room = layout
            .rooms
            .get(usize::from(cursor.room))
            .copied()
            .flatten()?;
        let context = world::ExpansionContext {
            stage: core.progression.stage.0,
            room: cursor.room,
            room_in_row: cursor.room_in_row,
            ring: cursor.ring,
            split: core.split_transition != 0,
            stage_three_control: core.progression.hit_dialogue_count,
            stage_four_control: core.progression.escort_prerequisite,
        };
        self.runtime
            .bank
            .select(runtime::PrgBank::from_selector(core.terrain_bank));
        let block = world::Room::block(cursor.ring, &self.content.raw_rooms[room]);
        let mut expanded = world::expand(
            &self.content.world.terrain[usize::from(core.terrain_bank)],
            block,
            context,
        );
        self.video
            .retain_terrain_staging(&expanded.tiles, cursor.ring & 7 == 7);
        expanded.tiles = self.video.terrain_staging;
        let result = world::column(layout, context, &expanded, self.video.display.suspended);
        if let Some((index, value)) = result.collision {
            core.cache.collision[index] = value;
        }
        if let Some(upload) = result.upload {
            const TILE_RECORD_BYTES: usize = 7;
            const TILE_PAYLOAD: usize = 3;
            const ATTRIBUTE_RECORD_BYTES: u8 = 4;
            const NAMETABLE_BASE: u16 = 0x2000;
            const NAMETABLE_BYTES: u16 = 0x400;
            const FOUR_TILE_ROWS: u16 = 128;
            const NAMETABLE_ROW: u16 = 32;
            let base = NAMETABLE_BASE
                + u16::from(cursor.room_in_row & 1) * NAMETABLE_BYTES
                + u16::from(cursor.ring & 7) * FOUR_TILE_ROWS
                + u16::from(cursor.ring >> 3) * 4;
            // Header operands retain byte Y but add each absolute field offset
            // without wrapping. Payload Y is recomputed with byte addition for
            // each quadrant. These differ when a large camera batch crossesFF.
            for row in 0..4 {
                let at = usize::from(queue_offset) + row * TILE_RECORD_BYTES;
                let address = base + row as u16 * NAMETABLE_ROW;
                for (field, value) in [(0, (address >> 8) as u8), (1, address as u8), (2, 3)] {
                    self.video
                        .write_primary_byte(at + field, value, &mut core.cache.attributes);
                }
            }
            for quadrant in (0..4).rev() {
                let source = (quadrant / 2) * 8 + (quadrant % 2) * 2;
                let row_offset = (quadrant / 2) * 2 * TILE_RECORD_BYTES + (quadrant % 2) * 2;
                let at = usize::from(queue_offset.wrapping_add(row_offset as u8));
                for (field, source) in [
                    (TILE_PAYLOAD, source),
                    (TILE_PAYLOAD + 1, source + 1),
                    (TILE_PAYLOAD + TILE_RECORD_BYTES, source + 4),
                    (TILE_PAYLOAD + TILE_RECORD_BYTES + 1, source + 5),
                ] {
                    self.video.write_primary_byte(
                        at + field,
                        self.video.terrain_staging[source],
                        &mut core.cache.attributes,
                    );
                }
            }
            let top_edge =
                core.progression.stage.0 < 5 && cursor.room_in_row & 1 == 0 && cursor.ring & 7 == 1;
            if top_edge {
                for index in 0..2 * TILE_RECORD_BYTES {
                    let at = usize::from(queue_offset.wrapping_add(index as u8));
                    let value = self
                        .video
                        .primary_byte(at + 2 * TILE_RECORD_BYTES, &core.cache.attributes);
                    self.video
                        .write_primary_byte(at, value, &mut core.cache.attributes);
                }
            }
            let tile_bytes = if top_edge || cursor.ring & 7 == 7 {
                2 * TILE_RECORD_BYTES
            } else {
                4 * TILE_RECORD_BYTES
            };
            let attribute_at = queue_offset.wrapping_add(tile_bytes as u8);
            let coordinate = (cursor.ring & 7) * 8 + (cursor.ring >> 3);
            let high = 0x23
                | self
                    .video
                    .staging_queue_byte(9 + usize::from(attribute_at), &core.cache.attributes);
            let palette = result
                .attribute
                .expect("visible column has an attribute byte")
                .1;
            for (field, value) in [high, 0xc0 | coordinate, 0, palette]
                .into_iter()
                .enumerate()
            {
                self.video.write_primary_byte(
                    usize::from(attribute_at) + field,
                    value,
                    &mut core.cache.attributes,
                );
            }
            let (attribute_index, attribute_value) =
                result.attribute.expect("column attribute cache write");
            core.cache.attributes[attribute_index] = attribute_value;
            // Source C94D terminates only after C92x's canonical cache store;
            // that final absolute +4 can itself alias attribute cell0.
            if terminate {
                self.video.write_primary_byte(
                    usize::from(attribute_at) + usize::from(ATTRIBUTE_RECORD_BYTES),
                    u8::MAX,
                    &mut core.cache.attributes,
                );
            }
            if terminate && upload.commit == crate::assets::formats::UploadCommit::AtInterrupt {
                self.video.display.primary.pending = u8::MAX;
            }
            if terminate && upload.commit == crate::assets::formats::UploadCommit::Immediate {
                let shared = self.video.primary_bytes(&core.cache.attributes);
                self.video.display.flush(
                    presentation::Queue::Primary,
                    &shared,
                    presentation::Stride::Across,
                );
                self.video.commit_writes();
            }
            return Some(attribute_at.wrapping_add(ATTRIBUTE_RECORD_BYTES));
        }
        None
    }
    pub fn build_terrain(&mut self, core: &mut Core, whole_ring: bool) {
        let layout = if usize::from(core.layout) < self.content.world.layouts.len() {
            &self.content.world.layouts[usize::from(core.layout)]
        } else {
            self.content.raw_layouts[usize::from(core.layout)]
                .as_ref()
                .expect("raw layout pointer consumer")
        };
        let visits = if whole_ring {
            // C78F leaves the counter at40hex, without advancing the room.
            // Its sole source caller is land setup immediately after D9CA,
            // which installs room_in_row = camera.room & 7; whole_ring uses
            // that established invariant and does not write in-row itself.
            core.stream.room = core.camera.room;
            core.stream.ring = world::ROOM_CELLS as u8;
            world::whole_ring(core.camera.room)
        } else {
            let plan = world::initial_columns(layout, core.camera.room, core.stream.room_in_row);
            core.stream = plan.final_cursor;
            core.previous_scroll_direction = 1;
            plan.visits
        };
        for cursor in visits {
            self.build_column(core, cursor);
        }
    }
}
/// Each source row uses a destination and last-index byte, then contiguous
/// tiles. This encoder is transient; Video owns all surviving queue bytes.
pub(super) fn encode_upload(writes: &[presentation::VideoWrite]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut cursor = 0;
    while cursor < writes.len() {
        let start = cursor;
        let address = writes[start].address;
        cursor += 1;
        while cursor < writes.len()
            && cursor - start < 128
            && writes[cursor].address == address.wrapping_add((cursor - start) as u16)
        {
            cursor += 1;
        }
        bytes.extend([
            (address >> 8) as u8,
            address as u8,
            (cursor - start - 1) as u8,
        ]);
        bytes.extend(writes[start..cursor].iter().map(|write| write.value));
    }
    bytes.push(0xff);
    bytes
}

impl Services {
    /// Special spawns run before the ordinary window. Candidate callbacks apply
    /// replacement/recovery allocation now, so subsequent candidates see them.
    pub fn spawn(&mut self, core: &mut Core) {
        let before = super::journal::Snapshot::core(core);
        let special_before = core.actors;
        let mut context = actors::SpawnContext {
            stage: core.progression.stage.0,
            room_row: core.row,
            camera: core.camera.x,
            direction: core.spawn_window.direction,
            split_transition: core.split_transition,
            late_mode: core.progression.late_mode,
            late_route: core.progression.late_route,
            stage_progress: core.progression.progress_bits,
            boss_hit_points: core.progression.boss_hit_points,
            special: core.special_spawn,
        };
        if let Some(definition) = self.content.spawns.source(core.progression.stage) {
            actors::special_spawn(
                &mut core.actors,
                &definition.special_rows,
                &self.content.spawns.templates,
                &mut context,
            );
        }
        super::journal::actor_changes(
            &self.events,
            &special_before,
            &core.actors,
            &self.content,
            "special_spawn",
        );
        core.special_spawn = context.special;
        core.progression.boss_hit_points = context.boss_hit_points;
        let planes = &self.content.spawn_planes[&core.spawn_configuration];
        let mut records = Vec::new();
        let mut cursor = planes.room_starts[usize::from(core.row)];
        loop {
            let index = usize::from(cursor);
            let high = planes.x_high[index];
            if high & actors::ACTIVE != 0 {
                break;
            }
            let template = usize::from(planes.templates[index]);
            assert!(
                template < self.content.spawns.templates.len(),
                "raw spawn template requires an authored native template"
            );
            records.push(actors::SpawnRecord {
                identity: cursor,
                world_x: u16::from(high) << 8 | u16::from(planes.x_low[index]),
                screen_y: planes.y[index],
                template,
            });
            cursor = cursor.wrapping_add(1);
            if cursor == planes.room_starts[usize::from(core.row)] {
                break;
            }
        }
        let terrain = &self.content.world.terrain[usize::from(core.terrain_bank)];
        // Observation-only prior boundary, dropped with this nonyielding call.
        // It never supplies allocation or gameplay decisions.
        let mut previous_pool = core.actors;
        actors::scan_window_with(
            &mut core.actors,
            &core.suppression,
            &records,
            &self.content.spawns.templates,
            &mut core.spawn_window,
            &mut context,
            &mut |pool, context, report| {
                super::journal::actor_changes(
                    &self.events,
                    &previous_pool,
                    pool,
                    &self.content,
                    "ordinary_spawn_candidate",
                );
                previous_pool = *pool;
                core.progression.boss_hit_points = context.boss_hit_points;
                core.special_spawn = context.special;
                for &slot in &report.terrain_replacements {
                    // B044 enters F4D8, not the active-object recovery gate.
                    let source = pool[slot];
                    let index = world::replacement_index(world::Replacement::Animation {
                        stage: crate::identifiers::StageId(context.stage),
                        animation: source.animation,
                    });
                    let selector = self.content.changed_terrain_selectors[usize::from(index)];
                    let upload = core.cache.replace(
                        terrain,
                        world::Mutation {
                            world_x: source.position.world_x,
                            spawn_y: source.spawn_y,
                            selector,
                            marks_changed: false,
                        },
                    );
                    self.video.display.secondary.append_changed_shared(
                        &mut core.secondary_cursor,
                        &upload,
                        &mut core.cache.attributes,
                    );
                }
                super::journal::actor_changes(
                    &self.events,
                    &previous_pool,
                    pool,
                    &self.content,
                    "spawn_recovery",
                );
                previous_pool = *pool;
            },
        );
        core.special_spawn = context.special;
        core.progression.boss_hit_points = context.boss_hit_points;
        self.runtime.bank.select(runtime::PrgBank::ActorsSix);
        before.finish_domain_core(&self.events, core, &self.content, "spawn");
    }
}

#[cfg(test)]
mod spawn_tests {
    use super::*;
    #[test]
    fn persistent_candidate_uses_animation_terrain_representation_without_recovery() {
        use romatlas_port::assets::{MemoryAssets, Rom};
        let rom = Rom::from_bytes(
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap(),
            crate::config::ROM_SHA256,
        )
        .unwrap();
        let overrides = MemoryAssets::default();
        let raw = crate::assets::Assets::load(&rom, &overrides).unwrap();
        let mut content = Content::load(&rom, &raw, &overrides).unwrap();
        content.spawns.templates[0] = actors::Template {
            flags_and_local: 0x90,
            animation: 0x9a,
            handler: actors::objects::TERRAIN_HANDLER,
            profile: 0,
        };
        let mut planes = actors::SpawnPlanes {
            x_high: [0xff; 256],
            x_low: [0; 256],
            room_starts: [0; 256],
            y: [0x80; 256],
            templates: [0; 256],
        };
        planes.x_high[..2].fill(0);
        planes.x_low[..2].copy_from_slice(&[0x80, 0x90]);
        content.spawn_planes.insert(0, planes);
        let mut services = Services::new(content);
        let mut core = Core::reset();
        core.special_spawn = actors::SpecialControl::Cooldown(1);
        core.spawn_window.direction = 1;
        services.spawn(&mut core);
        assert_eq!(
            (
                core.actors[15].spawn_identity,
                core.actors[14].spawn_identity
            ),
            (Some(0), Some(1))
        );
        assert_eq!((core.actors[15].handler, core.actors[14].handler), (8, 8));
        assert_eq!(
            (core.actors[15].animation, core.actors[14].animation),
            (0x1a, 0x1a)
        );
        assert_eq!(
            (core.cache.changed_this_frame, core.secondary_cursor),
            (0, 28)
        );
        assert_eq!((core.spawn_window.upper, core.spawn_window.lower), (2, 0));
        let events = services.events.observe();
        let slots = events["entries"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|entry| entry["kind"] == "ActorSpawned")
            .map(|entry| entry["payload"]["slot"].as_u64().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            slots,
            [15, 14],
            "B044 installs terrain representation without allocating recovery"
        );
    }
}

impl Services {
    /// Camera mode and its entire terrain-stream batch return in this foreground
    /// call. Only the eventual foreground Yield permits NMI to consume the queue.
    pub fn camera(&mut self, core: &mut Core) {
        const DIRECTION_MASK: u8 = 3;
        const FORWARD: u8 = 1;
        const BACKWARD: u8 = 2;
        let layout = if usize::from(core.layout) < self.content.world.layouts.len() {
            &self.content.world.layouts[usize::from(core.layout)]
        } else {
            self.content.raw_layouts[usize::from(core.layout)]
                .as_ref()
                .expect("camera raw layout consumer")
        };
        let automatic = core.progression.special_room & BACKWARD != 0;
        let active = core.scroll_direction & DIRECTION_MASK != 0;
        let result = world::camera_step(
            core.camera,
            world::CameraPlayer {
                x: core.actors[0].position.world_x,
                fraction: core.actors[0].position.x_fraction,
            },
            world::CameraInput {
                active,
                automatic,
                departure: self.departure_signal,
                held: self.runtime.controllers.ports[0].held.0,
                late: core.progression.late_mode,
            },
            layout,
        );
        core.camera = result.camera;
        core.actors[0].position.world_x = result.player.x;
        core.actors[0].position.x_fraction = result.player.fraction;
        if active && !automatic {
            core.scroll_direction = FORWARD;
        }
        let Some(direction) = result.direction else {
            return;
        };
        let encoded = if direction == world::Direction::Forward {
            FORWARD
        } else {
            BACKWARD
        };
        core.scroll_direction = encoded;
        let displacement = (core.camera.x as u8).wrapping_sub(core.camera.previous_x) as i8;
        core.camera_displacement = displacement.unsigned_abs();
        let mut queue_offset = 0;
        for index in 0..result.work {
            let last = index == result.work - 1;
            let previous = if core.previous_scroll_direction == encoded {
                direction
            } else if direction == world::Direction::Forward {
                world::Direction::Backward
            } else {
                world::Direction::Forward
            };
            core.stream = world::advance_cursor(core.stream, previous, direction);
            core.previous_scroll_direction = encoded;
            if let Some(next) = self.build_column_record(core, core.stream, queue_offset, last) {
                queue_offset = next;
            } else if last && queue_offset != 0 {
                self.video.write_primary_byte(
                    usize::from(queue_offset),
                    u8::MAX,
                    &mut core.cache.attributes,
                );
                if self.video.display.suspended == 0 {
                    self.video.display.primary.pending = u8::MAX;
                } else {
                    let bytes = self.video.primary_bytes(&core.cache.attributes);
                    self.video.display.flush(
                        presentation::Queue::Primary,
                        &bytes,
                        presentation::Stride::Across,
                    );
                    self.video.commit_writes();
                }
            }
        }
    }
}

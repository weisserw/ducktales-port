//! Read-only semantic views of canonical native owners. No observation drains
//! a queue, advances a continuation, renders audio or changes a source counter.
use super::Port;
use crate::{actors, player};
use serde_json::{Value, json};
pub fn action_name(action: player::Action) -> &'static str {
    use player::Action::*;
    match action {
        Standing => "standing",
        Airborne => "airborne",
        ForcedJump => "forced_jump",
        Climbing => "climbing",
        Swing => "swing",
        Protected => "protected",
        Damage => "damage",
        TerrainPose => "terrain_pose",
        FallOut => "fall_out",
        WaitA => "wait_a",
        CarriedLocked => "carried_locked",
        CarriedJumpable => "carried_jumpable",
        ExternalLaunch => "external_launch",
        Death => "death",
        Hold => "hold",
        Dialogue => "dialogue",
        ConvertTimer => "convert_timer",
    }
}
pub fn actor(slot: usize, a: &actors::ActorContact, label: Option<&str>) -> Value {
    json!({"slot":slot,"active":a.state_code() & actors::ACTIVE != 0,"state":a.state_code(),"render":a.render,"profile":a.profile,
        "handler":a.handler,"family":label,"spawn_identity":a.spawn_identity,
        "position":{"world_x":a.position.world_x,"screen_y":a.position.screen_y,"x_fraction":a.position.x_fraction,"y_fraction":a.position.y_fraction},
        "screen_x":a.screen_x,"horizontal_magnitude":a.horizontal_magnitude,"vertical_velocity":a.vertical_velocity,
        "animation":{"selector":a.animation,"frame":a.playback.frame_code(),"tick":a.playback.tick_code(),"restart":a.playback.restart_code()},
        "local":{"state":a.local_state_code(),"counter":a.local_counter_code(),"auxiliary":a.local_aux2,"sequence_cursor":a.sequence_cursor,"boss_protection":a.boss_protection},
        "hit_marker":a.hit_marker,"direction":a.direction,"motion_flags":a.motion_flags,"spawn_y":a.spawn_y})
}
pub(super) fn player_value(player: &player::Player) -> Value {
    json!({"action":action_name(player.action),"selector":player.action as u8,"queued":player.queued.map(action_name),"protection":player.protection,"special_mode":player.special_mode,"special_ticks":player.special_ticks,"gravity":player.gravity,"animation_delay":player.animation_delay,"pose_direction":player.pose_direction,"momentum_control":player.momentum_control,"momentum_direction":player.momentum_direction,"external_motion":player.external_motion,"actor_contacts":player.actor_contacts,"pogo_height_control":player.pogo_height_control,"jump_velocity":player.jump_velocity})
}
pub(super) fn status_value(status: &crate::status::Status) -> Value {
    json!({"reward":status.reward.display_codes(),"total":status.total.display_codes(),"lives":status.lives,"health":{"segments":status.health.overlay_segments().iter().map(|segment|json!({"y":segment.y,"tile":segment.tile,"depleted":segment.depleted,"x":segment.x})).collect::<Vec<_>>(),"end":status.health.overlay_end(),"alternating":status.health.alternating},"countdown":status.countdown.display_codes(),"divider":status.divider,"dirty":status.dirty,"ui_pending":status.ui_pending})
}
pub(super) fn progression_value(p: &crate::progression::Progression, row: u8) -> Value {
    json!({"stage":p.stage.0,"room_row":row,"progress_bits":p.progress_bits,"accepted_stages":p.accepted_stages,"escort_prerequisite":p.escort_prerequisite,"hit_dialogue_count":p.hit_dialogue_count,"reward_progress":p.reward_progress,"seen_messages":p.seen_messages,"stage_one_extension":p.stage_one_extension,"boss_hit_points":p.boss_hit_points,"impact_latch":p.impact_latch,"late_route":p.late_route,"late_mode":p.late_mode,"scene_signal":p.scene_signal,"ending_latch":p.ending_latch,"race_intro":p.race_intro,"completion_actor":p.completion_actor,"life_loss_variant":p.life_loss_variant})
}
impl Port {
    pub fn observe(&self) -> Value {
        let machine = &self.machine;
        let core = &machine.core;
        let services = &machine.services;
        let p = &core.progression;
        let player = &core.player;
        let video = &services.video;
        let actors = core
            .actors
            .iter()
            .enumerate()
            .map(|(slot, a)| {
                actor(
                    slot,
                    a,
                    services
                        .content
                        .spawns
                        .handlers
                        .get(usize::from(a.handler))
                        .map(|entry| entry.label.as_str()),
                )
            })
            .collect::<Vec<_>>();
        let channels = services.audio.channels.iter().enumerate().map(|(voice, channel)| json!({
            "voice":voice,"primary":{"cursor":channel.primary.cursor,"duration":channel.primary.duration,"scale":channel.primary.scale,"repeat":channel.primary.repeat_control,"note":channel.primary.note_control,"period_table":channel.primary.period_table,"period":channel.primary.period,"control":channel.primary.control,"pitch_step":channel.primary.pitch_step,"envelope":{"interval":channel.primary.envelope.interval,"counter":channel.primary.envelope.counter,"level":channel.primary.envelope.step_level}},
            "secondary":{"period":channel.secondary.period,"control":channel.secondary.control,"pitch_step":channel.secondary.pitch_step},
            "modulation":{"pitch_interval":channel.modulation.pitch_interval,"pitch_shape":channel.modulation.pitch_shape,"pitch_counter":channel.modulation.pitch_counter,"pitch_phase":channel.modulation.pitch_phase,"pitch_offset":channel.modulation.pitch_offset,"level_interval":channel.modulation.level_interval,"level_step":channel.modulation.level_step,"level_counter":channel.modulation.level_counter,"level":channel.modulation.level,"cached_high":channel.modulation.cached_high}
        })).collect::<Vec<_>>();
        json!({"schema_version":2,"implementation":"native_integration","ticks":machine.ticks,"mode":self.mode().name(),
            "assets":{"source_slots":self.assets.source.assets.len(),"decoded":true,"level_slots":services.content.levels.levels.len(),"active_level":services.content.active_file.as_ref().map(|level|level.id.0.clone()).or_else(||services.content.levels.original_level(p.stage).map(|level|level.id.0.clone())),"pending_successor":machine.pending_level()},
            "input":services.runtime.controllers.ports.iter().enumerate().map(|(port, input)|json!({"port":port,"held":input.held.0,"pressed":input.pressed.0})).collect::<Vec<_>>(),
            "scheduler":{"interrupt_tick":services.runtime.interrupt_tick,"slots":services.runtime.scheduler.slots.iter().map(|slot|format!("{slot:?}")).collect::<Vec<_>>(),"bank":format!("{:?}",services.runtime.bank.foreground),"random":services.runtime.random.words,"sample_return":services.runtime.controllers.last_sample_return.map(|result|result.primary_opposite_edges)},
            "progression":progression_value(p, core.row),
            "collision_return":{"bits":services.collision_outcome.bits},
            "player":player_value(player),
            "actors":actors,
            "world":{"layout":core.layout,"terrain_bank":core.terrain_bank,"camera":{"x":core.camera.x,"room":core.camera.room,"previous_x":core.camera.previous_x,"displacement":core.camera_displacement},"stream":{"ring":core.stream.ring,"room":core.stream.room,"room_in_row":core.stream.room_in_row,"scroll_direction":core.scroll_direction,"previous_direction":core.previous_scroll_direction},"terrain":{"collision":core.cache.collision.as_slice(),"attributes":core.cache.attributes.as_slice(),"changed_this_frame":core.cache.changed_this_frame},"spawn":{"configuration":core.spawn_configuration,"lower":core.spawn_window.lower,"upper":core.spawn_window.upper,"room_start":core.spawn_window.room_start,"direction":core.spawn_window.direction,"suppression":core.suppression.as_slice(),"special":format!("{:?}",core.special_spawn),"special_selector":core.special_spawn.encode()}},
            "status":status_value(&core.status),
            "timers":{"protection":player.protection,"special_ticks":player.special_ticks,"countdown_divider":core.status.divider,"composition":video.composition.clock},
            "flags":{"reload_request":p.reload_request,"boundary_request":p.boundary_request,"special_room":p.special_room,"status_selector":p.status_selector,"dialogue_cursor":p.dialogue_cursor},
            "calls":machine.observe_calls(),
            "transition":{"foreground":format!("{:?}",p.continuation),"awaiting_owner":p.awaiting_owner.map(|work|format!("{work:?}")),"first_actor_increment":p.first_actor_increment.map(|increment|format!("{increment:?}")),"menu":format!("{:?}",machine.menu.flow),"split_control":core.split_transition,"split_address":video.split_address},
            "dialogue":{"message":services.dialogue.message,"actor_slot":services.dialogue.actor_slot,"phase":format!("{:?}",services.dialogue.phase),"line":services.dialogue.line,"row_glyphs":services.dialogue.row_glyphs,"acknowledgment":services.dialogue.acknowledgment.map(|buttons|buttons.0),"pending_wait":services.dialogue.pending_wait.map(|work|format!("{work:?}")),"rows":video.display.primary.rows()},
            "presentation":{"display_mode":video.gameplay_display,"suspend_depth":video.display.suspended,"primary_pending":video.display.primary.pending,"secondary_pending":video.display.secondary_pending,"secondary_cursor":core.secondary_cursor,"palette_pending":video.display.palette_pending,"base_palette":video.display.base,"active_palette":video.display.active,"primary":video.display.primary.encoded,"secondary_prefix":video.display.secondary.encoded,"oam_guard":video.oam_guard,"committed_oam":video.committed_oam.iter().map(|sprite|[sprite.y,sprite.tile,sprite.attributes,sprite.x]).collect::<Vec<_>>(),"raster":{"ui_scroll":video.raster.ui_scroll,"ui_control":video.raster.ui_control,"ui_mask":video.raster.ui_mask,"playfield_scroll":video.raster.playfield_scroll,"playfield_control":video.raster.playfield_control,"playfield_mask":video.raster.playfield_mask,"split_address":video.raster.split_address}},
            "audio":{"implemented":true,"stage_command":services.commands.stage_command,"commands":{"producer":services.commands.producer,"consumer":services.commands.consumer,"slots":services.commands.slots},"frames":services.audio.frames,"channels":channels,"priority":services.audio.priority,"secondary_mask":services.audio.secondary_mask,"secondary_cursor":services.audio.secondary_cursor,"secondary_repeat":services.audio.secondary_repeat,"shared_delay":services.audio.shared_delay,"tempo":services.audio.tempo,"fade":services.audio.fade,"fade_level":services.audio.fade_level,"phase":services.audio.phase,"busy":services.audio.busy,"preset_base":services.audio.preset_base,"reset_seed":{"selector":services.audio.reset_selector.selector,"producer":format!("{:?}",services.audio.reset_selector.producer)},"synthesis_clocks":services.synthesizer.elapsed_clocks},
            "events":services.events.observe()})
    }
}

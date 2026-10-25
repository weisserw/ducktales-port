//! A complete player invocation, including text/conversion waits and the
//! nonlocal vertical tail. No body, sound or pre-dispatch timer is replayed when
//! its nested owner resumes on a later host tick.
use super::{
    contact::{PlayerCallReturn, PlayerWorld},
    dialogue_call::{self, DialogueCall},
    simulation::{Core, Services},
    tasks::{ConversionReturn, Tasks},
};
use crate::{
    player,
    progression::{FirstActorIncrement, OwnerStep, PlayerReturn},
};

#[derive(Default, Clone)]
enum Phase {
    #[default]
    Entry,
    Dialogue,
    Conversion,
    Vertical,
}
#[derive(Default, Clone)]
pub struct PlayerCall {
    phase: Phase,
    dialogue: DialogueCall,
}
pub enum Poll {
    Complete(PlayerReturn),
    VerticalTail,
    Suspended,
}
impl PlayerCall {
    pub fn dialogue_active(&self) -> bool {
        matches!(self.phase, Phase::Dialogue)
    }
    pub fn poll(
        &mut self,
        core: &mut Core,
        services: &mut Services,
        tasks: &mut Tasks,
        entry: FirstActorIncrement,
    ) -> Poll {
        loop {
            match self.phase {
                Phase::Entry => {
                    let content = std::sync::Arc::clone(&services.content);
                    let layout = if usize::from(core.layout) < content.world.layouts.len() {
                        &content.world.layouts[usize::from(core.layout)]
                    } else {
                        content.raw_layouts[usize::from(core.layout)]
                            .as_ref()
                            .expect("validated player room-layout consumer")
                    };
                    let rooms = layout
                        .rooms
                        .iter()
                        .map(|room| content.room_presence(*room))
                        .collect::<Vec<_>>();
                    let mut room = player::RoomContext {
                        camera: core.camera.x,
                        camera_room: core.camera.room,
                        terrain_room: core.stream.room,
                        row: core.row,
                        rows: layout.rows,
                        columns: layout.columns,
                        late_route: core.progression.late_route,
                        previous_scroll: core.previous_scroll_direction,
                        terrain_column: core.stream.ring,
                        special_spawn: core.special_spawn.encode(),
                    };
                    let mut world = PlayerWorld {
                        content: &content,
                        collision: &core.cache.collision,
                        row: core.row,
                        layout: core.layout,
                        bank: core.terrain_bank,
                        collision_outcome: &mut services.collision_outcome,
                        last_sample: None,
                        last_actor: None,
                        last_motion_return: None,
                        terrain_return: None,
                        final_operation: None,
                        special: &mut core.special_spawn,
                        total: &mut core.status.total,
                        ui_pending: &mut core.status.ui_pending,
                    };
                    let effects = core.player.step(
                        &mut core.actors,
                        &mut services.runtime.controllers,
                        &mut player::StepContext {
                            room: &mut room,
                            rooms: &rooms,
                            health: &mut core.status.health,
                            title_rule: services.option.status_rule(),
                            stage: core.progression.stage.0,
                            life_loss_variant: core.progression.life_loss_variant,
                            current_stage_audio: services.commands.stage_command,
                        },
                        &content.player,
                        &mut world,
                    );
                    let final_operation = world.final_operation;
                    // Commit the real helper's call-local projections before
                    // starting any owner that can yield or be externally seen.
                    core.camera.x = room.camera;
                    core.camera.room = room.camera_room;
                    core.stream.room = room.terrain_room;
                    core.stream.ring = room.terrain_column;
                    core.row = room.row;
                    if effects.transition {
                        core.special_spawn =
                            crate::actors::SpecialControl::decode(room.special_spawn);
                    }
                    if let Some(direction) = effects.scroll {
                        core.scroll_direction = direction;
                    }
                    if let Some(direction) = effects.spawn {
                        core.spawn_window.direction = direction;
                    }
                    if let Some(request) = effects.reload {
                        core.progression.reload_request = request;
                    }
                    if let Some(message) = effects.message {
                        services.dialogue.message = message;
                    }
                    // No queue consumer occurs within the nonyielding B06 body.
                    // Preserve author order (pre-dispatch stage commands first)
                    // and retain the actual reported submission, including drop.
                    let mut audio_return = None;
                    for command in effects.stage_sounds {
                        let submission = services.commands.submit_stage(command);
                        if final_operation
                            == Some(PlayerCallReturn::Audio {
                                command,
                                stage_command: true,
                            })
                        {
                            audio_return = Some((command, true, submission));
                        }
                    }
                    for command in effects.sounds {
                        let submission = services.commands.submit(command);
                        if final_operation
                            == Some(PlayerCallReturn::Audio {
                                command,
                                stage_command: false,
                            })
                        {
                            audio_return = Some((command, false, submission));
                        }
                    }
                    if effects.transition {
                        self.phase = Phase::Vertical;
                        continue;
                    }
                    if effects.work.contains(&player::Work::Dialogue) {
                        self.dialogue.begin(services);
                        self.phase = Phase::Dialogue;
                        continue;
                    }
                    if effects.work.contains(&player::Work::ConvertTimer) {
                        self.phase = Phase::Conversion;
                        continue;
                    }
                    let first_actor_increment = final_operation.map_or(entry, |operation| {
                        operation.first_actor_increment(audio_return)
                    });
                    services.events.record("PlayerReturn", serde_json::json!({"operation":final_operation.map(|operation|format!("{operation:?}")).unwrap_or_else(||"InheritedGameplayComparison".into()),"first_actor_increment":format!("{first_actor_increment:?}")}));
                    return Poll::Complete(PlayerReturn {
                        first_actor_increment,
                    });
                }
                Phase::Dialogue => match self.dialogue.poll(core, services) {
                    dialogue_call::Poll::Requested(work) => {
                        if tasks.dialogue_work(work, core, services) == OwnerStep::Suspended {
                            return Poll::Suspended;
                        }
                        self.dialogue.completed(work);
                    }
                    dialogue_call::Poll::Complete(operation) => {
                        self.phase = Phase::Entry;
                        let returned = operation.player_return();
                        services.events.record("PlayerReturn", serde_json::json!({"operation":format!("{operation:?}"),"first_actor_increment":format!("{:?}",returned.first_actor_increment)}));
                        return Poll::Complete(returned);
                    }
                },
                Phase::Conversion => match tasks.conversion(core, services) {
                    OwnerStep::Suspended => return Poll::Suspended,
                    OwnerStep::Complete(operation) => {
                        let increment = match operation {
                            ConversionReturn::EvenComposition => FirstActorIncrement::Zero,
                            ConversionReturn::ValueSound(submission) => {
                                submission.first_actor_increment()
                            }
                            ConversionReturn::FinishedWait => sample_return(services),
                        };
                        self.phase = Phase::Entry;
                        return Poll::Complete(PlayerReturn {
                            first_actor_increment: increment,
                        });
                    }
                },
                Phase::Vertical => {
                    if tasks.vertical_transition(core, services) == OwnerStep::Suspended {
                        return Poll::Suspended;
                    }
                    self.phase = Phase::Entry;
                    services.events.record("PlayerNonlocalReturn", serde_json::json!({"continuation":"AfterSpawn","source":"committed_vertical_transition"}));
                    return Poll::VerticalTail;
                }
            }
        }
    }
}
/// The resumed slot-zero wait returns the final pre-filter primary edge
/// comparison. It never substitutes held conflicts or already-filtered edges.
pub(super) fn sample_return(services: &Services) -> FirstActorIncrement {
    if services
        .runtime
        .controllers
        .last_sample_return
        .expect("a completed slot-zero wait sampled controllers")
        .primary_opposite_edges
    {
        FirstActorIncrement::One
    } else {
        FirstActorIncrement::Zero
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Core, Services, Tasks) {
        use romatlas_port::assets::{MemoryAssets, Rom};
        let bytes =
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap();
        let rom = Rom::from_bytes(bytes, crate::config::ROM_SHA256).unwrap();
        let overrides = MemoryAssets::default();
        let assets = crate::assets::Assets::load(&rom, &overrides).unwrap();
        let mut core = Core::reset();
        let services =
            Services::new(super::super::content::Content::load(&rom, &assets, &overrides).unwrap());
        core.player.action = player::Action::Hold;
        (core, services, Tasks::default())
    }
    fn complete(
        call: &mut PlayerCall,
        core: &mut Core,
        services: &mut Services,
        tasks: &mut Tasks,
        entry: FirstActorIncrement,
    ) -> FirstActorIncrement {
        let Poll::Complete(result) = call.poll(core, services, tasks, entry) else {
            panic!("nonyielding source body must return in this host turn");
        };
        result.first_actor_increment
    }
    #[test]
    fn held_body_preserves_entry_until_actual_special_expiry_audio_replaces_it() {
        let (mut core, mut services, mut tasks) = fixture();
        let mut call = PlayerCall::default();
        assert_eq!(
            complete(
                &mut call,
                &mut core,
                &mut services,
                &mut tasks,
                FirstActorIncrement::One
            ),
            FirstActorIncrement::One
        );
        for (occupied, expected) in [
            (0x11, FirstActorIncrement::Zero),
            (0xff, FirstActorIncrement::One),
        ] {
            services.commands = crate::audio::CommandQueue::default();
            services.commands.slots[1] = occupied;
            core.player.special_mode = 1;
            core.player.special_ticks = 1;
            assert_eq!(
                complete(
                    &mut call,
                    &mut core,
                    &mut services,
                    &mut tasks,
                    FirstActorIncrement::One
                ),
                expected
            );
            assert_eq!(services.commands.slots[0], player::SPECIAL_AUDIO);
            assert_eq!(services.commands.slots[1], occupied);
            assert_eq!(
                services.commands.producer, 1,
                "expiry restore was attempted after special audio, then dropped once"
            );
            assert_eq!(
                (core.player.special_mode, core.player.special_ticks),
                (0, 0)
            );
        }
    }
    #[test]
    fn blocked_swing_returns_the_second_real_submission_without_replaying_on_return() {
        let (mut core, mut services, mut tasks) = fixture();
        core.player.action = player::Action::Swing;
        core.actors[0].animation = player::SWING_ANIMATION;
        {
            let animation_value = 2;
            core.actors[0].playback.set_frame_code(animation_value);
        };
        core.actors[0].playback.restart_clock();
        core.actors[0].position.screen_y = 0x80;
        core.actors[0].position.world_x = 0x100;
        core.actors[0].vertical_velocity = 0x100;
        services.commands.slots[1] = 0x11;
        let increment = complete(
            &mut PlayerCall::default(),
            &mut core,
            &mut services,
            &mut tasks,
            FirstActorIncrement::One,
        );
        assert_eq!(increment, FirstActorIncrement::Zero);
        assert_eq!(services.commands.slots[0], player::SWING_SOUND);
        assert_eq!(services.commands.slots[1], 0x11);
        assert_eq!(services.commands.producer, 1);
        assert_eq!(core.actors[0].animation, player::BLOCKED_SWING_ANIMATION);
    }
}

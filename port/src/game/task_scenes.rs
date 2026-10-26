//! Resumable scene/special-room/ending calls over the same canonical owners.
//! A pending operation resumes before its enclosing scene policy advances.
use super::*;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct ContentSession {
    content: scenes::Content,
    pending: Option<scenes::ContentWork>,
}
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct SpecialSession {
    session: scenes::SpecialSession,
    pending: Option<scenes::SpecialWork>,
}
impl Tasks {
    pub fn scene_content(
        &mut self,
        scene: scenes::Scene,
        core: &mut Core,
        services: &mut Services,
    ) -> OwnerStep<()> {
        if self.scene_content.is_none() {
            self.scene_content = Some(ContentSession {
                content: scenes::Content::new(scene),
                pending: None,
            });
        }
        loop {
            let session = self.scene_content.as_mut().unwrap();
            let work = if let Some(work) = session.pending.take() {
                work
            } else {
                session.content.resume(
                    &services.content.scenes,
                    &mut scenes::ContentContext {
                        actors: &mut core.actors,
                        display: &mut services.video.display,
                        camera: &mut core.camera,
                        cursor: &mut core.stream,
                        scene_cursor: &mut services.scene_cursor,
                        display_mode: &mut services.video.gameplay_display,
                        split_y: &mut services.video.split_y,
                        title_option: services.option.source_selector(),
                        ending_variation: self.ending.variation,
                    },
                    scenes::OwnerResult::Complete,
                )
            };
            let step = match work.clone() {
                scenes::ContentWork::FadeOut => {
                    self.fade(services, presentation::FadeDirection::Out)
                }
                scenes::ContentWork::FadeIn => self.fade(services, presentation::FadeDirection::In),
                scenes::ContentWork::ClearNonplayer => {
                    clear_nonplayer(core);
                    OwnerStep::Complete(())
                }
                scenes::ContentWork::HideOam => {
                    services.video.hide_sprites();
                    OwnerStep::Complete(())
                }
                scenes::ContentWork::Yield => self.wait(WaitOwner::SceneYield, 1),
                scenes::ContentWork::Disable => {
                    services.video.display.disable();
                    OwnerStep::Complete(())
                }
                scenes::ContentWork::Enable => {
                    services.video.display.enable();
                    OwnerStep::Complete(())
                }
                scenes::ContentWork::FillNametable => {
                    services
                        .video
                        .display
                        .fill(0x24, 0x24, 0, presentation::Stride::Across);
                    services.video.commit_writes();
                    OwnerStep::Complete(())
                }
                scenes::ContentWork::Palette(scene) => {
                    let parameters = services.content.scenes.parameters[scene as usize];
                    services.video.display.scene_palette(
                        &services.content.scene_palette_colors,
                        parameters.background_end,
                        parameters.sprite_end,
                        usize::from(services.option.source_selector()),
                        &services.content.scene_option_palettes,
                    );
                    OwnerStep::Complete(())
                }
                scenes::ContentWork::TerrainSpan {
                    scene,
                    bank,
                    room_in_row,
                    columns,
                } => {
                    core.terrain_bank = bank;
                    for ring in 0..columns {
                        services.build_column(
                            core,
                            world::StreamCursor {
                                room: scene as u8,
                                ring,
                                room_in_row,
                            },
                        );
                    }
                    OwnerStep::Complete(())
                }
                scenes::ContentWork::ScreenScript(selector) => {
                    screen_script(core, services, usize::from(selector));
                    OwnerStep::Complete(())
                }
                scenes::ContentWork::Compose => {
                    select_bank(services, 5);
                    services.compose(core);
                    OwnerStep::Complete(())
                }
                scenes::ContentWork::Chr(selector) => {
                    copy_chr(services, usize::from(selector));
                    OwnerStep::Complete(())
                }
                scenes::ContentWork::Complete => {
                    self.scene_content = None;
                    return OwnerStep::Complete(());
                }
                scenes::ContentWork::Suspended => {
                    unreachable!("scene policy advances only after the real owner completes")
                }
            };
            if step == OwnerStep::Suspended {
                self.scene_content.as_mut().unwrap().pending = Some(work);
                return step;
            }
        }
    }
    pub fn special_room(&mut self, core: &mut Core, services: &mut Services) -> OwnerStep<()> {
        if self.special.is_none() {
            self.special = Some(SpecialSession {
                session: scenes::SpecialSession::new(core.progression.special_room)
                    .expect("source special-room producer"),
                pending: None,
            });
        }
        loop {
            let session = self.special.as_mut().unwrap();
            let work = if let Some(work) = session.pending.take() {
                work
            } else {
                let mut spawn_control = core.special_spawn.encode();
                let work = session.session.resume(
                    &services.content.scenes,
                    &mut core.actors,
                    &mut core.progression,
                    &mut core.player,
                    &mut scenes::SpecialControls {
                        display: &mut services.video.gameplay_display,
                        spawn_control: &mut spawn_control,
                        split: &mut core.split_transition,
                        upper_cursor: &mut core.spawn_window.upper,
                        lower_cursor: &mut core.spawn_window.lower,
                        room_start: &mut core.spawn_window.room_start,
                        scan_direction: &mut core.spawn_window.direction,
                        terrain_bank: &mut core.terrain_bank,
                    },
                    scenes::OwnerResult::Complete,
                );
                core.special_spawn = actors::SpecialControl::decode(spawn_control);
                work
            };
            let step = match work.clone() {
                scenes::SpecialWork::FadeOut => {
                    self.fade(services, presentation::FadeDirection::Out)
                }
                scenes::SpecialWork::FadeIn => self.fade(services, presentation::FadeDirection::In),
                scenes::SpecialWork::ClearNonplayer => {
                    clear_nonplayer(core);
                    OwnerStep::Complete(())
                }
                scenes::SpecialWork::Disable => {
                    services.video.display.disable();
                    OwnerStep::Complete(())
                }
                scenes::SpecialWork::Layout(selector) => {
                    core.layout = selector;
                    OwnerStep::Complete(())
                }
                scenes::SpecialWork::SpawnTables(selector) => {
                    assert!(services.content.spawn_planes.contains_key(&selector));
                    core.spawn_configuration = selector;
                    OwnerStep::Complete(())
                }
                scenes::SpecialWork::Chr(selector) => {
                    copy_chr(services, usize::from(selector));
                    OwnerStep::Complete(())
                }
                scenes::SpecialWork::Checkpoint(selector) => {
                    load_checkpoint(core, services, selector);
                    OwnerStep::Complete(())
                }
                scenes::SpecialWork::Palette(offset) => {
                    const PALETTE_BYTES: usize = presentation::PALETTE_COLORS / 2;
                    let palette = &services.content.raw_stage_presets
                        [usize::from(offset) / PALETTE_BYTES]
                        .background_palette;
                    services
                        .video
                        .display
                        .load_palette(palette, &services.content.progression.sprite_palette);
                    OwnerStep::Complete(())
                }
                scenes::SpecialWork::StageAudio(command) => {
                    services.commands.submit_stage(command);
                    OwnerStep::Complete(())
                }
                scenes::SpecialWork::Enable => {
                    services.video.display.enable();
                    OwnerStep::Complete(())
                }
                scenes::SpecialWork::Yield => self.wait(WaitOwner::SpecialYield, 1),
                scenes::SpecialWork::Complete => {
                    self.special = None;
                    return OwnerStep::Complete(());
                }
                scenes::SpecialWork::Suspended => {
                    unreachable!("special-room policy advances only after owner completion")
                }
            };
            if step == OwnerStep::Suspended {
                self.special.as_mut().unwrap().pending = Some(work);
                return step;
            }
        }
    }
    fn scene_actor(&mut self, core: &mut Core, services: &mut Services) -> OwnerStep<()> {
        if self.scene_actor_rebuild {
            if self.scene_content(scenes::Scene::Interior, core, services) == OwnerStep::Suspended {
                return OwnerStep::Suspended;
            }
            self.scene_actor_rebuild = false;
        }
        loop {
            match scenes::update_scene_actor(
                &mut core.actors,
                &services.content.scenes,
                core.player.gravity,
                &mut NativeMotion,
                &mut self.scene_actor,
                &mut services.video.display,
            ) {
                scenes::HandlerStep::Complete => return OwnerStep::Complete(()),
                scenes::HandlerStep::Compose => {
                    select_bank(services, 5);
                    services.compose(core);
                }
                scenes::HandlerStep::RebuildInterior => {
                    self.scene_actor_rebuild = true;
                    if self.scene_content(scenes::Scene::Interior, core, services)
                        == OwnerStep::Suspended
                    {
                        return OwnerStep::Suspended;
                    }
                    self.scene_actor_rebuild = false;
                }
                scenes::HandlerStep::Suspended => {
                    unreachable!("scene actor continuation retains its entered rebuild")
                }
            }
        }
    }
    pub fn ending(&mut self, core: &mut Core, services: &mut Services) -> OwnerStep<()> {
        self.busy_interrupt_only = false;
        loop {
            let work = if let Some(work) = self.ending_pending.take() {
                work
            } else {
                self.ending.resume(
                    &mut finale::EndingContext {
                        scene_cursor: &mut services.scene_cursor,
                        display_mode: &mut services.video.gameplay_display,
                        progression: &mut core.progression,
                        status: &mut core.status,
                        pool: &mut core.actors,
                        composition: &mut services.video.composition,
                        display: &mut services.video.display,
                        current_stage_audio: &mut services.commands.stage_command,
                    },
                    finale::OwnerResult::Complete,
                )
            };
            let step = match work {
                finale::EndingWork::TransferValues => {
                    // E1B7 calls FFC7 before building ending scenes: use the HUD.
                    self.transfer(core, services, status::TransferLayout::Selection)
                }
                finale::EndingWork::Wait(count) => self.wait(WaitOwner::EndingDelay, count),
                finale::EndingWork::StageAudio(command) => {
                    services.commands.submit_stage(command);
                    OwnerStep::Complete(())
                }
                finale::EndingWork::Layout(selector) => {
                    core.layout = selector;
                    OwnerStep::Complete(())
                }
                finale::EndingWork::Build(scene) => {
                    if self.scene_content.is_none() {
                        self.scene_actor = scenes::HandlerContinuation::Entry;
                        self.scene_actor_rebuild = false;
                    }
                    self.scene_content(scene, core, services)
                }
                finale::EndingWork::UpdateSceneActors => self.scene_actor(core, services),
                finale::EndingWork::Yield => self.wait(WaitOwner::EndingYield, 1),
                finale::EndingWork::SplitRaster(control) => {
                    if !services.video.ui_sprite_zero_hit() {
                        self.busy_interrupt_only = true;
                        OwnerStep::Suspended
                    } else {
                        services.video.raster.playfield_control = Some(control);
                        services.video.raster.playfield_scroll =
                            Some(services.video.raster.ui_scroll);
                        services.video.raster.playfield_mask = services.video.raster.ui_mask;
                        OwnerStep::Complete(())
                    }
                }
                finale::EndingWork::Complete => return OwnerStep::Complete(()),
                finale::EndingWork::Suspended => {
                    unreachable!("ending resumes only after its actual callee completes")
                }
            };
            if step == OwnerStep::Suspended {
                self.ending_pending = Some(work);
                return step;
            }
        }
    }
}
fn copy_chr(services: &mut Services, selector: usize) {
    let content = std::sync::Arc::clone(&services.content);
    let prepared = &content.chr[selector];
    services.video.display.apply_chr(prepared);
    services.video.commit_writes();
    if let Some(bank) = prepared.selected_bank {
        select_bank(services, bank);
    }
}

//! Cooperative source calls. Each suspended owner retains its own continuation;
//! the single task wait only represents the innermost outstanding frame wait.
use super::{
    menu::Menu,
    simulation::{Core, Services},
};
use crate::{
    actors, assets::formats, collision, finale, player, presentation, progression, scenes,
    selection, status, world,
};
use progression::OwnerStep;
use std::collections::VecDeque;
#[path = "task_scenes.rs"]
mod scene_calls;
#[path = "task_vertical.rs"]
mod vertical_calls;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WaitOwner {
    MenuYield,
    MenuDelay,
    Transfer,
    Completion,
    InitialYield,
    StatusRow,
    Conversion,
    Composed,
    HorizontalYield,
    RaceYield,
    SceneYield,
    SpecialYield,
    EndingYield,
    EndingDelay,
    VerticalYield,
    DialogueWait,
    ForegroundYield,
}
#[derive(Clone)]
struct Delay {
    owner: WaitOwner,
    remaining: u16,
}
#[derive(Clone)]
struct StatusRows {
    rows: VecDeque<status::RowUpload>,
    waiting: bool,
}
#[derive(Clone)]
struct Conversion {
    phase: status::ConversionPhase,
    wait_ticks: u16,
}
/// Source final operation exposed to the enclosing player-return adapter.
pub enum ConversionReturn {
    EvenComposition,
    ValueSound(crate::audio::Submission),
    FinishedWait,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleReturn {
    Gameplay,
    LandSelection,
    Reset,
}
#[derive(Clone)]
struct Initial {
    kind: progression::Lifecycle,
    phase: progression::Phase,
    operations: VecDeque<progression::Operation>,
    saved_background: Option<(crate::identifiers::StageId, u8)>,
    exit: LifecycleReturn,
}
#[derive(Clone, Copy, Debug)]
enum HorizontalPhase {
    Move,
    Wait,
}
#[derive(Clone)]
struct Horizontal {
    saved: progression::SavedHorizontalContext,
    direction: u8,
    phase: HorizontalPhase,
}
#[derive(Clone)]
struct ComposedWait {
    remaining: u16,
    waiting: bool,
}
#[derive(Default, Clone)]
pub struct Tasks {
    delay: Option<Delay>,
    fade: Option<presentation::Fade>,
    background: Option<(formats::BackgroundUpload, formats::BackgroundSelection)>,
    transfer: Option<status::TransferSession>,
    transfer_wait: bool,
    completion: Option<scenes::Completion>,
    completion_wait: Option<u16>,
    initial: Option<Initial>,
    composed_wait: Option<ComposedWait>,
    horizontal: Option<Horizontal>,
    vertical: Option<vertical_calls::VerticalCall>,
    race: Option<finale::RaceTerrain>,
    race_wait: bool,
    scene_content: Option<scene_calls::ContentSession>,
    special: Option<scene_calls::SpecialSession>,
    scene_actor: scenes::HandlerContinuation,
    scene_actor_rebuild: bool,
    ending: finale::Ending,
    ending_pending: Option<finale::EndingWork>,
    /// A raster polling loop can wait for interrupts without yielding its task.
    pub busy_interrupt_only: bool,
    status_rows: Option<StatusRows>,
    conversion: Option<Conversion>,
    /// Exact value from the last B12 message/transfer producer, retained through
    /// acknowledgement yields. This is not a global arithmetic flag or queue.
    pub submission_value: u8,
}
impl Tasks {
    pub fn observe(&self, services: &Services) -> serde_json::Value {
        use serde_json::json;
        json!({"wait":self.delay.as_ref().map(|delay|json!({"owner":format!("{:?}",delay.owner),"remaining":delay.remaining})),
            "fade":self.fade.as_ref().map(|fade|json!({"direction":format!("{:?}",fade.direction),"subtraction":fade.subtraction,"waits_remaining":fade.waits_remaining,"complete":fade.complete})),
            "transfer":self.transfer.as_ref().map(|transfer|json!({"source":transfer.source.display_codes(),"started":transfer.started,"waiting":self.transfer_wait})),
            "conversion":self.conversion.as_ref().map(|conversion|json!({"phase":format!("{:?}",conversion.phase),"wait_ticks":conversion.wait_ticks})),
            "status_rows":self.status_rows.as_ref().map(|rows|json!({"remaining":rows.rows.len(),"waiting":rows.waiting})),
            "lifecycle":self.initial.as_ref().map(|call|json!({"kind":format!("{:?}",call.kind),"phase":format!("{:?}",call.phase),"pending":call.operations.front().map(|operation|format!("{operation:?}"))})),
            "horizontal":self.horizontal.as_ref().map(|call|json!({"direction":call.direction,"phase":format!("{:?}",call.phase)})),
            "vertical":self.vertical.as_ref().map(|call|call.observe()),
            "background":self.background.as_ref().map(|(upload,selection)|{ let (record, chunk) = upload.position(&services.content.backgrounds[selection.list]); json!({"complete":upload.completed(),"record":record,"chunk":chunk}) }),
            "ending":{"phase":format!("{:?}",self.ending.phase),"variation":self.ending.variation,"pauses_left":self.ending.pauses_left,"split_delay":self.ending.split_delay,"pending":self.ending_pending.map(|work|format!("{work:?}"))},
            "scene_content_active":self.scene_content.is_some(),"special_room_active":self.special.is_some(),"busy_interrupt_only":self.busy_interrupt_only})
    }

    pub fn transition_active(&self) -> bool {
        self.initial.is_some()
            || self.horizontal.is_some()
            || self.vertical.is_some()
            || self.special.is_some()
            || self.scene_content.is_some()
    }
    pub fn foreground_yield(&mut self) -> OwnerStep<()> {
        self.wait(WaitOwner::ForegroundYield, 1)
    }
    /// Only these text callees can suspend. The dialogue borrow boundary itself
    /// completes in the current foreground turn; an actual wait uses this owner.
    pub fn dialogue_work(
        &mut self,
        work: crate::dialogue::Work,
        core: &mut Core,
        services: &mut Services,
    ) -> OwnerStep<()> {
        match work {
            crate::dialogue::Work::BuildRaceTerrain => self.race_terrain(core, services),
            crate::dialogue::Work::StatusStreams => self.status_streams(core, services),
            crate::dialogue::Work::Yield => self.wait(WaitOwner::DialogueWait, 1),
            crate::dialogue::Work::Wait(count) => self.wait(WaitOwner::DialogueWait, count),
            _ => unreachable!("only entered cooperative text owners leave the dialogue borrow"),
        }
    }
    fn wait(&mut self, owner: WaitOwner, count: u16) -> OwnerStep<()> {
        if let Some(delay) = &mut self.delay {
            assert_eq!(
                delay.owner, owner,
                "resume the actual innermost source wait"
            );
            delay.remaining -= 1;
            if delay.remaining == 0 {
                self.delay = None;
                OwnerStep::Complete(())
            } else {
                OwnerStep::Suspended
            }
        } else {
            self.delay = Some(Delay {
                owner,
                remaining: if count == 0 { 256 } else { count },
            });
            OwnerStep::Suspended
        }
    }
    fn fade(
        &mut self,
        services: &mut Services,
        direction: presentation::FadeDirection,
    ) -> OwnerStep<()> {
        if let Some(fade) = &mut self.fade {
            assert_eq!(fade.direction, direction);
            if fade.resumed() {
                if fade.complete {
                    self.fade = None;
                    return OwnerStep::Complete(());
                }
                fade.publish(&mut services.video.display);
            }
        } else {
            let fade = services.video.display.begin_fade(
                direction,
                &mut services.video.composition,
                services.video.gameplay_display != 0,
            );
            fade.publish(&mut services.video.display);
            self.fade = Some(fade);
        }
        OwnerStep::Suspended
    }
    /// B580 expands the clearing stream, then the selected stream. Each row is
    /// committed through the one primary queue before its actual one-frame wait.
    pub fn status_streams(&mut self, core: &mut Core, services: &mut Services) -> OwnerStep<()> {
        if self.status_rows.is_none() {
            let rows = services
                .content
                .status
                .update_streams(
                    usize::from(core.progression.status_selector),
                    core.player.action as u8,
                    &core.status,
                    &[
                        core.progression.escort_prerequisite,
                        core.progression.reward_progress,
                    ],
                )
                .expect("validated status stream");
            self.status_rows = Some(StatusRows {
                rows: rows.into(),
                waiting: false,
            });
        }
        if self.status_rows.as_ref().unwrap().waiting {
            if self.wait(WaitOwner::StatusRow, 1) == OwnerStep::Suspended {
                return OwnerStep::Suspended;
            }
            self.status_rows.as_mut().unwrap().waiting = false;
        }
        let Some(row) = self.status_rows.as_mut().unwrap().rows.pop_front() else {
            self.status_rows = None;
            return OwnerStep::Complete(());
        };
        services
            .video
            .prepare_primary(&row.bytes, u8::MAX, &mut core.cache.attributes);
        self.status_rows.as_mut().unwrap().waiting = true;
        self.wait(WaitOwner::StatusRow, 1)
    }
    pub fn refresh_status(&mut self, core: &mut Core, services: &mut Services, total: bool) {
        let busy = services.video.display.primary.pending != 0;
        let upload = if total {
            core.status.queue_ui(busy)
        } else {
            core.status.queue_reward(busy)
        };
        if let Some(bytes) = upload {
            services
                .video
                .prepare_primary(&bytes, u8::MAX, &mut core.cache.attributes);
        }
    }
    /// A conversion call can suspend inside either real wait. Its entry is not
    /// rerun on resume, and normal even/odd calls return to the foreground loop.
    pub fn conversion(
        &mut self,
        core: &mut Core,
        services: &mut Services,
    ) -> OwnerStep<ConversionReturn> {
        let phase = if let Some(conversion) = &self.conversion {
            let (phase, wait_ticks) = (conversion.phase, conversion.wait_ticks);
            if self.wait(WaitOwner::Conversion, wait_ticks) == OwnerStep::Suspended {
                return OwnerStep::Suspended;
            }
            self.conversion = None;
            phase
        } else {
            status::ConversionPhase::Entry
        };
        let outcome = core.status.convert_timer(
            phase,
            services.commands.stage_command,
            services.video.composition.clock,
        );
        if let Some(command) = outcome.stage_sound {
            services.commands.submit_stage(command);
        }
        if outcome.freeze_player_animation {
            core.actors[0].playback.restart_clock();
        }
        if outcome.completion_increment {
            core.progression.scene_signal = core.progression.scene_signal.wrapping_add(1);
            return OwnerStep::Complete(ConversionReturn::FinishedWait);
        }
        if outcome.wait_ticks != 0 {
            let next = if outcome.wait_ticks == status::AUDIO_SETTLE_WAIT {
                status::ConversionPhase::AfterAudioWait
            } else {
                status::ConversionPhase::AfterFinishWait
            };
            self.conversion = Some(Conversion {
                phase: next,
                wait_ticks: outcome.wait_ticks,
            });
            self.wait(WaitOwner::Conversion, outcome.wait_ticks);
            return OwnerStep::Suspended;
        }
        if let Some(command) = outcome.value_sound {
            return OwnerStep::Complete(ConversionReturn::ValueSound(
                services.commands.submit(command),
            ));
        }
        OwnerStep::Complete(ConversionReturn::EvenComposition)
    }
    pub fn selection(
        &mut self,
        operation: selection::Operation,
        menu: &mut Menu,
        core: &mut Core,
        services: &mut Services,
    ) -> OwnerStep<()> {
        use selection::Operation as O;
        match operation {
            O::Yield => return self.wait(WaitOwner::MenuYield, 1),
            O::Wait(count) => return self.wait(WaitOwner::MenuDelay, count),
            O::FadeIn => return self.fade(services, presentation::FadeDirection::In),
            O::FadeOut => return self.fade(services, presentation::FadeDirection::Out),
            O::TransferDisplayedValue => {
                return self.transfer(core, services, status::TransferLayout::Selection);
            }
            O::CompletionActor => return self.completion(core, services),
            O::EnterStageRuntime => return self.initial(core, services),
            O::Acknowledge => match selection::acknowledge(
                services.runtime.controllers.ports[0],
                self.submission_value,
            ) {
                selection::Acknowledgement::WaitOneFrame => return OwnerStep::Suspended,
                selection::Acknowledgement::Accepted {
                    primary_upload_pending,
                } => services.video.display.primary.pending = primary_upload_pending,
            },
            O::InstallControlShadow => {
                services.video.raster.ui_control = services.video.display.control
            }
            O::CopyBaseOverlays => services.copy_base_overlays(core),
            O::StageAudio(command) => {
                services.commands.submit_stage(command);
            }
            O::Sound(command) => {
                services.commands.submit(command);
            }
            O::ClearGameplayAndProtection => {
                services.video.gameplay_display = 0;
                core.player.protection = 0;
            }
            O::ClearGameplay => services.video.gameplay_display = 0,
            O::DisableRendering => services.video.display.disable(),
            O::EnableRendering => services.video.display.enable(),
            O::ResetTitleScroll => {
                services.video.ui_x = 0;
                services.video.ui_y = 0;
                core.camera.x = 0;
            }
            O::FillFirstNametable => {
                services
                    .video
                    .display
                    .fill(0x20, 0x24, 0, presentation::Stride::Across);
                services.video.commit_writes();
            }
            O::HideSprites => services
                .video
                .composition
                .hide(services.video.gameplay_display != 0),
            O::ClearNonplayerActors => clear_nonplayer(core),
            O::ScreenScript(selector) => screen_script(core, services, selector),
            O::Chr(selector) => {
                let prepared = &services.content.chr[selector];
                let bank = prepared.selected_bank;
                services.video.display.apply_chr(prepared);
                services.video.commit_writes();
                if let Some(bank) = bank {
                    select_bank(services, bank);
                }
            }
            O::StageChr => {
                let prepared = services.content.raw_chr[usize::from(core.progression.stage.0)]
                    .as_ref()
                    .expect("raw source CHR consumer assessed at stage entry");
                let bank = prepared.selected_bank;
                services.video.display.apply_chr(prepared);
                services.video.commit_writes();
                if let Some(bank) = bank {
                    select_bank(services, bank);
                }
            }
            O::TitlePalette => {
                let palette = &services.content.raw_stage_presets[6].background_palette;
                services
                    .video
                    .display
                    .load_palette(palette, &services.content.progression.sprite_palette);
                for (index, value) in selection::title_palette_edits().before_actors {
                    services.video.display.base[index] = value;
                    services.video.display.active[index] = value;
                }
            }
            O::TitlePaletteTail => {
                for (index, value) in selection::title_palette_edits().after_actors {
                    services.video.display.base[index] = value;
                    services.video.display.active[index] = value;
                }
            }
            O::InitializeTitleActors => {
                selection::initialize_title_actors(&mut core.actors, &services.content.selection)
            }
            O::InitializeLandActors => {
                selection::initialize_land_actors(&mut core.actors, &services.content.selection)
            }
            O::SaveStageSelector => menu.saved_stage = core.progression.stage.0,
            O::UseLandLayoutSelector => core.progression.stage.0 = 5,
            O::RestoreStageSelector => core.progression.stage.0 = menu.saved_stage,
            O::ClearOverlayGate => services.video.overlay_gate = 0,
            O::CopyTitleOverlay => core.status.lives = services.content.title_overlay[9],
            O::LoadLandPresentation | O::LoadStagePresentation => {
                services.load_stage_presentation(core, core.progression.stage.0)
            }
            O::BuildWholeTerrainRing => services.build_terrain(core, true),
            O::BuildInitialTerrainColumns => services.build_terrain(core, false),
            O::StatusMessage => {
                let bytes = services
                    .content
                    .status
                    .build_message(
                        core.progression.completion_actor,
                        core.progression.scene_signal,
                        core.progression.special_room,
                        core.progression.stage.0,
                        &mut core.actors,
                        &core.status,
                    )
                    .expect("status message format");
                self.submission_value = (bytes.len() - 1) as u8;
                services.video.prepare_primary(
                    &bytes,
                    self.submission_value,
                    &mut core.cache.attributes,
                );
            }
            O::Message(selector) => {
                let message = services
                    .content
                    .status
                    .write_message(
                        usize::from(selector),
                        core.progression.stage.0,
                        &core.status,
                    )
                    .expect("direct status message format");
                self.submission_value = message.submission_value;
                services.video.prepare_primary(
                    &message.bytes,
                    self.submission_value,
                    &mut core.cache.attributes,
                );
            }
            O::Compose => {
                select_bank(services, 5);
                services.compose(core);
            }
            O::ClearActorOne => {
                let actor_value = 0;
                core.actors[1].set_state_code(actor_value);
            }
        }
        OwnerStep::Complete(())
    }
    pub fn transfer(
        &mut self,
        core: &mut Core,
        services: &mut Services,
        layout: status::TransferLayout,
    ) -> OwnerStep<()> {
        if self.transfer_wait {
            if self.wait(WaitOwner::Transfer, status::TRANSFER_WAIT) == OwnerStep::Suspended {
                return OwnerStep::Suspended;
            }
            self.transfer_wait = false;
        }
        let session = self.transfer.get_or_insert(status::TransferSession {
            source: core.status.reward,
            started: false,
        });
        let result = core.status.transfer_batch(layout, session);
        if let Some(value) = result.submission_value {
            self.submission_value = value;
        }
        if let Some(upload) = result.upload {
            services.video.prepare_primary(
                &upload,
                self.submission_value,
                &mut core.cache.attributes,
            );
        }
        if let Some(sound) = result.sound {
            services.commands.submit(sound);
        }
        if result.complete {
            self.transfer = None;
            return OwnerStep::Complete(());
        }
        self.transfer_wait = true;
        self.wait(WaitOwner::Transfer, result.wait_ticks)
    }
    pub fn completion(&mut self, core: &mut Core, services: &mut Services) -> OwnerStep<()> {
        if let Some(count) = self.completion_wait {
            if self.wait(WaitOwner::Completion, count) == OwnerStep::Suspended {
                return OwnerStep::Suspended;
            }
            self.completion_wait = None;
        }
        loop {
            let completion = self
                .completion
                .get_or_insert_with(scenes::Completion::default);
            match completion.resume(
                &mut core.actors,
                &mut core.progression,
                &mut services.video.display,
                &mut core.player.gravity,
                &mut NativeMotion,
            ) {
                scenes::CompletionStep::Compose => services.compose(core),
                scenes::CompletionStep::Yield => {
                    self.completion_wait = Some(1);
                    return self.wait(WaitOwner::Completion, 1);
                }
                scenes::CompletionStep::Wait(count) => {
                    self.completion_wait = Some(count);
                    return self.wait(WaitOwner::Completion, count);
                }
                scenes::CompletionStep::Complete => {
                    self.completion = None;
                    return OwnerStep::Complete(());
                }
            }
        }
    }
    /// One actual C42C call prepares at most one64-byte chunk and never yields.
    /// Its caller determines whether to poll again now or after a real wait.
    pub fn poll_background(&mut self, core: &mut Core, services: &mut Services) -> bool {
        let (upload, selection) = self.background.get_or_insert_with(|| {
            (
                formats::BackgroundUpload::default(),
                services
                    .content
                    .background_parameters
                    .select(formats::BackgroundContext {
                        stage: core.progression.stage,
                        room_row: core.row,
                        camera_page: (core.camera.x >> 8) as u8,
                        display_suspend_depth: services.video.display.suspended,
                    }),
            )
        });
        let prepared = &services.content.backgrounds[selection.list];
        match upload
            .poll_prepared(
                prepared,
                selection.palette,
                services.video.display.suspended,
            )
            .expect("decoded background stream")
        {
            formats::BackgroundEffect::Chunk {
                destination,
                bytes,
                count_minus_one,
                commit,
                selected_bank,
                queue_terminator,
            } => {
                let mut encoded =
                    vec![(destination >> 8) as u8, destination as u8, count_minus_one];
                encoded.extend(bytes);
                encoded.push(queue_terminator);
                services.video.prepare_primary(
                    &encoded,
                    queue_terminator,
                    &mut core.cache.attributes,
                );
                select_bank(services, selected_bank);
                if commit == formats::UploadCommit::Immediate {
                    let shared = services.video.primary_bytes(&core.cache.attributes);
                    services.video.display.flush(
                        presentation::Queue::Primary,
                        &shared,
                        presentation::Stride::Across,
                    );
                    services.video.commit_writes();
                }
                false
            }
            formats::BackgroundEffect::PaletteTail { base, active, .. } => {
                services.video.display.base[28..].copy_from_slice(&base);
                services.video.display.active[28..].copy_from_slice(&active);
                true
            }
            formats::BackgroundEffect::None => true,
        }
    }
    fn background(&mut self, core: &mut Core, services: &mut Services) -> OwnerStep<()> {
        while !self.poll_background(core, services) {}
        self.background = None;
        OwnerStep::Complete(())
    }
    pub fn boundary(&mut self, core: &mut Core, services: &mut Services) -> OwnerStep<()> {
        if self.horizontal.is_none() {
            let transition = progression::boundary(
                &mut core.progression,
                core.row,
                core.actors[0].position.world_x,
                &services.content.progression,
            );
            let progression::Boundary::Horizontal {
                direction,
                background_row,
            } = transition
            else {
                return OwnerStep::Complete(());
            };
            core.scroll_direction = direction;
            clear_nonplayer(core);
            let saved =
                progression::begin_horizontal(&mut core.progression, &mut core.row, background_row);
            self.background = None;
            self.horizontal = Some(Horizontal {
                saved,
                direction,
                phase: HorizontalPhase::Move,
            });
        }
        let horizontal = self.horizontal.as_ref().unwrap();
        if matches!(horizontal.phase, HorizontalPhase::Wait) {
            if self.wait(WaitOwner::HorizontalYield, 1) == OwnerStep::Suspended {
                return OwnerStep::Suspended;
            }
            self.horizontal.as_mut().unwrap().phase = HorizontalPhase::Move;
        }
        let direction = self.horizontal.as_ref().unwrap().direction;
        let actor = &mut core.actors[0];
        let low_fixed =
            u16::from(actor.position.world_x as u8) << 8 | u16::from(actor.position.x_fraction);
        let step = progression::horizontal_step(core.camera.x, low_fixed, direction);
        core.camera.x = step.camera;
        actor.position.world_x = (actor.position.world_x & 0xff00) | (step.player_low_fixed >> 8);
        actor.position.x_fraction = step.player_low_fixed as u8;
        self.poll_background(core, services);
        if step.done {
            let saved = self.horizontal.take().unwrap().saved;
            let sound = progression::finish_horizontal(
                saved,
                &mut core.progression,
                &mut core.row,
                &mut services.video.display.palette_pending,
            );
            services.commands.submit_stage(sound);
            return OwnerStep::Complete(());
        }
        core.camera.previous_x = step
            .previous_x
            .expect("continuing horizontal transition has previous camera byte");
        if step.clear_displacement {
            core.camera_displacement = 0;
        }
        services
            .video
            .composition
            .hide(services.video.gameplay_display != 0);
        select_bank(services, 5);
        services.compose(core);
        self.horizontal.as_mut().unwrap().phase = HorizontalPhase::Wait;
        self.wait(WaitOwner::HorizontalYield, 1)
    }
    /// Final-race terrain owns its explicit per-column and final-upload yields.
    /// A background poll itself returns immediately and retains its progress.
    pub fn race_terrain(&mut self, core: &mut Core, services: &mut Services) -> OwnerStep<()> {
        if self.race.is_none() {
            self.race = Some(finale::RaceTerrain::default());
            self.background = None;
        }
        if self.race_wait {
            if self.wait(WaitOwner::RaceYield, 1) == OwnerStep::Suspended {
                return OwnerStep::Suspended;
            }
            self.race_wait = false;
        }
        loop {
            let mut state = if self
                .background
                .as_ref()
                .is_some_and(|(upload, _)| upload.completed())
            {
                0x80
            } else {
                0
            };
            let work = self.race.as_mut().unwrap().resume(
                &mut finale::RaceContext {
                    pool: &mut core.actors,
                    progression: &mut core.progression,
                    camera: &mut core.camera,
                    cursor: &mut core.stream,
                    row: &mut core.row,
                    background_state: &mut state,
                    display: &mut services.video.display,
                },
                finale::OwnerResult::Complete,
            );
            match work {
                finale::RaceWork::ClearNonplayer => clear_nonplayer(core),
                finale::RaceWork::PollBackground => {
                    self.poll_background(core, services);
                }
                finale::RaceWork::ResolveForwardColumn => services.build_column(core, core.stream),
                finale::RaceWork::Yield => {
                    self.race_wait = true;
                    return self.wait(WaitOwner::RaceYield, 1);
                }
                finale::RaceWork::FindSpawnWindow => {
                    let planes = &services.content.spawn_planes[&core.spawn_configuration];
                    core.spawn_window = actors::reset_window(
                        planes.room_starts[usize::from(core.row)],
                        &planes.x_high,
                        core.camera.x,
                    );
                    core.spawn_window.direction = 1;
                }
                finale::RaceWork::SpawnWindow => services.spawn(core),
                finale::RaceWork::Complete => {
                    self.race = None;
                    return OwnerStep::Complete(());
                }
                finale::RaceWork::Suspended => {
                    unreachable!("race is resumed only after its actual inner owner completes")
                }
            }
        }
    }
    /// DD2F freezes player tick after each resumed wait; game-over and selection
    /// use their distinct source loops, preserving their hide/freeze differences.
    pub fn composed_wait(
        &mut self,
        core: &mut Core,
        services: &mut Services,
        count: u16,
        kind: progression::Lifecycle,
    ) -> OwnerStep<()> {
        self.composed_wait.get_or_insert(ComposedWait {
            remaining: if count == 0 { 256 } else { count },
            waiting: false,
        });
        if self.composed_wait.as_ref().unwrap().waiting {
            if self.wait(WaitOwner::Composed, 1) == OwnerStep::Suspended {
                return OwnerStep::Suspended;
            }
            let wait = self.composed_wait.as_mut().unwrap();
            let (remaining, done) = progression::composed_wait_step(
                wait.remaining,
                &mut core.actors[0].playback,
                kind == progression::Lifecycle::Reload,
            );
            wait.remaining = remaining;
            if done {
                self.composed_wait = None;
                return OwnerStep::Complete(());
            }
            wait.waiting = false;
        }
        services.video.oam_guard = services.video.oam_guard.wrapping_add(1);
        if kind != progression::Lifecycle::Selection {
            services
                .video
                .composition
                .hide(services.video.gameplay_display != 0);
        }
        services
            .runtime
            .bank
            .select(crate::runtime::PrgBank::PresentationFive);
        services.compose(core);
        services.video.oam_guard = services.video.oam_guard.wrapping_sub(1);
        self.composed_wait.as_mut().unwrap().waiting = true;
        self.wait(WaitOwner::Composed, 1)
    }
    fn initial(&mut self, core: &mut Core, services: &mut Services) -> OwnerStep<()> {
        match self.lifecycle(progression::Lifecycle::Initial, core, services) {
            OwnerStep::Suspended => OwnerStep::Suspended,
            OwnerStep::Complete(LifecycleReturn::Gameplay) => OwnerStep::Complete(()),
            OwnerStep::Complete(_) => unreachable!("initial-stage lifecycle has no other exit"),
        }
    }
    pub fn lifecycle(
        &mut self,
        kind: progression::Lifecycle,
        core: &mut Core,
        services: &mut Services,
    ) -> OwnerStep<LifecycleReturn> {
        if self.initial.is_none() {
            if kind == progression::Lifecycle::Initial {
                self.background = None;
            }
            self.initial = Some(Initial {
                kind,
                phase: progression::Phase::Entry,
                operations: VecDeque::new(),
                saved_background: None,
                exit: LifecycleReturn::Gameplay,
            });
        }
        loop {
            let initial = self.initial.as_mut().expect("initial stage continuation");
            if initial.operations.is_empty() {
                if initial.phase == progression::Phase::Done {
                    let exit = initial.exit;
                    self.initial = None;
                    return OwnerStep::Complete(exit);
                }
                let batch = progression::lifecycle(
                    initial.kind,
                    initial.phase,
                    core.progression.reload_request,
                    core.progression.special_room,
                );
                initial.phase = batch.next;
                initial.operations.extend(batch.operations);
            }
            let operation = self
                .initial
                .as_ref()
                .and_then(|initial| initial.operations.front())
                .expect("nonempty initial batch")
                .clone();
            use progression::Operation as O;
            let step = match operation {
                O::PollBackground => self.background(core, services),
                O::Yield => self.wait(WaitOwner::InitialYield, 1),
                O::FadeIn => self.fade(services, presentation::FadeDirection::In),
                O::LoadSpawnTables => {
                    assert!(
                        services
                            .content
                            .spawn_planes
                            .contains_key(&core.progression.stage.0)
                    );
                    core.spawn_configuration = core.progression.stage.0;
                    OwnerStep::Complete(())
                }
                O::ResetSpawnWindow => {
                    let planes = &services.content.spawn_planes[&core.spawn_configuration];
                    core.spawn_window = actors::reset_window(
                        planes.room_starts[usize::from(core.row)],
                        &planes.x_high,
                        core.camera.x,
                    );
                    core.spawn_window.direction = 1;
                    OwnerStep::Complete(())
                }
                O::InitializePlayer => {
                    progression::initialize_player(
                        &mut core.player,
                        &mut core.actors,
                        &mut core.camera,
                    );
                    if let Some(level) = &services.content.active_file {
                        core.actors[0].position.screen_y = level.rooms.player_y;
                    }
                    core.split_transition = 0;
                    services.video.split_address = 0x24c0;
                    OwnerStep::Complete(())
                }
                O::CopyStageUi => {
                    let mut bytes = services.content.stage_ui_template.clone();
                    bytes[4..11].copy_from_slice(&core.status.total.display_codes());
                    services
                        .video
                        .prepare_primary(&bytes, 0xff, &mut core.cache.attributes);
                    OwnerStep::Complete(())
                }
                O::EnableRendering => {
                    services.video.display.enable();
                    OwnerStep::Complete(())
                }
                O::SetGameplayDisplay => {
                    if self.initial.as_ref().unwrap().kind == progression::Lifecycle::Initial {
                        services.video.split_y = 0x2c;
                    }
                    services.video.gameplay_display = 0x40;
                    OwnerStep::Complete(())
                }
                O::ClearInitialControls => {
                    let mut actor_slot = services.dialogue.actor_slot as u8;
                    progression::clear_control_range(
                        progression::ControlReset::Initial,
                        &mut core.progression,
                        &mut core.player,
                        &mut core.status,
                        &mut progression::TransientOwners {
                            departure_signal: &mut services.departure_signal,
                            dialogue_actor_slot: &mut actor_slot,
                            overlay_gate: &mut services.video.overlay_gate,
                        },
                    );
                    services.dialogue.actor_slot = usize::from(actor_slot);
                    core.progression.late_mode = 0;
                    core.special_spawn = actors::SpecialControl::Ready;
                    core.player.action = player::Action::Standing;
                    core.progression.scene_signal = 0;
                    core.camera.previous_x = 0;
                    OwnerStep::Complete(())
                }
                O::CopyStatusAndTimer => {
                    services.video.timer_layout = super::video::TimerLayout::Countdown;
                    core.status.reward =
                        <[u8; 6]>::try_from(&services.content.status_template[4..10])
                            .expect("reward template width")
                            .into();
                    core.status.countdown = [
                        services.content.timer_overlay[17],
                        services.content.timer_overlay[21],
                        services.content.timer_overlay[25],
                    ]
                    .into();
                    OwnerStep::Complete(())
                }
                O::RestoreHealth => {
                    core.status.health.restore();
                    OwnerStep::Complete(())
                }
                O::InitializeCountdown => {
                    let pair = services.content.raw_stage_presets
                        [usize::from(core.progression.stage.0)]
                    .countdown_pair;
                    core.status.countdown.replace_upper_codes(pair);
                    core.status.divider = status::COUNTDOWN_PERIOD;
                    OwnerStep::Complete(())
                }
                O::InitializeOverlaySentinel => {
                    services.video.overlay_gate = 0xff;
                    let actor = &mut core.actors[1];
                    {
                        let actor_value = 0xff;
                        actor.set_state_code(actor_value);
                    };
                    actor.render = 0xff;
                    actor.spawn_identity = None;
                    actor.position.screen_y = 0;
                    actor.position.world_x &= 0xff00;
                    actor.profile = 0;
                    OwnerStep::Complete(())
                }
                O::RebuildSuppression => {
                    rebuild_suppression(core, services);
                    OwnerStep::Complete(())
                }
                O::EnterGameplay => OwnerStep::Complete(()),
                O::ClearImpact => {
                    core.progression.impact_latch = 0;
                    OwnerStep::Complete(())
                }
                O::SelectReloadRoute => {
                    if core.progression.reload_request >= progression::LIFE_LOSS_FIRST {
                        services.commands.submit_stage(progression::LIFE_LOSS_SOUND);
                    }
                    match progression::reload_route(
                        core.progression.reload_request,
                        core.progression.special_room,
                        core.status.lives,
                    ) {
                        progression::ReloadRoute::GameOver => {
                            self.initial = Some(Initial {
                                kind: progression::Lifecycle::GameOver,
                                phase: progression::Phase::Entry,
                                operations: VecDeque::new(),
                                saved_background: None,
                                exit: LifecycleReturn::Reset,
                            });
                            continue;
                        }
                        progression::ReloadRoute::Checkpoint { sound, stage_sound } => {
                            if !stage_sound {
                                services.commands.submit(sound);
                            }
                        }
                        progression::ReloadRoute::LandSelection => {
                            unreachable!("special-room route exits before ordinary reload dispatch")
                        }
                    }
                    OwnerStep::Complete(())
                }
                O::GrantReloadProtection => {
                    core.player.protection = 1;
                    OwnerStep::Complete(())
                }
                O::WaitComposed(count) => {
                    let kind = self.initial.as_ref().unwrap().kind;
                    self.composed_wait(core, services, count, kind)
                }
                O::FadeOut => self.fade(services, presentation::FadeDirection::Out),
                O::ClearReloadControls => {
                    core.special_spawn = actors::SpecialControl::Ready;
                    core.progression.late_mode = 0;
                    let mut actor_slot = services.dialogue.actor_slot as u8;
                    progression::clear_control_range(
                        progression::ControlReset::Reload,
                        &mut core.progression,
                        &mut core.player,
                        &mut core.status,
                        &mut progression::TransientOwners {
                            departure_signal: &mut services.departure_signal,
                            dialogue_actor_slot: &mut actor_slot,
                            overlay_gate: &mut services.video.overlay_gate,
                        },
                    );
                    services.dialogue.actor_slot = usize::from(actor_slot);
                    core.split_transition = 0;
                    services.video.split_address = (services.video.split_address & 0xff00) | 0xc0;
                    OwnerStep::Complete(())
                }
                O::DisableRendering => {
                    services.video.display.disable();
                    OwnerStep::Complete(())
                }
                O::LoadCheckpoint => {
                    load_checkpoint(core, services, core.progression.reload_request);
                    OwnerStep::Complete(())
                }
                O::ClearActors => {
                    clear_nonplayer(core);
                    OwnerStep::Complete(())
                }
                O::InstallTemporaryBackground => {
                    self.background = None;
                    self.initial.as_mut().unwrap().saved_background =
                        Some((core.progression.stage, core.row));
                    let background = progression::reload_background(
                        core.progression.reload_request,
                        core.progression.stage,
                        core.row,
                    );
                    core.progression.stage = background.stage;
                    core.row = background.row;
                    core.progression.late_mode = background.late_mode;
                    if background.decrement_life {
                        core.status.lives = core.status.lives.wrapping_sub(1);
                    }
                    if background.reset_animation {
                        core.actors[0].animation = 0;
                        core.actors[0].playback.reset_frame();
                        core.actors[0].playback.restart_clock();
                    }
                    if background.restore_health {
                        core.status.health.restore();
                    }
                    OwnerStep::Complete(())
                }
                O::RestoreBackgroundContext => {
                    let (stage, row) = self
                        .initial
                        .as_mut()
                        .unwrap()
                        .saved_background
                        .take()
                        .expect("saved reload background selectors");
                    core.progression.stage = stage;
                    core.row = row;
                    OwnerStep::Complete(())
                }
                O::SetForwardScroll => {
                    core.scroll_direction = 1;
                    core.previous_scroll_direction = 1;
                    OwnerStep::Complete(())
                }
                O::RestoreAudioCountdown => {
                    if core.progression.late_mode & actors::ACTIVE != 0 {
                        services
                            .commands
                            .submit_stage(progression::LATE_STAGE_SOUND);
                    } else if services.commands.stage_command == progression::LIFE_LOSS_SOUND {
                        let pair = services.content.raw_stage_presets
                            [usize::from(core.progression.stage.0)]
                        .countdown_pair;
                        core.status.countdown = [pair[0], pair[1], 0].into();
                        core.status.divider = status::COUNTDOWN_PERIOD;
                        services.commands.submit_stage(core.progression.stage.0);
                    }
                    OwnerStep::Complete(())
                }
                O::Spawn => {
                    services.spawn(core);
                    OwnerStep::Complete(())
                }
                O::ActivatePlayer => {
                    progression::activate_reload_player(&mut core.actors);
                    OwnerStep::Complete(())
                }
                O::ClearReloadPlayerAndRequest => {
                    progression::finish_reload(
                        &mut core.progression,
                        &mut core.player,
                        &mut core.actors,
                    );
                    OwnerStep::Complete(())
                }
                O::StageSound(command) => {
                    services.commands.submit_stage(command);
                    OwnerStep::Complete(())
                }
                O::CopyGameOverOverlay => {
                    services.video.timer_layout = super::video::TimerLayout::GameOver;
                    let overlay = &services.content.progression.game_over_overlay;
                    core.status.countdown = [overlay[17], overlay[21], overlay[25]].into();
                    OwnerStep::Complete(())
                }
                O::Reset => {
                    self.initial.as_mut().unwrap().exit = LifecycleReturn::Reset;
                    OwnerStep::Complete(())
                }
                O::ReturnToLandSelect => {
                    self.initial.as_mut().unwrap().exit = LifecycleReturn::LandSelection;
                    OwnerStep::Complete(())
                }
            };
            if step == OwnerStep::Suspended {
                return OwnerStep::Suspended;
            }
            self.initial
                .as_mut()
                .expect("initial continuation")
                .operations
                .pop_front();
        }
    }
}
fn load_checkpoint(core: &mut Core, services: &mut Services, selector: u8) {
    if let Some(level) = services
        .content
        .active_file
        .clone()
        .filter(|level| core.layout == level.rooms.rule_stage)
    {
        let room = level.rooms.entry_room;
        let page = room & 7;
        core.row = room / level.rooms.layout.columns;
        core.camera.room = room;
        core.camera.x = u16::from(page) << 8;
        core.actors[0].position.screen_y = level.rooms.player_y;
        core.actors[0].position.world_x = u16::from(page) << 8 | u16::from(level.rooms.player_x);
        let build = world::initial_columns(
            &services.content.world.layouts[usize::from(core.layout)],
            room,
            page,
        );
        for cursor in build.visits {
            services.build_column(core, cursor);
        }
        core.stream = build.final_cursor;
        return;
    }

    let layout = if usize::from(core.layout) < services.content.world.layouts.len() {
        &services.content.world.layouts[usize::from(core.layout)]
    } else {
        services.content.raw_layouts[usize::from(core.layout)]
            .as_ref()
            .expect("checkpoint raw layout consumer")
    };
    let rooms = layout
        .rooms
        .iter()
        .map(|room| services.content.room_presence(*room))
        .collect::<Vec<_>>();
    let plan = progression::checkpoint_build(selector, &services.content.progression, &rooms)
        .expect("source checkpoint record");
    core.row = plan.checkpoint.row;
    core.camera.x = plan.camera_position;
    core.camera.room = plan.camera_room;
    core.actors[0].position.screen_y = plan.checkpoint.player_y;
    core.actors[0].position.world_x =
        u16::from(plan.player_page) << 8 | u16::from(plan.checkpoint.player_x);
    for cursor in plan.visits {
        services.build_column(core, cursor);
    }
    core.stream = plan.final_cursor;
}

fn clear_nonplayer(core: &mut Core) {
    collision::clear_transition_actors(&mut core.actors, core.progression.stage.0, core.row);
}
fn screen_script(core: &mut Core, services: &mut Services, selector: usize) {
    select_bank(services, 5);
    let writes = &services.content.presentation.scripts[selector];
    for write in writes {
        services.video.memory.write(*write);
    }
    if selector == 11 {
        let mut address = writes
            .last()
            .map_or(0, |write| write.address.wrapping_add(1));
        for value in core
            .status
            .total
            .display_codes()
            .into_iter()
            .chain(std::iter::once(0))
        {
            services.video.write_direct(address, value);
            address = address.wrapping_add(1);
        }
    }
}
pub(super) fn rebuild_suppression(core: &mut Core, services: &Services) {
    let rules = services
        .content
        .spawns
        .suppression
        .get(usize::from(core.progression.stage.0))
        .map_or(&[][..], Vec::as_slice);
    actors::rebuild_suppression(
        &mut core.suppression,
        rules,
        actors::SuppressionFlags {
            stage_bits: core.progression.progress_bits,
            source_a1: core.progression.escort_prerequisite,
            source_a3: core.progression.reward_progress,
            source_a6: core.progression.accepted_stages,
            source_a8: core.progression.seen_messages[0],
            source_ae: core.progression.stage_one_extension,
        },
    );
}
fn select_bank(services: &mut Services, bank: u8) {
    services
        .runtime
        .bank
        .select(crate::runtime::PrgBank::from_selector(bank));
}

pub struct NativeMotion;
impl scenes::Motion for NativeMotion {
    fn apply(
        &mut self,
        actor: &mut actors::ActorContact,
        operation: scenes::Integration,
        gravity: u8,
    ) -> bool {
        match operation {
            scenes::Integration::Gravity => {
                collision::gravity(actor, gravity);
                false
            }
            scenes::Integration::AddX => {
                collision::integrate_horizontal(actor, collision::HorizontalDirection::Positive)
            }
            scenes::Integration::SubtractX => {
                !collision::integrate_horizontal(actor, collision::HorizontalDirection::Negative)
            }
            scenes::Integration::AddY => collision::integrate_vertical(
                actor,
                collision::VerticalOperation::AddEncodedVelocity,
            ),
            scenes::Integration::SubtractY => !collision::integrate_vertical(
                actor,
                collision::VerticalOperation::SubtractEncodedVelocity,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn services() -> Services {
        use romatlas_port::assets::{MemoryAssets, Rom};
        let rom = Rom::from_bytes(
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap(),
            crate::config::ROM_SHA256,
        )
        .unwrap();
        let overrides = MemoryAssets::default();
        let assets = crate::assets::Assets::load(&rom, &overrides).unwrap();
        Services::new(super::super::content::Content::load(&rom, &assets, &overrides).unwrap())
    }
    #[test]
    fn status_stream_owner_waits_after_every_original_row_and_preserves_secondary_bytes() {
        let mut services = services();
        let mut core = Core::reset();
        let mut tasks = Tasks::default();
        services.video.display.secondary.encoded[0] = 0x73;
        let mut waits = 0;
        loop {
            if tasks.status_streams(&mut core, &mut services) == OwnerStep::Complete(()) {
                break;
            }
            waits += 1;
            assert_eq!(services.video.display.primary.pending, u8::MAX);
            // This fixture models the completed interrupt flush before resume.
            services.video.display.primary.pending = 0;
            assert!(
                waits <= 10,
                "original clearing+ordinary streams have ten rows"
            );
        }
        assert_eq!(waits, 10);
        assert_eq!(services.video.display.secondary.encoded[0], 0x73);
        assert!(tasks.status_rows.is_none());
    }
    #[test]
    fn conversion_resumes_after_exact_audio_and_finish_waits_and_returns_actual_submission() {
        let mut services = services();
        let mut core = Core::reset();
        let mut tasks = Tasks::default();
        core.status.reward = [status::BLANK; 6].into();
        core.status.countdown = [0, 1, 9].into();
        {
            let animation_value = 9;
            core.actors[0].playback.set_tick_code(animation_value);
        };
        services.video.composition.clock = 1;
        services.commands.slots[1] = 0x11;
        assert!(matches!(
            tasks.conversion(&mut core, &mut services),
            OwnerStep::Suspended
        ));
        for _ in 1..status::AUDIO_SETTLE_WAIT {
            assert!(matches!(
                tasks.conversion(&mut core, &mut services),
                OwnerStep::Suspended
            ));
        }
        let OwnerStep::Complete(ConversionReturn::ValueSound(submission)) =
            tasks.conversion(&mut core, &mut services)
        else {
            panic!("256th resume must finish audio wait and convert this odd frame");
        };
        assert!(!submission.accepted);
        assert_eq!(
            submission.comparison,
            crate::audio::ProducerComparison::BelowEmpty
        );
        assert_eq!(
            (services.commands.slots[0], services.commands.producer),
            (status::STOP_STAGE_SOUND, 1)
        );
        assert_eq!(
            (
                core.status.countdown.display_codes(),
                core.status.reward.display_codes()[3],
                core.actors[0].playback.tick_code()
            ),
            ([0, 0, 0], 1, 0)
        );
        assert!(matches!(
            tasks.conversion(&mut core, &mut services),
            OwnerStep::Suspended
        ));
        for _ in 1..status::CONVERSION_FINISH_WAIT {
            assert!(matches!(
                tasks.conversion(&mut core, &mut services),
                OwnerStep::Suspended
            ));
        }
        assert!(matches!(
            tasks.conversion(&mut core, &mut services),
            OwnerStep::Complete(ConversionReturn::FinishedWait)
        ));
        assert_eq!(core.progression.scene_signal, 1);
    }
    #[test]
    fn authored_main_checkpoint_restores_entry_after_original_special_room_layout() {
        let base = super::super::levels::tests::content();
        let mut services = Services::new(base.authored_view("file/level").unwrap());
        let mut core = Core::reset();
        services.load_stage_presentation(&mut core, 0);
        core.layout = 7;
        let special_layout = services.content.world.layouts[7].clone();
        assert_eq!(
            special_layout, base.world.layouts[7],
            "authored main map preserves source special rooms"
        );
        core.layout = 0;
        core.camera.x = 0x1234;
        core.row = 9;
        core.actors[0].position.screen_y = 1;
        load_checkpoint(&mut core, &mut services, 0x81);
        assert_eq!(
            (
                core.row,
                core.camera.room,
                core.camera.x,
                core.actors[0].position.screen_y
            ),
            (0, 0, 0, 96)
        );
        assert_eq!(core.actors[0].position.world_x, 0x80);
        assert!(core.cache.collision.contains(&0x55));
        assert_eq!(
            services.content.active_file.as_ref().unwrap().id.0,
            "file/level"
        );
    }
}

//! Shared host executor. One tick performs one interrupt and resumes the same
//! source task until its next real yield. Borrow-boundary requests never create
//! extra frames. Busy raster/menu loops retain the running task without sampling.
use super::{
    content::Content,
    families,
    foreground::{self, Foreground, Request, Returned},
    menu::{self, Menu},
    player_call::{self, PlayerCall},
    simulation::{Core, Services},
    tasks::{LifecycleReturn, Tasks},
};
use crate::{
    progression::{self, OwnerStep, Work},
    runtime,
};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Route {
    Menu,
    Gameplay,
}
#[derive(Clone)]
pub struct Machine {
    pub core: Core,
    pub ticks: u64,
    pub services: Services,
    pub menu: Menu,
    pub route: Route,
    base_content: std::sync::Arc<Content>,
    pending_level: Option<String>,
    tasks: Tasks,
    foreground: Foreground,
    player: PlayerCall,
}
impl Machine {
    pub fn mode(&self) -> crate::identifiers::Mode {
        use crate::identifiers::Mode;
        if self.route == Route::Menu {
            return match self.menu.flow {
                menu::Flow::Title(_) => Mode::Title,
                menu::Flow::Confirm => Mode::Confirmation,
                menu::Flow::Stalled => Mode::Stalled,
                menu::Flow::Gameplay => Mode::Gameplay,
                _ => Mode::LandSelection,
            };
        }
        if self.core.progression.ending_latch != 0 {
            Mode::Ending
        } else if self.tasks.transition_active() {
            Mode::Transition
        } else if self.player.dialogue_active()
            || self.core.player.action == crate::player::Action::Dialogue
        {
            Mode::Dialogue
        } else if self.core.progression.status_selector != 0 {
            Mode::StatusPanel
        } else {
            Mode::Gameplay
        }
    }
    pub fn new(content: Content) -> Self {
        let base_content = std::sync::Arc::new(content);
        let mut machine = Self {
            base_content: base_content.clone(),
            pending_level: None,
            core: Core::reset(),
            ticks: 0,
            services: Services::with_content(base_content),
            menu: Menu::title(),
            route: Route::Menu,
            tasks: Tasks::default(),
            foreground: Foreground::default(),
            player: PlayerCall::default(),
        };
        // Reset dispatches the title entry before the first playable interrupt.
        // Scheduler Entry deliberately does not acquire input. Warm-up vblank
        // polling is not converted into extra native gameplay frames.
        machine.dispatch([runtime::PortSample::default(); runtime::CONTROLLER_PORTS]);
        machine
    }
    pub fn tick(&mut self, samples: [runtime::PortSample; runtime::CONTROLLER_PORTS]) {
        self.ticks += 1;
        self.services.events.set_tick(self.ticks);
        let busy = self.tasks.busy_interrupt_only
            || self.route == Route::Menu && self.menu.flow == menu::Flow::Stalled;
        self.services.interrupt(&mut self.core);
        if busy {
            self.run_foreground();
        } else {
            self.dispatch(samples);
        }
    }
    fn dispatch(&mut self, samples: [runtime::PortSample; runtime::CONTROLLER_PORTS]) {
        if let Some(dispatch) = self
            .services
            .runtime
            .scheduler
            .dispatch(&mut self.services.runtime.controllers, samples)
        {
            assert_eq!(
                dispatch.slot,
                runtime::TaskSlot::Foreground,
                "the original main call installs only slot zero"
            );
            self.run_foreground();
        }
    }
    fn reset(&mut self) {
        let before = self.mode();
        self.services.content = self.base_content.clone();
        self.pending_level = None;
        self.core = Core::reset();
        self.services.reset();
        self.menu = Menu::title();
        self.route = Route::Menu;
        self.tasks = Tasks::default();
        self.foreground = Foreground::default();
        self.player = PlayerCall::default();
        self.mode_change(before, "source_reset");
    }
    fn enter_gameplay(&mut self) {
        let before = self.mode();
        self.route = Route::Gameplay;
        self.foreground = Foreground::default();
        self.player = PlayerCall::default();
        self.core.progression.continuation = progression::FrameContinuation::Entry;
        self.core.progression.awaiting_owner = None;
        self.core.progression.first_actor_increment = None;
        self.mode_change(before, "enter_gameplay");
    }
    fn enter_lands(&mut self) {
        let before = self.mode();
        if self.services.content.active_file.is_some() {
            self.stop_authored_audio();
        }
        self.services.content = self.base_content.clone();
        self.menu.enter_lands();
        self.route = Route::Menu;
        self.mode_change(before, "enter_land_selection");
    }
    fn confirm_stage(&mut self) {
        let before = self.mode();
        self.menu.confirm();
        self.route = Route::Menu;
        self.mode_change(before, "confirm_stage");
    }
    fn poll_menu(&mut self) -> menu::Poll {
        if self.menu.operations.is_empty()
            && matches!(self.menu.flow, menu::Flow::Completion(_))
            && let Some(next) = self.pending_level.take()
        {
            if self.menu.flow
                == menu::Flow::Completion(crate::selection::CompletionRoute::LateStage)
            {
                self.core.progression.late_route = self.core.progression.late_route.wrapping_add(1);
            }
            self.activate_level(&next);
        }
        let before = super::journal::Snapshot::core(&self.core);
        let mode = self.mode();
        let result = self.menu.poll(&mut self.core, &mut self.services);
        before.finish_core(
            &self.services.events,
            &self.core,
            &self.services.content,
            "menu_policy",
        );
        self.mode_change(mode, "menu_policy");
        result
    }
    fn yield_task(&mut self) {
        if self.tasks.busy_interrupt_only
            || self.route == Route::Menu && self.menu.flow == menu::Flow::Stalled
        {
            return;
        }
        self.services
            .runtime
            .scheduler
            .yield_one_frame(runtime::TaskSlot::Foreground, runtime::BootTask::GameMain);
    }
    fn run_foreground(&mut self) {
        loop {
            if self.route == Route::Menu {
                match self.poll_menu() {
                    menu::Poll::Operation(operation) => {
                        let before = super::journal::Snapshot::core(&self.core);
                        let mode = self.mode();
                        let owner = format!("menu::{operation:?}");
                        let step = self.tasks.selection(
                            operation,
                            &mut self.menu,
                            &mut self.core,
                            &mut self.services,
                        );
                        before.finish_core(
                            &self.services.events,
                            &self.core,
                            &self.services.content,
                            &owner,
                        );
                        self.boundary(&owner, step == OwnerStep::Complete(()), mode);
                        if step == OwnerStep::Suspended {
                            self.yield_task();
                            return;
                        }
                        self.menu.completed();
                    }
                    menu::Poll::Gameplay => self.enter_gameplay(),
                    menu::Poll::Stalled => return,
                }
                continue;
            }
            let before = super::journal::Snapshot::core(&self.core);
            let mode = self.mode();
            let poll = self.foreground.poll(&mut self.core, &mut self.services);
            before.finish_core(
                &self.services.events,
                &self.core,
                &self.services.content,
                "foreground",
            );
            self.mode_change(mode, "foreground_policy");
            match poll {
                foreground::Poll::Requested(Request::Player) => {
                    let before = super::journal::Snapshot::core(&self.core);
                    let mode = self.mode();
                    let result = self.player.poll(
                        &mut self.core,
                        &mut self.services,
                        &mut self.tasks,
                        self.foreground.player_entry(),
                    );
                    before.finish_core(
                        &self.services.events,
                        &self.core,
                        &self.services.content,
                        "player_call",
                    );
                    self.boundary(
                        "player_call",
                        !matches!(result, player_call::Poll::Suspended),
                        mode,
                    );
                    match result {
                        player_call::Poll::Suspended => {
                            self.yield_task();
                            return;
                        }
                        player_call::Poll::Complete(result) => {
                            self.foreground.completed(Returned::Player(result))
                        }
                        player_call::Poll::VerticalTail => {
                            self.foreground.vertical_completed(&mut self.core)
                        }
                    }
                }
                foreground::Poll::Requested(Request::Owner(work)) => {
                    let before = (!matches!(work, Work::Actors(_) | Work::Spawn))
                        .then(|| super::journal::Snapshot::core(&self.core));
                    let mode = self.mode();
                    let mut exit = None;
                    let step = match work {
                        Work::Reload => match self.tasks.lifecycle(
                            progression::Lifecycle::Reload,
                            &mut self.core,
                            &mut self.services,
                        ) {
                            OwnerStep::Suspended => OwnerStep::Suspended,
                            OwnerStep::Complete(result) => {
                                exit = Some(result);
                                OwnerStep::Complete(())
                            }
                        },
                        _ => self.owner(work),
                    };
                    let owner = format!("foreground::{work:?}");
                    if let Some(before) = before {
                        before.finish_core(
                            &self.services.events,
                            &self.core,
                            &self.services.content,
                            &owner,
                        );
                    }
                    self.boundary(&owner, step == OwnerStep::Complete(()), mode);
                    if step == OwnerStep::Suspended {
                        self.yield_task();
                        return;
                    }
                    match exit {
                        Some(LifecycleReturn::LandSelection) => {
                            self.enter_lands();
                            continue;
                        }
                        Some(LifecycleReturn::Reset) => {
                            self.reset();
                            continue;
                        }
                        _ => {}
                    }
                    self.foreground.completed(Returned::Owner(work));
                }
                foreground::Poll::Frame(exit) => match exit {
                    progression::FrameExit::NextFrame => {} // Resume at loop entry in this SAME host turn.
                    progression::FrameExit::LandSelection => {
                        self.leave_completed_stage();
                    }
                    progression::FrameExit::StageOneConfirmation => {
                        self.confirm_stage();
                    }
                    progression::FrameExit::Suspended => {
                        unreachable!("all cooperative suspension exposes its entered request")
                    }
                },
            }
        }
    }
    fn leave_completed_stage(&mut self) {
        self.pending_level = self.successor();
        self.enter_lands();
    }
    pub(super) fn observe_calls(&self) -> serde_json::Value {
        self.tasks.observe(&self.services)
    }
    pub(super) fn pending_level(&self) -> Option<&str> {
        self.pending_level.as_deref()
    }
    fn stop_authored_audio(&mut self) {
        self.services.audio = crate::audio::Driver::default();
        let writes = [0x4000, 0x4004, 0x4008, 0x400c]
            .map(|address| crate::audio::RegisterWrite { address, value: 0 });
        self.services.synthesizer.apply(&writes);
    }
    fn successor(&self) -> Option<String> {
        // An ordinary return/cancellation does not count as authored completion.
        if self.core.progression.scene_signal == 0 && self.core.progression.completion_actor == 0 {
            return None;
        }
        let id = self
            .services
            .content
            .active_file
            .as_ref()
            .map(|level| level.id.clone())
            .or_else(|| {
                self.base_content
                    .levels
                    .original_level(self.core.progression.stage)
                    .map(|level| level.id.clone())
            })?;
        self.base_content.additions.successors.get(&id.0).cloned()
    }
    fn activate_level(&mut self, id: &str) {
        let view = self
            .base_content
            .authored_view(id)
            .expect("authored graph/payloads validated at startup");
        let stage = view
            .active_file
            .as_ref()
            .expect("authored view identity")
            .rooms
            .rule_stage;
        self.services.content = std::sync::Arc::new(view);
        // Old encoded cursors belong to the previous immutable program. Source
        // queue commands stay canonical; only the extension song driver stops.
        self.stop_authored_audio();
        self.core.progression.stage.0 = stage;
        self.core.progression.scene_signal = 0;
        self.core.progression.completion_actor = 0;
        self.tasks = Tasks::default();
        self.foreground = Foreground::default();
        self.player = PlayerCall::default();
        self.services.events.record(
            "LevelActivated",
            serde_json::json!({"id":id,"rule_stage":stage}),
        );
        self.confirm_stage();
    }
    fn mode_change(&self, before: crate::identifiers::Mode, owner: &str) {
        let after = self.mode();
        if before != after {
            self.services.events.record(
                "ModeChanged",
                serde_json::json!({"before":before.name(),"after":after.name(),"owner":owner}),
            );
        }
    }
    fn boundary(&self, owner: &str, complete: bool, previous_mode: crate::identifiers::Mode) {
        self.mode_change(previous_mode, owner);
        let mode = self.mode();
        self.services.events.record("OwnerBoundary", serde_json::json!({"owner":owner,"complete":complete,"mode":mode.name(),"primary_pending":self.services.video.display.primary.pending,"secondary_pending":self.services.video.display.secondary_pending,"palette_pending":self.services.video.display.palette_pending,"busy_interrupt_only":self.tasks.busy_interrupt_only,"spawn":{"lower":self.core.spawn_window.lower,"upper":self.core.spawn_window.upper,"special_control":self.core.special_spawn.encode()}}));
    }
    fn owner(&mut self, work: Work) -> OwnerStep<()> {
        let core = &mut self.core;
        let services = &mut self.services;
        match work {
            Work::Ending => return self.tasks.ending(core, services),
            Work::Compose => services.compose(core),
            Work::StatusStreams => return self.tasks.status_streams(core, services),
            Work::Actors(increment) => {
                let content = std::sync::Arc::clone(&services.content);
                let mut pending = u8::from(services.video.display.secondary_pending);
                let mut guard = services.video.oam_guard;
                families::update_actors(
                    &mut progression::Frame {
                        progression: &mut core.progression,
                        player: &mut core.player,
                        actors: &mut core.actors,
                        camera: &mut core.camera,
                        cache: &mut core.cache,
                        status: &mut core.status,
                        camera_displacement: &mut core.camera_displacement,
                        secondary_cursor: &mut core.secondary_cursor,
                        secondary_pending: &mut pending,
                        oam_guard: &mut guard,
                        room_row: &mut core.row,
                        catalog: &content.progression,
                    },
                    services,
                    &mut core.suppression,
                    &mut core.special_spawn,
                    core.layout,
                    core.terrain_bank,
                    &mut core.split_transition,
                    increment,
                );
                services.video.display.secondary_pending = pending != 0;
                services.video.oam_guard = guard;
            }
            Work::Countdown => {
                let result = core
                    .status
                    .timer_tick(core.player.action as u8, services.video.composition.clock);
                if let Some(action) = result.player_state {
                    core.player.action = super::contact::action(action);
                }
                if result.clear_late_route {
                    core.progression.late_route = 0;
                }
                if let Some(command) = result.sound {
                    services.commands.submit(command);
                }
            }
            Work::Camera => services.camera(core),
            Work::StageBoundary => return self.tasks.boundary(core, services),
            Work::Selection | Work::ConfirmStageOne => {} // Foreground returns the nonlocal menu route next.
            Work::RefreshStatus => self.tasks.refresh_status(core, services, false),
            Work::StageUi => self.tasks.refresh_status(core, services, true),
            Work::SpecialRoom => return self.tasks.special_room(core, services),
            Work::Spawn => services.spawn(core),
            Work::Yield => return self.tasks.foreground_yield(),
            Work::Reload => {
                unreachable!("reload can replace the main route and is handled by its caller")
            }
            Work::Sound(_)
            | Work::HideSprites
            | Work::PresentationBank
            | Work::ActorBank
            | Work::ClearActors
            | Work::AdvanceRandom => {
                unreachable!("nonyielding leaves complete inside the canonical foreground borrow")
            }
        }
        OwnerStep::Complete(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Machine {
        use romatlas_port::assets::{MemoryAssets, Rom};
        let bytes =
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap();
        let rom = Rom::from_bytes(bytes, crate::config::ROM_SHA256).unwrap();
        let overrides = MemoryAssets::default();
        let assets = crate::assets::Assets::load(&rom, &overrides).unwrap();
        let content = super::super::content::Content::load(&rom, &assets, &overrides).unwrap();
        let mut core = Core::reset();
        core.player.action = crate::player::Action::Hold;
        core.status.divider = crate::status::COUNTDOWN_PERIOD;
        core.progression.late_mode = 0x80;
        core.progression.race_intro = 1; // Source tail bypasses spawn; isolates scheduling.
        let base_content = std::sync::Arc::new(content);
        Machine {
            base_content: base_content.clone(),
            pending_level: None,
            core,
            ticks: 0,
            services: Services::with_content(base_content),
            menu: Menu::title(),
            route: Route::Gameplay,
            tasks: Tasks::default(),
            foreground: Foreground::default(),
            player: PlayerCall::default(),
        }
    }
    #[test]
    fn countdown_owner_preserves_warning_submission_and_next_call_death() {
        // E28C/E2E6: composition owns warning cadence; the countdown owner
        // commits its sound and next-call death into canonical services/player.
        let mut machine = fixture();
        machine.core.status.countdown = [0, 3, 0].into();
        machine.core.status.divider = 1;
        machine.services.video.composition.clock = 0;
        assert!(matches!(
            machine.owner(Work::Countdown),
            OwnerStep::Complete(())
        ));
        assert_eq!(machine.core.status.countdown.display_codes(), [0, 2, 9]);
        assert_eq!(
            machine.services.commands.drain(),
            vec![crate::status::WARNING_SOUND]
        );
        machine.core.status.countdown = [0, 0, 1].into();
        machine.core.status.divider = 1;
        machine.core.progression.late_route = 0x80;
        machine.owner(Work::Countdown);
        assert_eq!(
            machine.core.status.countdown.display_codes(),
            crate::status::EXPIRED
        );
        assert_eq!(machine.core.player.action, crate::player::Action::Hold);
        assert!(machine.services.commands.drain().is_empty());
        machine.owner(Work::Countdown);
        assert_eq!(machine.core.player.action, crate::player::Action::Death);
        assert_eq!(machine.core.progression.late_route, 0);
        assert_eq!(machine.core.status.divider, crate::status::COUNTDOWN_PERIOD);
        assert_eq!(
            super::super::observation::status_value(&machine.core.status)["countdown"],
            serde_json::json!(crate::status::EXPIRED)
        );
    }
    #[test]
    fn immediate_owner_requests_finish_before_the_one_actual_yield_and_resume_same_tick() {
        let mut machine = fixture();
        machine.dispatch([runtime::PortSample::default(); runtime::CONTROLLER_PORTS]);
        assert_eq!(machine.services.runtime.interrupt_tick, 0);
        assert_eq!(
            machine.core.status.divider,
            crate::status::COUNTDOWN_PERIOD - 1
        );
        assert_eq!(machine.services.video.composition.clock, 1);
        assert_eq!(
            machine.services.runtime.controllers.last_sample_return,
            None
        );
        assert!(matches!(
            machine.services.runtime.scheduler.slots[0],
            runtime::TaskState::Sleeping {
                ticks_remaining: 1,
                ..
            }
        ));
        let samples = [
            runtime::PortSample {
                standard: runtime::Buttons::UP | runtime::Buttons::DOWN,
                auxiliary: runtime::Buttons::NONE,
            },
            runtime::PortSample::default(),
        ];
        machine.tick(samples);
        assert_eq!(machine.services.runtime.interrupt_tick, 1);
        assert_eq!(
            machine.core.status.divider,
            crate::status::COUNTDOWN_PERIOD - 2
        );
        assert_eq!(machine.services.video.composition.clock, 2);
        assert_eq!(
            machine.services.runtime.controllers.last_sample_return,
            Some(runtime::SampleReturn {
                primary_opposite_edges: true
            })
        );
        assert_eq!(
            machine.services.runtime.controllers.ports[0].held,
            runtime::Buttons::NONE
        );
        assert!(matches!(
            machine.services.runtime.scheduler.slots[0],
            runtime::TaskState::Sleeping {
                ticks_remaining: 1,
                ..
            }
        ));
    }
    #[test]
    fn stalled_raw_selector_loop_runs_interrupt_audio_without_resuming_or_sampling() {
        let mut machine = fixture();
        machine.route = Route::Menu;
        machine.menu.flow = menu::Flow::Stalled;
        machine.services.runtime.scheduler.slots[0] =
            runtime::TaskState::Running(runtime::BootTask::GameMain);
        machine.tick(
            [runtime::PortSample {
                standard: runtime::Buttons::A,
                auxiliary: runtime::Buttons::NONE,
            }; runtime::CONTROLLER_PORTS],
        );
        assert_eq!(machine.services.runtime.interrupt_tick, 1);
        assert_eq!(
            machine.services.runtime.controllers.last_sample_return,
            None
        );
        assert_eq!(
            machine.services.runtime.controllers.ports[0].pressed,
            runtime::Buttons::NONE
        );
        assert!(matches!(
            machine.services.runtime.scheduler.slots[0],
            runtime::TaskState::Running(_)
        ));
        assert_eq!(machine.services.video.composition.clock, 0);
    }
    #[test]
    fn menu_policy_and_gameplay_handoff_record_each_actual_mode_transition_once() {
        let mut machine = fixture();
        machine.route = Route::Menu;
        machine.menu.flow = menu::Flow::LandInput;
        machine.services.runtime.controllers.ports[0].pressed = runtime::Buttons::A;
        assert!(matches!(machine.poll_menu(), menu::Poll::Operation(_)));
        let snapshot = machine.services.events.observe();
        let changes = snapshot["entries"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|entry| entry["kind"] == "ModeChanged")
            .collect::<Vec<_>>();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0]["payload"]["before"], "land_selection");
        assert_eq!(changes[0]["payload"]["after"], "confirmation");
        machine.menu.operations.clear();
        assert!(matches!(machine.poll_menu(), menu::Poll::Gameplay));
        machine.enter_gameplay();
        let snapshot = machine.services.events.observe();
        let changes = snapshot["entries"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|entry| entry["kind"] == "ModeChanged")
            .collect::<Vec<_>>();
        assert_eq!(changes.len(), 2);
        assert_eq!(changes[1]["payload"]["before"], "confirmation");
        assert_eq!(changes[1]["payload"]["after"], "gameplay");
    }

    #[test]
    fn completed_authored_chain_restores_base_and_runs_source_completion_before_next_view() {
        let mut content = super::super::levels::tests::content();
        let mut second = (**content.additions.levels.get("file/level").unwrap()).clone();
        second.id.0 = "file/second".into();
        second.rooms.after.0 = "file/level".into();
        let registry = std::sync::Arc::make_mut(&mut content.additions);
        registry
            .successors
            .insert("file/level".into(), "file/second".into());
        registry
            .levels
            .insert("file/second".into(), std::sync::Arc::new(second));
        let mut machine = fixture();
        machine.base_content = std::sync::Arc::new(content);
        machine.services.content = machine.base_content.clone();
        machine.activate_level("file/level");
        machine
            .services
            .audio
            .command(&machine.services.content.audio, 0, 0, &mut Vec::new());
        machine
            .services
            .audio
            .frame(&machine.services.content.audio, 0);
        assert!(machine.services.audio.channels[0].primary.cursor < 0x8000);
        machine.core.progression.progress_bits = 1;
        machine.core.progression.scene_signal = 1;
        machine.leave_completed_stage();
        assert!(std::sync::Arc::ptr_eq(
            &machine.services.content,
            &machine.base_content
        ));
        assert_eq!(machine.pending_level.as_deref(), Some("file/second"));
        machine.services.interrupt(&mut machine.core);
        assert_eq!(
            machine.services.audio.channels[0].primary.cursor, 0,
            "base interrupt cannot follow discarded authored pointers"
        );
        // The source menu's transfer/message/acknowledgement work drains before
        // its Completion continuation hands off to the next authored entry.
        machine.menu.flow =
            menu::Flow::Completion(crate::selection::CompletionRoute::OrdinaryAcknowledgement);
        machine.menu.operations.clear();
        assert!(matches!(
            machine.poll_menu(),
            menu::Poll::Operation(crate::selection::Operation::FadeOut)
        ));
        assert_eq!(
            machine.services.content.active_file.as_ref().unwrap().id.0,
            "file/second"
        );
        assert_eq!(
            (
                machine.core.progression.scene_signal,
                machine.core.progression.completion_actor,
                machine.core.progression.progress_bits
            ),
            (0, 0, 1)
        );
        assert_eq!(
            machine.services.content.raw_rooms.len(),
            machine.base_content.raw_rooms.len() + 1
        );
        assert!(
            machine.successor().is_none(),
            "new entry cannot complete from stale source controls"
        );
        machine.reset();
        assert!(std::sync::Arc::ptr_eq(
            &machine.services.content,
            &machine.base_content
        ));
    }
}

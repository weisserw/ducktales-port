//! Source title/land call continuations. Work is returned to the canonical
//! services executor and acknowledged only after its real owner returns.
use super::simulation::{Core, Services};
use crate::{runtime::Buttons, selection};
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flow {
    Title(selection::TitlePhase),
    Lands(selection::LandPhase),
    Completion(selection::CompletionRoute),
    LandInput,
    Confirm,
    Gameplay,
    Stalled,
}
pub enum Poll {
    Operation(selection::Operation),
    Gameplay,
    Stalled,
}
#[derive(Clone)]
pub struct Menu {
    pub flow: Flow,
    pub operations: VecDeque<selection::Operation>,
    pub saved_stage: u8,
}
impl Menu {
    pub fn title() -> Self {
        Self {
            flow: Flow::Title(selection::TitlePhase::Entry),
            operations: VecDeque::new(),
            saved_stage: 0,
        }
    }
    pub fn enter_lands(&mut self) {
        self.operations.clear();
        self.flow = Flow::Lands(selection::LandPhase::Entry);
    }
    pub fn confirm(&mut self) {
        self.operations = selection::confirmation_plan().into();
        self.flow = Flow::Confirm;
    }
    pub fn completed(&mut self) {
        self.operations
            .pop_front()
            .expect("actual pending menu operation");
    }
    pub fn poll(&mut self, core: &mut Core, services: &mut Services) -> Poll {
        loop {
            if let Some(operation) = self.operations.front() {
                return Poll::Operation(operation.clone());
            }
            match self.flow {
                Flow::Title(selection::TitlePhase::Input) => {
                    let result = selection::title_input(
                        &mut services.option,
                        &mut core.actors,
                        services.runtime.controllers.ports[0],
                        &services.content.selection,
                    );
                    if result.decision == selection::Decision::LandSelection {
                        self.enter_lands();
                        continue;
                    }
                    self.operations.extend(result.operations);
                    self.operations.extend(selection::title_animation(
                        &mut core.actors,
                        &mut services.runtime.random,
                        &mut services.video.composition,
                    ));
                }
                Flow::Title(phase) => {
                    let batch = selection::title_resume(
                        phase,
                        &mut selection::TitleContext {
                            actors: &mut core.actors,
                            random: &mut services.runtime.random,
                            option: &mut services.option,
                        },
                        &services.content.selection,
                    );
                    self.flow = Flow::Title(batch.next);
                    self.operations.extend(batch.operations);
                }
                Flow::Lands(selection::LandPhase::Completion) => {
                    let plan = selection::completion_plan(selection::CompletionInput {
                        completion_actor: core.progression.completion_actor,
                        scene_signal: core.progression.scene_signal,
                        progress_bits: core.progression.progress_bits,
                    });
                    self.flow = Flow::Completion(plan.route);
                    self.operations.extend(plan.operations);
                }
                Flow::Lands(phase) => {
                    let batch = selection::land_resume(phase);
                    self.flow = Flow::Lands(batch.next);
                    self.operations.extend(batch.operations);
                }
                Flow::Completion(selection::CompletionRoute::LateStage) => {
                    core.progression.late_route = core.progression.late_route.wrapping_add(1);
                    core.progression.stage.0 = 1;
                    self.confirm();
                }
                Flow::Completion(_) => {
                    core.progression.stage = services
                        .content
                        .raw_lands
                        .first_available(core.progression.progress_bits);
                    services.content.raw_lands.position(
                        core.progression.stage,
                        &mut core.actors,
                        true,
                    );
                    self.flow = Flow::LandInput;
                }
                Flow::LandInput => {
                    let pressed = services.runtime.controllers.ports[0].pressed.0;
                    if pressed & (Buttons::UP.0 | Buttons::DOWN.0) != 0 {
                        let next = services.content.raw_lands.navigate(
                            core.progression.stage,
                            core.progression.progress_bits,
                            pressed & Buttons::DOWN.0 != 0,
                            &services.content.selection,
                        );
                        let Some(next) = next else {
                            self.flow = Flow::Stalled;
                            continue;
                        };
                        core.progression.stage = next;
                        services
                            .content
                            .raw_lands
                            .position(next, &mut core.actors, false);
                        // D755 clears only these two selection actors' clocks.
                        for slot in [2, 4] {
                            core.actors[slot].animation_frame = 0;
                            core.actors[slot].animation_tick = 0;
                        }
                        self.operations
                            .push_back(selection::Operation::Sound(selection::MENU_SOUND));
                    } else if pressed & Buttons::A.0 != 0 {
                        self.confirm();
                        continue;
                    }
                    self.operations.extend([
                        selection::Operation::HideSprites,
                        selection::Operation::Compose,
                        selection::Operation::Yield,
                    ]);
                }
                Flow::Confirm => {
                    self.flow = Flow::Gameplay;
                }
                Flow::Gameplay => return Poll::Gameplay,
                Flow::Stalled => return Poll::Stalled,
            }
        }
    }
}

//! Native text call boundary. Internal owner requests release borrows, not time:
//! the Machine must service them in the same foreground turn until a real wait.
use super::{
    families,
    simulation::{self, Core, Services},
};
use crate::{
    actors, collision, dialogue, player,
    progression::{OwnerStep, PlayerReturn},
    runtime, world,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReturnFact {
    Outcome(dialogue::OutcomeReturn),
    RevealSound(crate::audio::Submission),
}
impl ReturnFact {
    pub fn player_return(self) -> PlayerReturn {
        PlayerReturn {
            first_actor_increment: match self {
                Self::Outcome(outcome) => outcome.first_actor_increment(),
                Self::RevealSound(submission) => submission.first_actor_increment(),
            },
        }
    }
}
pub enum Poll {
    Requested(dialogue::Work),
    Complete(ReturnFact),
}
#[derive(Default, Clone)]
pub struct DialogueCall {
    pending: Option<dialogue::Work>,
    returned: Option<dialogue::Work>,
    final_operation: Option<ReturnFact>,
}
impl DialogueCall {
    pub fn begin(&mut self, services: &mut Services) {
        assert!(self.pending.is_none() && self.returned.is_none());
        self.final_operation = None;
        services.dialogue.phase = dialogue::Phase::Entry;
        services.dialogue.acknowledgment = None;
        services.dialogue.pending_wait = None;
        services.dialogue.after_scroll = false;
    }
    pub fn completed(&mut self, work: dialogue::Work) {
        assert_eq!(self.pending.take(), Some(work));
        assert!(self.returned.replace(work).is_none());
    }
    pub fn poll(&mut self, core: &mut Core, services: &mut Services) -> Poll {
        if let Some(work) = self.pending {
            return Poll::Requested(work);
        }
        let content = std::sync::Arc::clone(&services.content);
        // Move the ONE panel and dialogue into this call scope. No interrupt,
        // render, debug observation or external owner can run until both have
        // been restored below. All nested leaf uploads borrow this same panel.
        let mut panel = std::mem::take(&mut services.video.display.primary);
        let mut running = std::mem::take(&mut services.dialogue);
        let mut input = services.runtime.controllers.ports[0];
        let result = dialogue::resume(
            &mut dialogue::Context {
                dialogue: &mut running,
                progression: &mut core.progression,
                player: &mut core.player,
                actors: &mut core.actors,
                status: &mut core.status,
                cache: &mut core.cache,
                camera: &core.camera,
                suppression: &mut core.suppression,
            },
            &mut input,
            &mut panel,
            &content.dialogue,
            &mut Boundary {
                services,
                pending: &mut self.pending,
                returned: &mut self.returned,
                final_operation: &mut self.final_operation,
                terrain_bank: core.terrain_bank,
            },
        )
        .expect("validated dialogue resource/cursor consumer");
        services.runtime.controllers.ports[0] = input;
        services.dialogue = running;
        services.video.display.primary = panel;
        match result {
            OwnerStep::Complete(()) => Poll::Complete(
                self.final_operation
                    .expect("every dialogue outcome reports its actual final operation"),
            ),
            OwnerStep::Suspended => Poll::Requested(
                self.pending
                    .expect("text suspension entered a real cooperative owner"),
            ),
        }
    }
}
struct Boundary<'a> {
    services: &'a mut Services,
    pending: &'a mut Option<dialogue::Work>,
    returned: &'a mut Option<dialogue::Work>,
    final_operation: &'a mut Option<ReturnFact>,
    terrain_bank: u8,
}
impl dialogue::Systems for Boundary<'_> {
    fn outcome_return(&mut self, outcome: dialogue::OutcomeReturn) {
        *self.final_operation = Some(ReturnFact::Outcome(outcome));
    }
    fn perform(
        &mut self,
        work: dialogue::Work,
        context: &mut dialogue::Context<'_>,
    ) -> OwnerStep<()> {
        if let Some(returned) = self.returned.take() {
            assert_eq!(returned, work);
            return OwnerStep::Complete(());
        }
        use dialogue::Work;
        match work {
            Work::BuildRaceTerrain | Work::StatusStreams | Work::Yield | Work::Wait(_) => {
                *self.pending = Some(work);
                return OwnerStep::Suspended;
            }
            Work::SelectGraphicsTwo => self
                .services
                .runtime
                .bank
                .select(runtime::PrgBank::GraphicsTwo),
            Work::SelectGraphicsOne => self
                .services
                .runtime
                .bank
                .select(runtime::PrgBank::GraphicsOne),
            Work::SelectPresentation => self
                .services
                .runtime
                .bank
                .select(runtime::PrgBank::PresentationFive),
            Work::SelectTextBank => self
                .services
                .runtime
                .bank
                .select(runtime::PrgBank::ActorsFour),
            Work::RestoreActorBank => self
                .services
                .runtime
                .bank
                .select(runtime::PrgBank::ActorsSix),
            Work::HideSprites => self
                .services
                .video
                .composition
                .hide(self.services.video.gameplay_display != 0),
            Work::Compose => simulation::compose_owners(
                &self.services.content,
                &mut self.services.video,
                context.actors,
                context.player,
                context.status,
                context.progression,
                context.camera,
                &context.cache.attributes,
            ),
            Work::AddSuppression => {
                let progress = &context.progression;
                let rules = self
                    .services
                    .content
                    .spawns
                    .suppression
                    .get(usize::from(progress.stage.0))
                    .map_or(&[][..], Vec::as_slice);
                actors::add_suppression(
                    context.suppression,
                    rules,
                    actors::SuppressionFlags {
                        stage_bits: progress.progress_bits,
                        source_a1: progress.escort_prerequisite,
                        source_a3: progress.reward_progress,
                        source_a6: progress.accepted_stages,
                        source_a8: progress.seen_messages[0],
                        source_ae: progress.stage_one_extension,
                    },
                );
            }
            Work::Sound(command) => {
                self.services.commands.submit(command);
            }
            Work::ClearPanel | Work::RestorePanel => {} // The caller already authored the borrowed panel.
            Work::ChangedBurst(_) => {
                unreachable!("running text lends its actual panel to the burst")
            }
        }
        OwnerStep::Complete(())
    }
    fn perform_with_panel(
        &mut self,
        work: dialogue::Work,
        context: &mut dialogue::Context<'_>,
        panel: &mut dialogue::Panel,
    ) -> OwnerStep<()> {
        if let dialogue::Work::ChangedBurst(selector) = work {
            let submission = families::burst_owners(
                &self.services.content,
                &mut self.services.commands,
                panel,
                &mut self.services.video.display.secondary,
                context.cache,
                context.actors,
                self.terrain_bank,
                selector,
            );
            self.services
                .runtime
                .bank
                .select(runtime::PrgBank::ActorsFour);
            *self.final_operation = Some(ReturnFact::RevealSound(submission));
            OwnerStep::Complete(())
        } else {
            self.perform(work, context)
        }
    }
    // The text resume/outcome call graph never invokes actor-family callbacks;
    // the live actor dispatcher owns their actual implementations in families.
    fn contact(&mut self, _: &mut actors::ActorPool, _: &mut player::Player, _: usize) -> bool {
        unreachable!("text call has no actor-family contact")
    }
    fn distance(&mut self, _: &actors::ActorPool, _: usize) -> u8 {
        unreachable!("text call has no actor-family distance")
    }
    fn motion(
        &mut self,
        _: &mut actors::ActorPool,
        _: &mut player::Player,
        _: &world::WorldCache,
        _: usize,
        _: collision::Movement,
        _: u8,
        _: u8,
    ) -> u8 {
        unreachable!("text call has no actor-family motion")
    }
    fn integrate(&mut self, _: &mut actors::ActorPool, _: usize, _: bool) {
        unreachable!("text call has no actor-family integration")
    }
    fn allocate(&mut self, _: &actors::ActorPool) -> Option<usize> {
        unreachable!("text call has no actor-family allocation")
    }
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
            Services::new(super::super::content::Content::load(&rom, &assets, &overrides).unwrap()),
        )
    }
    #[test]
    fn internal_status_request_restores_the_canonical_panel_before_owner_service() {
        let (mut core, mut services) = fixture();
        services.dialogue.phase = dialogue::Phase::StatusStreams;
        services.video.display.primary.encoded[63] = 0x65;
        services.video.display.primary.pending = 0xff;
        let mut call = DialogueCall::default();
        assert!(matches!(
            call.poll(&mut core, &mut services),
            Poll::Requested(dialogue::Work::StatusStreams)
        ));
        assert_eq!(services.video.display.primary.encoded.len(), 64);
        assert_eq!(services.video.display.primary.encoded[63], 0x65);
        assert_eq!(services.video.display.primary.pending, 0xff);
        assert_eq!(services.dialogue.phase, dialogue::Phase::StatusStreams);
    }
    #[test]
    fn reveal_outcome_returns_its_actual_queue_comparison_and_restores_panel_storage() {
        let (mut core, mut services) = fixture();
        services.dialogue.phase = dialogue::Phase::Outcome;
        services.dialogue.message = dialogue::OutcomeMessage::Reveal as u8;
        services.dialogue.actor_slot = 1;
        services.commands.slots[0] = 0x11;
        let mut call = DialogueCall::default();
        let Poll::Complete(ReturnFact::RevealSound(submission)) =
            call.poll(&mut core, &mut services)
        else {
            panic!("reveal must return the actual burst submission");
        };
        assert!(!submission.accepted);
        assert_eq!(
            submission.comparison,
            crate::audio::ProducerComparison::BelowEmpty
        );
        assert_eq!(
            submission.first_actor_increment(),
            crate::progression::FirstActorIncrement::Zero
        );
        assert_eq!(
            services.commands.producer, 0,
            "dropped command does not advance/replay"
        );
        assert_eq!(services.video.display.primary.encoded.len(), 64);
        assert_eq!(services.video.display.primary.pending, 0xff);
        assert_eq!(services.dialogue.phase, dialogue::Phase::Done);
    }
}

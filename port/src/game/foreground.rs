//! Borrow boundary between the canonical foreground policy and its cooperative
//! owners. A request suspends the same call until the actual owner returns.
//! Requests here are native continuations, never a second simulation/effect queue.
use super::simulation::{Core, Services};
use crate::{
    collision,
    progression::{self, OwnerStep, Work},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Request {
    Player,
    Owner(Work),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Returned {
    Player(progression::PlayerReturn),
    Owner(Work),
}
pub enum Poll {
    Requested(Request),
    Frame(progression::FrameExit),
}
#[derive(Default, Clone)]
pub struct Foreground {
    pending: Option<Request>,
    returned: Option<Returned>,
    player_entry: Option<progression::FirstActorIncrement>,
}
impl Foreground {
    pub fn player_entry(&self) -> progression::FirstActorIncrement {
        self.player_entry
            .expect("foreground entry established the player's source comparison")
    }
    /// E8C8 discarded the enclosing player return. Its sixty-tick owner already
    /// spawned, so DBE4 enters the post-spawn tail without inventing an RNG bit.
    pub fn vertical_completed(&mut self, core: &mut Core) {
        assert_eq!(self.pending.take(), Some(Request::Player));
        assert!(self.returned.is_none());
        core.progression.first_actor_increment = None;
        core.progression.continuation = progression::FrameContinuation::AfterSpawn;
    }
    pub fn completed(&mut self, returned: Returned) {
        let expected = match returned {
            Returned::Player(_) => Request::Player,
            Returned::Owner(work) => Request::Owner(work),
        };
        assert_eq!(
            self.pending.take(),
            Some(expected),
            "only the entered owner can complete this call"
        );
        assert!(self.returned.replace(returned).is_none());
    }
    pub fn poll(&mut self, core: &mut Core, services: &mut Services) -> Poll {
        if let Some(request) = self.pending {
            return Poll::Requested(request);
        }
        if core.progression.continuation == progression::FrameContinuation::Entry {
            // GameplayLoopEntry's CMP #Dialogue survives normal input gates.
            // A nested status-stream wait can replace this at its actual return.
            self.player_entry = Some(
                if core.player.action as u8 >= crate::player::Action::Dialogue as u8 {
                    progression::FirstActorIncrement::One
                } else {
                    progression::FirstActorIncrement::Zero
                },
            );
        }
        let content = std::sync::Arc::clone(&services.content);
        let input = services.runtime.controllers.ports[0];
        // These two small source controls are call-local writeback projections:
        // no interrupt/render/inspection runs until they are committed below.
        // Video owns them at every host boundary; other callbacks do not use them.
        let mut pending = u8::from(services.video.display.secondary_pending);
        let mut guard = services.video.oam_guard;
        let result = progression::foreground(
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
            input,
            &mut Boundary {
                services,
                pending: &mut self.pending,
                returned: &mut self.returned,
                player_entry: &mut self.player_entry,
            },
        );
        services.video.display.secondary_pending = pending != 0;
        services.video.oam_guard = guard;
        match self.pending {
            Some(request) => Poll::Requested(request),
            None => Poll::Frame(result),
        }
    }
}
struct Boundary<'a> {
    services: &'a mut Services,
    pending: &'a mut Option<Request>,
    returned: &'a mut Option<Returned>,
    player_entry: &'a mut Option<progression::FirstActorIncrement>,
}
impl progression::Systems for Boundary<'_> {
    fn player(&mut self, _: &mut progression::Frame<'_>) -> OwnerStep<progression::PlayerReturn> {
        match self.returned.take() {
            Some(Returned::Player(returned)) => return OwnerStep::Complete(returned),
            Some(Returned::Owner(_)) => unreachable!("owner completion cannot satisfy player call"),
            None => {}
        }
        *self.pending = Some(Request::Player);
        OwnerStep::Suspended
    }
    fn perform(&mut self, work: Work, frame: &mut progression::Frame<'_>) -> OwnerStep<()> {
        if let Some(returned) = self.returned.take() {
            assert_eq!(
                returned,
                Returned::Owner(work),
                "resume the same original foreground call"
            );
            if work == Work::StatusStreams {
                *self.player_entry = Some(super::player_call::sample_return(self.services));
            }
            return OwnerStep::Complete(());
        }
        match work {
            Work::Sound(command) => {
                self.services.commands.submit(command);
            }
            Work::HideSprites => self
                .services
                .video
                .composition
                .hide(self.services.video.gameplay_display != 0),
            Work::PresentationBank => self
                .services
                .runtime
                .bank
                .select(crate::runtime::PrgBank::PresentationFive),
            Work::ActorBank => self
                .services
                .runtime
                .bank
                .select(crate::runtime::PrgBank::ActorsSix),
            Work::ClearActors => collision::clear_transition_actors(
                frame.actors,
                frame.progression.stage.0,
                *frame.room_row,
            ),
            Work::AdvanceRandom => self.services.runtime.random.advance(),
            // Callees borrow Core directly after this Frame borrow ends. The
            // caller resumes only after completed() receives their real result.
            _ => {
                *self.pending = Some(Request::Owner(work));
                return OwnerStep::Suspended;
            }
        }
        OwnerStep::Complete(())
    }
}

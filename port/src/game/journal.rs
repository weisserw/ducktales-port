//! Transient owner-boundary snapshots for semantic changes. These are dropped
//! immediately after recording; they never drive rules or survive a host wait.
use super::{content::Content, observation, simulation::Core};
use crate::{actors, events::Events, player, progression, status, world};
use serde_json::json;
struct Parts<'a> {
    actors: &'a actors::ActorPool,
    player: &'a player::Player,
    progression: &'a progression::Progression,
    status: &'a status::Status,
    cache: &'a world::WorldCache,
    camera: &'a world::Camera,
    row: u8,
}
impl<'a> Parts<'a> {
    fn core(core: &'a Core) -> Self {
        Self {
            actors: &core.actors,
            player: &core.player,
            progression: &core.progression,
            status: &core.status,
            cache: &core.cache,
            camera: &core.camera,
            row: core.row,
        }
    }
    fn frame(frame: &'a progression::Frame<'_>) -> Self {
        Self {
            actors: frame.actors,
            player: frame.player,
            progression: frame.progression,
            status: frame.status,
            cache: frame.cache,
            camera: frame.camera,
            row: *frame.room_row,
        }
    }
}
pub(super) struct Snapshot {
    actors: actors::ActorPool,
    player: player::Player,
    progression: progression::Progression,
    status: status::Status,
    cache: world::WorldCache,
    camera: world::Camera,
    row: u8,
}
impl Snapshot {
    fn capture(parts: Parts<'_>) -> Self {
        Self {
            actors: *parts.actors,
            player: parts.player.clone(),
            progression: parts.progression.clone(),
            status: parts.status.clone(),
            cache: parts.cache.clone(),
            camera: *parts.camera,
            row: parts.row,
        }
    }
    pub fn core(core: &Core) -> Self {
        Self::capture(Parts::core(core))
    }
    pub fn frame(frame: &progression::Frame<'_>) -> Self {
        Self::capture(Parts::frame(frame))
    }
    pub fn finish_core(self, events: &Events, core: &Core, content: &Content, owner: &str) {
        self.finish(events, Parts::core(core), content, owner, true);
    }
    pub fn finish_frame(
        self,
        events: &Events,
        frame: &progression::Frame<'_>,
        content: &Content,
        owner: &str,
    ) {
        self.finish(events, Parts::frame(frame), content, owner, true);
    }
    pub fn finish_domain_core(self, events: &Events, core: &Core, content: &Content, owner: &str) {
        self.finish(events, Parts::core(core), content, owner, false);
    }
    fn finish(
        mut self,
        events: &Events,
        now: Parts<'_>,
        content: &Content,
        owner: &str,
        include_actors: bool,
    ) {
        if include_actors {
            actor_changes(events, &self.actors, now.actors, content, owner);
        }
        if self.player != *now.player {
            events.record("PlayerChanged", json!({"owner":owner,"before":observation::player_value(&self.player),"after":observation::player_value(now.player)}));
        }
        if self.status != *now.status {
            events.record("StatusChanged", json!({"owner":owner,"before":observation::status_value(&self.status),"after":observation::status_value(now.status)}));
        }
        // Native borrow/return bookkeeping is observed separately as OwnerBoundary;
        // it is not a second source progression transition.
        self.progression.continuation = now.progression.continuation;
        self.progression.awaiting_owner = now.progression.awaiting_owner;
        self.progression.first_actor_increment = now.progression.first_actor_increment;
        if self.progression != *now.progression || self.row != now.row {
            events.record("ProgressionChanged", json!({"owner":owner,"before":observation::progression_value(&self.progression,self.row),"after":observation::progression_value(now.progression,now.row)}));
        }
        if self.camera != *now.camera {
            events.record("CameraChanged", json!({"owner":owner,"before":{"x":self.camera.x,"room":self.camera.room,"previous_x":self.camera.previous_x},"after":{"x":now.camera.x,"room":now.camera.room,"previous_x":now.camera.previous_x}}));
        }
        if self.cache != *now.cache {
            let delta = |before: &[u8], after: &[u8]| {
                before
                    .iter()
                    .zip(after)
                    .enumerate()
                    .filter(|&(_, (&old, &new))| old != new)
                    .map(|(index, (&old, &new))| json!({"index":index,"before":old,"after":new}))
                    .collect::<Vec<_>>()
            };
            events.record("TerrainChanged", json!({"owner":owner,"collision":delta(&self.cache.collision,&now.cache.collision),"attributes":delta(&self.cache.attributes,&now.cache.attributes),"changed_before":self.cache.changed_this_frame,"changed_after":now.cache.changed_this_frame}));
        }
    }
}
pub(super) fn actor_changes(
    events: &Events,
    before: &actors::ActorPool,
    after: &actors::ActorPool,
    content: &Content,
    owner: &str,
) {
    for (slot, (old, new)) in before.iter().zip(after).enumerate() {
        if old == new {
            continue;
        }
        let was_active = old.state_code() & actors::ACTIVE != 0;
        let active = new.state_code() & actors::ACTIVE != 0;
        let kind = if !was_active && active {
            "ActorSpawned"
        } else if was_active && !active {
            "ActorRemoved"
        } else if old.spawn_identity != new.spawn_identity || old.handler != new.handler {
            "ActorReplaced"
        } else {
            "ActorChanged"
        };
        let label = |selector: u8| {
            content
                .spawns
                .handlers
                .get(usize::from(selector))
                .map(|entry| entry.label.as_str())
        };
        events.record(kind, json!({"owner":owner,"slot":slot,"before":observation::actor(slot,old,label(old.handler)),"after":observation::actor(slot,new,label(new.handler))}));
    }
}

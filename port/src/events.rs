//! Bounded semantic observations, separate from source simulation/pending work.
//! Cloned handles refer to ONE journal so callbacks can record actual author
//! order without retaining another command queue or mutable game-state mirror.
use serde_json::{Value, json};
use std::{cell::RefCell, collections::VecDeque, rc::Rc};
#[derive(Clone, Debug)]
struct Journal {
    tick: u64,
    capacity: usize,
    next_sequence: u64,
    dropped: u64,
    entries: VecDeque<Value>,
}
#[derive(Clone, Debug)]
pub(crate) struct Events(Rc<RefCell<Journal>>);
impl Events {
    pub fn new(capacity: usize) -> Self {
        Self(Rc::new(RefCell::new(Journal {
            tick: 0,
            capacity,
            next_sequence: 0,
            dropped: 0,
            entries: VecDeque::new(),
        })))
    }
    /// Fork the journal; normal Clone intentionally shares callback handles.
    pub fn fork(&self) -> Self {
        Self(Rc::new(RefCell::new(self.0.borrow().clone())))
    }
    pub fn set_tick(&self, tick: u64) {
        self.0.borrow_mut().tick = tick;
    }
    pub fn record(&self, kind: &str, payload: Value) {
        let mut journal = self.0.borrow_mut();
        let event = json!({"sequence":journal.next_sequence,"tick":journal.tick,"kind":kind,"payload":payload});
        journal.next_sequence += 1;
        if journal.capacity == 0 {
            journal.dropped += 1;
            return;
        }
        if journal.entries.len() == journal.capacity {
            journal.entries.pop_front();
            journal.dropped += 1;
        }
        journal.entries.push_back(event);
    }
    pub fn observe(&self) -> Value {
        let journal = self.0.borrow();
        json!({"implemented":true,"capacity":journal.capacity,"next_sequence":journal.next_sequence,"dropped":journal.dropped,"entries":journal.entries})
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snapshot_fork_is_independent_while_callbacks_share_the_fork() {
        let original = Events::new(8);
        original.record("before", json!({}));
        let fork = original.fork();
        let callback = fork.clone();
        callback.record("branch", json!({}));
        assert_eq!(original.observe()["next_sequence"], 1);
        assert_eq!(fork.observe()["next_sequence"], 2);
        original.record("other", json!({}));
        assert_ne!(original.observe(), fork.observe());
    }
    #[test]
    fn shared_handles_preserve_author_order_bound_retention_and_never_drain() {
        let events = Events::new(2);
        let callback = events.clone();
        events.set_tick(7);
        events.record("first", json!({"value":1}));
        callback.record("second", json!({"value":2}));
        events.set_tick(8);
        callback.record("third", json!({"value":3}));
        let snapshot = events.observe();
        assert_eq!(snapshot, callback.observe());
        assert_eq!(snapshot["next_sequence"], 3);
        assert_eq!(snapshot["dropped"], 1);
        assert_eq!(snapshot["entries"][0]["sequence"], 1);
        assert_eq!(snapshot["entries"][1]["tick"], 8);
    }
}

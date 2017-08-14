//! Provide a queue-based callback for virtualizing OS abstractions.

use common::list::{List, ListLink, ListNode};
use core::cell::Cell;

pub trait Dequeued<'a> {
    fn dequeued(&'a self);
    fn id(&'a self) -> u32;
}

pub struct QueuedCall<'a> {
    next: ListLink<'a, QueuedCall<'a>>,
    callback: Cell<Option<&'a Dequeued<'a>>>,
    active: Cell<bool>,
    queue: &'a CallQueue<'a>,
}

impl<'a> ListNode<'a, QueuedCall<'a>> for QueuedCall<'a> {
    fn next(&self) -> &'a ListLink<QueuedCall<'a>> {
        &self.next
    }
}

impl<'a> QueuedCall<'a> {
    pub fn new(queue: &'a CallQueue<'a>) -> QueuedCall<'a> {
        QueuedCall {
            next: ListLink::empty(),
            callback: Cell::new(None),
            active: Cell::new(false),
            queue: queue,
        }
    }

    pub fn set_callback(&'a self, callback: &'a Dequeued<'a>) {
        self.callback.set(Some(callback));
        self.queue.queued_calls.push_head(self);
    }

    /// Returns true if it was inserted, false if it was already
    /// in the queue.
    pub fn insert(&'a self) -> bool {
        let rval = !self.active.get();
        self.active.set(true);
        rval
    }

    /// Returns true if removed from the queue, false if it was not
    /// in the queue.
    pub fn remove(&'a self) -> bool {
        let rval = self.active.get();
        self.active.set(false);
        rval
    }

    pub fn is_inserted(&'a self) -> bool {
        self.active.get()
    }
}

pub struct CallQueue<'a> {
    queued_calls: List<'a, QueuedCall<'a>>,
    next: Cell<Option<&'a QueuedCall<'a>>>,
}

impl<'a> CallQueue<'a> {
    pub const fn new() -> CallQueue<'a> {
        CallQueue {
            queued_calls: List::new(),
            next: Cell::new(None),
        }
    }

    // This triggers the next queued element by performing
    // a linear scan of the list, starting at the front. It
    // keeps a reference to the 'next' element after the last one
    // triggered (or None, if it was the tail of the queue).
    // It keeps track of the 'first' active element before
    // the 'next' element. It then walks forward from next,
    // triggering the earlist active element. If it reaches the
    // end of the queue, it triggers 'first' if there is one,
    // or returns false if there is no 'first' (no element in the
    // queue was marked active).
    pub fn dequeue_and_trigger(&self) -> bool {
        // If the last scan reached the end of the queue,
        // set the first element to look at to be the first element
        // of the queue.
        if self.next.get().is_none() {
            self.next.set(self.queued_calls.head());
        }
        let mut next = false;
        let mut passed = false;
        let mut first = None;
        for call in self.queued_calls.iter() {
            // Haven't passed next
            if !passed {
                // Reached next
                if call as *const QueuedCall == self.next.get().unwrap() as *const QueuedCall {
                    passed = true;
                    self.next.set(None);
                    if call.active.get() {
                        next = true;
                        call.active.set(false);
                        call.callback.get().map(|c| c.dequeued());
                    }
                } else if call.active.get() && first.is_none() {
                    // We're before next, so set first
                    first = Some(call);
                }
            } else if next {
                // Previous item triggered, so set next
                self.next.set(Some(call));
                return true;
            } else if call.active.get() {
                call.active.set(false);
                call.callback.get().map(|c| c.dequeued());
                next = true;
                self.next.set(None);
            }
        }
        if first.is_some() {
            let val = first.unwrap();
            val.active.set(false);
            val.callback.get().map(|c| c.dequeued());
            return true;
        }
        next
    }
}

//! This example capsule illustrates how to create a `List`
//! of trait objects
//!
//! A potential usage of this example might look like:
//!
//! ```
//! let manager = static_init!(traitobj_list::Manager<'static>,
//!                            traitobj_list::Manager::new());
//! let jazz = static_init!(traitobj_list::Jazz<'static>,
//!                         traitobj_list::Jazz::new());
//! let cheese = static_init!(traitobj_list::Cheese<'static>,
//!                           traitobj_list::Cheese::new());
//!
//! manager.manage(jazz);
//! manager.manage(cheese);
//! manager.report();
//! ```

use kernel::common::list::*;

pub trait Funky<'a> {
    fn name(&self) -> &'static str;
    fn next_funky_thing(&'a self) -> &'a ListLink<'a, Funky<'a>>;
}

impl<'a> ListNode<'a, Funky<'a>> for Funky<'a> {
    fn next(&'a self) -> &'a ListLink<'a, Funky<'a>> {
        &self.next_funky_thing()
    }
}

// A manager holds a list of funky things
pub struct Manager<'a> {
    funky_things: List<'a, Funky<'a>>,
}

impl<'a> Manager<'a> {
    pub fn new() -> Manager<'a> {
        Manager {
            funky_things: List::new(),
        }
    }

    pub fn manage(&mut self, thing: &'a (Funky<'a>)) {
        self.funky_things.push_head(thing);
    }

    pub fn report(&self) {
        for t in self.funky_things.iter() {
            debug!("Funky thing: {}", t.name());
        }
    }
}

// Jazz is a funky thing
pub struct Jazz<'a> {
    next: ListLink<'a, Funky<'a>>,
}

impl<'a> Jazz<'a> {
    pub fn new() -> Self {
        Jazz {
            next: ListLink::empty(),
        }
    }
}

impl<'a> Funky<'a> for Jazz<'a> {
    fn name(&self) -> &'static str {
        "Jazz"
    }

    fn next_funky_thing(&'a self) -> &'a ListLink<'a, Funky<'a>> {
        &self.next
    }
}

// Cheese is a funky thing
pub struct Cheese<'a> {
    next: ListLink<'a, Funky<'a>>,
}

impl<'a> Cheese<'a> {
    pub fn new() -> Self {
        Cheese {
            next: ListLink::empty(),
        }
    }
}

impl<'a> Funky<'a> for Cheese<'a> {
    fn name(&self) -> &'static str {
        "Cheese"
    }
    fn next_funky_thing(&'a self) -> &'a ListLink<'a, Funky<'a>> {
        &self.next
    }
}

use core::cell::Cell;
use core::cmp::PartialEq;

pub struct ListLink<'a, T:'a + PartialEq>(Cell<Option<&'a T >>);

impl<'a, T: PartialEq > ListLink<'a, T> {
    pub const fn empty() -> ListLink<'a, T> {
        ListLink(Cell::new(None))
    }
}

pub trait ListNode<'a, T> where T: PartialEq {
    fn next(&'a self) -> &'a ListLink<'a, T>;
}

pub struct List<'a, T: 'a + ListNode<'a, T> + PartialEq > {
    head: ListLink<'a, T>,
    tail: ListLink<'a, T>
}

pub struct ListIterator<'a, T: 'a + ListNode<'a, T>> where T: PartialEq {
    cur: Option<&'a T>
}

impl<'a, T: ListNode<'a, T>> Iterator for ListIterator<'a, T> where T: PartialEq {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        match self.cur {
            Some(res) => {
                self.cur = res.next().0.get();
                Some(res)
            },
            None => None
        }
    }
}

impl<'a, T: ListNode<'a, T> + PartialEq> List<'a, T> where T: PartialEq {
    pub const fn new() -> List<'a, T> {
        List {
            head: ListLink(Cell::new(None)),
            tail: ListLink(Cell::new(None))
        }
    }

    pub fn head(&self) -> Option<&'a T> {
        self.head.0.get()
    }
    
    pub fn pop_tail(&mut self) -> Option<&'a T> {
        if self.tail.0.get().is_none() {
            None
        } else if self.tail.0 == self.head.0 {
            let node = self.head.0.get();
            self.head.0.set(None);
            self.tail.0.set(None);
            node
        } else {
            let mut iterator = self.iter();
            let mut node: Option<&'a T> = None;
            let mut curr = iterator.next();
            while !curr.is_none() {
                if curr.unwrap().next().0.get().unwrap() == self.tail.0.get().unwrap() {
                    node = self.tail.0.get();
                    self.tail = ListLink(Cell::new(curr));
                    curr.unwrap().next().0.set(None);
                    break;
                }

                curr = iterator.next();
            }
            node
        }
    }

    pub fn push_head(&self, node: &'a T) {
        node.next().0.set(self.head.0.get());
        if self.head.0.get().is_none() {
            self.tail.0.set(Some(node));
        }
        self.head.0.set(Some(node));
    }

    pub fn iter(&self) -> ListIterator<'a, T> {
        ListIterator {
            cur: self.head.0.get()
        }
    }
}


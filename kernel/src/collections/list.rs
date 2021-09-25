//! Interfaces to and implementations of linked-list data structures.
//!
//! This module contains traits useful for manipulating linked-list
//! data structures:
//!
//! - [`ListNode`], representing a single node in a singly-linked list
//! - [`SinglyLinkedList`], containing a basic interface to manipulate
//!   singly-linked list data structures
//!
//! Furthermore, it contains some implementations of linked-list data
//! structures:
//!
//! - [`simple_linked_list::SimpleLinkedList`] is a singly-linked list
//!   implementation which defines its own
//!   [`SimpleLinkedListNode`](simple_linked_list::SimpleLinkedListNode) node
//!   type. By defining its own node type it has full control over the list's
//!   structure and can thereby enforce that the list contains no loops and the
//!   list nodes cannot dynamically change the inherent list structure. It also
//!   does not require implementations to provide their own list node type.
//!
//! - [`generic_linked_list::GenericLinkedList`] is a singly-list implementation
//!   generic over the underlying list node type. It requires list nodes to
//!   implement [`ModifyableListNode`]. Implementations are free to define the
//!   inherent list behavior through custom implementations of the
//!   [`ListNode::next`] operator. For example, this type allows to build lazily
//!   evaluated lists or lists which change on the fly.

/// Node of a singly-linked list.
///
/// The implementation of the [`next`](ListNode::next) method defines the
/// inherent list structure. Thus implementors of this interface should be
/// careful to avoid building loops in the list, if this is not desired
/// behavior.
pub trait ListNode<'a> {
    type Content: ?Sized;

    /// Get a reference to the list node's content
    ///
    /// This method is provided for when the ListNode is a container type
    /// distinct from the content type `C`. Implementations generic over the
    /// `ListNode` container are thus expected to always call `content` to get a
    /// reference of type `C`.
    fn content<'c>(&'c self) -> &'c Self::Content;

    /// Get a reference to the next list node
    ///
    /// Implementors must be careful to avoid building loops in the list, if
    /// this is not desired.
    ///
    /// The call to [`next`](ListNode::next) may be used to allocate the next
    /// list element dynamically.
    fn next(&'a self) -> Option<&'a Self>;
}

/// Modifyable node of a singly-linked list.
///
/// This trait is a subtrait of [`ListNode`]. It can be used to allow external
/// modification of the list's structure.
pub trait ModifyableListNode<'a>: ListNode<'a> {
    /// Set the next list element.
    ///
    /// Implementations MUST ensure the that next list element (returned on the
    /// call to [`next`](ListNode::next)) is the passed in reference, or `None`
    /// if the supplied `next` is `None`.
    fn set_next(&'a self, next: Option<&'a Self>);
}

/// Iterator over a list consisting of [`ListNode`]s.
///
/// Iterates over a chain of list nodes, returning their contents. Per
/// iteration, this calls [`next`](ListNode::next) and
/// [`content`](ListNode::content) for each visited list node.
pub struct ListIterator<'a, L: ListNode<'a>> {
    cur: Option<&'a L>,
}

impl<'a, L: ListNode<'a>> Iterator for ListIterator<'a, L> {
    type Item = &'a L::Content;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.cur {
            self.cur = node.next();
            Some(node.content())
        } else {
            None
        }
    }
}

/// Interface to a singly-linked list
pub trait SinglyLinkedList<'a, L: ListNode<'a>> {
    /// Get a reference to the first list element
    ///
    /// If the list is not empty, return a reference to the first list
    /// node. Otherwise returns `None`.
    fn head(&self) -> Option<&'a L>;

    /// Construct a [`ListIterator`], starting with the list head
    fn iter(&self) -> ListIterator<'a, L>;

    /// Prepend a [`ListNode`] to the list
    ///
    /// Prepends the passed [`ListNode`] in front of the list's head.
    ///
    /// Implementations are free to refuse this call, for instance when it would
    /// create a loop in the list. If the call is refused this method returns
    /// `false`, otherwise `true`.
    ///
    /// This method may modify the passed [`ListNode`]'s `next` value, either
    /// through an internal mechanism or through
    /// [`ModifyableListNode::set_next`]. It will not modify the [`ListNode`]'s
    /// `next` value if the call is refused, indicated by a return value of
    /// `false.
    ///
    /// This method may walk the entire list by calling [`ListNode::next`] on
    /// each element, to perform internal consistency checks.
    fn push_head(&self, node: &'a L) -> bool;

    /// Append a [`ListNode`] to the list.
    ///
    /// Appends the passed [`ListNode`] after the last list element. To find the
    /// last list element, this method will walk the entire list by calling
    /// [`ListNode::next`] on each element.
    ///
    /// Implementations are free to refuse this call, for instance when it would
    /// create a loop in the list. If the call is refused this method returns
    /// `false`, otherwise `true`.
    ///
    /// This method may modify the passed [`ListNode`]'s `next` value, either
    /// through an internal mechanism or through
    /// [`ModifyableListNode::set_next`]. It will not modify the [`ListNode`]'s
    /// `next` value if the call is refused, indicated by a return value of
    /// `false.
    fn push_tail(&self, node: &'a L) -> bool;

    /// Remove and return the first [`ListNode`].
    ///
    /// Removes the first [`ListNode`] in the list. Implementations should not
    /// walk the list for this operation.
    ///
    /// Returns the removed [`ListNode`] if the list was not empty, otherwise
    /// `None`.
    fn pop_head(&self) -> Option<&'a L>;
}

pub mod generic_linked_list {
    //! Module containing a singly-list implementation generic over the
    //! underlying list node type. It requires list nodes to implement
    //! [`ModifyableListNode`]. Implementations are free to define the inherent
    //! list behavior through custom implementations of the
    //! [`ListNode::next`](super::ListNode::next) operator. For example, this
    //! type allows to build lazily evaluated lists or lists which change on the
    //! fly.

    use core::cell::Cell;

    use super::{ListIterator, ModifyableListNode, SinglyLinkedList};

    /// Singly-linked list generic over the underlying [`ModifyableListNode`] type.
    ///
    /// This singly-linked list implementation can be used to build
    /// highly-flexible linked-list like data structures. By being generic over
    /// the underlying [`ListNode`](super::ListNode) type, implementations can,
    /// for example, determine the list's structure dynamically at runtime or
    /// lazily allocate and evaluate the list's contents.
    ///
    /// This list type requires it's nodes to implement
    /// [`ModifyableListNode`]. This is required such that the
    /// [`GenericLinkedList`] can manipulate the list structure through the
    /// generic list nodes.
    ///
    /// This list type does not implement any internal consistency checks. Users
    /// must be careful to avoid building loops if this is not desired. A call
    /// to [`push_tail`](GenericLinkedList) will iterate over the entire
    /// list. In case of a looping list structure, this may thus recurse
    /// infinitely.
    pub struct GenericLinkedList<'a, L: ModifyableListNode<'a>> {
        head: Cell<Option<&'a L>>,
    }

    impl<'a, L: ModifyableListNode<'a>> GenericLinkedList<'a, L> {
        pub const fn new() -> GenericLinkedList<'a, L> {
            GenericLinkedList {
                head: Cell::new(None),
            }
        }
    }

    impl<'a, L: ModifyableListNode<'a>> SinglyLinkedList<'a, L> for GenericLinkedList<'a, L> {
        fn head(&self) -> Option<&'a L> {
            self.head.get()
        }

        fn iter(&self) -> ListIterator<'a, L> {
            ListIterator {
                cur: self.head.get(),
            }
        }

        fn push_head(&self, node: &'a L) -> bool {
            node.set_next(self.head.get());
            self.head.set(Some(node));
            true
        }

        fn push_tail(&self, node: &'a L) -> bool {
            if let Some(current_head) = self.head.get() {
                let mut current_node = current_head;

                // Iterate through all list nodes in the chain. Cheaper than a
                // recursive implementation. This emulates a do-while loop,
                // which isn't directly supported in Rust.
                while {
                    if let Some(next_node) = current_node.next() {
                        current_node = next_node;
                        true
                    } else {
                        false
                    }
                } {}

                current_node.set_next(Some(node));
            } else {
                self.head.set(Some(node));
            }

            // This method will always succeed, there are no intern
            true
        }

        fn pop_head(&self) -> Option<&'a L> {
            if let Some(current_head) = self.head.get() {
                self.head.set(current_head.next());
                Some(current_head)
            } else {
                None
            }
        }
    }
}

pub mod simple_linked_list {
    //! Module containing a simple singly-list implementation . It defines and
    //! requires usage of its own [`SimpleLinkedListNode`], which help to
    //! maintain internal consistency within the list. Thus, this implementation
    //! does not support loops within the list, and the
    //! [`ListNode::content`](super::ListNode::content) and
    //! [`ListNode::next`](super::ListNode::content) methods cannot be
    //! overwritten. These constraints make the behavior of this list
    //! implementation very predictable.

    use core::cell::Cell;

    use super::{ListIterator, ListNode, SinglyLinkedList};

    /// Outgoing link of a [`SimpleLinkedListNode`]
    ///
    /// This enum can represent that the list node is not currently part of any
    /// list (`None`), it is the end of list of which it is a member (`End`), or
    /// it is followed by a successor element `Some(ref)`.
    enum SimpleLinkedListNodeLink<'a, C> {
        None,
        End,
        Next(&'a SimpleLinkedListNode<'a, C>),
    }

    // #[derive(Copy, Clone)] does not work here, because that macro cannot
    // infer that C: Clone + Copy is not a requirement for
    // SimpleLinkedListNodeLink<'_, C>: Clone + Copy.
    impl<'a, C> Clone for SimpleLinkedListNodeLink<'a, C> {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl<'a, C> Copy for SimpleLinkedListNodeLink<'a, C> {}

    impl<'a, C> core::cmp::PartialEq for SimpleLinkedListNodeLink<'a, C> {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (SimpleLinkedListNodeLink::None, SimpleLinkedListNodeLink::None) => true,
                (SimpleLinkedListNodeLink::End, SimpleLinkedListNodeLink::End) => true,
                (SimpleLinkedListNodeLink::Next(ref_a), SimpleLinkedListNodeLink::Next(ref_b)) => {
                    *ref_a as *const SimpleLinkedListNode<'a, C>
                        == *ref_b as *const SimpleLinkedListNode<'a, C>
                }
                _ => false,
            }
        }
    }
    impl<'a, C> core::cmp::Eq for SimpleLinkedListNodeLink<'a, C> {}

    /// Node of a [`SimpleLinkedList`]
    ///
    /// A node which can be part of a [`SimpleLinkedList`]. It holds an
    /// arbitrary, sized generic type `C`. A reference to this value is provided
    /// by calling [`ListNode::content`].
    ///
    /// The inner `link` field can only be modified through operations on the
    /// [`SimpleLinkedList`].
    pub struct SimpleLinkedListNode<'a, C> {
        content: C,
        link: Cell<SimpleLinkedListNodeLink<'a, C>>,
    }

    impl<'a, C> SimpleLinkedListNode<'a, C> {
        pub fn new(content: C) -> SimpleLinkedListNode<'a, C> {
            SimpleLinkedListNode {
                content,
                link: Cell::new(SimpleLinkedListNodeLink::None),
            }
        }
    }

    impl<'a, C> ListNode<'a> for SimpleLinkedListNode<'a, C> {
        type Content = C;

        fn content<'c>(&'c self) -> &'c C {
            &self.content
        }

        fn next(&'a self) -> Option<&'a Self> {
            match self.link.get() {
                SimpleLinkedListNodeLink::Next(next_node) => Some(next_node),
                _ => None,
            }
        }
    }

    /// Simple singly-linked list.
    ///
    /// This type implements a conceptually simple linked-list data
    /// structure. This is achieved by using a custom type for the list nodes,
    /// which helps to maintain internal consistency of the list. Thus this
    /// implementation does not allow loops within the list, and the
    /// [`ListNode::content`](super::ListNode::content) and
    /// [`ListNode::next`](super::ListNode::content) methods cannot be
    /// overwritten.
    ///
    /// The internal consistency checks do require walking the list for every
    /// `push` operation. The
    /// [`GenericLinkedList`](super::generic_linked_list::GenericLinkedList) can
    /// be used as an efficient and flexible alternative.
    pub struct SimpleLinkedList<'a, C> {
        head: Cell<Option<&'a SimpleLinkedListNode<'a, C>>>,
    }

    impl<'a, C> SimpleLinkedList<'a, C> {
        pub const fn new() -> SimpleLinkedList<'a, C> {
            SimpleLinkedList {
                head: Cell::new(None),
            }
        }
    }

    impl<'a, C> SinglyLinkedList<'a, SimpleLinkedListNode<'a, C>> for SimpleLinkedList<'a, C> {
        fn head(&self) -> Option<&'a SimpleLinkedListNode<'a, C>> {
            self.head.get()
        }

        fn iter(&self) -> ListIterator<'a, SimpleLinkedListNode<'a, C>> {
            ListIterator {
                cur: self.head.get(),
            }
        }

        fn push_head(&self, node: &'a SimpleLinkedListNode<'a, C>) -> bool {
            // First, check whether this node is already part of some list
            if node.link.get() != SimpleLinkedListNodeLink::None {
                return false;
            }

            if let Some(current_head) = self.head.get() {
                // Have a head-node already, prepend it and create a link to the
                // previous head.
                node.link.set(SimpleLinkedListNodeLink::Next(current_head));
                self.head.set(Some(node));
                true
            } else {
                // No head-node, mark this list node as the list end.
                node.link.set(SimpleLinkedListNodeLink::End);
                self.head.set(Some(node));
                true
            }
        }

        fn push_tail(&self, node: &'a SimpleLinkedListNode<'a, C>) -> bool {
            // First, check whether this node is already part of some list
            if node.link.get() != SimpleLinkedListNodeLink::None {
                return false;
            }

            if let Some(mut iter_node) = self.head.get() {
                // We have some head-element, walk the list to find the tail
                // element:
                while let SimpleLinkedListNodeLink::Next(next_node) = iter_node.link.get() {
                    iter_node = next_node;
                }

                node.link.set(SimpleLinkedListNodeLink::End);
                iter_node.link.set(SimpleLinkedListNodeLink::Next(node));
                true
            } else {
                // No head-node, mark this list node as the list end.
                node.link.set(SimpleLinkedListNodeLink::End);
                self.head.set(Some(node));
                true
            }
        }

        fn pop_head(&self) -> Option<&'a SimpleLinkedListNode<'a, C>> {
            if let Some(current_head) = self.head.get() {
                if let SimpleLinkedListNodeLink::Next(next_head) = current_head.link.get() {
                    self.head.set(Some(next_head));
                } else {
                    self.head.set(None);
                }

                current_head.link.set(SimpleLinkedListNodeLink::None);
                Some(current_head)
            } else {
                None
            }
        }
    }
}

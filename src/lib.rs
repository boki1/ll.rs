mod list {
    use std::cell::{Ref, RefCell, RefMut};
    use std::rc::Rc;

    pub struct List<T> {
        head: Link<T>,
        tail: Link<T>,
        size: usize,
    }

    type Link<T> = Option<Rc<RefCell<Node<T>>>>;

    struct Node<T> {
        element: T,
        next: Link<T>,
        prev: Link<T>,
    }

    impl<T> Node<T> {
        fn new(element: T) -> Rc<RefCell<Self>> {
            Rc::new(RefCell::new(Node {
                element,
                prev: None,
                next: None,
            }))
        }
    }

    impl<T> List<T> {
        pub fn new() -> Self {
            List {
                head: None,
                tail: None,
                size: 0,
            }
        }

        pub fn push_front(&mut self, element: T) {
            let new_head = Node::new(element);
            match self.head.take() {
                Some(old_head) => {
                    old_head.borrow_mut().prev = Some(new_head.clone());
                    new_head.borrow_mut().next = Some(old_head);
                    self.head = Some(new_head);
                }
                None => {
                    self.tail = Some(new_head.clone());
                    self.head = Some(new_head);
                }
            }

            self.size += 1;
        }

        pub fn push_back(&mut self, element: T) {
            let new_tail = Node::new(element);
            match self.tail.take() {
                Some(old_tail) => {
                    old_tail.borrow_mut().next = Some(new_tail.clone());
                    new_tail.borrow_mut().prev = Some(old_tail);
                    self.tail = Some(new_tail);
                }
                None => {
                    self.tail = Some(new_tail.clone());
                    self.head = Some(new_tail);
                }
            }

            self.size += 1;
        }

        pub fn pop_front(&mut self) -> Option<T> {
            self.head.take().map(|old_head| {
                match old_head.borrow_mut().next.take() {
                    Some(next_head) => {
                        next_head.borrow_mut().prev.take();
                        self.head = Some(next_head);
                    }
                    None => {
                        self.tail.take();
                    }
                }
                // Cannot do that
                // old_head.into_inner().element
                // because it's behind an Rc<> and we may only get shared refs.

                self.size -= 1;
                Rc::try_unwrap(old_head).ok().unwrap().into_inner().element
            })
        }

        pub fn pop_back(&mut self) -> Option<T> {
            self.tail.take().map(|old_tail| {
                match old_tail.borrow_mut().prev.take() {
                    Some(new_tail) => {
                        new_tail.borrow_mut().next.take();
                        self.tail = Some(new_tail);
                    }
                    None => {
                        self.head.take();
                    }
                }
                // Cannot do that
                // Some(old_tail.into_inner().element)
                // because it's behind an Rc<> and we may only get shared refs.
                self.size -= 1;
                Rc::try_unwrap(old_tail).ok().unwrap().into_inner().element
            })
        }

        pub fn size(&self) -> usize {
            self.size
        }

        pub fn peek_back(&self) -> Option<Ref<T>> {
            self.tail
                .as_ref()
                .map(|node| Ref::map(node.borrow(), |node| &node.element))
        }

        // BAD: Exposes the implementation detail of using Ref<T>
        pub fn peek_front(&self) -> Option<Ref<T>> {
            self.head
                .as_ref()
                .map(|node| Ref::map(node.borrow(), |node| &node.element))
        }

        pub fn peek_back_mut(&mut self) -> Option<RefMut<T>> {
            self.tail
                .as_ref()
                .map(|node| RefMut::map(node.borrow_mut(), |node| &mut node.element))
        }

        // BAD: Exposes the implementation detail of using Ref<T>
        pub fn peek_front_mut(&mut self) -> Option<RefMut<T>> {
            self.head
                .as_ref()
                .map(|node| RefMut::map(node.borrow_mut(), |node| &mut node.element))
        }
    }

    pub struct IntoIter<T>(List<T>);

    impl<T> List<T> {
        pub fn into_iter(self) -> IntoIter<T> {
            IntoIter(self)
        }
    }

    impl<T> Iterator for IntoIter<T> {
        type Item = T;

        fn next(&mut self) -> Option<Self::Item> {
            self.0.pop_front()
        }
    }

    impl<T> DoubleEndedIterator for IntoIter<T> {
        fn next_back(&mut self) -> Option<Self::Item> {
            self.0.pop_back()
        }
    }

    pub struct Iter<'a, T>(Option<Ref<'a, Node<T>>>);

    impl<T> List<T> {
        pub fn iter(&self) -> Iter<T> {
            Iter(self.head.as_ref().map(|node| node.borrow()))
        }
    }

    /*
    impl<'a, T> Iterator for Iter<'a, T> {
        type Item = Ref<'a, T>;

        fn next(&mut self) -> Option<Self::Item> {
            self.0.take().map(|node_ref| {
                let (next, element) = Ref::map_split(node_ref, |node| (&node.next, &node.element));

                self.0 = next.as_ref().map(|head| head.borrow());
                element
            })
        }
    }
    */
}

#[cfg(test)]
mod test {
    use crate::list::List;

    #[test]
    fn basic() {
        let mut list: List<u32> = List::new();
        // Check empty list behaves right
        assert_eq!(list.pop_front(), None);

        // Populate list
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);
        assert_eq!(list.size(), 3);

        // Check normal removal
        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.pop_front(), Some(2));
        assert_eq!(list.size(), 1);

        // Push some more just to make sure nothing's corrupted
        list.push_front(4);
        list.push_front(5);
        assert_eq!(list.size(), 3);

        // Check normal removal
        assert_eq!(list.pop_front(), Some(5));
        assert_eq!(list.pop_front(), Some(4));
        assert_eq!(list.size(), 1);

        // Check exhaustion
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.size(), 0);

        assert_eq!(list.size(), 0);
        list.push_front(3);
        assert_eq!(list.size(), 1);
    }

    #[test]
    fn peeks() {
        let mut list: List<u32> = List::new();
        assert!(list.peek_front().is_none());
        list.push_front(1);
        assert_eq!(&*list.peek_front().unwrap(), &1);
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next_back(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next(), None);
    }
}

mod list {
    // Makes out struct List a covariant.
    use std::marker::PhantomData;
    use std::ptr::NonNull;

    pub struct List<T> {
        front: Link<T>,
        back: Link<T>,
        len: usize,
        _ghost: PhantomData<T>,
    }

    impl<T> List<T> {
        pub fn new() -> Self {
            Self {
                front: None,
                back: None,
                len: 0,
                _ghost: PhantomData,
            }
        }

        pub fn push_front(&mut self, element: T) {
            unsafe {
                let new = NonNull::new_unchecked(Box::into_raw(Box::new(Node::new(element))));

                if let Some(old) = self.front {
                    (*old.as_ptr()).front = Some(new);
                    (*new.as_ptr()).back = Some(old);
                } else {
                    debug_assert!(self.back.is_none());
                    debug_assert!(self.front.is_none());
                    debug_assert!(self.len == 0);

                    self.back = Some(new);
                }
                self.front = Some(new);
                self.len += 1;
            }
        }

        pub fn pop_front(&mut self) -> Option<T> {
            unsafe {
                self.front.map(|node| {
                    let boxed_node = Box::from_raw(node.as_ptr());
                    let element = boxed_node.element;
                    self.front = boxed_node.back;

                    if let Some(new) = self.front {
                        (*new.as_ptr()).front = None;
                    } else {
                        // Empty list here
                        debug_assert!(self.len == 1);
                        self.back = None;
                    }

                    self.len -= 1;
                    element
                })
            }
        }

        pub fn len(&self) -> usize {
            self.len
        }
    }

    type Link<T> = Option<NonNull<Node<T>>>;

    struct Node<T> {
        front: Link<T>,
        back: Link<T>,
        element: T,
    }

    impl<T> Node<T> {
        pub fn new(element: T) -> Self {
            Self {
                front: None,
                back: None,
                element,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::list::List;

    #[test]
    fn test_basic_front() {
        let mut list = List::new();

        // Try to break an empty list
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        // Try to break a one item list
        list.push_front(10);
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        // Mess around
        list.push_front(10);
        assert_eq!(list.len(), 1);
        list.push_front(20);
        assert_eq!(list.len(), 2);
        list.push_front(30);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(30));
        assert_eq!(list.len(), 2);
        list.push_front(40);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(40));
        assert_eq!(list.len(), 2);
        assert_eq!(list.pop_front(), Some(20));
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);
    }
}

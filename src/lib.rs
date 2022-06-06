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

        pub fn push_back(&mut self, element: T) {
            // SAFETY: it's a linked-list, what do you want?
            unsafe {
                let new = NonNull::new_unchecked(Box::into_raw(Box::new(Node::new(element))));
                if let Some(old) = self.back {
                    (*old.as_ptr()).back = Some(new);
                    (*new.as_ptr()).front = Some(old);
                } else {
                    self.front = Some(new);
                }

                self.back = Some(new);
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

        pub fn pop_back(&mut self) -> Option<T> {
            unsafe {
                self.back.map(|node| {
                    let boxed_node = Box::from_raw(node.as_ptr());
                    let element = boxed_node.element;

                    self.back = boxed_node.front;
                    if let Some(new) = self.back {
                        (*new.as_ptr()).back = None;
                    } else {
                        self.front = None;
                    }

                    self.len -= 1;
                    element
                })
            }
        }

        pub fn len(&self) -> usize {
            self.len
        }

        pub fn front(&self) -> Option<&T> {
            unsafe { self.front.as_ref().map(|node| &node.as_ref().element) }
        }

        pub fn front_mut(&mut self) -> Option<&mut T> {
            unsafe { self.front.as_mut().map(|node| &mut node.as_mut().element) }
        }

        pub fn back(&self) -> Option<&T> {
            unsafe { self.back.as_ref().map(|node| &node.as_ref().element) }
        }

        pub fn back_mut(&mut self) -> Option<&mut T> {
            unsafe { self.back.as_mut().map(|node| &mut node.as_mut().element) }
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

    impl<T> Drop for List<T> {
        fn drop(&mut self) {
            while let Some(_) = self.pop_front() {}
        }
    }

    pub struct IntoIter<T> {
        list: List<T>,
    }

    impl<T> List<T> {
        pub fn into_iter(self) -> IntoIter<T> {
            IntoIter { list: self }
        }
    }

    impl<T> IntoIterator for List<T> {
        type IntoIter = IntoIter<T>;
        type Item = T;

        fn into_iter(self) -> Self::IntoIter {
            self.into_iter()
        }
    }

    impl<T> Iterator for IntoIter<T> {
        type Item = T;

        fn next(&mut self) -> Option<Self::Item> {
            self.list.pop_front()
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.list.len, Some(self.list.len))
        }
    }

    impl<T> DoubleEndedIterator for IntoIter<T> {
        fn next_back(&mut self) -> Option<Self::Item> {
            self.list.pop_back()
        }
    }

    impl<T> ExactSizeIterator for IntoIter<T> {
        fn len(&self) -> usize {
            self.list.len
        }
    }

    pub struct Iter<'a, T> {
        front: Link<T>,
        back: Link<T>,
        len: usize,
        _ghost: PhantomData<&'a T>,
    }

    impl<T> List<T> {
        pub fn iter(&self) -> Iter<T> {
            Iter {
                front: self.front,
                back: self.back,
                len: self.len,
                _ghost: PhantomData,
            }
        }
    }

    impl<'a, T> IntoIterator for &'a List<T> {
        type IntoIter = Iter<'a, T>;
        type Item = &'a T;

        fn into_iter(self) -> Self::IntoIter {
            self.iter()
        }
    }

    impl<'a, T> Iterator for Iter<'a, T> {
        type Item = &'a T;

        fn next(&mut self) -> Option<Self::Item> {
            // While self.front == self.back is a tempting condition to check here,
            // it won't do the right for yielding the last element! That sort of
            // thing only works for arrays because of "one-past-the-end" pointers.
            if self.len > 0 {
                // We could unwrap front, but this is safer and easier
                self.front.map(|node| unsafe {
                    self.len -= 1;
                    self.front = (*node.as_ptr()).back;
                    &(*node.as_ptr()).element
                })
            } else {
                None
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.len, Some(self.len))
        }
    }

    impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
        fn next_back(&mut self) -> Option<Self::Item> {
            if self.len > 0 {
                self.back.map(|node| unsafe {
                    self.len -= 1;
                    self.back = (*node.as_ptr()).front;
                    &(*node.as_ptr()).element
                })
            } else {
                None
            }
        }
    }

    impl<'a, T> ExactSizeIterator for Iter<'a, T> {
        fn len(&self) -> usize {
            self.len
        }
    }

    struct IterMut<'a, T> {
        front: Link<T>,
        back: Link<T>,
        len: usize,
        _ghost: PhantomData<&'a mut T>,
    }

    impl<'a, T> Iterator for IterMut<'a, T> {
        type Item = &'a mut T;

        fn next(&mut self) -> Option<Self::Item> {
            // While self.front == self.back is a tempting condition to check here,
            // it won't do the right for yielding the last element! That sort of
            // thing only works for arrays because of "one-past-the-end" pointers.
            if self.len > 0 {
                // We could unwrap front, but this is safer and easier
                self.front.map(|node| unsafe {
                    self.len -= 1;
                    self.front = (*node.as_ptr()).back;
                    &mut (*node.as_ptr()).element
                })
            } else {
                None
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.len, Some(self.len))
        }
    }

    impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
        fn next_back(&mut self) -> Option<Self::Item> {
            if self.len > 0 {
                self.back.map(|node| unsafe {
                    self.len -= 1;
                    self.back = (*node.as_ptr()).front;
                    &mut (*node.as_ptr()).element
                })
            } else {
                None
            }
        }
    }

    impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
        fn len(&self) -> usize {
            self.len
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

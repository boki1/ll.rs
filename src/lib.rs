mod list {
    use std::cmp::Ordering;
    use std::fmt::{self, Debug};
    use std::hash::{Hash, Hasher};
    use std::iter::FromIterator;
    use std::marker::PhantomData; // Makes out struct List a covariant.
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

        pub fn iter_mut(&mut self) -> IterMut<T> {
            IterMut {
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

    pub struct IterMut<'a, T> {
        front: Link<T>,
        back: Link<T>,
        len: usize,
        _ghost: PhantomData<&'a mut T>,
    }

    impl<T> List<T> {}

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

    impl<T> List<T> {
        pub fn is_empty(&self) -> bool {
            self.len == 0
        }

        pub fn clear(&mut self) {
            while let Some(_) = self.pop_front() {}
        }
    }

    impl<T> Default for List<T> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<T: Clone> Clone for List<T> {
        fn clone(&self) -> Self {
            let mut new_list = Self::new();
            for item in self {
                new_list.push_back(item.clone());
            }
            new_list
        }
    }

    impl<T> Extend<T> for List<T> {
        fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
            for item in iter {
                self.push_back(item);
            }
        }
    }

    impl<T> FromIterator<T> for List<T> {
        fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
            let mut list = Self::new();
            list.extend(iter);
            list
        }
    }

    impl<T: Debug> Debug for List<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_list().entries(self).finish()
        }
    }

    impl<T: PartialEq> PartialEq for List<T> {
        fn eq(&self, other: &Self) -> bool {
            self.len() == other.len() && self.iter().eq(other)
        }

        fn ne(&self, other: &Self) -> bool {
            self.len() != other.len() || self.iter().ne(other)
        }
    }

    impl<T: Eq> Eq for List<T> {}

    impl<T: PartialOrd> PartialOrd for List<T> {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            self.iter().partial_cmp(other)
        }
    }

    impl<T: Ord> Ord for List<T> {
        fn cmp(&self, other: &Self) -> Ordering {
            self.iter().cmp(other)
        }
    }

    impl<T: Hash> Hash for List<T> {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.len().hash(state);
            for item in self {
                item.hash(state);
            }
        }
    }

    // Markers
    unsafe impl<T: Send> Send for List<T> {}
    unsafe impl<T: Sync> Sync for List<T> {}

    unsafe impl<'a, T: Send> Send for Iter<'a, T> {}
    unsafe impl<'a, T: Sync> Sync for Iter<'a, T> {}

    unsafe impl<'a, T: Send> Send for IterMut<'a, T> {}
    unsafe impl<'a, T: Sync> Sync for IterMut<'a, T> {}

    // Cursors
    pub struct CursorMut<'a, T> {
        curr: Link<T>,
        list: &'a mut List<T>,
        index: Option<usize>,
    }

    impl<T> List<T> {
        pub fn cursor_mut(&mut self) -> CursorMut<T> {
            CursorMut {
                curr: None,
                list: self,
                index: None,
            }
        }
    }

    impl<'a, T> CursorMut<'a, T> {
        pub fn index(&self) -> Option<usize> {
            self.index
        }

        pub fn move_next(&mut self) {
            if let Some(curr) = self.curr {
                unsafe {
                    self.curr = (*curr.as_ptr()).back;
                    if self.curr.is_some() {
                        *self.index.as_mut().unwrap() += 1;
                    } else {
                        self.index = None
                    }
                }
            } else if !self.list.is_empty() {
                self.curr = self.list.front;
                self.index = Some(0);
            } else {
                // At ghost - the only element.
                // Skip.
            }
        }

        pub fn move_prev(&mut self) {
            if let Some(curr) = self.curr {
                unsafe {
                    // We're on a real element, go to its previous (front)
                    self.curr = (*curr.as_ptr()).front;
                    if self.curr.is_some() {
                        *self.index.as_mut().unwrap() -= 1;
                    } else {
                        // We just walked to the ghost, no more index
                        self.index = None;
                    }
                }
            } else if !self.list.is_empty() {
                self.curr = self.list.back;
                self.index = Some(self.list.len - 1)
            } else {
                // At ghost - the only element.
                // Skip.
            }
        }

        pub fn current(&mut self) -> Option<&mut T> {
            unsafe { self.curr.map(|node| &mut (*node.as_ptr()).element) }
        }

        pub fn peek_next(&mut self) -> Option<&mut T> {
            unsafe {
                self.curr
                    .and_then(|node| (*node.as_ptr()).back)
                    .map(|node| &mut (*node.as_ptr()).element)
            }
        }

        pub fn peek_prev(&mut self) -> Option<&mut T> {
            unsafe {
                self.curr
                    .and_then(|node| (*node.as_ptr()).front)
                    .map(|node| &mut (*node.as_ptr()).element)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::list::List;

    fn generate_test() -> List<i32> {
        list_from(&[0, 1, 2, 3, 4, 5, 6])
    }

    fn list_from<T: Clone>(v: &[T]) -> List<T> {
        v.iter().map(|x| (*x).clone()).collect()
    }

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

    #[test]
    fn test_basic() {
        let mut m = List::new();
        assert_eq!(m.pop_front(), None);
        assert_eq!(m.pop_back(), None);
        assert_eq!(m.pop_front(), None);
        m.push_front(1);
        assert_eq!(m.pop_front(), Some(1));
        m.push_back(2);
        m.push_back(3);
        assert_eq!(m.len(), 2);
        assert_eq!(m.pop_front(), Some(2));
        assert_eq!(m.pop_front(), Some(3));
        assert_eq!(m.len(), 0);
        assert_eq!(m.pop_front(), None);
        m.push_back(1);
        m.push_back(3);
        m.push_back(5);
        m.push_back(7);
        assert_eq!(m.pop_front(), Some(1));

        let mut n = List::new();
        n.push_front(2);
        n.push_front(3);
        {
            assert_eq!(n.front().unwrap(), &3);
            let x = n.front_mut().unwrap();
            assert_eq!(*x, 3);
            *x = 0;
        }
        {
            assert_eq!(n.back().unwrap(), &2);
            let y = n.back_mut().unwrap();
            assert_eq!(*y, 2);
            *y = 1;
        }
        assert_eq!(n.pop_front(), Some(0));
        assert_eq!(n.pop_front(), Some(1));
    }

    #[test]
    fn test_iterator() {
        let m = generate_test();
        for (i, elt) in m.iter().enumerate() {
            assert_eq!(i as i32, *elt);
        }
        let mut n = List::new();
        assert_eq!(n.iter().next(), None);
        n.push_front(4);
        let mut it = n.iter();
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(it.next().unwrap(), &4);
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_iterator_double_end() {
        let mut n = List::new();
        assert_eq!(n.iter().next(), None);
        n.push_front(4);
        n.push_front(5);
        n.push_front(6);
        let mut it = n.iter();
        assert_eq!(it.size_hint(), (3, Some(3)));
        assert_eq!(it.next().unwrap(), &6);
        assert_eq!(it.size_hint(), (2, Some(2)));
        assert_eq!(it.next_back().unwrap(), &4);
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(it.next_back().unwrap(), &5);
        assert_eq!(it.next_back(), None);
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_rev_iter() {
        let m = generate_test();
        for (i, elt) in m.iter().rev().enumerate() {
            assert_eq!(6 - i as i32, *elt);
        }
        let mut n = List::new();
        assert_eq!(n.iter().rev().next(), None);
        n.push_front(4);
        let mut it = n.iter().rev();
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(it.next().unwrap(), &4);
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_mut_iter() {
        let mut m = generate_test();
        let mut len = m.len();
        for (i, elt) in m.iter_mut().enumerate() {
            assert_eq!(i as i32, *elt);
            len -= 1;
        }
        assert_eq!(len, 0);
        let mut n = List::new();
        assert!(n.iter_mut().next().is_none());
        n.push_front(4);
        n.push_back(5);
        let mut it = n.iter_mut();
        assert_eq!(it.size_hint(), (2, Some(2)));
        assert!(it.next().is_some());
        assert!(it.next().is_some());
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert!(it.next().is_none());
    }

    #[test]
    fn test_iterator_mut_double_end() {
        let mut n = List::new();
        assert!(n.iter_mut().next_back().is_none());
        n.push_front(4);
        n.push_front(5);
        n.push_front(6);
        let mut it = n.iter_mut();
        assert_eq!(it.size_hint(), (3, Some(3)));
        assert_eq!(*it.next().unwrap(), 6);
        assert_eq!(it.size_hint(), (2, Some(2)));
        assert_eq!(*it.next_back().unwrap(), 4);
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(*it.next_back().unwrap(), 5);
        assert!(it.next_back().is_none());
        assert!(it.next().is_none());
    }

    #[test]
    fn test_eq() {
        let mut n: List<u8> = list_from(&[]);
        let mut m = list_from(&[]);
        assert!(n == m);
        n.push_front(1);
        assert!(n != m);
        m.push_back(1);
        assert!(n == m);

        let n = list_from(&[2, 3, 4]);
        let m = list_from(&[1, 2, 3]);
        assert!(n != m);
    }

    #[test]
    fn test_ord() {
        let n = list_from(&[]);
        let m = list_from(&[1, 2, 3]);
        assert!(n < m);
        assert!(m > n);
        assert!(n <= n);
        assert!(n >= n);
    }

    #[test]
    fn test_ord_nan() {
        let nan = 0.0f64 / 0.0;
        let n = list_from(&[nan]);
        let m = list_from(&[nan]);
        assert!(!(n < m));
        assert!(!(n > m));
        assert!(!(n <= m));
        assert!(!(n >= m));

        let n = list_from(&[nan]);
        let one = list_from(&[1.0f64]);
        assert!(!(n < one));
        assert!(!(n > one));
        assert!(!(n <= one));
        assert!(!(n >= one));

        let u = list_from(&[1.0f64, 2.0, nan]);
        let v = list_from(&[1.0f64, 2.0, 3.0]);
        assert!(!(u < v));
        assert!(!(u > v));
        assert!(!(u <= v));
        assert!(!(u >= v));

        let s = list_from(&[1.0f64, 2.0, 4.0, 2.0]);
        let t = list_from(&[1.0f64, 2.0, 3.0, 2.0]);
        assert!(!(s < t));
        assert!(s > one);
        assert!(!(s <= one));
        assert!(s >= one);
    }

    #[test]
    fn test_debug() {
        let list: List<i32> = (0..10).collect();
        assert_eq!(format!("{:?}", list), "[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]");

        let list: List<&str> = vec!["just", "one", "test", "more"]
            .iter()
            .copied()
            .collect();
        assert_eq!(format!("{:?}", list), r#"["just", "one", "test", "more"]"#);
    }

    #[test]
    fn test_hashmap() {
        // Check that HashMap works with this as a key

        let list1: List<i32> = (0..10).collect();
        let list2: List<i32> = (1..11).collect();
        let mut map = std::collections::HashMap::new();

        assert_eq!(map.insert(list1.clone(), "list1"), None);
        assert_eq!(map.insert(list2.clone(), "list2"), None);

        assert_eq!(map.len(), 2);

        assert_eq!(map.get(&list1), Some(&"list1"));
        assert_eq!(map.get(&list2), Some(&"list2"));

        assert_eq!(map.remove(&list1), Some("list1"));
        assert_eq!(map.remove(&list2), Some("list2"));

        assert!(map.is_empty());
    }

    #[test]
    #[allow(dead_code)]
    fn markers() {
        use crate::list::*;

        fn is_send<T: Send>() {}
        fn is_sync<T: Sync>() {}

        is_send::<List<i32>>();
        is_sync::<List<i32>>();

        is_send::<IntoIter<i32>>();
        is_sync::<IntoIter<i32>>();

        is_send::<Iter<i32>>();
        is_sync::<Iter<i32>>();

        is_send::<IterMut<i32>>();
        is_sync::<IterMut<i32>>();

        fn linked_list_covariant<'a, T>(x: List<&'static T>) -> List<&'a T> {
            x
        }
        fn iter_covariant<'i, 'a, T>(x: Iter<'i, &'static T>) -> Iter<'i, &'a T> {
            x
        }
        fn into_iter_covariant<'a, T>(x: IntoIter<&'static T>) -> IntoIter<&'a T> {
            x
        }
    }
}

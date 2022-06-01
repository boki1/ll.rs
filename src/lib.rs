mod list {
    pub struct List<T> {
        head: Link<T>,
    }

    impl<T> List<T> {
        pub fn new() -> Self {
            List {
                head: None
            }
        }

        pub fn push(&mut self, element: T) {
            let new_head = Box::new(Node {
                element,
                link: self.head.take(),
            });

            self.head = Some(new_head);
        }

        pub fn pop(&mut self) -> Option<T> {
            self.head.take().map(|head_node| {
                self.head = head_node.link;
                head_node.element
            })
        }

        pub fn peek(&self) -> Option<&T> {
            self.head.as_ref().map(|head_node| {
                &head_node.element
            })
        }

        pub fn peek_mut(&mut self) -> Option<&mut T> {
            self.head.as_mut().map(|head_node| {
                &mut head_node.element
            })
        }
    }

    impl<T> Drop for List<T> {
        fn drop(&mut self) {
            let mut curr_link = self.head.take();

            while let Some(mut boxed_node) = curr_link {
                curr_link = boxed_node.link.take();
            }
        }
    }

    type Link<T> = Option<Box<Node<T>>>;

    struct Node<T> {
        element: T,
        link: Link<T>,
    }

    /// An owning iterator over the entries of `List<T>`.
    /// Out of https://doc.rust-lang.org/std/iter/index.html#implementing-iterator:
    /// "Creating an iterator of your own involves two steps: creating a struct to hold the
    /// iterator’s state, and then implementing Iterator for that struct. This is why there are so
    /// many structs in this module: there is one for each iterator and iterator adapter."
    /// It seems IntoIter is an idiomatic name for iterator adapter.
    pub struct IntoIter<T>(List<T>);

    impl<T> List<T> {
        pub fn into_iter(self) -> IntoIter<T> {
            IntoIter(self)
        }
    }

    /// There are three common methods which can create iterators from a collection:
    ///     - iter(), which iterates over &T.
    ///     - iter_mut(), which iterates over &mut T.
    ///     - into_iter(), which iterates over T.

    impl<T> Iterator for IntoIter<T> {
        type Item = T;
        fn next(&mut self) -> Option<Self::Item> {
            self.0.pop()
        }
    }

    pub struct Iter<'a, T> {
        next: Option<&'a Node<T>>,
    }

    impl<T> List<T> {
        pub fn iter(&self) -> Iter<T> {
            Iter { next: self.head.as_deref() }
        }
    }

    impl<'a, T> Iterator for Iter<'a, T> {
        type Item = &'a T;

        fn next(&mut self) -> Option<Self::Item> {
            self.next.map(|node| {
                // Turbofish (::<>) usage here is an alterantive to using `.as_deref()`.
                // This lets the compiler know that `&node` should have _deref coercion_ applied to
                // so we don't need to manually apply all those *'s! (which would look like: &**node).
                self.next = node.link.as_ref().map::<&Node<T>, _>(|node| &*node);
                &node.element
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::list::List;

    #[test]
    fn it_works() {
        let mut list = List::new();
        for i in 0..10 {
            list.push(i);
        }
        for i in 10..0 {
            assert_eq!(list.pop().unwrap(), i);
        }

        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop(), None);

        // Populate list
        list.push(1);
        list.push(2);
        list.push(3);

        // Check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // Check normal removal
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn generic() {
        let mut list: List<f32> = List::new();
        list.push(3.14);
        assert_eq!(list.pop().unwrap(), 3.14);
    }

    #[test]
    fn peek() {
        let mut list: List<f64> = List::new();
        assert_eq!(list.peek(), None);
        assert_eq!(list.peek_mut(), None);

        list.push(3.14);
        assert_eq!(list.peek(), Some(&3.14));
        assert_eq!(list.peek_mut(), Some(&mut 3.14));

        list.peek_mut().map(|element| {
            *element = 42.0;
        });

        assert_eq!(list.peek_mut(), Some(&mut 42.0));
        assert_eq!(list.peek(), Some(&42.0));
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);
        let mut iter = list.iter();

        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
    }
}

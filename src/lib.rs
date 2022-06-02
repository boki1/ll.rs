mod list {
    use std::rc::Rc;

    pub struct List<T> {
        head: Link<T>,
    }

    type Link<T> = Option<Rc<Node<T>>>;

    pub struct Node<T> {
        link: Link<T>,
        element: T,
    }

    impl<T> List<T> {
        pub fn new() -> Self {
            List { head: None }
        }

        /// Makes a "new" list with a new element, followed by the other nodes.
        pub fn prepend(&self, element: T) -> List<T> {
            let node = Node {
                // Bump self.head's ref count
                link: self.head.clone(),
                element,
            };

            List { head: Some(Rc::new(node)) }
        }

        /// Takes a list with all nodes which follow the head.
        pub fn tail(&self) -> List<T> {
            List { head: self.head.as_ref().and_then(|node| node.link.clone()) }
        }

        pub fn head(&self) -> Option<&T> {
            self.head.as_ref().map(|node| &node.element)
        }
    }

    pub struct Iter<'a, T> {
        link: Option<&'a Node<T>>,
    }

    impl<T> List<T> {
        pub fn iter(&self) -> Iter<'_, T> {
            Iter { link: self.head.as_deref() }
        }
    }

    impl<'a, T> Iterator for Iter<'a, T> {
        type Item = &'a T;

        fn next(&mut self) -> Option<Self::Item> {
            self.link.map(|node| {
                self.link = node.link.as_deref();
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
        let list = List::new();
        assert_eq!(list.head(), None);

        let list = list.prepend(1).prepend(2).prepend(3);
        assert_eq!(list.head(), Some(&3));

        let list = list.tail();
        assert_eq!(list.head(), Some(&2));

        let list = list.tail();
        assert_eq!(list.head(), Some(&1));

        let list = list.tail();
        assert_eq!(list.head(), None);

        // Make sure empty tail works
        let list = list.tail();
        assert_eq!(list.head(), None);
    }

    #[test]
    fn iter() {
        let list = List::new().prepend(1).prepend(2).prepend(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
    }
}

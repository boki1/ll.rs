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
}

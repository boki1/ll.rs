mod list {
    use std::mem;

    pub struct List {
        head: Link,
    }

    impl List {
        pub fn new() -> Self {
            List {
                head: Link::Empty
            }
        }

        pub fn push(&mut self, element: i32) {
            let new_head = Box::new(Node {
                element,
                link: mem::replace(&mut self.head, Link::Empty),
            });

            self.head = Link::More(new_head);
        }

        pub fn pop(&mut self) -> Option<i32> {
            match mem::replace(&mut self.head, Link::Empty) {
                Link::Empty => None,
                Link::More(head_node) => {
                    self.head = head_node.link;
                    Some(head_node.element)
                }
            }
        }
    }

    impl Drop for List {
        fn drop(&mut self) {
            let mut curr_link = mem::replace(&mut self.head, Link::Empty);

            while let Link::More(mut boxed_node) = curr_link {
                curr_link = mem::replace(&mut boxed_node.link, Link::Empty);
            }
        }
    }

    enum Link {
        Empty,
        More(Box<Node>),
    }

    struct Node {
        element: i32,
        link: Link,
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
}

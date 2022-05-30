mod list {
    pub struct List {
        head: Link,
    }

    impl List {
        pub fn new() -> Self {
            List {
                head: None
            }
        }

        pub fn push(&mut self, element: i32) {
            let new_head = Box::new(Node {
                element,
                link: self.head.take(),
            });

            self.head = Some(new_head);
        }

        pub fn pop(&mut self) -> Option<i32> {
            match self.head.take() {
                None => None,
                Some(head_node) => {
                    self.head = head_node.link;
                    Some(head_node.element)
                }
            }
        }
    }

    impl Drop for List {
        fn drop(&mut self) {
            let mut curr_link = self.head.take();

            while let Some(mut boxed_node) = curr_link {
                curr_link = boxed_node.link.take();
            }
        }
    }

    type Link = Option<Box<Node>>;

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

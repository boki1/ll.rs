mod list {
    pub struct List<'a, T> {
        head: Link<T>,
        tail: Option<&'a mut Node<T>>,
    }

    type Link<T> = Option<Box<Node<T>>>;

    struct Node<T> {
        next: Link<T>,
        element: T,
    }

    impl<T> Node<T> {
        pub fn new(element: T) -> Self {
            Self {
                next: None,
                element,
            }
        }
    }

    impl<'a, T> List<'a, T> {
        pub fn new() -> Self {
            Self {
                head: None,
                tail: None,
            }
        }

        pub fn push(&'a mut self, element: T) {
            let new_tail = Box::new(Node::new(element));
            let new_tail = match self.tail.take() {
                Some(mut old_tail) => {
                    old_tail.next = Some(new_tail);
                    old_tail.next.as_deref_mut()
                }
                None => {
                    self.head = Some(new_tail);
                    self.head.as_deref_mut()
                }
            };

            self.tail = new_tail;
        }

        pub fn pop(&mut self) -> Option<T> {
            self.head.take().map(|node| {
                let head = *node;
                self.head = head.next;

                if self.head.is_none() {
                    self.tail = None;
                }

                head.element
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::list::List;

    #[test]
    fn it_works() {
        let mut list: List<u32> = List::new();
        assert_eq!(list.pop(), None);
        list.push(42);
        assert_eq!(list.pop(), Some(42));
    }
}

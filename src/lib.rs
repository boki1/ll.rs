mod list {
    use std::borrow::BorrowMut;
    use std::cell::RefCell;
    use std::rc::Rc;

    pub struct List<T> {
        head: Link<T>,
        tail: Link<T>,
    }

    impl<T> List<T> {
        pub fn new() -> Self {
            Self { head: None, tail: None }
        }

        pub fn push_front(&mut self, element: T) {
            let mut new_node = Node::new(element);
            match self.head.take() {
                Some(old_head) => {
                    new_node.borrow_mut().next = Some(old_head);
                    old_head.borrow_mut().prev = Some(new_node.clone());
                    self.head = Some(new_node);
                }
                None => {
                    self.head = Some(new_node.clone());
                    self.tail = Some(new_node);
                }
            }
        }

        pub fn push_back(&mut self, element: T) {
            let mut new_node = Node::new(element);
            match self.tail.take() {
                Some(mut old_tail) => {
                    new_node.borrow_mut().prev = Some(old_tail);
                    old_tail.borrow_mut().next = Some(new_node.clone());
                    self.tail = Some(new_node);
                }
                None => {
                    self.head = Some(new_node.clone());
                    self.tail = Some(new_node);
                }
            }
        }

        pub fn pop_front(&mut self) -> Result<T, ()> {
            unimplemented!()
        }

        pub fn pop_back(&mut self) -> Result<T, ()> {
            unimplemented!()
        }

        pub fn peek_front(&self) -> Option<&T> {
            self.head.as_ref().map(|head| {
                &head.borrow().element
            })
        }

        pub fn peek_back(&self) -> Option<&T> {
            self.tail.as_ref().map(|tail| {
                &tail.borrow().element
            })
        }
    }

    type Link<T> = Option<Rc<RefCell<Node<T>>>>;

    struct Node<T> {
        element: T,
        next: Link<T>,
        prev: Link<T>,
    }

    impl<T> Node<T> {
        // FIXME: This interface looks bad to me. I would expect that Node::new() creates a Node
        //        instance for me and not being wrapped by something. Maybe something that goes with
        //        the Default trait?
        pub fn new(element: T) -> Rc<RefCell<Self>> {
            Rc::new(RefCell::new(Self {
                element,
                next: None,
                prev: None,
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::list::List;

    #[test]
    fn it_works() {
        let mut list: List<f64> = List::new();
        list.push_front(3.14);
        assert_eq!(list.peek_front(), 3.14);
        list.push_back(2.12);
        assert_eq!(list.peek_back(), 2.12);
        list.push_back(10.04);
        assert_eq!(list.peek_back(), 10.04);
    }
}

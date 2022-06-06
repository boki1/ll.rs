mod list {
    use std::ptr;

    pub struct List<T> {
        head: Link<T>,
        tail: *mut Node<T>,
    }

    // Do not mix safe and unsafe primitives in order to escape from UB.
    type Link<T> = *mut Node<T>;

    struct Node<T> {
        next: Link<T>,
        element: T,
    }

    impl<T> Node<T> {
        pub fn new(element: T) -> Self {
            Self {
                next: ptr::null_mut(),
                element,
            }
        }
    }

    impl<T> List<T> {
        pub fn new() -> Self {
            Self {
                head: ptr::null_mut(),
                tail: ptr::null_mut(),
            }
        }

        // Note:
        // If you never actually dereference a raw pointer those are totally safe things to do.
        // You're just reading and writing an integer! The only time you can actually get into
        // trouble with a raw pointer is if you actually dereference it. So Rust says only that
        // operation is unsafe, and everything else is totally safe.
        // Super. Pedantic. But technically correct.

        pub fn push(&mut self, element: T) {
            let raw_tail = Box::into_raw(Box::new(Node::new(element)));
            if !self.tail.is_null() {
                // Hello Compiler, I Know I Am Doing Something Dangerous And
                // I Promise To Be A Good Programmer Who Never Makes Mistakes.
                //
                // Safety: ???
                unsafe {
                    (*self.tail).next = raw_tail;
                }
            } else {
                self.head = raw_tail;
            }

            self.tail = raw_tail;
        }

        pub fn pop(&mut self) -> Option<T> {
            if self.head.is_null() {
                None
            } else {
                let head = unsafe { Box::from_raw(self.head) };
                self.head = head.next;

                if self.head.is_null() {
                    self.tail = ptr::null_mut();
                }

                Some(head.element)
            }
        }
    }

    impl<T> Drop for List<T> {
        fn drop(&mut self) {
            while let Some(_) = self.pop() {}
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

    #[test]
    fn basics() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop(), None);

        // Populate list
        list.push(1);
        list.push(2);
        list.push(3);

        // Check normal removal
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // Check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), None);

        // Check the exhaustion case fixed the pointer right
        list.push(6);
        list.push(7);

        // Check normal removal
        assert_eq!(list.pop(), Some(6));
        assert_eq!(list.pop(), Some(7));
        assert_eq!(list.pop(), None);
    }
}

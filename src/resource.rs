use std::{
    cell::{Ref, RefCell},
    ops::{Deref, DerefMut},
    rc::Rc,
};

/// A shared resource wrapper that provides interior mutability.
#[derive(Debug, PartialEq, Eq, Default)]
pub struct Resource<T> {
    inner: Rc<RefCell<T>>,
}

impl<T> Resource<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(value)),
        }
    }

    /// Borrows the resource immutably.
    pub fn get(&self) -> std::cell::Ref<'_, T> {
        self.inner.borrow()
    }

    /// Borrows the resource mutably.
    pub fn get_mut(&self) -> std::cell::RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}

impl<T> From<T> for Resource<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> Clone for Resource<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

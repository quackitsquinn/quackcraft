use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use anymap::AnyMap;

/// A database for storing components of various types.
pub struct ComponentDB {
    map: Rc<AnyMap>,
}

impl ComponentDB {
    /// Creates a new, empty component database.
    pub fn new() -> Self {
        Self {
            map: Rc::new(AnyMap::new()),
        }
    }

    /// Gets a reference to a component of the specified type.
    pub fn get_checked<T: 'static>(&self) -> Option<Ref<'_, T>> {
        let component = self.map.get::<Component<T>>()?;
        Some(component.get())
    }

    /// Gets a reference to a component of the specified type.
    pub fn get<T: 'static>(&self) -> Ref<T> {
        self.get_checked()
            .expect("Component not found in ComponentDB")
    }

    /// Gets a mutable reference to a component of the specified type.
    pub fn get_mut_checked<T: 'static>(&self) -> Option<std::cell::RefMut<'_, T>> {
        let component = self.map.get::<Component<T>>()?;
        Some(component.get_mut())
    }

    /// Gets a mutable reference to a component of the specified type.
    pub fn get_mut<T: 'static>(&self) -> std::cell::RefMut<'_, T> {
        self.get_mut_checked()
            .expect("Component not found in ComponentDB")
    }

    /// Inserts a component into the database.
    ///
    /// There must be no other references to the database when calling this method.
    pub fn insert<T: 'static>(&mut self, component: T) {
        let map = Rc::get_mut(&mut self.map)
            .expect("Cannot insert into ComponentDB while there are other references");

        map.insert::<Component<T>>(Component::new(component));
    }
}

/// Internal representation of a component.
struct Component<T> {
    // This struct mainly exists right now just as future-proofing in case we want to add
    // more functionality to components later.
    data: RefCell<T>,
}

impl<T> Component<T> {
    /// Creates a new component.
    fn new(data: T) -> Self {
        Self {
            data: RefCell::new(data),
        }
    }

    fn get(&self) -> std::cell::Ref<T> {
        self.data.borrow()
    }

    fn get_mut(&self) -> std::cell::RefMut<T> {
        self.data.borrow_mut()
    }
}

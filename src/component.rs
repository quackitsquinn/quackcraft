use std::{
    any::TypeId,
    cell::{Cell, OnceCell, Ref, RefCell},
    fmt::Debug,
    rc::{Rc, Weak},
};

use anymap::{AnyMap, any::UncheckedAnyExt, raw::RawMap};
use wgpu::naga::Type;

/// A database for storing components of various types.
#[derive(Clone)]
pub struct State {
    map: Rc<RawMap>,
    public_ref: Rc<OnceCell<State>>,
}

impl State {
    /// Creates a new, empty component database.
    pub fn new() -> Self {
        Self {
            map: Rc::new(RawMap::new()),
            public_ref: Rc::new(OnceCell::new()),
        }
    }

    /// Finalizes the initialization of the component database.
    pub fn finish_initialization(&self) {
        let _ = self.public_ref.set(self.clone());
    }

    /// Gets a reference to a component of the specified type.
    pub fn get_checked<T: 'static>(&self) -> Option<Ref<'_, T>> {
        let component = self.map.get(&TypeId::of::<Resource<T>>())?;
        Some(unsafe { component.downcast_ref_unchecked::<Resource<T>>().get() })
    }

    /// Gets a reference to a component of the specified type.
    pub fn get<T: 'static>(&self) -> Ref<T> {
        self.get_checked()
            .expect("Component not found in ComponentDB")
    }

    /// Gets a mutable reference to a component of the specified type.
    pub fn get_mut_checked<T: 'static>(&self) -> Option<std::cell::RefMut<'_, T>> {
        let component = self.map.get(&TypeId::of::<Resource<T>>())?;
        Some(unsafe { component.downcast_ref_unchecked::<Resource<T>>().get_mut() })
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

        unsafe {
            map.insert(
                TypeId::of::<Resource<T>>(),
                Box::new(Resource::new(component)),
            )
        };
    }

    /// Creates a handle for a component of the specified type.
    pub fn handle_for<T: 'static>(&self) -> ResourceHandle<T> {
        ResourceHandle::new(self.handle())
    }

    /// Creates a handle to the component map.
    pub fn handle(&self) -> StateHandle {
        StateHandle::new(self)
    }
}

impl Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut type_names: Vec<&'static str> = vec![];
        for component in self.map.iter() {
            // SAFETY: We only insert Component<T> types into the map, and Component<T> always has a &'static str type_name field.
            let type_name = unsafe { component.downcast_ref_unchecked::<&'static str>() };
            type_names.push(type_name);
        }
        f.debug_struct("State")
            .field("resources", &type_names)
            .finish()
    }
}

/// Internal representation of a resource.
///
/// I promise this codebase won't have multiple Resource structs with different meanings, the old one is being removed soon.
#[repr(C)]
struct Resource<T> {
    // WARNING: field order matters here!
    type_name: &'static str,
    data: RefCell<T>,
}

impl<T> Resource<T> {
    /// Creates a new component.
    fn new(data: T) -> Self {
        Self {
            data: RefCell::new(data),
            type_name: std::any::type_name::<T>(),
        }
    }

    fn get(&self) -> std::cell::Ref<'_, T> {
        self.data.borrow()
    }

    fn get_mut(&self) -> std::cell::RefMut<'_, T> {
        self.data.borrow_mut()
    }
}

/// A handle to a component stored in a `ComponentDB`.
pub struct ResourceHandle<T: 'static> {
    handle: StateHandle,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> ResourceHandle<T> {
    fn new(state_handle: StateHandle) -> Self {
        Self {
            handle: state_handle,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Gets a reference to the component.
    pub fn get(&self) -> Ref<T> {
        self.handle.get::<T>()
    }

    /// Gets a mutable reference to the component.
    pub fn get_mut(&self) -> std::cell::RefMut<T> {
        self.handle.get_mut::<T>()
    }
}

impl<T> Debug for ResourceHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ResourceHandle<{}>", std::any::type_name::<T>())
    }
}

impl<T> Clone for ResourceHandle<T> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}

/// A handle to a ComponentMap that allows checking its state.
#[derive(Clone)]
pub struct StateHandle {
    // TODO: Figure out a way to optimize this into a single pointer sized field.
    // This is gonna need some unsafe code and weird pointer tagging so this is a later task.
    handle: OnceCell<State>,
    global_handle: Rc<OnceCell<State>>,
}

impl StateHandle {
    pub fn new(component_map: &State) -> Self {
        Self {
            handle: OnceCell::new(),
            global_handle: component_map.public_ref.clone(),
        }
    }

    fn get_map(&self) -> Option<&State> {
        if let Some(map) = self.handle.get() {
            return Some(map);
        }

        self.global_handle.get().and_then(|map| {
            let _ = self.handle.set(map.clone());
            self.handle.get()
        })
    }

    /// Gets a reference to a component of the specified type.
    pub fn get_checked<T: 'static>(&self) -> Option<Ref<T>> {
        let map = self.get_map()?;
        Some(map.get())
    }

    /// Gets a reference to a component of the specified type.
    pub fn get<T: 'static>(&self) -> Ref<T> {
        self.get_checked()
            .expect("Component not found in ComponentDB")
    }

    /// Gets a mutable reference to a component of the specified type.
    pub fn get_mut_checked<T: 'static>(&self) -> Option<std::cell::RefMut<'_, T>> {
        let map = self.get_map()?;
        Some(map.get_mut())
    }

    /// Gets a mutable reference to a component of the specified type.
    pub fn get_mut<T: 'static>(&self) -> std::cell::RefMut<'_, T> {
        self.get_mut_checked()
            .expect("Component not found in ComponentDB")
    }

    /// Creates a handle for a component of the specified type.
    pub fn handle_for<T: 'static>(&self) -> ResourceHandle<T> {
        ResourceHandle::new(self.clone())
    }
}

impl Debug for StateHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateHandle").finish()
    }
}

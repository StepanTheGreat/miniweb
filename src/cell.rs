//! Minimal cell structures

pub use core::cell::UnsafeCell;
use core::ops::Deref;

/// This leads to huge explosion in size
pub struct InitCell<T>(UnsafeCell<Option<T>>);

impl<T> InitCell<T> {
    pub const fn new() -> Self {
        Self(UnsafeCell::new(None))
    }

    pub fn is_init(&self) -> bool {
        unsafe { &*self.0.get() }.is_some()
    }

    pub fn init(&self, value: T) {
        if !self.is_init() {
            unsafe { &mut *self.0.get() }.replace(value);
        }
    }

    pub fn get(&self) -> Option<&T> {
        unsafe { (&*self.0.get()).as_ref() }
    } 

    pub unsafe fn get_unchecked(&self) -> &T {
        unsafe { self.get().unwrap_unchecked() }
    }
}

// Wasm has only one thread
unsafe impl<T> Sync for InitCell<T> {}
unsafe impl<T> Send for InitCell<T> {}


enum InitOrSome<T, F>
where F: FnOnce() -> T {
    Init(F),
    Some(T)
}

pub struct AutoCell<T, F = fn() -> T>(UnsafeCell<InitOrSome<T, F>>)
where F: FnOnce() -> T;

impl<T, F> AutoCell<T, F>
where F: FnOnce() -> T {
    pub const fn new(f: F) -> Self {
        Self(UnsafeCell::new(InitOrSome::Init(f)))
    }

    fn is_init(&self) -> bool {
        let data = unsafe { &*self.0.get() };

        matches!(data, InitOrSome::Some(_))
    }

    fn init(&self) {
        if !self.is_init() {
            unsafe {
                let data_ptr = self.0.get();

                let init_value = match data_ptr.read() {
                    InitOrSome::Init(f) => InitOrSome::Some((f)()),
                    
                    // This is unreachable, but, to avoid unneccessary codegen from the
                    // unreachable macro, let's pretend like this succeeds no matter what
                    InitOrSome::Some(value) => InitOrSome::Some(value)
                };

                data_ptr.write(init_value);
            }
        }
    }

    pub fn get(&self) -> &T {
        self.init();

        match unsafe { &*self.0.get() } {
            InitOrSome::Some(value) => value,
            _ => unreachable!()
        }
    } 
}

impl<T, F> Deref for AutoCell<T, F>
where F: FnOnce() -> T {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

// Wasm has only one thread
unsafe impl<T> Sync for AutoCell<T> {}
unsafe impl<T> Send for AutoCell<T> {}
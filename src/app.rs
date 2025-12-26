use core::cell::UnsafeCell;

#[cfg(target_family = "wasm")]
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    let msg = info.message().as_str();
    let location = info.location().unwrap();

    crate::js::panic(msg, location.file(), location.line());

    // Unreachable
    loop {}
}

pub trait AppHandler {
    /// The animation loop itself (identical to update)
    fn draw(&mut self);
}

pub struct AppCell<T: AppHandler>(UnsafeCell<Option<T>>);
unsafe impl<T: AppHandler> Send for AppCell<T> {}
unsafe impl<T: AppHandler> Sync for AppCell<T> {}

impl<T: AppHandler> AppCell<T> {
    pub const fn new() -> Self {
        Self(UnsafeCell::new(None))
    }

    pub const fn is_init(&self) -> bool {
        unsafe { & *self.0.get() }.is_some()
    }

    pub unsafe fn init(&self, app: T) {
        if !self.is_init() {
            unsafe { &mut *self.0.get() }.replace(app);
        }
    }

    pub const unsafe fn get_mut(&self) -> Option<&mut T> {
        if !self.is_init() {
            return None;
        }
        // assert!(self.is_init(), "The app isn't initialised");

        Some(
            unsafe { (&mut *self.0.get()).as_mut().unwrap() }
        )
    }

    pub const unsafe fn get(&self) -> Option<&T> {
        if !self.is_init() {
            return None;
        }

        Some(
            unsafe { (&*self.0.get()).as_ref().unwrap() }
        )
    }
}

/// The global macro for generating an app 
#[macro_export]
macro_rules! make_app {
    ($ty:ident) => {
        static APP: AppCell<$ty> = AppCell::new();

        unsafe fn init_app(app: $ty) {
            unsafe { APP.init(app) };
        } 

        unsafe fn get_app<'a>() -> &'a mut $ty {
            unsafe { APP.get_mut().unwrap() }
        }

        
        #[unsafe(no_mangle)]
        pub extern "C" fn __main() {
            let app = main();
            
            unsafe { init_app(app) };
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn __draw() {
            unsafe { get_app() }.draw();
        }
    };
}
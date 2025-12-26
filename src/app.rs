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

/// The global macro for generating an app 
#[macro_export]
macro_rules! make_app {
    ($ty:ident) => {
        static APP: InitCell<UnsafeCell<$ty>> = InitCell::new();

        fn init_app(app: $ty) {
            APP.init(UnsafeCell::new(app));
        } 

        unsafe fn get_app<'a>() -> &'a mut $ty {
            unsafe { &mut *APP.get().unwrap().get() }
        }

        
        #[unsafe(no_mangle)]
        pub extern "C" fn __main() {
            let app = main();
            
            init_app(app);
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn __draw() {
            unsafe { get_app() }.draw();
        }
    };
}
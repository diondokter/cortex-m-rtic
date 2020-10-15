//! examples/message.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    #[init]
    fn init(_c: init::Context) -> init::LateResources {
        foo::spawn(1, 2).unwrap();

        init::LateResources {}
    }

    #[task]
    fn foo(_c: foo::Context, x: i32, y: u32) {
        hprintln!("foo {}, {}", x, y).unwrap();
        if x == 2 {
            debug::exit(debug::EXIT_SUCCESS);
        }
        foo2::spawn(2).unwrap();
    }

    #[task]
    fn foo2(_c: foo2::Context, x: i32) {
        hprintln!("foo2 {}", x).unwrap();
        foo::spawn(x, 0).unwrap();
    }

    // RTIC requires that unused interrupts are declared in an extern block when
    // using software tasks; these free interrupts will be used to dispatch the
    // software tasks.
    extern "C" {
        fn SSI0();
    }
}

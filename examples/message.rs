//! examples/message.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::{debug, hprintln};
use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    #[init(spawn = [foo])]
    fn init(c: init::Context) -> init::LateResources {
        c.spawn.foo(/* no message */).unwrap();

        init::LateResources {}
    }

    #[task(spawn = [bar])]
    fn foo(c: foo::Context) {
        static mut COUNT: u32 = 0;

        hprintln!("foo").unwrap();

        c.spawn.bar(*COUNT).unwrap();
        *COUNT += 1;
    }

    #[task(spawn = [baz])]
    fn bar(c: bar::Context, x: u32) {
        hprintln!("bar({})", x).unwrap();

        c.spawn.baz(x + 1, x + 2).unwrap();
    }

    #[task(spawn = [foo])]
    fn baz(c: baz::Context, x: u32, y: u32) {
        hprintln!("baz({}, {})", x, y).unwrap();

        if x + y > 4 {
            debug::exit(debug::EXIT_SUCCESS);
        }

        c.spawn.foo().unwrap();
    }

    // RTIC requires that unused interrupts are declared in an extern block when
    // using software tasks; these free interrupts will be used to dispatch the
    // software tasks.
    extern "C" {
        fn SSI0();
    }
}

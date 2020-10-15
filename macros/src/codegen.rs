use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::ast::App;

use crate::{analyze::Analysis, check::Extra};

mod assertions;
mod dispatchers;
mod hardware_tasks;
mod idle;
mod init;
mod locals;
mod module;
mod post_init;
mod pre_init;
mod resources;
mod resources_struct;
mod software_tasks;
mod timer_queue;
mod util;

// TODO document the syntax here or in `rtic-syntax`
pub fn app(app: &App, analysis: &Analysis, extra: &Extra) -> TokenStream2 {
    let mut mod_app = vec![];
    let mut mains = vec![];
    let mut root = vec![];
    let mut user = vec![];
    let mut imports = vec![];

    // Generate the `main` function
    let assertion_stmts = assertions::codegen(analysis);

    let pre_init_stmts = pre_init::codegen(&app, analysis, extra);

    let (mod_app_init, root_init, user_init, user_init_imports, call_init) =
        init::codegen(app, analysis, extra);

    let post_init_stmts = post_init::codegen(&app, analysis);

    let (mod_app_idle, root_idle, user_idle, user_idle_imports, call_idle) =
        idle::codegen(app, analysis, extra);

    user.push(quote!(
        #user_init

        #user_idle
    ));

    imports.push(quote!(
        #(#user_init_imports)*
        #(#user_idle_imports)*
    ));

    root.push(quote!(
        #(#root_init)*

        #(#root_idle)*
    ));

    mod_app.push(quote!(
        #mod_app_init

        #mod_app_idle
    ));

    let main = util::suffixed("main");
    mains.push(quote!(
        #[no_mangle]
        unsafe extern "C" fn #main() -> ! {
            let _TODO: () = ();

            #(#assertion_stmts)*

            #(#pre_init_stmts)*

            #call_init

            #(#post_init_stmts)*

            #call_idle
        }
    ));

    let (mod_app_resources, mod_resources, mod_resources_imports) =
        resources::codegen(app, analysis, extra);

    let (
        mod_app_hardware_tasks,
        root_hardware_tasks,
        user_hardware_tasks,
        user_hardware_tasks_imports,
    ) = hardware_tasks::codegen(app, analysis, extra);

    let (
        mod_app_software_tasks,
        root_software_tasks,
        user_software_tasks,
    ) = software_tasks::codegen(app, analysis, extra);

    let mod_app_dispatchers = dispatchers::codegen(app, analysis, extra);
    let mod_app_timer_queue = timer_queue::codegen(app, analysis, extra);
    let user_imports = &app.user_imports;
    let user_code = &app.user_code;
    let name = &app.name;
    let device = extra.device;

    quote!(

        /// Implementation details
        mod #name {
            /// Always include the device crate which contains the vector table
            use #device as _;
            #(#user)*

            #(#user_hardware_tasks)*

            #(#user_software_tasks)*

            #(#root)*

            #mod_resources

            #(#root_hardware_tasks)*

            #(#root_software_tasks)*

            #(#imports)*
            #(#user_imports)*

            /// User code from within the module
            #(#user_code)*
            /// User code end


            #(#user_hardware_tasks_imports)*

            #(#mod_resources_imports)*

            /// app module
            #(#mod_app)*

            #(#mod_app_resources)*

            #(#mod_app_hardware_tasks)*

            #(#mod_app_software_tasks)*

            #(#mod_app_dispatchers)*

            #(#mod_app_timer_queue)*

            #(#mains)*
        }
    )
}

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::ast::App;

use crate::{analyze::Analysis, check::Extra, codegen::util};

/// Generates timer queues and timer queue handlers
pub fn codegen(app: &App, analysis: &Analysis, extra: &Extra) -> Vec<TokenStream2> {
    let mut items = vec![];

    if let Some(timer_queue) = &analysis.timer_queues.first() {
        let t = util::schedule_t_ident();

        // Enumeration of `schedule`-able tasks
        {
            let variants = timer_queue
                .tasks
                .iter()
                .map(|name| {
                    let cfgs = &app.software_tasks[name].cfgs;

                    quote!(
                        #(#cfgs)*
                        #name
                    )
                })
                .collect::<Vec<_>>();

            let doc = format!("Tasks that can be scheduled");
            items.push(quote!(
                #[doc = #doc]
                #[allow(non_camel_case_types)]
                #[derive(Clone, Copy)]
                enum #t {
                    #(#variants,)*
                }
            ));
        }

        let tq = util::tq_ident();

        // Static variable and resource proxy
        {
            let doc = format!("Timer queue");
            let m = extra.monotonic();
            let n = util::capacity_typenum(timer_queue.capacity, false);
            let tq_ty = quote!(rtic::export::TimerQueue<#m, #t, #n>);

            items.push(quote!(
                #[doc = #doc]
                static mut #tq: #tq_ty = rtic::export::TimerQueue(
                    rtic::export::BinaryHeap(
                        rtic::export::iBinaryHeap::new()
                    )
                );

                struct #tq<'a> {
                    priority: &'a rtic::export::Priority,
                }
            ));

            items.push(util::impl_mutex(
                extra,
                &[],
                false,
                &tq,
                tq_ty,
                timer_queue.ceiling,
                quote!(&mut #tq),
            ));
        }

        // Timer queue handler
        {
            let arms = timer_queue
                .tasks
                .iter()
                .map(|name| {
                    let task = &app.software_tasks[name];

                    let cfgs = &task.cfgs;
                    let priority = task.args.priority;
                    let rq = util::rq_ident(priority);
                    let rqt = util::spawn_t_ident(priority);
                    let enum_ = util::interrupt_ident();
                    let interrupt = &analysis.interrupts.get(&priority);

                    let pend = {
                        quote!(
                            rtic::pend(you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml::#enum_::#interrupt);
                        )
                    };

                    quote!(
                        #(#cfgs)*
                        #t::#name => {
                            (#rq { priority: &rtic::export::Priority::new(PRIORITY) }).lock(|rq| {
                                rq.split().0.enqueue_unchecked((#rqt::#name, index))
                            });

                            #pend
                        }
                    )
                })
                .collect::<Vec<_>>();

            let priority = timer_queue.priority;
            let sys_tick = util::suffixed("SysTick");
            items.push(quote!(
                #[no_mangle]
                unsafe fn #sys_tick() {
                    use rtic::Mutex as _;

                    /// The priority of this handler
                    const PRIORITY: u8 = #priority;

                    rtic::export::run(PRIORITY, || {
                        while let Some((task, index)) = (#tq {
                            // NOTE dynamic priority is always the static priority at this point
                            priority: &rtic::export::Priority::new(PRIORITY),
                        })
                        // NOTE `inline(always)` produces faster and smaller code
                            .lock(#[inline(always)]
                                |tq| tq.dequeue())
                        {
                            match task {
                                #(#arms)*
                            }
                        }
                    });
                }
            ));
        }
    }
    items
}

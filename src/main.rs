
#![no_main]
#![no_std]

// RTIC provides the concurrency model and application macro.
use rtic::app;

// Host-visible logging through QEMU semihosting.
use cortex_m_semihosting::{debug, hprintln};

// Halt safely if an unrecoverable panic occurs.
use panic_halt as _;

#[app(
    device = lm3s6965,
    dispatchers = [SSI0]
)]
mod application {
    use super::*;

    /// Resources shared between RTIC tasks.
    ///
    /// RTIC generates the synchronization required for safe access.
    #[shared]
    struct Shared {
        /// Example application state.
        ///
        /// In a real system, this could represent active sensor
        /// consumers, a sample count, or a protected state variable.
        active_users: u32,
    }

    /// Resources owned by individual tasks.
    ///
    /// This demo needs no task-local state, but defining Local
    /// explicitly makes the application structure easy to extend.
    #[local]
    struct Local {}

    /// Application initialization.
    ///
    /// RTIC invokes this once during startup, before normal tasks run.
    /// The returned Shared and Local values initialize the resources
    /// declared above.
    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        hprintln!("[init] RTIC application starting").unwrap();

        // Request execution of the low-priority software task.
        //
        // RTIC schedules this task at priority 1.
        worker::spawn().unwrap();

        (
            Shared { active_users: 0 },
            Local {},
            init::Monotonics(),
        )
    }

    /// Low-priority software task.
    ///
    /// Priority 1 is lower than the priority-2 interrupt handler.
    /// The worker deliberately pends that interrupt while it is
    /// executing, making the preemption order observable in QEMU.
    #[task(priority = 1, shared = [active_users])]
    fn worker(mut cx: worker::Context) {
        hprintln!("[worker] START: low-priority task running").unwrap();

        // Access the shared resource through RTIC's lock API.
        //
        // The closure is the critical section for this resource.
        // RTIC prevents conflicting task access while it is locked.
        cx.shared.active_users.lock(|active_users| {
            *active_users += 1;
            hprintln!(
                "[worker] active_users incremented to {}",
                *active_users
            )
            .unwrap();
        });

        hprintln!(
            "[worker] Requesting high-priority GPIOA interrupt"
        )
        .unwrap();

        // This requests an interrupt through the Cortex-M NVIC.
        //
        // GPIOA is the interrupt vector for the emulated LM3S6965,
        // not an STM32 TIM2 timer. The interrupt is pended explicitly
        // so the demo does not depend on physical GPIO input.
        //
        // SAFETY: GPIOA is a valid interrupt in the selected device
        // PAC. The interrupt is used by this application as its
        // high-priority demonstration handler.
        unsafe {
            cortex_m::peripheral::NVIC::pend(
                lm3s6965::Interrupt::GPIOA,
            );
        }

        // With the higher-priority interrupt pending and enabled,
        // the processor can preempt this task before the next log.
        hprintln!(
            "[worker] RESUMED: high-priority handler has returned"
        )
        .unwrap();

        cx.shared.active_users.lock(|active_users| {
            hprintln!(
                "[worker] Final active_users = {}",
                *active_users
            )
            .unwrap();
        });

        hprintln!("[worker] DEMO COMPLETE").unwrap();

        // Exit QEMU rather than leaving the emulated MCU in an
        // infinite idle loop. This is simulation-only behavior.
        debug::exit(debug::EXIT_SUCCESS);
    }

    /// High-priority hardware interrupt handler.
    ///
    /// On the emulated Cortex-M3, GPIOA is a real interrupt vector.
    /// Here it is pended by the worker to demonstrate preemption.
    #[task(binds = GPIOA, priority = 2, shared = [active_users])]
    fn gpioa_interrupt(mut cx: gpioa_interrupt::Context) {
        hprintln!(
            "[GPIOA IRQ] PREEMPTION: priority-2 handler entered"
        )
        .unwrap();

        // This handler can access the same shared resource safely.
        cx.shared.active_users.lock(|active_users| {
            hprintln!(
                "[GPIOA IRQ] active_users observed = {}",
                *active_users
            )
            .unwrap();
        });

        hprintln!(
            "[GPIOA IRQ] Handler complete; worker can resume"
        )
        .unwrap();
    }
}

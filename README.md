# RTIC-Preemption-Demo

# RTIC Preemption Demo — Embedded Rust on QEMU

A bare-metal Rust project demonstrating priority-based preemption, interrupt-driven concurrency, and protected shared state using **RTIC (Real-Time Interrupt-driven Concurrency)** on an emulated Cortex-M3 processor.

The repository is designed to be cloned, built, and executed on a developer's Linux, macOS, or Windows host without a physical microcontroller.

## Project goals

- Demonstrate RTIC software-task scheduling and interrupt priorities.
- Show a higher-priority interrupt preempting a lower-priority task.
- Share mutable state safely between concurrent execution contexts.
- Observe execution order through semihosting logs.
- Reproduce the demonstration using QEMU and Cargo.
- Establish a foundation for future sensor-monitoring and actuator-control projects.

## Architecture

```text
                    QEMU: Cortex-M3
                           |
                    RTIC initialization
                           |
                    Spawn worker task
                       Priority 1
                           |
                  Update shared state
                           |
                    Pend GPIOA IRQ
                           |
                  +--------v---------+
                  | GPIOA interrupt  |
                  |   Priority 2    |
                  +--------+---------+
                           |
                  Inspect shared state
                           |
                  Return from interrupt
                           |
                  Worker resumes
                           |
                     Exit QEMU
```

### Why RTIC?

RTIC provides a framework for structuring concurrent embedded applications around task priorities and hardware interrupts.

Unlike a conventional cooperative loop, a higher-priority interrupt can interrupt a lower-priority task when the interrupt becomes pending and is enabled. The processor saves the interrupted execution context, runs the handler, and subsequently restores the lower-priority task.

This behavior is particularly important for embedded systems that must respond promptly to sensor events, communication deadlines, or safety-related signals.

### Priority-based preemption

This example defines two execution contexts:

| Context | Priority | Responsibility |
|---|---:|---|
| `worker` | 1 | Update application state and request an interrupt |
| `gpioa_interrupt` | 2 | Inspect shared state and demonstrate preemption |

The GPIOA handler is pended through the Nested Vectored Interrupt Controller (NVIC). Once the higher-priority interrupt is taken, the handler executes before the worker continues.

The GPIOA vector is selected because the QEMU machine used here is an LM3S6965 Cortex-M3 board, not an STM32. This is a reproducible interrupt demonstration, not an implementation of a physical sensor peripheral.

### Shared resources and SRP

The application contains a shared `active_users` counter. Both tasks access it through RTIC's generated resource-locking API.

This illustrates important real-time concurrency principles:

- Shared mutable state should have an explicit synchronization strategy.
- Critical sections should be kept short.
- Task priorities and resource-access rules must be considered together.
- Blocking and priority inversion must be accounted for in timing analysis.

**Important:** Priority-based preemption and safe resource locking demonstrate concepts associated with real-time resource protocols, but this small example is not a formal proof of all Stack Resource Policy (SRP) properties. SRP analysis also requires resource ceilings, system ceilings, and the relevant scheduling assumptions to be specified and verified.

## Technology stack

- Rust and Cargo
- RTIC 1.1.x
- Cortex-M3 / `thumbv7m-none-eabi`
- QEMU system emulation
- Cortex-M NVIC interrupt handling
- Semihosting diagnostics

## Hardware requirements

No physical hardware is required.

QEMU emulates the LM3S6965 evaluation board, which contains a Cortex-M3 processor and peripherals. The application is compiled for the ARMv7-M bare-metal target and executed by the emulator.

**Simulation limitations:** QEMU is not a cycle-accurate model of every microcontroller behavior. This project demonstrates execution order and software structure; it does not establish worst-case execution times, interrupt latency guarantees, or safety certification.

## Prerequisites

Install the following tools:

1. Rust via [rustup](https://rustup.rs/).
2. The ARM bare-metal compilation target.
3. QEMU's ARM system emulator, `qemu-system-arm`.

Install the Rust target:

```bash
rustup target add thumbv7m-none-eabi
```

Check the toolchain:

```bash
rustc --version
cargo --version
rustup target list --installed
```

The installed target list should include:

```text
thumbv7m-none-eabi
```

### Install QEMU

**Ubuntu / Debian**

```bash
sudo apt update
sudo apt install qemu-system-arm
```

**macOS with Homebrew**

```bash
brew install qemu
```

**Windows**

Install a QEMU distribution that provides `qemu-system-arm`, using the official [QEMU download page](https://www.qemu.org/download/). Ensure the QEMU installation directory is available on your `PATH`.

Verify the installation:

```bash
qemu-system-arm --version
```

If the executable is not found, correct your `PATH` before continuing.

## Clone and run

Clone the repository:

```bash
git clone https://github.com/YOUR_USERNAME/rtic-preemption-demo.git
cd rtic-preemption-demo
```

Replace `YOUR_USERNAME` with your GitHub username.

Build and execute the application:

```bash
cargo run
```

Cargo reads `.cargo/config.toml`, compiles for `thumbv7m-none-eabi`, and invokes QEMU automatically.

To build without running the emulator:

```bash
cargo build
```

To build an optimized release binary:

```bash
cargo build --release
```

To run the optimized binary:

```bash
cargo run --release
```

The program exits through the semihosting debug-exit mechanism when the demonstration finishes.

## Expected behavior

The terminal should show a sequence similar to:

```text
[init] RTIC application starting
[worker] START: low-priority task running
[worker] active_users incremented to 1
[worker] Requesting high-priority GPIOA interrupt
[GPIOA IRQ] PREEMPTION: priority-2 handler entered
[GPIOA IRQ] active_users observed = 1
[GPIOA IRQ] Handler complete; worker can resume
[worker] RESUMED: high-priority handler has returned
[worker] Final active_users = 1
[worker] DEMO COMPLETE
```

The important observation is that the priority-2 interrupt logs its execution before the priority-1 worker logs that it has resumed.

## Troubleshooting

### `qemu-system-arm` is not found

Install QEMU and verify that its executable is on your `PATH`.

### Rust cannot find the target

Run:

```bash
rustup target add thumbv7m-none-eabi
```

### The build fails

Check the installed Rust toolchain and dependency versions:

```bash
rustc --version
cargo --version
cargo check
```

If a dependency API has changed or the compiler reports an RTIC macro error, inspect the reported line and compare it with the RTIC version declared in `Cargo.toml`.

### No output appears

Run the application using the configured `cargo run` command rather than launching the ELF directly. Confirm that the runner contains the semihosting option:

```text
-semihosting-config enable=on,target=native
```

## Extending the project

Possible next steps include:

1. Add a periodic timer interrupt and a simulated sensor-sampling task.
2. Introduce multiple priorities and measure observed event ordering.
3. Add bounded message queues for sensor-to-control communication.
4. Model actuator commands with Rust enums and validate their ranges.
5. Add host-side unit tests for scheduling-independent application logic.
6. Port the architecture to a physical STM32 board using its PAC or HAL.
7. Add a timing-analysis document covering WCET, interrupt latency, jitter, and resource ceilings.

For a physical STM32 port, replace the LM3S6965 device crate, startup configuration, interrupt vector, and peripheral initialization with those appropriate to the chosen STM32 part. A real timer such as TIM2 can then replace the simulated GPIOA interrupt trigger.

## License

 MIT 


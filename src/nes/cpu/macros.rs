#! Macros for CPU.

#[cfg(debug_assertions)]
macro_rules! print_cpu_state {
    ($cpu: ident) => { $cpu.print_cpu_state(); }
}

#[cfg(not(debug_assertions))]
macro_rules! print_cpu_state {
    ($cpu: ident) => { }
}

#[cfg(debug_assertions)]
macro_rules! check_stack_overflow {
    ($cpu: ident) => { $cpu.check_stack_overflow(); }
}

#[cfg(not(debug_assertions))]
macro_rules! check_stack_overflow {
    ($cpu: ident) => { }
}

#[cfg(debug_assertions)]
macro_rules! check_stack_underflow {
    ($cpu: ident) => { $cpu.check_stack_underflow(); }
}

#[cfg(not(debug_assertions))]
macro_rules! check_stack_underflow {
    ($cpu: ident) => { }
}
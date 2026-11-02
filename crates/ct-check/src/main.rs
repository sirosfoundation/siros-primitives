#![doc = include_str!("../README.md")]

#[cfg(target_arch = "x86_64")]
mod check;
#[cfg(target_arch = "x86_64")]
mod operations;
#[cfg(target_arch = "x86_64")]
mod request;

use std::process::ExitCode;

#[cfg(target_arch = "x86_64")]
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [] => check::main(),
        [flag] if flag == check::OPERATIONS => operations::main(),
        _ => {
            eprintln!("usage: ct-check");
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(target_arch = "x86_64"))]
fn main() -> ExitCode {
    eprintln!("ct-check: the client requests of memcheck are x86-64 instructions");
    ExitCode::FAILURE
}

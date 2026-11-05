//! Sets `cfg(wide_arithmetic)` for a WebAssembly build whose RUSTFLAGS enable the
//! `wide-arithmetic` target feature. The feature is unstable, so rustc sets no
//! `cfg(target_feature)` for it; its `i64.mul_wide_u` replaces the split products that the other
//! WebAssembly builds take.

use std::env;

fn main() -> Result<(), env::VarError> {
    println!("cargo::rustc-check-cfg=cfg(wide_arithmetic)");
    println!("cargo::rerun-if-env-changed=CARGO_ENCODED_RUSTFLAGS");
    if env::var("CARGO_CFG_TARGET_ARCH")? == "wasm32"
        && enables_wide_arithmetic(&env::var("CARGO_ENCODED_RUSTFLAGS")?)
    {
        println!("cargo::rustc-cfg=wide_arithmetic");
    }
    Ok(())
}

/// Whether the last `wide-arithmetic` of the `-C target-feature` lists in `flags`, separated by
/// `0x1f`, enables it.
fn enables_wide_arithmetic(flags: &str) -> bool {
    let mut words = flags.split('\x1f');
    let mut enabled = false;
    while let Some(word) = words.next() {
        let list = match word {
            "-C" | "--codegen" => words
                .next()
                .and_then(|next| next.strip_prefix("target-feature=")),
            _ => word
                .strip_prefix("-Ctarget-feature=")
                .or_else(|| word.strip_prefix("--codegen=target-feature=")),
        };
        for feature in list.into_iter().flat_map(|list| list.split(',')) {
            match feature {
                "+wide-arithmetic" => enabled = true,
                "-wide-arithmetic" => enabled = false,
                _ => {}
            }
        }
    }
    enabled
}

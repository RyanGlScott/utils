//! Backends for `cmov`, only one of which will be selected at compile-time.

// Architecture-specific backends for target architectures with native predication instructions
#[cfg(false)]
mod aarch64;
#[cfg(false)]
mod x86;

// Fallback portable implementation for targets which don't have native predication instructions
// (or if they do, aren't currently supported)
#[cfg(true)]
mod soft;

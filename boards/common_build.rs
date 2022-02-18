//! Shared build.rs options and flags.
//!
//! These are common
//! [build.rs](https://doc.rust-lang.org/cargo/reference/build-scripts.html)
//! configurations that Tock boards can use in their own build.rs.

/// Standard linker arguments that boards typically pass to rustc.
pub fn tock_default_linker_args() {
    // Default name of the linker script.
    println!("cargo:rustc-link-arg=-Tlayout.ld");

    // lld by default uses a default page size to align program sections. Tock
    //  expects that program sections are set back-to-back. `-nmagic` instructs
    //  the linker to not page-align sections.
    println!("cargo:rustc-link-arg=-nmagic");

    // Identical Code Folding (ICF) set to all. This tells the linker to be more
    // aggressive about removing duplicate code. The default is `safe`, and the
    // downside to `all` is that different functions in the code can end up with
    // the same address in the binary. However, it can save a fair bit of code
    // size.
    println!("cargo:rustc-link-arg=-icf=all");
}

fn main() {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    // One kernel, two hardware ports: pick the memory layout for the target.
    let script = match std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("x86_64") => "linker_x86.ld",
        _ => "linker.ld",
    };
    println!("cargo:rustc-link-arg=-T{dir}/{script}");
    println!("cargo:rerun-if-changed=linker.ld");
    println!("cargo:rerun-if-changed=linker_x86.ld");
    for asm in [
        "src/boot/entry.S", "src/trap/trap.S", "src/task/switch.S",
        "src/x86/boot.S", "src/x86/trap.S", "src/task/switch_x86.S",
    ] {
        println!("cargo:rerun-if-changed={asm}");
    }
}

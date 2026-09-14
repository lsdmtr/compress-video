fn main() {
    println!("cargo:rerun-if-env-changed=STATIC_VCRUNTIME");
    #[cfg(feature = "desktop")]
    tauri_build::build()
}

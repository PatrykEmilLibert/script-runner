fn main() {
    // `option_env!` bakes these in at compile time but does NOT, by itself, make
    // cargo recompile when the value changes. Declaring them here ensures a build
    // with a freshly set service push token actually re-embeds it instead of
    // reusing a stale object compiled without it.
    println!("cargo:rerun-if-env-changed=SR_SCRIPTS_PUSH_TOKEN");
    println!("cargo:rerun-if-env-changed=SR_SCRIPTS_PUSH_TOKEN_B64");

    // Surface, in the build log, whether the service push token reached the
    // compiler — length only, never the value. len=0 / "NOT set" means the
    // compiled token will be empty and non-admin publishing will fail.
    match std::env::var("SR_SCRIPTS_PUSH_TOKEN_B64") {
        Ok(v) => println!(
            "cargo:warning=SR_SCRIPTS_PUSH_TOKEN_B64 present at build (len={})",
            v.trim().len()
        ),
        Err(_) => println!("cargo:warning=SR_SCRIPTS_PUSH_TOKEN_B64 NOT set at build time"),
    }

    tauri_build::build()
}

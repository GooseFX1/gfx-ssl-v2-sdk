#[cfg(feature = "jupiter_amm")]
use std::env::var;

fn main() {
    #[cfg(feature = "jupiter_amm")]
    {
        let manifest_dir = var("CARGO_MANIFEST_DIR").unwrap();

        println!("cargo:rustc-link-lib=dylib=ssl_v2_black_box");

        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        println!("cargo:rustc-link-search={}/lib/darwin/arm64", manifest_dir);

        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        println!("cargo:rustc-link-search={}/lib/darwin/x86_64", manifest_dir);

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        println!("cargo:rustc-link-search={}/lib/linux/x86_64", manifest_dir);
    }
}

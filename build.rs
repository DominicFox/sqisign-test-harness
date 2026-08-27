use std::env;
use std::path::PathBuf;

fn main() {
    // 1. Point directly to your existing compiled C directory
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let src_dir = PathBuf::from(manifest_dir).join("../Schemes/the-sqisign/build/src");

    // 2. Add the exact search path for every mathematical submodule
    println!("cargo:rustc-link-search=native={}", src_dir.display());
    println!("cargo:rustc-link-search=native={}/signature/ref/lvl1", src_dir.display());
    println!("cargo:rustc-link-search=native={}/verification/ref/lvl1", src_dir.display());
    println!("cargo:rustc-link-search=native={}/id2iso/ref/lvl1", src_dir.display());
    println!("cargo:rustc-link-search=native={}/hd/ref/lvl1", src_dir.display());
    println!("cargo:rustc-link-search=native={}/ec/ref/lvl1", src_dir.display());
    println!("cargo:rustc-link-search=native={}/gf/ref/lvl1", src_dir.display());
    println!("cargo:rustc-link-search=native={}/precomp/ref/lvl1", src_dir.display());
    println!("cargo:rustc-link-search=native={}/quaternion/ref/generic", src_dir.display());
    println!("cargo:rustc-link-search=native={}/mp/ref/generic", src_dir.display());
    println!("cargo:rustc-link-search=native={}/common/generic", src_dir.display());

    // 1. Tell Cargo to look in the Apple Silicon Homebrew directory
    println!("cargo:rustc-link-search=native=/opt/homebrew/lib");

    // 3. Link every required component in reverse dependency order
    println!("cargo:rustc-link-lib=static=sqisign_lvl1_nistapi");
    println!("cargo:rustc-link-lib=static=sqisign_lvl1");
    println!("cargo:rustc-link-lib=static=sqisign_signature_lvl1");
    println!("cargo:rustc-link-lib=static=sqisign_verification_lvl1");
    println!("cargo:rustc-link-lib=static=sqisign_id2iso_lvl1");
    println!("cargo:rustc-link-lib=static=sqisign_hd_lvl1");
    println!("cargo:rustc-link-lib=static=sqisign_ec_lvl1");
    println!("cargo:rustc-link-lib=static=sqisign_gf_lvl1");
    println!("cargo:rustc-link-lib=static=sqisign_precomp_lvl1");
    println!("cargo:rustc-link-lib=static=sqisign_quaternion_generic");
    println!("cargo:rustc-link-lib=static=sqisign_mp_generic");
    println!("cargo:rustc-link-lib=static=sqisign_common_sys");

    // 2. Link the GMP library dynamically
    println!("cargo:rustc-link-lib=dylib=gmp");
}
use std::env;
use std::path::PathBuf;

fn main() {
    //Point directly to your existing compiled C directory
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let src_dir = PathBuf::from(manifest_dir).join("variant/sqisign-v2/build/src");

    // Add the search path for every mathematical submodule
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

    // Tell Cargo to look in the Apple Silicon Homebrew directory
    println!("cargo:rustc-link-search=native=/opt/homebrew/lib");

    // Link every required component
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

    // Link the GMP library dynamically
    println!("cargo:rustc-link-lib=dylib=gmp");
}
use criterion::{criterion_group, criterion_main, Criterion};
use std::os::raw::{c_uchar, c_ulonglong, c_int};

// 1. Added the 'unsafe' keyword here
unsafe extern "C" {
    #[link_name = "sqisign_lvl1_ref_crypto_sign_keypair"]
    pub fn crypto_sign_keypair(pk: *mut u8, sk: *mut u8) -> std::os::raw::c_int;

    #[link_name = "sqisign_lvl1_ref_crypto_sign"]
    pub fn crypto_sign(
        sm: *mut u8,
        smlen: *mut usize, // Changed to usize to map correctly to size_t
        m: *const u8,
        mlen: usize,       // Changed to usize
        sk: *const u8,
    ) -> std::os::raw::c_int;

    #[link_name = "sqisign_lvl1_ref_crypto_sign_open"]
    pub fn crypto_sign_open(
        m: *mut u8,
        mlen: *mut usize,  // Changed to usize
        sm: *const u8,
        smlen: usize,      // Changed to usize
        pk: *const u8,
    ) -> std::os::raw::c_int;
}

const PK_LEN: usize = 1000; 
const SK_LEN: usize = 2000; 

fn bench_sqisign_keygen(c: &mut Criterion) {
    let mut pk = vec![0u8; PK_LEN];
    let mut sk = vec![0u8; SK_LEN];

    c.bench_function("sqisign_keygen_flamegraph", |b| {
        // 2. We keep the unsafe block here to call the specific functions
        b.iter(|| unsafe {
            crypto_sign_keypair(pk.as_mut_ptr(), sk.as_mut_ptr())
        })
    });
}

criterion_group!(benches, bench_sqisign_keygen);
criterion_main!(benches);
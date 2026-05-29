use criterion::{criterion_group, criterion_main, black_box, Criterion};
use forgottenserver_common::xtea::{Key, expand_key, encrypt, decrypt};
use forgottenserver_common::base64;
use forgottenserver_common::tools::{transform_to_sha1, hmac_sha1};

fn xtea_expand_key(c: &mut Criterion) {
    let key = Key([0x01234567u32, 0x89ABCDEFu32, 0xFEDCBA98u32, 0x76543210u32]);
    c.bench_function("xtea_expand_key", |b| {
        b.iter(|| expand_key(black_box(&key)))
    });
}

fn xtea_encrypt_64b(c: &mut Criterion) {
    let key = Key([0x01234567u32, 0x89ABCDEFu32, 0xFEDCBA98u32, 0x76543210u32]);
    let round_keys = expand_key(&key);
    let plaintext: [u8; 8] = [0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48];
    c.bench_function("xtea_encrypt_64b", |b| {
        b.iter(|| {
            let mut data = *black_box(&plaintext);
            encrypt(&mut data, black_box(&round_keys));
            data
        })
    });
}

fn xtea_encrypt_1kb(c: &mut Criterion) {
    let key = Key([0xDEADBEEFu32, 0xCAFEBABEu32, 0x12345678u32, 0x9ABCDEFu32]);
    let round_keys = expand_key(&key);
    let plaintext: Vec<u8> = (0u8..=255u8).cycle().take(1024).collect();
    c.bench_function("xtea_encrypt_1kb", |b| {
        b.iter(|| {
            let mut data = black_box(plaintext.clone());
            encrypt(&mut data, black_box(&round_keys));
            data
        })
    });
}

fn xtea_decrypt_1kb(c: &mut Criterion) {
    let key = Key([0xDEADBEEFu32, 0xCAFEBABEu32, 0x12345678u32, 0x9ABCDEFu32]);
    let round_keys = expand_key(&key);
    let plaintext: Vec<u8> = (0u8..=255u8).cycle().take(1024).collect();
    let mut ciphertext = plaintext.clone();
    encrypt(&mut ciphertext, &round_keys);
    c.bench_function("xtea_decrypt_1kb", |b| {
        b.iter(|| {
            let mut data = black_box(ciphertext.clone());
            decrypt(&mut data, black_box(&round_keys));
            data
        })
    });
}

fn base64_encode_256b(c: &mut Criterion) {
    let input: Vec<u8> = (0u8..=255u8).collect();
    c.bench_function("base64_encode_256b", |b| {
        b.iter(|| base64::encode(black_box(&input)))
    });
}

fn base64_decode_256b(c: &mut Criterion) {
    let input: Vec<u8> = (0u8..=255u8).collect();
    let encoded = base64::encode(&input);
    c.bench_function("base64_decode_256b", |b| {
        b.iter(|| base64::decode(black_box(encoded.as_str())))
    });
}

fn sha1_hash_256b(c: &mut Criterion) {
    let input: Vec<u8> = (0u8..=255u8).collect();
    c.bench_function("sha1_hash_256b", |b| {
        b.iter(|| transform_to_sha1(black_box(&input)))
    });
}

fn hmac_sha1_256b(c: &mut Criterion) {
    let key: [u8; 16] = [
        0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b,
        0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b,
    ];
    let message: Vec<u8> = (0u8..=255u8).collect();
    c.bench_function("hmac_sha1_256b", |b| {
        b.iter(|| hmac_sha1(black_box(&key), black_box(&message)))
    });
}

criterion_group!(
    benches,
    xtea_expand_key,
    xtea_encrypt_64b,
    xtea_encrypt_1kb,
    xtea_decrypt_1kb,
    base64_encode_256b,
    base64_decode_256b,
    sha1_hash_256b,
    hmac_sha1_256b,
);
criterion_main!(benches);

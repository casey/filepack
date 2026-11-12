//! Runs in a wasm runtime (node), through `wasm-bindgen-test-runner`.

#![cfg(target_arch = "wasm32")]

use {
  filepack_wasm::{Hasher, Manifest, hash_bytes, parse_public_key},
  wasm_bindgen_test::wasm_bindgen_test,
};

// Created and signed by the native CLI:
//
//   filepack keygen
//   filepack create --sign --timestamp
//
// over `hello.txt` (HELLO) and `sub/pattern.bin` (`pattern()`), then read back
// with `filepack fingerprint`, `filepack hash` and `filepack manifest --format json`.
const FINGERPRINT: &str =
  "package10ad31182bf53cd89bd07bf226e5561490517b3468f75c6937e6f0633c709c372";
const HELLO: &[u8] = b"hello filepack\n";
const HELLO_HASH: &str = "hash1310f8fa30b894bfdbbe1dbd624bb8943259c63e65047494c778b4b0aa41c1222";
const MANIFEST: &[u8] = include_bytes!("fixtures/manifest.filepack");
const MANIFEST_JSON: &str = include_str!("fixtures/manifest.json");
const PATTERN_HASH: &str = "hash11f0b5ef5bc3734ced2bb2f01127c05ea31f46be8f2fc30b4524e2a91ec462ab4";
const PUBLIC_KEY: &str = "public1fd6505c06ea5f96e361f9b535d933d791402ed2ae35534dbe7bb0cf78bff0490";

#[wasm_bindgen_test]
fn clock_reads_javascript_time() {
  let now = filepack::now().unwrap();
  let date = js_sys::Date::now() / 1000.0;
  #[expect(clippy::cast_precision_loss)]
  let difference = (now as f64 - date).abs();
  assert!(difference < 5.0, "{now} vs {date}");
}

#[wasm_bindgen_test]
fn hash_bytes_matches_native() {
  assert_eq!(hash_bytes(HELLO), HELLO_HASH);
  assert_eq!(hash_bytes(&pattern()), PATTERN_HASH);
}

#[wasm_bindgen_test]
fn hasher_streams_chunks() {
  let pattern = pattern();
  let mut hasher = Hasher::new();
  for chunk in pattern.chunks(4096 + 17) {
    hasher.update(chunk);
  }
  assert_eq!(hasher.finalize(), PATTERN_HASH);
  assert_eq!(hasher.size(), 300_000);
}

#[wasm_bindgen_test]
fn manifest_round_trips() {
  let manifest = Manifest::from_bytes(MANIFEST).unwrap();
  assert_eq!(manifest.fingerprint().unwrap(), FINGERPRINT);
  assert_eq!(manifest.to_json().unwrap(), MANIFEST_JSON.trim_end());
  assert_eq!(manifest.to_bytes().unwrap(), MANIFEST);
}

#[wasm_bindgen_test]
fn manifest_signatures_verify() {
  let manifest = Manifest::from_bytes(MANIFEST).unwrap();
  assert_eq!(manifest.verify_signatures().unwrap(), [PUBLIC_KEY]);
}

fn pattern() -> Vec<u8> {
  (0..300_000u32)
    .map(|i| u8::try_from((i * 31 + 7) % 251).unwrap())
    .collect()
}

#[wasm_bindgen_test]
fn public_key_parses() {
  assert_eq!(parse_public_key(PUBLIC_KEY).unwrap(), PUBLIC_KEY);
}

//! WebAssembly bindings for filepack's byte-level core: hashing, manifests,
//! fingerprints and signature verification. Nothing here touches the
//! filesystem; callers hand over bytes and get back strings and bytes.

use {
  filepack::{ContentHasher, DecodeOptions, Hash, PublicKey},
  wasm_bindgen::prelude::*,
};

/// Incremental hasher, for content that arrives in chunks.
#[wasm_bindgen]
#[derive(Default)]
pub struct Hasher {
  inner: ContentHasher,
}

#[wasm_bindgen]
impl Hasher {
  /// Hash of the content so far, as `hash1…`.
  pub fn finalize(&self) -> String {
    self.inner.finalize().to_string()
  }

  #[wasm_bindgen(constructor)]
  pub fn new() -> Self {
    Self::default()
  }

  /// Bytes hashed so far.
  #[wasm_bindgen(getter)]
  pub fn size(&self) -> u64 {
    self.inner.size()
  }

  pub fn update(&mut self, bytes: &[u8]) {
    self.inner.update(bytes);
  }
}

/// A manifest decoded from the bytes of a `manifest.filepack` file.
#[wasm_bindgen]
pub struct Manifest {
  inner: filepack::Manifest,
}

#[wasm_bindgen]
impl Manifest {
  /// Package fingerprint, as `package1…`.
  pub fn fingerprint(&self) -> Result<String, JsError> {
    Ok(self.inner.fingerprint()?.to_string())
  }

  /// Decode a manifest from the bytes of a `manifest.filepack` file.
  #[wasm_bindgen(js_name = fromBytes)]
  pub fn from_bytes(bytes: &[u8]) -> Result<Manifest, JsError> {
    Ok(Self {
      inner: filepack::Manifest::from_bytes(bytes, DecodeOptions::default())?,
    })
  }

  /// Encode the manifest back into `manifest.filepack` bytes.
  #[wasm_bindgen(js_name = toBytes)]
  pub fn to_bytes(&self) -> Result<Vec<u8>, JsError> {
    Ok(self.inner.to_bytes()?)
  }

  /// The manifest as JSON, as `filepack manifest --format json` prints it.
  #[wasm_bindgen(js_name = toJson)]
  pub fn to_json(&self) -> Result<String, JsError> {
    Ok(serde_json::to_string(&self.inner)?)
  }

  /// Verify every signature against the manifest's fingerprint and return
  /// the public keys that signed it, as `public1…`.
  #[wasm_bindgen(js_name = verifySignatures)]
  pub fn verify_signatures(&self) -> Result<Vec<String>, JsError> {
    Ok(
      self
        .inner
        .verify_signatures(self.inner.fingerprint()?)?
        .iter()
        .map(ToString::to_string)
        .collect(),
    )
  }
}

/// Hash of `bytes`, as `hash1…`.
#[wasm_bindgen(js_name = hashBytes)]
pub fn hash_bytes(bytes: &[u8]) -> String {
  Hash::bytes(bytes).to_string()
}

/// Parse a public key and return it in canonical form, as `public1…`.
#[wasm_bindgen(js_name = parsePublicKey)]
pub fn parse_public_key(key: &str) -> Result<String, JsError> {
  Ok(key.parse::<PublicKey>()?.to_string())
}

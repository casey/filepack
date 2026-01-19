Filepack Design
===============

Open Questions
--------------

- *Should filepack re-use an existing signature system, like SSH or PGP?* I
  looked into both, but the formats and algorithms are incredibly complex, and
  allow a huge number of unnecessary degrees of freedom.

- *Should filepack use a derived key when signing messages?* I would like to,
  since deriving a key with an explicit context seems like good practice, but
  `ed25519_dalek` doesn't support it.

Closed Questions
----------------

- *Should Bech32m be used for fingerprints, public keys, private keys, and
  signatures?* This would make them distinct and easy to identify, and give
  private keys names like `private1…` which suggests that they shouldn't be
  exposed. **Conclusion: Use bech32m everything. Having easy-to-distinguish
  keys, fingerprints, and signatures is a huge benefit. BLAKE3 file hashes are
  standard hexadecimal.**

- *Should bech32m-encoded strings add a version character?* Similar to segwit
  addresses, we could add a version character to fingerprints, public keys,
  private keys, and signatures. This would allow us to print a better error
  message if for some reason these formats change in the future. It seems
  unlikely public keys, private keys, and fingerprints would change, but it
  does seem conceivable that it might be desirable to add additional data to
  signatures, for example, a per-signature timestamp. **Conclusion: Add the
  version character `q`, and complain if a bech32m string starts with a
  different character.**

- *Should signatures be included in the manifest or in a subdirectory?*
  Currently, signatures are stored in the manifest in an object under the
  `signatures` key. This has pros and cons. The major pro is one only needs to
  download a single file in order to verify the contents of a directory and
  signatures. The major con is that adding a signature requires modifying the
  manifest, and care must be taken to avoid conflicts if multiple people add
  signatures concurrently. **Conclusion: keeping signatures with the manifest
  has safety and usability benefits, and merging multiple sources of signatures
  into a manifest can be made easier with tooling. Also, the manifest is now
  pretty-printed, to make merge conflicts easier to deal with.**

- *Should the manifest use a binary serialization format?* The main advantage
  would be being able to easily include large amounts of binary data in the
  manifest. For example, parity information, or intermediate file hash tiers.
  **Conclusion: Keep JSON, since it is so much easier for humans to deal with
  and so widely supported. If we need to include large amounts of binary data,
  we can keep them in separate files.**

- *Should the signature algorithm use BLAKE3 instead of the EdDSA default of
  SHA-512?* This would allow us to avoid double-hashing and remove a dependency
  on SHA-512, but would make our signatures nonstandard, which is crazy.
  **Conclusion: using non-standard ed25519 signatures for such limited benefit
  is indeed crazy.**

- *Should fingerprint hashes be calculated over CBOR, instead of TLV fields?*
  Currently, fingerprints are created by encoding data as a sequence of TLV
  fields. This is extremely simple. However, we could also encode data as CBOR,
  which is much more complicated, but has the advantage of being standardized,
  and enforces domain separation between types. **Conclusion: Stick with the
  TLV-encoding. The complexity of CBOR canonicalization offsets any benefit of
  standardization.**

- *Should I worry about quantum computers?* I'm leaning towards no, since
  filepack can likely be reactive instead of proactive on this front.
  **Conclusion: If there was an obvious choice of post-quantum signature
  scheme, that would be one thing, but right now it's such a moving target that
  it's almost certainly better to wait.**

Manifest Format
---------------

A filepack manifest contains all information needed to verify the contents of a
directory. The `files` key of the manifest is a directory object mapping
filenames to directory entries, which may themselves be directories, or files,
in which case they contain the hash of the file contents, as well as the length
of the file.

The length of the file is not strictly necessary for verification, but is
included so that truncation or extension can be explicitly identified, which
may help in understanding verification failures.

File Hashes
-----------

The contents of files are hashed with
[BLAKE3](https://github.com/BLAKE3-team/BLAKE3), using the official Rust
implementation. BLAKE3 was chosen both for its speed, and for the fact that it
utilizes a Merkle tree construction. A Merkle tree allows for verified file
streaming and subrange inclusion proofs, which both seem useful in the context
of file hashing and verification.

Signatures
----------

Filepack allows for the creation of Ed25519 signatures over the contents of a
manifest, which thus commit to the contents of the directory covered by the
manifest. Signatures are made not over serialized manifest, but over a message
containing a "fingerprint" hash, a Merkle tree hash created from the contents
of the manifest. This keeps signatures independent of the manifest format,
avoids issues with canonicalization of the manifest JSON, avoids hash loops due
to the inclusion of signatures in the manifest itself, and allows proving the
inclusion of files covered by a signature using a Merkle receipt.

Fingerprints
------------

Although only manifest fingerprints are exposed externally, several types of
fingerprints are used internally, namely directory, entry, file, and message
fingerprints.

Fingerprints are constructed to be unique, both between and within types,
meaning that it is impossible two different values with different types or
contents but which have the same fingerprint.

Fingerprints are BLAKE3 hashes. To guarantee that fingerprints are unique
between types, the hasher is first initialized with a length-prefixed string
unique to each type.

After the prefix, the value is hashed as a sequence of TLV fields.

Fields are hashed in order, but may be skipped, in the case of optional fields,
or repeated, in the case of fields containing multiple values.

Currently, no fingerprint test vectors exist, and the best documentation is the
code itself.

In particular, see:

- [FingerprintHasher](src/fingerprint_hasher.rs)
- [FingerprintPrefix](src/fingerprint_prefix.rs)
- [Manifest](src/manifest.rs)
- [Directory](src/directory.rs)
- [Entry](src/entry.rs)
- [Files](src/file.rs)
- [Message](src/message.rs)

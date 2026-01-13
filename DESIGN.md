Design
======

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

Filepack allows for the creation of signatures over the contents of a manifest,
which thus commit to the contents of the directory covered by the manifest.
Signatures are made not over the hash of the literal JSON bytes of the
manifest, but over a fingerprint hash, a Merkle tree hash created from the
contents of the manifest. This keeps signatures independent of the manifest
format, avoids issues with canonicalization of the manifest JSON, avoids hash
loops due to the inclusion of signatures in the manifest itself, and allows
proving the inclusion of files covered by a signature using a Merkle receipt.

Fingerprints
------------

Although only manifest fingerprints are exposed externally, several types of
fingerprints are used internally.

Fingerprints are constructed to be unique, both between and within types,
meaning that it is impossible two different values with different types or
contents, but which have the same fingerprint.

Fingerprints are BLAKE3 hashes. To guarantee that fingerprints are unique
between types, the hasher is first initialized with a prefix string unique to
each type.

After the prefix, the value is hashed as a sequence of TLV fields.

Fields are hashed in order, but may be skipped, in the case of optional fields,
or be repeated, in the case of fields containing multiple values.

Currently, no fingerprint test vectors exist, and the best documentation is
the code itself.

In particular, see:

- [src/fingerprint_hasher.rs]
- [src/fingerprint_prefix.rs]
- [src/manifest.rs]
- [src/directory.rs]
- [src/entry.rs]
- [src/file.rs]

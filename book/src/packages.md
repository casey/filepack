Packages
========

Manifest Format
---------------

A filepack manifest contains all information needed to verify the contents of a
directory. The `package` key of the manifest is a directory object mapping
filenames to directory entries, which may themselves be directories, or files,
in which case they contain the hash of the file contents, as well as the length
of the file.

The length of the file is not strictly necessary for verification, but is
included so that truncated, empty, and overlong files can be identified, which
may help in understanding verification failures.

File contents may be embedded in the `manifest.filepack` archive under the
`embedded` key.

File Hashes
-----------

The contents of files are hashed with
[BLAKE3](https://github.com/BLAKE3-team/BLAKE3) using the official Rust
implementation. BLAKE3 was chosen both for its speed, and for the fact that it
utilizes a Merkle tree construction. A Merkle tree allows for verified file
streaming and subrange inclusion proofs, which both seem useful in the context
of file hashing and verification.

Signatures
----------

Filepack allows for the creation of Ed25519 signatures over the contents of a
manifest, which thus commit to the contents of the directory covered by the
manifest. Signatures are made over a statement containing a "fingerprint" hash
of a canonical deco serialization of the manifest's `package` directory. This
keeps signatures independent of the manifest format, avoids the need for a
canonical serialization of the manifest as a whole, avoids hash loops due to
the inclusion of signatures in the manifest itself, and allows proving the
inclusion of files covered by a signature.

Fingerprints
------------

Package fingerprints are the BLAKE3 hash of a canonical deco serialization of
the manifest's `package` directory. Fingerprints are constructed to be unique,
meaning that it is impossible for two different packages with different
contents to have the same fingerprint.

Currently, no fingerprint test vectors exist, and the best documentation is the
code itself.

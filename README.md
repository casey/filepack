filepack
========

`filepack` is a command-line utility for verifying the integrity of collections
of files.

It is currently unstable: the interface and file format may change at any time.
Additionally, the code has not been extensively reviewed and should be
considered experimental.

Files are hashed using [BLAKE3](https://github.com/BLAKE3-team/BLAKE3/), a
fast, cryptographic hash function, and hash digests and file paths are stored
in a manifest file named `filepack.json`.

Files can later be verified against the hashes saved in the manifest to protect
against accidental or malicious corruption, as long as the manifest has not
been tampered with.

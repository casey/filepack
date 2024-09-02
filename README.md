filepack
========

`filepack` is a command line utility for verifying the integrity of collections
of files.

It is currently unstable: the command-line interface and file format may change
at any time. Additionally, the code has not been extensively reviewed, and
should be considered experimental.

Files are hashed using [BLAKE3](https://github.com/BLAKE3-team/BLAKE3/), a
fast, cryptographic hash function, and a map of hashes to file paths are stored
in a file named `filepack.json`.

These hashes can later be verified, to protect against accidental or malicious
corruption, as long as `filepack.json` has not been tampered with.

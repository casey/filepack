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

Installation
------------

`filepack` is written in [Rust](https://www.rust-lang.org/) and can be built
from source and installed from this repo with `cargo install --path .`, or from
[crates.io](https://crates.io/crates/filepack) with `cargo install filepack`.

See [rustup.rs](https://rustup.rs/) for installation instructions for Rust.

### Pre-Built Binaries

Pre-built binaries for Linux, MacOS, and Windows can be found on
[the releases page](https://github.com/casey/filepack/releases).

You can use the following command on Linux, MacOS, or Windows to download the
latest release, just replace `DEST` with the directory where you'd like to put
`filepack`:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://filepack.com/install.sh | bash -s -- --to DEST
```

For example, to install `filepack` to `~/bin`:

```sh
# create ~/bin
mkdir -p ~/bin

# download and extract filepack to ~/bin/filepack
curl --proto '=https' --tlsv1.2 -sSf https://filepack.com/install.sh | bash -s -- --to ~/bin

# add `~/bin` to the paths that your shell searches for executables
# this line should be added to your shell's initialization file,
# e.g. `~/.bashrc` or `~/.zshrc`
export PATH="$PATH:$HOME/bin"

# filepack should now be executable
filepack --help
```

Note that `install.sh` may fail on GitHub Actions or in other environments
where many machines share IP addresses. `install.sh` calls GitHub APIs in order
to determine the latest version of `filepack` to install, and those API calls
are rate-limited on a per-IP basis. To make `install.sh` more reliable in such
circumstances, pass a specific tag to install with `--tag`.

Alternatives and Prior Art
--------------------------

`filepack` serves the same purpose as programs like `shasum`, which hash files
and output a text file containing file hashes and paths, which can later be
used with the same program to verify that the files have not changed.

They output hashes and paths one per line, separated by whitespace, and mainly
differ in which hash function they use.

Some examples, with links to implementations and the hash functions they use:

| Binary | Hash Function |
|---|---|
| [`cksfv`](https://zakalwe.fi/~shd/foss/cksfv/) | [CRC-32](https://en.wikipedia.org/wiki/Cyclic_redundancy_check) |
| [`shasum`](https://metacpan.org/pod/Digest::SHA) | [SHA-1](https://en.wikipedia.org/wiki/SHA-1) and [SHA-2](https://en.wikipedia.org/wiki/SHA-2) |
| [`sha3sum`](https://codeberg.org/maandree/sha3sum) | [SHA-3](https://en.wikipedia.org/wiki/SHA-3) |
| [`b2sum`](https://github.com/BLAKE2/BLAKE2) | [BLAKE2](https://en.wikipedia.org/wiki/BLAKE_(hash_function)#BLAKE2) |
| [`b3sum`](https://github.com/BLAKE3-team/BLAKE3) | [BLAKE3](https://en.wikipedia.org/wiki/BLAKE_(hash_function)#BLAKE3) |

CRC-32 is not a cryptographic hash function and cannot be used to detect
intentional modifications. Similarly, SHA-1 was thought to be a cryptographic
hash function, but is now known to be insecure.

`filepack` and `b3sum` both use BLAKE3, a fast, general-purpose cryptographic
hash function.

New Releases
------------

New releases of `filepack` are made frequently so that users quickly get access to
new features.

Release commit messages use the following template:

```
Release x.y.z

- Bump version: x.y.z → x.y.z
- Update changelog
- Update changelog contributor credits
- Update dependencies
```

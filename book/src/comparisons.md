Comparisons
===========

Hashing
-------

`filepack` serves the same purpose as programs like `shasum`, which hash files
and output a text file containing file hashes and paths, which can later be
used with the same program to verify that the files have not changed.

They output hashes and paths one per line, separated by whitespace, and mainly
differ in which hash function they use.

Some examples, with links to implementations and the hash functions they use:

| binary | hash function |
|---|---|
| [`b2sum`](https://github.com/BLAKE2/BLAKE2) | [BLAKE2](https://en.wikipedia.org/wiki/BLAKE_(hash_function)#BLAKE2) |
| [`b3sum`](https://github.com/BLAKE3-team/BLAKE3) | [BLAKE3](https://en.wikipedia.org/wiki/BLAKE_(hash_function)#BLAKE3) |
| [`cksfv`](https://zakalwe.fi/~shd/foss/cksfv/) | [CRC-32](https://en.wikipedia.org/wiki/Cyclic_redundancy_check) |
| [`hashdeep`](https://github.com/jessek/hashdeep) | various |
| [`hashdir`](https://github.com/ultimateanu/hashdir/) | various |
| [`sha3sum`](https://codeberg.org/maandree/sha3sum) | [SHA-3](https://en.wikipedia.org/wiki/SHA-3) |
| [`shasum`](https://metacpan.org/pod/Digest::SHA) | [SHA-1](https://en.wikipedia.org/wiki/SHA-1) and [SHA-2](https://en.wikipedia.org/wiki/SHA-2) |

CRC-32 is not a cryptographic hash function and cannot be used to detect
intentional modifications. Similarly, SHA-1 was thought to be a cryptographic
hash function, but is now known to be insecure.

`filepack` and `b3sum` both use BLAKE3, a fast, general-purpose cryptographic
hash function.

Signing
-------

`filepack` can also create and verify signatures. Other signing and
verification utilities include:

| binary | about |
|---|---|
| [`gpg`](https://gnupg.org/) | general-purpose, [OpenPGP](https://www.openpgp.org/) implementation |
| [`ssh-keygen`](https://man.openbsd.org/ssh-keygen.1) | general-purpose, shipped with [OpenSSH](https://www.openssh.com/) |
| [`minisign`](https://github.com/jedisct1/minisign) | general-purpose |
| [`signify`](https://github.com/aperezdc/signify) | general-purpose |
| [`SignTool`](https://learn.microsoft.com/en-us/windows/win32/seccrypto/signtool) | Windows code signing |
| [`codesign`](https://developer.apple.com/library/archive/documentation/Security/Conceptual/CodeSigningGuide/Procedures/Procedures.html) | macOS code signing |
| [`jarsigner`](https://docs.oracle.com/javase/8/docs/technotes/tools/windows/jarsigner.html) | JDK code signing |

Content-addressed Distribution
------------------------------

`filepack` packages are Merkle trees over their contents, which allows them to
be used for content-addressed storage and distribution.

### BitTorrent

Filepack packages are similar to
[BitTorrent](https://en.wikipedia.org/wiki/BitTorrent) torrents.

[Version 1 torrents](https://www.bittorrent.org/beps/bep_0003.html) use the
[SHA-1](https://en.wikipedia.org/wiki/SHA-1) hash function, which is vulnerable
to collisions. Files do not have consistent hashes between torrents.

[Version 2 torrents](https://www.bittorrent.org/beps/bep_0052.html) use the
[SHA-256](https://en.wikipedia.org/wiki/SHA-2) hash function with a Merkle tree
construction that does give files consistent hashes between torrents.

Version 1 torrents are currently far more widely adopted than version 2
torrents.

Torrents are not composable. A torrent may contain `.torrent` files, but may
not natively contain other torrents.

Compared to BitTorrent, Filepack uses BLAKE3, a fast, modern cryptographic hash
function with built-in Merkle tree construction. Files are consistently hashed
regardless of the package they appear in. Packages are composable. Package
fingerprints identify the root directory of the package, which may itself
contain packages in subdirectories.

Filepack uses a more consistent content-addressing scheme. All objects,
including files and directories, are identified by BLAKE3 hash, which is itself
a Merkle tree, allowing any object to be incrementally streamed and verified.
By contrast, a BitTorrent infohash is not a tree hash, so the info dictionary
object it identifies cannot be downloaded using the same piece bartering
algorithm as the files in the torrent.

Filepack packages are directories of files, so packages can be transported
using BitTorrent. Additionally, Filepack metadata identifies assets by relative
path, so Filepack metadata can be used in torrents independently of Filepack
hashing or manifests.

### IPFS

Filepack and IPFS use a similar data model, where directories and files are
both identified by hash.

IPFS is notoriously complex, supports a wide variety of hash functions,
public-key cryptography algorithms, and peer to peer transport and addressing
schemes.

Compared to IPFS, Filepack is far simpler. Filepack uses a single hash
function, BLAKE3, and a single public-key cryptography implementation, Ed25519.
Additionally, Filepack can be used for hashing, signing, and verification
without depending on the network.

### Iroh

Iroh is a peer-to-peer networking stack that uses BLAKE3 for content
addressing.

Filepack and Iroh are largely orthogonal. Iroh is used for transport of flat
blobs, and does not define a directory encoding.

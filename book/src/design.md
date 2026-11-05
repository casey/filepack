Design
======

Cryptography
------------

### BLAKE3

Filepack uses the [BLAKE3](https://github.com/BLAKE3-team/BLAKE3) cryptographic
hash function for hashing files, directories, and signed messages.

BLAKE3 was chosen because it is fast, widely used, believed to be secure, and
has an internal tree structure.

Many Filepack operations involve hashing potentially large files. The content of
packages is hashed when packages are created, uploaded, downloaded, and
verified. All these operations benefit from a fast hash function, and BLAKE3 is
up to 14 times faster than SHA-256 when SHA-256 hardware acceleration is not
available, at least as fast when it is, and benefits from multiple cores when
hashing large files..

Filepack requires a secure, collision-resistant cryptographic hash function.
Using a cryptographic hash function allows Filepack to detect both accidental
corruption and malicious tampering, whereas a faster, non-cryptographic hash
function would only reliably detect accidental corruption. BLAKE3 is descended
from BLAKE, which received extensive scrutiny as a finalist in the
[NIST Cryptographic Hash Algorithm Competition](https://www.nist.gov/programs-projects/cryptographic-hash-algorithm-competition),
and is used by a variety of projects, including
[Bazel](https://github.com/bazelbuild/bazel), [ccache](https://ccache.dev/),
[LLVM](https://llvm.org/), [OpenZFS](https://openzfs.org/), and
[Tekken 8](https://x.com/rodarmor/status/1751567502050771189). This wide use and
cryptanalysis increases confidence in the security properties of BLAKE3.

Many Filepack applications could benefit from incremental and out-of-order
streaming and verification, enabled by BLAKE3's internal Merkle tree structure.
This enables, for example, BitTorrent-style piece barter over Filepack packages.

### Ed25519

Filepack uses [Ed25519](https://ed25519.cr.yp.to/), an
[EdDSA](https://en.wikipedia.org/wiki/EdDSA) signature scheme using
[SHA-512](https://en.wikipedia.org/wiki/SHA-2) and edwards25519, an elliptic
curve related to [Curve25519](https://en.wikipedia.org/wiki/Curve25519).

Ed25519 was chosen because it is widely used and studied, and has small private
keys, public keys, and signatures.

Public keys and signatures must be canonical, which protects them from
third-party malleability. Public keys must be large-order, so weak keys for
which anyone can create a valid signature are rejected. Public keys must also be
torsion-free, which allows implementations using the cofactored or cofactorless
verification equation to agree on signature validity.

### Algorithm Agility

Filepack's hash function and signature scheme are fixed.

Flexibility in the choice of cryptography, where multiple hash functions and
signature schemes are supported and may be mixed or negotiated, is a source of
bugs and complexity.

Additionally, hash functions and signature schemes are rarely perfect
substitutes. They have different properties, and characteristics of the system
as a whole are determined by the properties of the parts.

Although there are no specific provisions for algorithm agility, Filepack
provides a multitude of schema evolution mechanisms for introducing both
backwards-compatible and backwards-incompatible changes. Backwards-compatible
changes are ignored by old versions of Filepack, and backwards-incompatible
changes fail with an informative error message. These mechanisms could be used
to introduce a new hash function or signature scheme.

Ideally, new cryptography will never be necessary, but the possibility cannot be
ruled out.

### Public Key Infrastructure

Filepack does not include public key infrastructure, a web of trust, key
revocation, or key expiry.

While appealing, these features would be speculative additions with unclear
adoption and utility. Current key management, signing, and signature
verification functionality is intentionally minimal. Future features using
public key cryptography are contemplated, such as signed package feeds and
updates.

### Signatures

Filepack signatures are detached to allow multiple, after-the-fact signatures,
by both first and third parties.

The primary alternative would be embedded signatures, where a package
identifier, either a fingerprint or revision, commits to signatures. Detached
signatures allow packages to be signed after they are created, without changing
the package fingerprint or revision hash. Embedded signatures would have the
benefit of being inseparable from packages, whereas detached signatures must be
transported separately, and the Filepack server does not yet support upload and
download of signatures.

Messages are prehashed with BLAKE3 before signing with Ed25519. This makes the
signing interface a consistent 32-byte digest, and introduces no new security
security assumptions, since Filepack as a whole already relies on the security
of BLAKE3.

The signed message is a [deco-encoded](deco.md)
[Envelope](deco/envelope-statement.md), which serializes to an unambiguous
encoding that commits to the application, within-application context, and
message. The application, in this case the string `"filepack"`, ensures
cross-protocol domain separation, and the context, the integer `0` for package
signatures and the integer `1` for bearer token signatures, ensures domain
separation within Filepack.

Metadata
--------

Metadata is perhaps the central feature of Filepack. Although Filepack strives
to be useful as a general-purpose hashing, signing, validation tool, it accepts
compromises, such as treating `metadata.filemeta` files specially, that would
not be appropriate in a fully general tool.

Metadata is designed to be usable without relying of other Filepack features,
and suitable for use with any storage or transport. Adoption of Filepack
metadata and not the package format, feature, or tool, would be a successful
outcome of the project.
The value of metadata

Metadata is intended to facilitate features which require it to be uniform and
standardized, including:

- Structured search, such as by title, year, audio codec, and video bitrate.

- Automatic selection appropriate media, including by format, codec, or bitrate.

- Attractive display, including artwork, video placeholder images, image
  previews, and image thumbnails.

- Media playback and streaming.

All of these features require a standard schema with a fixed encoding and
semantics.

The encoding of metadata was designed with these requirements in mind, and is
schema driven and is not user-extensible.

Serialization
-------------

Filepack uses a custom serialization format, [deco](deco.md), wherever data is
serialized as binary, including the payload of tagged hex strings.

Deco has a number of desirable properties. It is simple, encoded values are
compact, it is fast to encoded and decode, unknown values can be skipped or
extracted without being parsed, and, where required, the encoding is canonical.

Filepack implementations must agree on the hashes of serialized data such as
directories, signed messages, and revisions. Deco provides canonical encodings
and a strict decoding mode that rejects non-canonical encodings. Additionally,
deco's only wire type is the length-delimited byte string, used by both scalers
and containers, so unknown values, such as struct fields, can be preserved
through decode-encode round trips without modificiation.

Deco has a comprehensive schema-evolution model. Invalid encodings are either
malformed or unknown. Malformed values are illegal and produce an error wherever
encountered. Unknown values also produce an error during strict decoding, but
are either ignored or preserved during non-strict decoding. Unknown values
include unknown struct fields, optional struct fields containing unknown enum
variants, and unsupported version fields.

Deco comes with downsides. It is schema-driven, binary, and used only by
Filepack. Encoded values cannot be decoded without knowing their type and are
not human-readable or -writable, and it lacks a formal specificiation.

Deco was chosen after evaluating other popular formats, including
[JSON](https://en.wikipedia.org/wiki/JSON) and [CBOR](https://cbor.io/), both of
which were used by earlier versions of Filepack.

Server
------

Packages are uploaded to the server bottom-up. Before a directory can be
verified, the files and subdirectories it contains must be uploaded, and those
subdirectories must themselves be verified. Before a package can be verified,
the root directory of the package must be verified.

Alternatively, packages could be uploaded top down. Bottom-up uploading was
chosen so that verified directories and packages are always complete, and do not
exist in a pending state.

The server uses garbage collection instead of reference counting. Garbage
collection was chosen mostly on the basis of simplicity. It may be desirable, in
the future, to use reference counting as a replacement or supplement to garbage
collecting. Especially if garbage collection sweeps become expensive on servers
with a large number of packages.

Peer-to-peer Networking
-----------------------

Filepack is designed with the distribution of packages over peer-to-peer
networks in mind. Packages are Merkle trees, and files are hashed with BLAKE3,
itself a Merkle tree hash. Starting with a trusted package fingerprint or
manifest, a peer can download a package from any source, including from
untrusted peers, using a piece-barter algorithm.

A native peer-to-peer network, however, would be a large undertaking, so the
initial focus is on the web and Filepack server.

Compared to BitTorrent, where the infohash is the hash of the complete torrent
file tree, a Filepack package fingerprint is only the hash of the package root
directory. This has the benefit of making packages composable. A package may
contain other packages, whereas a Torrent cannot natively contain other
torrents. However, package retrieval thus requires downloading each directory
individually, and the definition of a separate single-file storage format for
manifests.

An analysis of [MagnetDB](https://magnetdb.secretlab.page/), a large dataset of
torrents, shows that Filepack would need only 1.63 additional requests per
torrent to download subdirectories.

Packages
--------

Filepack records hashes both files and directories, so empty directories may be
part of a package.

Filenames are restricted to make paths safe from confusion across platforms.
Regardless of platform, they may not contain forwards slashes, backslashes, or
begin with a Windows drive prefix, in addition to other restrictions. This is in
tension with the desire for Filepack to be a useful, general, file hashing and
verification tool, since there are legitimate filenames that Filepack will
reject, but it was decided to prioritize safety and platform independence.

Directory entries contain the hash of a file or a directory, and files and
directories are hashed the same way, without domain separation. This keeps file
and directory hashes the plain BLAKE3 hash of their content, and is unambiguous,
since the entry identifies whether it is a file or directory.

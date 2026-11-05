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

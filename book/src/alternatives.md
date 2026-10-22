Alternatives and Prior Art
==========================

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

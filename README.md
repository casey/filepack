<div align=right>Table of Contents↗️</div>

<h1 align=center><code>filepack</code></h1>

<div align=center>
  <a href=https://crates.io/crates/filepack>
    <img src=https://img.shields.io/crates/v/filepack.svg alt="crates.io version">
  </a>
  <a href=https://github.com/casey/filepack/actions>
    <img src=https://github.com/casey/filepack/actions/workflows/ci.yaml/badge.svg alt="build status">
  </a>
  <a href=https://github.com/casey/filepack/releases>
    <img src=https://img.shields.io/github/downloads/casey/filepack/total.svg alt=downloads>
  </a>
</div>

<br>

`filepack` is a command-line file hashing and verification utility written in
Rust.

It is an alternative to `.sfv` files and tools like `shasum`. Files are hashed
using [BLAKE3](https://github.com/BLAKE3-team/BLAKE3/), a fast, cryptographic
hash function.

A manifest named `filepack.json` containing the hashes of files in a directory
can be created with:

```shell
filepack create path/to/directory
```

Which will write the manifest to `path/to/directory/filepack.json`.

Files can later be verified with:

```shell
filepack verify path/to/directory
```

To protect against accidental or malicious corruption, as long as the manifest
has not been tampered with.

If you run `filepack` a lot, you might want to `alias fp=filepack`.

`filepack` is currently unstable: the interface and file format may change at
any time. Additionally, the code has not been extensively reviewed and should
be considered experimental.

Installation
------------

`filepack` is written in [Rust](https://www.rust-lang.org/) and can be built
from source and installed from a checked-out copy of this repo with:

```sh
cargo install --path .
```

Or from [crates.io](https://crates.io/crates/filepack) with:

```sh
cargo install filepack
```

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

Usage
-----

Filepack supports a number of subcommands, including `filepack create` to
create a manifest, and `filepack verify` to verify a manifest.

See `filepack help` for supported subcommands and `filepack help SUBCOMMAND`
for information about a particular subcommand.

### `filepack create`

Create a manifest.

Recommended lints can be enabled with:

```shell
filepack create --deny distribution
```

### `filepack verify`

Verify the contents of a directory against a manifest.

To verify the contents of `DIR` against `DIR/filepack.json`:

```shell
filepack verify DIR
```

If the current directory contains `filepack.json`, `DIR` can be omitted:

```shell
filepack verify
```

`filepack verify` takes an optional `--print` flag, which prints the manifest
to standard output if verification succeeds. This can be used in a pipeline to
ensure that you the manifest has been verified before proceeding:

```shell
filepack verify --print | jq
```

Data Directory
--------------

Filepack stores local configuration in the `filepack` subdirectory of the
platform data directory.

The location of the platform data directory is platform dependent, with the
`$XDG_DATA_DIR` being respected on all platforms.

| Platform | Value                               |
| -------- | ----------------------------------- |
| All      | `$XDG_DATA_DIR`                     |
| Linux    | `$HOME`/.local/share                |
| macOS    | `$HOME`/Library/Application Support |
| Windows  | `{FOLDERID_LocalAppData}`           |

Filepack stores keys in the `keychain` subdirectory of the filepack data
directory. So if the platform data directory is `~/.local/share`, the
`keychain` directory is `~/.local/share/filepack/keychain`.

The location of the filepack data directory used by a command can be overridden
with the `--data-dir` option.

Manifest
--------

`filepack` manifests are conventionally named `filepack.json` and are placed
alongside the files they reference.

Manifests are [UTF-8](https://en.wikipedia.org/wiki/UTF-8)-encoded
[JSON](https://www.json.org/json-en.html).

Manifests contain an object with two mandatory keys, `files` and `notes`.

### `files`

The value of the mandatory `files` key is an object mapping path components to
directory entries. Directory entries may be subdirectories or files. Files are
objects with keys `hash`, the hex-encoded BLAKE3 hash of the file, and `size`,
the length of the file in bytes.

As a consequence of the manifest being UTF-8, all path components must be
valid Unicode.

Path components may not be `.` or `..`, contain the path separators `/` or `\`,
contain NUL, be longer than 255 bytes, or begin with a Windows drive prefix,
such as `C:`.

### `signatures`

The value of the mandatory `signatures` key is an array of signatures.
Signatures are bech32 strings that include an Ed25519 key, the package
fingerprint the signature is made over, an optional timestamp, and the
signature itself.

Public keys are Curve25519 points and signatures are Ed25519 signatures made
over the root of a Merkle tree which commits to the content of `files` via the
package fingerprint.

### Example

An manifest over a directory containing the files `README.md` and `src/main.c`,
signed by the public key
`public1a67dndhhmae7p6fsfnj0z37zf78cde6mwqgtms0y87h8ldlvvflyqcxnd63`:

```json
{
  "files": {
    "README.md": {
      "hash": "fc253b84551ce6b00e820a826ac18054dc7f63a318ce62f3175315f5c467a62a",
      "size": 11883
    },
    "src": {
      "main.rs": {
        "hash": "1fa48b95ed335369d45b91af8138bdccd1413364bcdbfa6e9034e8a2cfd6e17f",
        "size": 33
      }
    }
  },
  "signatures": ["…"]
}
```

The signature is elided for brevity. Signatures are bech32m-encoded strings
containing both a public key and an Ed25519 signature.

Keys, Signatures, Fingerprints, and Hashes
------------------------------------------

Public keys, private keys, signatures, and package fingerprints are all
[bech32m](https://github.com/bitcoin/bips/blob/master/bip-0350.mediawiki)-encoded
strings beginning with `public1…`, `private1…`, `signature1…`, and `package1…`
respectively.

BLAKE3 file hashes are 64-character lowercase hexadecimal.

Metadata
--------

Filepack packages may contain a file named `metadata.yaml` describing the
package and its content.

`filepack create` loads `metadata.yaml` if present and checks for validity and
unknown fields.

`filepack verify` also loads `metadata.yaml` if present and checks for
validity. Unknown fields, however, are not an error, so that future versions of
`filepack` may define new metadata fields in a backwards-compatible fashion.

Filepack metadata is intended to a broadly useful machine and human readable
description of the contents of a package, covering personal, distribution, and
archival use-cases.

Metadata follows a fixed schema and is not user-extensible. Future versions of
`filepack` may define new metadata fields, causing verification errors if those
fields are present and invalid according to the new schema.

Please feel free to open an issue with ideas for new metadata fields.

### Schema

Fields are given as `NAME: TYPE`.

Mandatory fields:

- `title: component`: The content's human-readable title.

Optional fields:

- `artwork: component.png`: The filename of an PNG file containing artwork for
  the content, for example, cover art for an album or key art for a movie.

- `creator: component`: The person or group who created the content.

- `date: date`: The date the content was created or released.

- `description: markdown`: A description of the content.

- `homepage: url`: Primary URL for the content. Should be the official homepage
  of the content, if any, and not, for example, a Wikipedia or media database
  link.

- `language: language`: The primary language of the content.

- `package: object`: The package metadata.

- `readme: component.md`: The filename of the content readme.

Optional `package` field describing the package itself, as opposed its content:

- `creator: component`: The person or group who created the package.

- `creator-tag: tag`: The tag of the person or group who created the
  package.

- `date: date`: The date the package was created.

- `description: markdown`: A description of the package.

- `homepage: url`: Primary URL for the package.

- `nfo: component.nfo`: The filename of the package nfo file.

Types:

- `component`: A string with the same restrictions as path components in the
  manifest `files` object, allowing them to be used as unix filesystem paths.
  Note that Windows imposes additional restrictions which are not enforced, so
  components may not be valid paths on Windows.

- `component.EXTENSION`: A component that must end with `.EXTENSION`.

- `date`: A string containing a date in one of several formats: as a year only,
  when the date and time is unknown, a date only, when the time is unknown, or
  a date and time with a mandatory time zone.

- `language`: A string containing an ISO 639-1 two-character language code. See
  `filepack languages` for valid language codes.

- `markdown`: A string containing CommonMark markdown.

- `tag`: A string containing a tag, commonly an abbreviation of a release group
  name. Must match the regular expression `[0-9A-Z]+(\.[0-9A-Z]+)*`.

- `url`: A string containing a URL.

Example dates:

```tsv
1970
1970-01-01
1970-01-01T00:00:00Z
1970-01-01 00:00:00Z
1970-01-01T00:00:00+00:00
1970-01-01 00:00:00 +00:00
```

### Example

```yaml
title: Tobin's Spirit Guide
creator: John Horace Tobin
artwork: cover.png
date: 1929
description: A compilation of supernatural occurrences, entities, and facts.
homepage: https://tobin-society.org/spirit-guide
language: en
readme: README.md
package:
  creator: Egon Spengler
  creator-tag: ES
  date: 1984-07-08 19:32:00 -04:00
  description: >
    First edition on loan from NYPL Main Branch research stacks. Captured via
    Microtek MS-300A flatbed scanner.
  homepage: https://ghost-busters.net/~egon
  nfo: tobins.nfo
```

The `homepage` URLs are of course anachronistic, as the World Wide Web was
created in 1989, some years after Egon first packaged Tobin's Spirit Guide.

Lints
-----

`filepack create` supports optional lints that can be enabled by group:

```shell
filepack create --deny distribution
```

The `distribution` lint group checks for issues which can cause problems if the
package is intended for distribution, such as non-portable paths that are
illegal on Windows, paths which would conflict on case-insensitive file
systems, and inclusion of junk files such as `.DS_Store`.

Lint group names and the lints they cover can be printed with:

```shell
filepack lints
```

Keys and Signatures
-------------------

`filepack` supports the generation of
[Curve25519](https://en.wikipedia.org/wiki/Curve25519) public/private keypairs,
and the creation and verification of
[EdDSA](https://en.wikipedia.org/wiki/EdDSA) signatures over manifests.

### Keypair Generation

Keypairs are generated with:

```shell
filepack keygen
```

Which creates `master.public` and `master.private` files in the `keychain`
subdirectory of the filepack data directory.

### Public Key Printing

Generated public keys can be printed with:

```shell
filepack key
```

### Signing

Signatures are created with:

```shell
filepack sign
```

Which signs the manifest in the current directory with your master key and adds
the signature to the manifest's `signatures` array. Signatures are made over a
fingerprint hash, recursively calculated from the contents of the manifest.

### Signature Verification

Signatures embedded in a manifest are verified whenever a manifest is verified.
The presence of a signature by a particular public key can be asserted with:

```sh
filepack verify --key PUBLIC_KEY
```

Which will fail if a valid signature for `PUBLIC_KEY` over the manifest
contents is not present.

Fingerprints
------------

Filepack signatures are made over the package fingerprint, which is the root of
a Merkle tree of the files and directories contained in the manifest.

Fingerprints are BLAKE3 hashes, constructed such that it is impossible to
produce objects which are different, either in type or content, but which have
the same fingerprint.

Fingerprints may be used as a globally unique identifier. If two packages have
the same fingerprint, they have the same content.

For details on how fingerprints are calculated, see [DESIGN.md](DESIGN.md).

Guide
-----

### Detecting Accidental Corruption

Create a filepack manifest with:

```shell
filepack create <PACKAGE>
```

This will create `<PACKAGE>/filepack.json`

To later verify the package against the manifest:

```shell
filepack verify <PACKAGE>
```

Because the manifest contains cryptographic hashes, accidental corruption to
the files or manifest will always be detected by `filepack verify`.

This is *not* the case with intentional, malicious corruption, since an
attacker could modify the files and replace the manifest hashes with the hashes
of the modified files.

### Detecting Malicious Corruption

Because an attacker could modify the files and replace the manifest hashes with
the hashes of the modified files, you must ensure that the manifest has not
been tampered with.

This can be accomplished in a number of ways, either by saving the manifest to
a secure location, saving the package fingerprint, or signing the package.

#### Manifest

To save the manifest in a secure location, use the `--manifest` option to save
the manifest somewhere other than the package:

```shell
filepack create <PACKAGE> --manifest <MANIFEST>
```

Then, verify the package against the saved manifest:

```shell
filepack verify <PACKAGE> --manifest <MANIFEST>
```

Because the manifest was protected, any modification to the package will be
detected. This has the advantage that not only will any modifications be
detected, but which files were modified can also be detected.

#### Fingerprint

Create the manifest in the package root with:

```shell
filepack create <PACKAGE>
```

Print the package fingerprint:

```shell
filepack fingerprint <PACKAGE>
```

Save the fingerprint in a secure location.

Then, verify the package against the saved fingerprint:

```sh
filepack verify <PACKAGE> --fingerprint <FINGERPRINT>
```

Because the fingerprint was protected, any modification to the package will be
detected. This has the advantage that you only have to save a small text
string, but the disadvantage that while any modifications will be detected, you
will not be able to determine which files have changed.

#### Signature

Create the manifest in the package root and sign it with your `master` key:

```shell
filepack create <PACKAGE> --sign
```

Then, verify the package and its signature:

```shell
filepack verify <PACKAGE> --key master
```

Any modification to the package or manifest will invalidate the signature,
which will be detected. This has the advantage of not needing to save the
manifest or fingerprint of packages you want to verify. However, you will need
to generate and secure your private key.

### Determining Authenticity

To check the authenticity of a package created by someone else, get their
public key and verify that the package contains a signature by that key:

```sh
filepack verify <PACKAGE> --key <KEY>
```

Alternatives and Prior Art
--------------------------

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
| [`signifiy`]( https://github.com/aperezdc/signify) | general-purpose |
| [`SignTool`](https://learn.microsoft.com/en-us/windows/win32/seccrypto/signtool) | Windows code signing |
| [`codesign`](https://developer.apple.com/library/archive/documentation/Security/Conceptual/CodeSigningGuide/Procedures/Procedures.html) | macOS code signing |
| [`jarsigner`](https://docs.oracle.com/javase/8/docs/technotes/tools/windows/jarsigner.html) | JDK code signing |

Filepack for SFV Enjoyers
-------------------------

If you package content for distribution, Filepack offers a number of benefits
over simple file verification with `.sfv` files.

- Filepack can detect both accidental corruption and intentional modification,
  whereas `.sfv` files can only detect accidental corruption.

- Because `filepack.json` manifests contain file sizes, Filepack can tell the
  user not just whether a file has been modified, but also whether it is empty,
  truncated, or too long.

- Filepack packages have a fingerprint, a short text string beginning with
  `package1…` which is guaranteed to be globally unique, allowing packages to
  be identified and referenced by fingerprint alone.

- Fingerprints can be used to verify that a `filepack.json` manifest itself has
  not been tampered with, proving authenticity of a package regardless of its
  source.

- Packages can be signed, allowing users to verify authenticity of any package
  from a packager by public key, a short string beginning with `public1…`.

- Filepack can warn you if package filenames might cause issues on other
  operating systems or file systems.

- Filepack packages are Merkle trees, both of files and within files. A user
  with a manifest, package fingerprint, or trusted public key can incrementally
  stream and verify files, or access and verify file content at random.
  Additionally, errors can be detected and recovered from by transmitting
  only those parts of the file which are corrupted.

- Packages may contain machine readable metadata following a filepack-defined
  schema, allowing packages to be searched, indexed, and exposed through rich,
  featureful interfaces.

Why is metadata important?
--------------------------

Metadata is the difference between a good user experience and a bad user
experience.

On Netflix, movies are presented in an attractive fashion with artwork, titles,
actors, and other information, all content is playable on whatever device you
happen to be using, streaming starts instantly, search is useful, and a
recommendation engine surfaces movies you might like. It is usable by anyone,
of any age and with any degree of technical proficiency.

On BitTorrent, movies are presented inscrutably in text listings with weirdly
formatted titles, spotty or missing information, and no artwork. Content may or
may not be playable on your device, streaming is impossible, search is
unstructured, and recommendations are unavailable. It is only usable by a small
and decreasing percentage of the population, and asking someone over to
"BitTorrent and chill" will be met by blinking incomprehension, followed by
derision as you fiddle with your torrent client like a digital steam engine
operator, desperately trying to find seeds for the movie you want to watch.

The difference between the two systems is not due to a difference in underlying
data, every piece of content one could possibly imagine is available on
BitTorrent. It is a difference in the availability of metadata.

Netflix has a database of standardized, machine-readable metadata, which makes
every feature of their system possible. BitTorrent has a disparate collection
of torrents, basically folders of files, with inconsistent and incomplete
metadata.

As polished services driven by metadata have become more popular, the user
experience that systems based on folders of files can offer has become ever
more unacceptable and alien to users, regardless of cost or underlying
availability of content.

Filepack seeks to rectify this by standardizing machine-readable metadata that
can be included with folder-of-files content.

Filepack metadata is stored a single file, `metadata.yaml`, in the root of a
filepack package. It contains information related to what the package contains,
which files in the package contain the package content and their file formats,
and who created the package. This metadata can serve as a base for the creation
of rich local and distributed applications and services, with user experiences
that compete with and exceed those of closed centralized alternatives.

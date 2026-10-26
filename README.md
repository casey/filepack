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

<!-- ANCHOR: introduction -->
Filepack is a content-addressed package format with self-describing metadata.

A package is a directory of files and a manifest containing their BLAKE3
hashes. File and directory hashes form a Merkle tree whose root hash, the
package fingerprint, uniquely identifies the package. Packages can be signed
with Ed25519 and verified against a manifest, fingerprint, or signature.

Packages may contain machine-readable metadata describing their content,
allowing programmatic search, preview, playback, and conversion.

`filepack` is a command-line tool for creating, signing, and verifying
packages, and includes an HTTP server for package upload, download, and
display.

The demo server at [filepack.com](https://filepack.com) hosts a selection of
example packages.

Filepack is currently experimental. The `filepack` interface and package format
may change at any time.
<!-- ANCHOR_END: introduction -->

Documentation is available at [filepack.com/book/](https://filepack.com/book/).

See [installation](https://filepack.com/book/installation.html) and
[quickstart](https://filepack.com/book/quickstart.html) to get started, and
[Why is metadata important?](https://filepack.com/book/why-metadata.html) for
some of the higher-level ideas behind Filepack.

Lints
-----

`filepack create` supports optional lints to check your package for issues:

```shell
filepack create --deny distribution
```

The `distribution` lint group checks for issues which can cause problems if the
package is intended for distribution, such as non-portable paths that are
illegal on Windows, paths which would conflict on case-insensitive file
systems, and inclusion of junk files such as `.DS_Store`.

Lint groups and the lints they contain may be printed with:

```shell
filepack lints
```

Individual lints may be allowed with `--allow`:

```shell
filepack create --deny distribution --allow windows-reserved-filename
```

Keys and Signatures
-------------------

`filepack` supports the generation of
[Curve25519](https://en.wikipedia.org/wiki/Curve25519) public/private key pairs,
and the creation and verification of
[EdDSA](https://en.wikipedia.org/wiki/EdDSA) signatures over manifests.

### Key Pair Generation

Key pairs are generated with:

```shell
filepack keygen
```

Which creates `master.public` and `master.private` files in the `keychain`
subdirectory of the Filepack data directory.

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

Guide
-----

The `[DIRECTORY]` argument in examples can be omitted, in which case it
defaults to the current directory.

### Creating a Package

To create a Filepack manifest:

```shell
filepack create [DIRECTORY]
```

This creates `manifest.filepack` containing hashes and file sizes of all files
in the `DIRECTORY` and subdirectories.

To enable linting, use the `--deny` flag with a lint or lint group:

```sh
filepack create --deny <LINT> [DIRECTORY]
```

To view all lints:

```shell
filepack lints
```

Please feel free to open issues requesting new lints.

### Creating a Package with Metadata

To create a package with metadata describing the package, create a file named
`metadata.yaml` in the package root, for example:

```yaml
title: The Necronomicon
creator: Abdul Alhazred
description: >-
  The Old Ones, their history, and the rites by which they may be summoned.
language: la
```

Then, to create the package:

```shell
filepack create [DIRECTORY]
```

Which will validate metadata if present and write it to `metadata.filemeta`, as
well as include it in `manifest.filepack`.

See [the book](https://filepack.com/book/authoring-metadata.html) for more
information.

### Signing a Package

To sign a package when creating it, first create a new public and private key
pair:

```shell
filepack keygen
```

This creates a new public key, `master.public`, with corresponding private key,
`master.private`, in the Filepack keychain directory.

The `filepack info` command will print the path to the Filepack data directory,
keychain directory, and any public keys in the keychain directory.

Your public key can be printed with:

```shell
filepack key
```

To sign a new package:

```shell
filepack create --sign [DIRECTORY]
```

To add a signature to an existing package:

```shell
filepack sign
```

### Getting the Package Fingerprint

To print the fingerprint of a package:

```shell
filepack fingerprint [DIRECTORY]
```

### Publishing a Manifest

Manifests contain only hashes and not file content, and so can be published
anywhere, including places where technical or legal limitations would preclude
publication of the content itself.

### Verifying a Package

Package content can be verified with:

```shell
filepack verify [DIRECTORY]
```

Any extra files, missing files, or modified files will produce an error.

This verifies files against hashes stored in the manifest.

Running `filepack verify` on its own can detect accidental corruption, but not
intentional modification, since an attacker could modify package content, and
then modify the manifest to make it match the modified content and thus pass
verification.

However, if the manifest was obtained from a trusted source, verification will
catch any modifications, intentional or otherwise, even if the package content
was obtained from an untrusted source.

To detect intentional modification, you can either verify a package against a
known-good fingerprint:

```sh
filepack verify --fingerprint <FINGERPRINT> [DIRECTORY]
```

This verifies both the contents of the package and that the manifest has the
expected fingerprint. The former protects against accidental corruption, and
the latter against intentional modification.

Packages can also be verified by checking for a signature by a known public
key. This also protects against intentional modification, as long as the
private key corresponding to the public key is protected, and only known to
you, or someone you trust.

To verify that a package has a signature from a particular public key:

```sh
filepack verify --key <PUBLIC_KEY> [DIRECTORY]
```

The `--key` option can be repeated to require signatures from multiple public
keys.

### Detecting Accidental Corruption

Create a Filepack manifest with:

```shell
filepack create <PACKAGE>
```

This will create `<PACKAGE>/manifest.filepack`

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

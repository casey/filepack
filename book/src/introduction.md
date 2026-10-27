Introduction
============

{{#include ../../README.md:introduction}}

See [Installation](installation.md) and [Quickstart](quickstart.md) to get
started, [Packages](packages.md) for an overview of Filepack packages, and [Why
is metadata important?](why-metadata.md) for some of the higher-level ideas
behind Filepack.

Features
--------

If you package content for distribution, Filepack offers a number of useful
features, even if you already use [SFV
files](https://en.wikipedia.org/wiki/Simple_file_verification):

- Filepack can detect both accidental corruption and intentional modification,
  whereas SFV files can only detect accidental corruption.

- Because `manifest.filepack` manifests contain file sizes, Filepack can tell
  the user not just whether a file has been modified, but also whether it is
  empty, truncated, or too long.

- Filepack packages have a fingerprint, a short text string beginning with
  `package1…` which is guaranteed to be globally unique, allowing packages to
  be identified and referenced by fingerprint alone.

- Fingerprints can be used to verify that a `manifest.filepack` manifest itself
  has not been tampered with, proving authenticity of a package regardless of
  its source.

- Packages can be signed, allowing users to verify authenticity of any package
  from a packager by public key, a short string beginning with `public1…`.

- Filepack can warn you if package filenames might cause issues on other
  operating systems or file systems.

- Filepack packages are Merkle trees, both of files and within files. A user
  with a manifest, package fingerprint, or trusted public key can incrementally
  stream and verify files, or access and verify file content at random.
  Additionally, errors can be detected and recovered from by transmitting
  only those parts of the file which are corrupted.

- Packages may contain machine readable metadata following a Filepack-defined
  schema, allowing packages to be searched, indexed, and exposed through rich,
  featureful interfaces.

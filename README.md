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

See [Installation](https://filepack.com/book/installation.html) and
[Usage](https://filepack.com/book/usage.html) to get started,
[Packages](https://filepack.com/book/packages.html) for an overview of Filepack
packages, and [Why?](https://filepack.com/book/why.html) for some of the
higher-level ideas behind Filepack.

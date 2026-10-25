Packages
========

A filepack package is a directory containing a manifest named
`manifest.filepack`, which records the names, sizes, and [BLAKE3
hashes](https://en.wikipedia.org/wiki/BLAKE_(hash_function)#BLAKE3) of all
files in the directory and its subdirectories.

A filepack manifest contains all the information needed to verify the contents
of a package. Additionally, packages may be signed using
[Ed25519](https://en.wikipedia.org/wiki/EdDSA#Ed25519), allowing verification
by checking that a package was signed by a trusted public key.

Filepack packages may include machine-readable metadata in a file named
`metadata.filemeta` in the package root.

A filepack package is a [Merkle
tree](https://en.wikipedia.org/wiki/Merkle_tree). The root hash of the tree is
the hash of the serialized root directory of the package, called the package
root. Serialized directories contain the names, sizes, hashes, and types of
their children, which may be files or subdirectories.

The root hash of a package Merkle tree is called the package fingerprint, and
is represented in using [tagged hex](hex.md):

```
package1af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262
```

A package fingerprint uniquely identifies the content of a package. Any change
to the content of a package results in a completely different fingerprint, and
it is computationally infeasible, assuming that BLAKE3 is collision-resistant,
to create two packages with different content, but with the same fingerprint.

BLAKE3 is itself a Merkle tree hash, whose leaves are the contents of the file
being hashed, broken into 1 KiB chunks.

Filepack's use of Merkle trees allows for efficient sharing and verification of
packages over the internet, for example using the [Filepack server](server.md).
A client, starting with a package fingerprint, first requests and verifies the
serialized package root directory, followed by each subdirectory, and file.

Because file hashes are Merkle tree roots, files can be streamed and verified
incrementally, with the server providing the client of a compact proof that
downloaded file chunks are included in the BLAKE3 Merkle tree>

Because filepack packages are trees of directories, they are composable. A
filepack package may contain other packages, with the directory entry of each
subpackage containing the subpackage fingerprint.

A package may be stored in forms other than as a directory tree and manifest.
For example, the Filepack server stores files and serialized directories by
hash in a flat directory, alongside a database recording package fingerprints.
This allows the filepack server to deduplicate files that appear in multiple
packages, and allows clients to skip uploading files that are already present
on the server, regardless of which package contains them.

Serialization
-------------

Package directories, `manifest.filepack` manifests, and `metadata.filemeta`
metadata are serialized as deco, a compact, canonical, schema-driven binary
serialization format.

The [deco](deco.md) chapter includes a description of deco itself, as well as
the schemas of all types serialized as deco.

A package fingerprint is the BLAKE3 hash of the deco-encoded package root
[directory](deco/directory.md), containing [entries](deco/entry.md) for each
child. Each entry contains the hash, name, and size of the child, along with
its [type and additional info](deco/entry-info.md). For directories, this
additional info includes [totals](deco/totals.md), the total number of files
and subdirectories, as well as their aggregate file size.

Filepack uses three other serialization formats, aside from deco:

- [Tagged hex](hex.md), a text format wrapping a hex-encoded deco payload. Used
  for fingerprints, hashes, public keys, private keys, and signatures.

- [YAML](yaml.md), a human-readable and writable text format, used for
  hand-authored files, namely `metadata.yaml`, the template from which a
  package's `metadata.filemeta` is generated.

- [JSON](json.md), a simple text format, used for command input and output, and
  machine-authored files.

Manifests
---------

Manifests are serialized with [deco](deco.md), a compact, binary serialization
format, and stored in a file named `manifest.filepack` in the package root
directory.

A `manifest.filepack` file is called a manifest archive, and is a
content-addressed format, containing a root hash, which is not the package
fingerprint, but instead to hash of an intermediate directory, and a map of
hashes to file contents. See [Archive](deco/archive.md) for the manifest
archive schema.

The manifest can be traversed by reading the root hash, reading the entry for
the root hash in the files map, deserializing the contents as a [deco
directory](deco/directory.md), and repeating the process for subdirectories.

The manifest archive root directory contains two subdirectories, `package` and
`signatures`.

The `package` directory is the package root, and the `signatures` directory
contains signatures over the package root. Each signature is stored as a file
whose name is the hash of the serialized signature.

All other subdirectory names are reserved for use by future versions of
Filepack. Unrecognized subdirectories of the manifest archive root are ignored
but preserved.

Manifest archives do not contain package files, only serialized directories.
The only exception to this is `metadata.filemeta`, which is included in the
archive if present. This allows the manifest archive to serve as a convenient,
single-file reference for a package, including both file hashes and sizes, as
well as metadata.

Manifests can be converted to JSON with `filepack manifest` for inspection and
manipulation. A JSON manifest can be converted to a deco manifest archive with
`filepack archive`. See [Manifest](json/manifest.md) for the JSON schema.

Filenames
---------

Path components in manifest directories are UTF-8 and may not be `.` or `..`,
contain the path separators `/` or `\`, contain control characters, or begin
with a Windows drive prefix, such as `C:`.

This ensures that file paths assembled from path components work on Windows and
Unix, and cannot escape the package root. Symbolic links are not supported.

Signatures
----------

Filepack allows for the creation of
[Ed25519](https://en.wikipedia.org/wiki/EdDSA#Ed25519) signatures over
a package fingerprint, which commits to the content of a package.

Signatures are made over a deco-encoded [Statement](deco/statement.md)
containing the package fingerprint, as well as an optional integer timestamp of
seconds since the Unix epoch.

Signatures are stored as files named after their hash in the the manifest
archive `signatures` directory.

Signatures are encoded as [tagged-hex](hex/signature.md), whose payload is a
deco-encoded [Signature object containing the signed
Statement](deco/signature-statement.md).

Before signing, and when verifying, the serialized statement is wrapped in an
[Envelope](deco/envelope-statement.md), which performs domain separation by
committing to the [application](deco/application.md) and
[context](deco/context.md).

The same signature scheme, with a different context and inner
[Claims](deco/claims.md) object, is used for server authorization bearer
tokens.

Metadata
--------

Filepack packages may contain a file named `metadata.filemeta` describing the
package and its contents. If present, `metadata.filemeta` is also embedded in
the `manifest.filepack` archive.

Metadata is authored by creating a file named `metadata.yaml` in the root of a
new package. `filepack create` then loads `metadata.yaml` if present, checks
for validity and unknown fields, and writes the deco serialization to
`metadata.filemeta` in the package root.

`metadata.yaml` is retained as a human-readable reference and for amending
metadata, but `metadata.filemeta` is the authoritative source of metadata. For
consumption by scripts and tools, `filepack metadata` prints the contents of
`metadata.filemeta` as JSON.

Metadata follows a fixed schema and is not user-extensible, with fields
describing the package contents, the package itself, and the media content of
the package. Future versions of `filepack` may define new metadata fields,
causing verification errors if those fields are present and invalid according
to the new schema.

Metadata allows for the attractive presentation of packages, including
browsing, viewing, search, and conversion. See
[filepack.com](https://filepack.com) for a selection of example packages,
covering a variety of media types. All metadata served by the demo server,
including package artwork, video placeholders, thumbnail images, and media
information, is sourced from package metadata.

See [Metadata](deco/metadata.md) for the deco schema, [Authoring
Metadata](authoring-metadata.md) for a description of `metadata.yaml`, the
template file used to generate package metadata.

Please feel free to [open an issue](https://github.com/casey/filepack/issues)
with ideas for new metadata fields.

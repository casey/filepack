Packages
========

A Filepack package is a directory containing a manifest named
`manifest.filepack`, which records the names, sizes, and [BLAKE3
hashes](https://en.wikipedia.org/wiki/BLAKE_(hash_function)#BLAKE3) of all
files in the directory and its subdirectories.

A Filepack manifest contains all the information needed to verify the contents
of a package. Additionally, packages may be signed using
[Ed25519](https://en.wikipedia.org/wiki/EdDSA#Ed25519), allowing verification
by checking that a package was signed by a trusted public key.

Filepack packages may include machine-readable metadata in a file named
`metadata.filemeta` in the package root.

A Filepack package is a [Merkle
tree](https://en.wikipedia.org/wiki/Merkle_tree). The root hash of the tree is
the hash of the serialized root directory of the package. Serialized
directories contain the names, sizes, hashes, and types of their children,
which may be files or subdirectories.

The root hash of a package Merkle tree is called the package fingerprint, and
is represented as [tagged hex](hex.md):

```
package12186aa6cefc1ae2f427b90e61a3ab91c88573f6f6719ca0188098d681b6c80ca
```

A package fingerprint uniquely identifies the content of a package. Any change
to the content of a package results in a completely different fingerprint, and
it is computationally infeasible, assuming that BLAKE3 is collision-resistant,
to create two packages with different content, but with the same fingerprint.

Filepack's use of Merkle trees allows for efficient sharing and verification of
packages over the internet, for example using the [Filepack server](server.md).
A client, starting with a package fingerprint, first requests and verifies the
serialized package root directory, followed by each subdirectory and file.

BLAKE3 is itself a Merkle tree hash, whose leaves are the contents of the file
being hashed, broken into 1 KiB chunks. This allows files to be streamed and
verified incrementally using compact proofs that downloaded file chunks are
included in the BLAKE3 Merkle tree.

A package may be stored in forms other than as a directory tree and manifest.
For example, the Filepack server stores files and serialized directories by
hash in a flat directory, alongside a database recording package fingerprints.
This allows the Filepack server to deduplicate files that appear in multiple
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
child. Directories map child names to entries containing the hash and size of
the child, along with its [type and additional info](deco/entry-info.md). For
directories, this additional info includes [totals](deco/totals.md), the total
number of files and subdirectories, as well as their aggregate sizes.

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

Manifests are serialized with [deco](deco.md) and stored in a file named
`manifest.filepack` in the package root directory.

A `manifest.filepack` file is called a manifest archive, and is a
content-addressed format, containing a root hash, which is not the package
fingerprint, but instead the hash of an intermediate directory, and a map of
hashes to file contents. See [Archive](deco/archive.md) for the manifest
archive schema.

The manifest can be traversed by reading the root hash, reading the entry for
the root hash in the files map, deserializing the contents as a [deco
directory](deco/directory.md), and repeating the process for subdirectories.

The manifest archive root directory contains two subdirectories, `package` and
`signatures`.

The `package` directory is the package root, and the `signatures` directory
contains signatures over the package fingerprint. Each signature is stored as a
file whose name is the hash of the serialized signature.

All other entry names are reserved for use by future versions of Filepack.
Unrecognized entries in the manifest archive root are ignored but preserved.

Manifest archives do not contain package files, only serialized directories and
signatures. The only exception to this is `metadata.filemeta`, which is
included in the archive if present. This allows the manifest archive to serve
as a convenient, single-file reference for a package, including both file
hashes and sizes, as well as signatures and metadata.

Manifests can be converted to JSON with `filepack manifest` for inspection and
manipulation. A JSON manifest can be converted to a deco manifest archive with
`filepack archive`. See [Manifest](json/manifest.md) for the JSON schema.

Filenames
---------

Filenames in manifest directories are non-empty UTF-8 strings and may not
contain control characters.

Furthermore, they may not be `.` or `..`, contain `/` or `\`, or begin with a
Windows drive prefix, such as `C:`.

This ensures that file paths assembled from path components work on Windows and
Unix and cannot escape the package root. Symbolic links are not supported.

The filenames `manifest.filepack` and `.filepack`, with any case, are reserved
in the package root.

Signatures
----------

Packages may be signed with
[Ed25519](https://en.wikipedia.org/wiki/EdDSA#Ed25519).

To sign a package, the signer constructs a [statement](deco/statement.md),
containing the package fingerprint, as well as an optional integer timestamp of
seconds since the Unix epoch.

The signature is made over the BLAKE3 hash of a deco-encoded
[envelope](deco/envelope-statement.md) containing the statement, which performs
domain separation by committing to the [application](deco/application.md) and
[context](deco/context.md).

For transport, the raw Ed25519 signature, statement, and public key are stored
in a [signature object](deco/signature-statement.md).

Signatures are serialized as deco and stored as files named after their hash in
the manifest archive `signatures` directory, and are represented in text as
[tagged hex](hex/signature.md).

To verify a signature, the statement is wrapped in an envelope, serialized,
hashed, and the raw Ed25519 signature over the hash is verified against the
public key.

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
the package. Future versions of Filepack may define new metadata fields,
causing verification errors if those fields are present and invalid according
to the new schema.

Metadata allows for the attractive presentation of packages, including
browsing, viewing, search, and conversion. See
[filepack.com](https://filepack.com) for a selection of example packages,
covering a variety of media types. All metadata served by the demo server,
including package artwork, video placeholders, thumbnail images, and media
information, is sourced from package metadata.

See [Metadata](deco/metadata.md) for the deco schema and [Authoring
Metadata](authoring-metadata.md) for a description of `metadata.yaml`, the
template file used to generate package metadata.

Please feel free to [open an issue](https://github.com/casey/filepack/issues)
with ideas for new metadata fields.

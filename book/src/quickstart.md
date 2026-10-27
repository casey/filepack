Quickstart
==========

Filepack supports a number of subcommands, including `filepack create` to
create a package, `filepack verify` to verify a package, and `filepack serve`
to start an HTTP package server.

See [Commands](commands.md) for all commands and their `--help` strings, or run
`filepack help` for supported subcommands and `filepack help SUBCOMMAND` for
information about a particular subcommand.

Creating Packages
-----------------

To create a package in the current directory:

```sh
filepack create
```

The manifest, containing the hashes and sizes of all files in the current
directory and its subdirectories, will be written to `manifest.filepack`.

To recreate a package in the current directory:

```sh
filepack create --force
```

Adding Package Metadata
-----------------------

Packages may optionally contain metadata in `metadata.filemeta`. This metadata
describes the contents of the package, the package itself, and the media
content of the package.

Metadata may be added by creating a metadata template in `metadata.yaml`:

```sh
echo "title: Fifty-three More Things to do in Zero Gravity" > metadata.yaml
```

Create a package and write the metadata to `metadata.filemeta`:

```sh
filepack create
```

Package metadata can be printed with:

```sh
filepack metadata
```

The metadata schema is extensive, with many fields and media types. See
[Metadata](metadata.md) for more information.

Linting Packages
----------------

To create a package with all lints enabled:

```sh
filepack create --deny all
```

Filepack supports a variety of [lints](lints.md). These include checks for
cross-platform compatibility issues, like case-conflicts on case-insensitive
filesystems; quality issues, like inclusion of junk files; and metadata issues,
like missing or incomplete metadata.

Creating a package with all lints enabled and fixing them is a great way to
learn more about Filepack and package metadata.

Verifying Packages
------------------

A package can be verified by manifest, fingerprint, or public key. Verification
will fail if files have been added, removed, or modified. The three
verification methods differ by what is trusted: the manifest, fingerprint, or
private key corresponding to the public key.

The package in the current directory can be verified against the hashes and
sizes in the manifest:

```sh
filepack verify
```

As long as the manifest has not been modified, any change to a file will cause
`filepack` to exit with a non-zero exit code.

A package can also be verified by fingerprint, the root hash of the package
Merkle tree.

First, print the fingerprint:

```sh
filepack fingerprint
```

Then, verify the package by fingerprint:

```sh
filepack verify --fingerprint <FINGERPRINT>
```

As long as the fingerprint has not been modified, any change to the package
will cause `filepack` to exit with a non-zero exit code.

Signing Packages
----------------

Generate a new key pair with the default name `master`:

```sh
filepack keygen
```

Sign the package with the newly generated private key:

```sh
filepack sign
```

Print the newly generated public key:

```sh
filepack key
```

And finally, verify that the package was signed by your private key:

```sh
filepack verify --key <PUBLIC_KEY>
```

Any change to the package will cause `filepack` to exit with a non-zero exit
code. As long as your private key is secure, nobody can create a signature
which verifies against your public key.

Sharing Packages
----------------

To serve packages locally on port 8000:

```sh
filepack serve --address localhost --http-port 8000
```

Upload the package in the current directory and print its server-assigned
package number with:

```sh
filepack upload --server localhost:8000
```

Open `http://localhost:8000` in your web browser to view packages. Media items
in packages with media metadata can be viewed and played.

Download the uploaded package to a new directory with:

```sh
filepack download --server localhost:8000 --package <FINGERPRINT> <DIRECTORY>
```

Delete the package from the server with:

```sh
filepack delete --server localhost:8000 <PACKAGE_NUMBER>
```

Run garbage collection to clean up files no longer referenced by any package:

```sh
filepack gc --server localhost:8000
```

See [Server](server.md) for more information.

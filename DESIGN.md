Design and Open Questions
=========================

Backwards Compatibility
-----------------------

How should future extensions to the manifest be handled? Extensions will likely
take the form of additional manifest keys, either at the top level, or in
manifest entries. Currently, unrecognized keys are a hard error. Ideally it
would be possible to partially relax this, such that unrecognized keys could be
categorized as required or optional, and only required unrecognized keys would
trigger an error.

With serialization formats using integer keys, it is common to designate even
keys as required and odd keys as optional.

It's a bit odd, but we could designate lowercase keys as required and uppercase
keys as optional.

Empty Directories
-----------------

Currently, empty directories are an error for both `filepack create` and
`filepack verify`. Should they be ignored, or recorded in the manifest by
`filepack create` and required to be present by `filepack verify`?

Entry Nesting
-------------

Currently, each manifest entry the complete relative path to the file. This
means that when a directory contains more than one file, the directory path
will be repeated.

```json
{
  "files": {
    "foo/bar": {
      …
    },
    "foo/baz": {
      …
    }
  }
}
```

We could instead opt to nest entries to avoid repetition:

```json
{
  "files": {
    "foo": {
      "bar": {
        …
      },
      "baz": {
        …
      }
    }
  }
}
```

This would save space in cases when a single long path component is repeated
many times, and provide a way to represent empty directories, but would make
manipulation with tools like `jq` more difficult.

Hash Function
-------------

Currently, `filepack` uses the BLAKE3 hash function, a fast, parallelizable,
general purpose cryptographic hash function.

There are other reasonable choices for hash functions:

- SHA-2 is used in many other systems, so by using SHA-2, those systems could
  adopt `filepack` without making any additional assumptions about the security
  of other hash functions. SHA-2 however is susceptible to length-extension
  attacks, which, although not a problem for `filepack`, does require
  additional caution in potential downstream applications.

- SHA-3 is less common than SHA-2, but hardware acceleration is more common for
  SHA-3 than BLAKE3.

- KangarooTwelve is a tree hash based on SHA-3, and so benefits from SHA-3
  hardware acceleration.

Another choice would be to allow users to select the hash function that they
wish to use, and record the hash function used in the manifest. I am very much
inclined not to do this. I would much rather support a single, all-purpose hash
function, to simplify implementations, and avoid additional security
considerations.

Overall, I'm happy with the current choice of BLAKE3. BLAKE3 is not incredible
common, but its membership in the BLAKE family has made it relatively well
reviewed and scrutinized.

A tree hash like BLAKE3 also allows for incremental streaming or random access
verification, which was actually the impetus for the creation of BLAKE3, and is
implemented in the [bao](https://github.com/oconnor663/bao) crate.

One slightly annoying aspect of BLAKE3 is that the chunk offset is used as an
input to produce the leaf hash, so it is not possible to deduplicate leaf
chunks.

Manifest Format
---------------

`filepack` manifests are currently JSON, but it would be worth considering
other serialization formats.

JSON was chosen because it is plain-text, more-or-less human readable, and
widely supported, with many libraries and tools like `jq` available.

Some alternative serialization formats are:

- TSV: the traditional format for `shasum` and `.sfv`. TSV is plain-text and
  even more human readable, but cannot be easily extended. Fields are
  identified by position, so it would not be possible to add additional fields
  in a backwards compatible fashion, and additional top-level keys are not
  supported.

- CBOR: a binary serialization format more-or-less morally equivalent to JSON.
  CBOR is less common and is not human readable, however, it is more compact
  than JSON. I suspect, however, that compactness is less of a benefit than
  human readability and ubiquitous support.

  CBOR would allow for easily embedding large amounts of additional binary
  data, for example, parity data or hash tiers.

- YAML: Easier for a human to read and write than JSON, but less widely
  supported. And, YAML canonicalization would be much harder, in the event that
  the manifest is canonicalized for hashing and signing.

Metadata
--------

A planned feature is the addition of optional metadata which semantically
describes the contents of a package. For example, it's title and author.

This metadata could be included in the manifest, or could be in a file placed
alongside the manifest.

Including it in the manifest would have the advantage of being a single file.

Including it alongside the manifest would have the advantage of being easier to
author, since it separates the manually created metadata and the automatically
generated manifest, and possibly allow the metadata to be written in a more
human-friendly format, such as YAML.

Signatures
----------

A planned feature is the addition of signing and signature verification.

When signing, signatures would be written to files in a `signatures` directory
alongside the manifest with the filename `KEY.SCHEME`, where `KEY` the public
key the signature was made with, and `SCHEME` is a short string identifying the
signature scheme, for example, `pgp` or `ssh`.

`filepack create` would exclude the `signatures` directory from hashing, since
signatures must be made over the hash of the manifest, `filepack` would provide
helpers for creating signatures and writing them to the appropriate file in the
`signatures` directory.

`filepack verify` would exclude the `signatures` directory from extraneous file
checks, since it would not be present in the manifest, and for each
`KEY.SCHEME` file, verify that the signature contained in the file is valid
according to `SCHEME` and was made with the private key corresponding to the
public key `KEY`.

Initially, existing schemes will be used, instead of trying to create a new
signing scheme, so `filepack create`, or the helper used to create signatures
will call to the appropriate binary to create signatures, and `filepack verify`
will call the appropriate binary to verify signatures.

There are a number of open questions related to signatures:

- Should signatures be made over the manifest or over the BLAKE3 hash of the
  manifest?

- Should the manifest be required to be in a canonical format? If not, there
  will be multiple valid manifests that describe the same file contents, but
  which have different hashes. Requiring the manifest to be canonical would
  require a canonical serialization and deserialization scheme.

- Should signatures be made over a message which attempts to ensure domain
  separation? For example, signatures could be made over the string `filepack:`
  prepended to the manifest hash. Decent signature schemes will already provide
  for domain separation, but if we ever want to introduce different kinds of
  signatures that, for example, represent different intents, then domain
  separation would be desirable.

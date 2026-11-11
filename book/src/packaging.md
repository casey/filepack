Packaging
=========

Welcome to the Filepack Packaging Guide!

Filepack packages are a complex format with rich metadata. We hope that the
experiences that can be built on top of structured packages with high quality
metadata will make the effort of packaging worth it, and that this guide can
help make the process as easy as possible.

Each section of this guide is a step in the packaging workflow. Some steps are
optional, but make sure to at least skim each section so you don't miss any
features or ways to improve your packages.

This guide assumes you are working in the terminal and have the `filepack`
binary installed. See [Installation](installation.md) for installation
instructions.

If you encounter a bug, unclear docs, or want to request a new feature, please
feel free to [open an issue](https://github.com/casey/filepack/issues/new) on
[GitHub](https://github.com/casey/filepack). There are a huge number of features
that could be added to Filepack, including support for more media types and file
formats, additional metadata fields, and different ways of sharing packages.
User feedback is vital in helping to decide what to implement.

Content
-------

First select the content that you want to package.

Content of any kind may be packaged, but Filepack supports media-specific
metadata for audio, images, and video. This metadata enables features like audio
playback, video playback, and image previews. Check out [Media](media.md) for
supported media types and file formats.

If you're packaging a type of media that is not yet supported, such as a PDF,
font, or ROM, or an unsupported file type, please
[open an issue!](https://github.com/casey/filepack/issues/new) Filepack aims to
support every form of media imaginable.

Layout
------

Assemble the content you want to package into a directory. "Directory" and
"folder" are synonymous, with "folder" being the GUI metaphor, and "directory"
being the lower-level primitive.

The files and subdirectories in a Filepack package should be organized and named
so that they are easy to understand and navigate, even without referring to the
metadata.

The directory that contains the package is the "package root" or "package root
directory".

Metadata
--------

Create the `metadata.yaml` metadata template in the package root. See
[Metadata](metadata.md) for an example metadata template.

Metadata is optional, so this step can be skipped. However, metadata is often
useful, if only for your own organization and records, so consider creating it,
even if the package is only for your own use.

Metadata from the metadata template, as well as media items, is read on package
creation and written to `metadata.filemeta`, the authoritative source of package
metadata.

By default, the metadata template is included in the package, so that package
metadata can be easily updated and regenerated. Treat it as having the same
audience as the rest of the package content.

The `package` field contains information about the package itself, as opposed to
its content. So if you're packaging a book, information about the book, such as
its title, creator, and language, would go at the top level, and information
about the package, such as the creator, in this case you, how it was scanned, or
how it was acquired, would go under the `package` field.

`description` fields are for single-line, plain-text descriptions. Longer
descriptions should be written to files, with the filename in the `readme` or
`package.colophon` field, depending on whether it concerns the content or the
package itself.

Supported `language` codes may be printed:

```sh
filepack languages
```

`time` fields support year or day precision. Please
[open an issue](https://github.com/casey/filepack/issues/new) if you require a
different precision.

Media
-----

The `media` field of `metadata.yaml` contains media-specific metadata. The
`type` field declares what type of media it is, and for media like audio or
images, the `items` field lists the media files in the package. Adding `media`
metadata enables features like preview and playback in the Filepack server, and
comprehensive per-file metadata is automatically extracted from media items,
such as resolution, sample rate, and codec.

Packages with `media` set may not have files that are not listed in the
metadata. Such files are effectively invisible, as apps like the Filepack server
have no useful way of surfacing them in the interface. Please
[open an issue](https://github.com/casey/filepack/issues/new) if you run into
this limitation, to discuss your use case and how it can be supported.

Tagging
-------

Filepack automatically extracts metadata from media item tags.

Tags are required for audio, and optional for all other media types. Consider
tagging all media items, since tagging can improve the user experience in file
explorers and media players.

See [Media](media.md) for supported tags and example tagging commands.

Assets
------

Packages may include assets, such as package artwork or video placeholder
images.

The `artwork` field contains the filename of the package artwork. It must be an
image whose width and height are equal.

The `placeholder` field of video media items contains the filename of a
placeholder image. It is used as the preview image on `/package` for that video,
as well as the `poster` attribute on the `/item` page, which is displayed before
the video begins playing. It should be an image file with the same resolution as
the video.

You may want to use [FFmpeg](https://www.ffmpeg.org/) to extract placeholder
images. For example, to extract the frame at 12 minutes 34 seconds and 467
milliseconds from `input.mp4` and write it to `input.jpg`:

```sh
ffmpeg -ss 00:12:34.467 -i input.mp4 -frames:video 1 input.jpg
```

Image assets should use their original and highest quality. When packages are
created, unless `--no-generate` is passed, thumbnails are generated for all
image assets. Thumbnails are used by the server for efficiency when the full
resolution of the original image is unnecessary. Thumbnails whose file size is
larger than the original image are omitted.

Creation
--------

Create your package by running:

```sh
filepack create
```

This writes file and directory hashes to the package manifest in
`manifest.filepack`, and is the only required step for creating a package.
However, keep reading for more options to use with `filepack create`.

Use `--ignore` to omit files and directories from the package:

```sh
filepack create --ignore .git
```

Lints
-----

Filepack supports a wide variety of lints that check for potential problems with
your package.

You can print supported lints with:

```sh
filepack lints
```

Related lints are collected into named lint groups, such as `all`, which
includes all lints. Lints are enabled with `--deny NAME`, where `NAME` is the
name of a lint or lint group, and disabled with `--allow NAME`.

Enabling all lints is a great way to learn about Filepack features and how to
improve your packages.

Update the package, this time with all lints enabled:

```sh
filepack create --deny all --force
```

Since the package was already created in the previous step, `--force` is
included to overwrite the old manifest.

`filepack` will fail with a hopefully informative error message telling you what
went wrong, including the name of the lint in brackets. The lint can be fixed,
or disabled by adding `--allow NAME` to the `filepack` invocation.

Lints may not apply to your package at all. For example, the `media-missing`
lint complains if your package does not have `media` metadata. However, your
package may not contain media, or may contain an unsupported media type, in
which case there is no sensible way to satisfy the lint.

Fingerprint
-----------

Print the package fingerprint:

```sh
filepack fingerprint
```

The fingerprint is a string starting with `package1…` which uniquely identifies
the content of the package. If the package changes, including any files,
filenames, metadata, or derived assets, the fingerprint will change.

Signing
-------

Packages may be signed, allowing authenticity to be verified in addition to
integrity. When verifying a package against a hash or a manifest we are
verifying integrity: Has the package been tampered with? When verifying a
package against a signature we are verifying authenticity: Was this package
vouched for by a particular person?

Signing involves three new types of objects: private keys, public keys, and
signatures.

Your private key should be kept secret. Anyone who has a private key can create
signatures for it. Private keys begin with `private1…`.

Your public key is used to verify the signatures that you create and does not
need to be kept secret. Public keys begin with `public1…`.

Signatures can be used, along with a package and a public key, to verify that
the package was signed by the person who holds the private key from which the
public key was derived.

If you keep your private key secure, you can create and sign packages and anyone
with your public key can verify that the package was created by you, even if
they get the package from an untrusted third party.

Please note that the Filepack server does not yet support uploading or
downloading signatures.

Generate a private key and public key, also called a key pair, named
`master.private` and `master.public`:

```sh
filepack keygen
```

Private keys are stored in `~/.filepack/keychain` by default. See
[Configuration](configuration.md) for details.

Sign the package you created in the previous steps:

```sh
filepack sign
```

To sign a package during creation:

```sh
filepack create --sign
```

To print your public key for verification:

```sh
filepack key
```

Signatures identify a package by fingerprint. If you update the package and its
fingerprint changes, signatures with the old fingerprint will not be valid for
the new package.

Regenerate
----------

Regenerate the package after any changes and with all options above:

```sh
filepack create --deny all --force --sign
```

Add `--allow NAME` with any lints you are disabling, and omit `--sign` if you
are not signing the package.

Preview
-------

The best way to preview your package is by uploading it to a local server.

Start a server listening on `127.0.0.1:8000` with writes enabled:

```sh
filepack serve --http-port 8000 --unrestricted-writes
```

Upload the package:

```sh
filepack upload --server 127.0.0.1:8000
```

Then visit [http://localhost:8000](http://localhost:8000) to view the uploaded
package, check for errors, and ensure that it displays properly.

Click the `media` link on the `/package` page to see extracted media metadata.

Upload
------

When you're happy with your package you're ready to publish it to a Filepack
server. To upload to the fictitious `example.com` server and authorize with your
`master` key pair:

```sh
filepack upload --auth master --server example.com
```

After uploading, the uploaded package number and revision will be saved to
`.filepack/state.json` in the package root.

The package may be regenerated and a new revision published to the same package
number by running the same upload command above.

Download
--------

To download your package from the fictitious `example.com` server to the `foo`
directory:

```sh
filepack download --package <FINGERPRINT> --server example.com foo
```

The `filepack download` command verifies the package being downloaded against
the provided fingerprint, making it possible to download packages from untrusted
servers and be assured that the downloaded contents match the fingerprint.

Share
-----

A Filepack package is just a directory of files, so it can be shared without a
server. Stick it on a thumb drive, blast it out over BitTorrent, or share it
with a Dropbox link.

Packages obtained without using `filepack download` can be verified by
fingerprint:

```sh
filepack verify --fingerprint <FINGERPRINT>
```

Or by signature:

```sh
filepack verify --key <PUBLIC_KEY>
```

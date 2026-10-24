Server
======

The package server can be started with `filepack serve`. A demo server is
available at [filepack.com](https://filepack.com).

The server has a web frontend that can be used to browse packages, as well as
an API for uploading and downloading packages.

See [Routes](routes.md) for documentation of both the web and API routes.

The server responds to requests for `/{fingerprint}`, `/{number}`, and
`/{revision}` which redirect to
[`/package/{package}`](routes/package-package.md), and can be used as
convenient shortcuts.

Authorization
-------------

By default, the filepack server does not require authorization. Anyone can
upload, download, and browse packages. Running `filepack serve` without any
flags listens on port 80 on all interfaces, not just localhost.

Writes can be restricted with `--restrict-writes`, which forbids unauthorized
write access to the API, including the ability to upload and delete packages.

The `--admin-key` option, along with `--domain`, can be used to authorize write
access by public key. The holder of the admin private key may authorize
requests by including a tagged-hex [bearer token](hex/token.md) with their
requests in the `Authorization` header. The token's audience must match the
value of `--domain`, and the timestamp must be current.

If `--restrict-writes` is used with no `--admin-key`, the server will be
read-only.

Note that authorization tokens are not scoped to a particular endpoint or
request, only by domain and time. They can be used for any purpose while valid.
As such, `filepack` commands with `--auth` will refuse to run unless the
connection to the server is over loopback or HTTPS.

HTTPS
-----

The Filepack server can request a [TLS
certificate](https://en.wikipedia.org/wiki/Transport_Layer_Security) from
[Let's Encrypt](https://letsencrypt.org/) using the
[ACME](https://en.wikipedia.org/wiki/Automatic_Certificate_Management_Environment)
protocol.

To serve HTTPS on port 443, request a TLS certificate for `example.com`, and
redirect requests on port 80 to port 443:

```sh
filepack serve --domain example.com --https --redirect-http-to-https
```

ACME TLS certificates are stored in `acme-cache/` in the [Data
Directory](configuration.md#data-directory).

Responses served over HTTPS include `Strict-Transport-Security:
max-age=31536000`. If you serve a domain over HTTPS and then later switch to
HTTP, browsers that have previously visited using HTTPS will refuse to connect.

Storage
-------

Server state, including the database, ACME cache, uploaded files, and
incomplete files, as well as the keychain containing admin keys, is stored in
the [Data Directory](configuration.md#data-directory).

The database is stored in `database.redb` and uses
[redb](https://github.com/cberner/redb).

ACME TLS certificates are cached in `acme-cache/`.

Uploaded files are stored in `files/{hash}`, where hash is the [tagged-hex
BLAKE3 hash of the file contents](hex/hash.md).

Uploads are written to temporary files in `incoming/` and atomically moved to
`files/{hash}` upon completion.

There is no disk quota, so a writable server can be filled by anyone with
access.

API
---

API routes begin with `/api`. See [Routes](routes.md) for details.

Typed request and response bodies are encoded with [deco](deco.md).

The optional `Filepack-API-Version` header may be included with requests. If
included, the value should be the version of the Filepack API that the client
supports. If omitted, requests are treated as having `Filepack-API-Version: 0`.
Requests with an unsupported API version return 400.

The only supported API version is `0`.

Caching
-------

All routes that return hashed files include an `ETag` header whose value is the
tagged-hex [hash](hex/hash.md) of the file as a strong entity tag. These routes
also respect the `If-None-Match` header for `GET` and `HEAD` requests,
including range requests, and will return a `304 Not Modified` with no body if
the `If-None-Match` header entity tag matches the response `ETag` header.

Immutable routes return `Cache-Control: public, max-age=31536000, immutable`
with successful responses. Immutable routes are those whose path contains a
hash which commits to the response. Note that web routes, such as
`/package/{fingerprint}`, are not immutable, since the HTML served may change,
for example if `filepack` is updated.

Package Numbers and Revisions
-----------------------------

When a client uploads a package to the server, it creates and uploads a
[revision object](deco/revision-object.md), similar to a
[Git](https://en.wikipedia.org/wiki/Git) commit.

Revision objects have a `package` field containing a package fingerprint, and a
`parents` field containing a set of parent revision hashes.

Uploaded packages are assigned sequential package numbers starting from zero.
Deleted package numbers are not reused.

A package revision with a number is a head.

Newly uploaded packages are assigned numbers and are thus heads. When a new
revision of the same package is published, the package number is updated to
point at the new revision, whose parents field contains the old head revision.

Server URLs
-----------

Commands like `filepack upload` which take server URLs also accept IP addresses
and domains without a scheme. IPs and `localhost` use `http://`; all other
domains use `https://`.

Uploading Packages
------------------

Packages can be uploaded to the server with [`filepack
upload`](commands/filepack-upload.md).

`filepack upload` actually performs three distinct operations: creation,
replacement, and update.

After successfully creating, replacing, or updating a package on the server,
the new package head for that server is recorded to `.filepack/state.json` in
the package root.

### Package Creation

Package creation assigns a new package number to an uploaded package and is the
default when running `filepack upload` for the first time in a given directory
to a particular server.

Running `filepack upload` from the same directory without modifying the package
is idempotent and will return the same package number.

After modifying the package, running `filepack upload` from the same directory
will update the package.

### Package Replacement

Package replacement makes an existing package number point at a new package
with no parents, making the old package and its parents collectable by
`filepack gc`, unless they are reachable from another head.

A previously uploaded package with `NUMBER` can be replaced with:

```sh
filepack upload --replace <NUMBER>
```

### Package Update

A package number may be updated to point at a new revision whose parent is the
current head for that number with:

```sh
filepack upload --update <NUMBER>
```

Or just `filepack upload`, if `.filepack/state.json` has an entry for the
current server.

If the current package head differs from the recorded head in
`.filepack/state.json`, for example if someone else updates the package,
`filepack upload` will abort. If two clients race to update the same package,
the server will accept the first and reject the second with `409 Conflict`.

Downloading Packages
--------------------

Packages and files may be downloaded from the server with [`filepack
download`](commands/filepack-download.md).

`filepack download` verifies the hash of all files it downloads, so servers do
not need to be trusted, as long as the client has a known-good package
fingerprint or file hash.

Deleting Packages
-----------------

Packages may be deleted from the server with [`filepack
delete`](commands/filepack-delete.md).

Garbage Collection
------------------

Deleting a package causes the package number to no longer point to a package,
and replacing a package causes the number to point at a different package.
These packages are still served by fingerprint and revision, and their files
are not immediately removed from the server.

After a package is deleted or replaced, running `filepack gc` will remove all
packages and package files which are not reachable from the current heads or
mounted packages.

Previous versions of updated packages are still reachable from the parent
revisions of the new head revisions, and will continue to be served by
fingerprint and revision.

If a file uploaded with `filepack upload --file` is not reachable from any
package head, it will be deleted by the next run of `filepack gc`.

Mounts
------

Web packages contain web content with a `static/index.html` entry point. They
are [currently unstable](unstable-features.md) and will be rejected for upload
unless the server is started with unstable features enabled.

By default, web packages are not mounted. Uploaded web packages can be viewed
and browsed at `/package/{package}`, but all active content, including HTML and
JavaScript, is served with `Content-Security-Policy: sandbox`, a non-active
`Content-Type` such as `text/plain;charset=utf-8` or
`application/octet-stream`, and `X-Content-Type-Options: nosniff`.

Web packages can be mounted with:

```sh
filepack serve --mount PACKAGE
```

When mounted, `/mount/{package}/`, where `package` is a mounted package
fingerprint, revision, or number, returns `static/index.html` of the mounted
package, and `/mount/{package}/{path}` returns the package file at
`static/{path}`.

Mounted files are served unsandboxed from the server origin with unrestricted
content types, including `text/html` and `text/javascript`, guessed from file
extensions. Furthermore, packages mounted by number serve the latest version of
the package. On a world-writable server, anyone may update the package, causing
arbitrary HTML and JavaScript to be served from the server origin. Make sure
you understand the consequences of this and only mount packages you trust, and
only mount packages by number on servers with `--restrict-writes`.

Mounts may optionally be given a name with `--mount PACKAGE:NAME`, in which
case they are also served under `/{NAME}/`. Built-in routes take precedence
over mounts.

This book is served as a web package, mounted at
[filepack.com/book/](https://filepack.com/book/).

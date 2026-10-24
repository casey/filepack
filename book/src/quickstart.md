Quickstart
==========

Optionally, add metadata describing your package:

```sh
echo "title: Packaging Guide" > metadata.yaml
```

Create `manifest.filepack`:

```sh
filepack create
```

Verify current directory against hashes in `manifest.filepack`:

```sh
filepack verify
```

Print package fingerprint:

```sh
filepack fingerprint
```

Print package metadata:

```sh
filepack metadata
```

Start a server:

```sh
filepack serve --http-port 8000
```

Upload the package to the server:

```sh
filepack upload --server localhost:8000
```

Download the package from the server:

```sh
filepack download --server localhost:8000 --package <FINGERPRINT> <DIRECTORY>
```

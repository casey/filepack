Quickstart
==========

Optionally, add metadata describing your package:

```sh
echo "title: Packaging Guide" > metadata.yaml
```

Create `manifest.filepack`:

```shell
filepack create
```

Verify current directory against hashes in `manifest.filepack`:

```shell
filepack verify
```

Print package fingerprint:

```shell
filepack fingerprint
```

Print package metadata:

```shell
filepack metadata
```

Start a server:

```shell
filepack serve --http-port 8000
```

Upload the package to the server:

```shell
filepack upload --server localhost:8000
```

Download the package from the server:

```shell
filepack download --server localhost:8000 --package <FINGERPRINT> <DIRECTORY>
```

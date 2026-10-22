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

Metadata
========

Filepack packages may contain metadata describing the package in
`metadata.filemeta`. This metadata is stored as [deco](deco.md), which is not
human readable or writable.

When authoring a package with `filepack create`, a metadata template named
`metadata.yaml` may be included, in which case it will be converted to deco and
saved to `metadata.filemeta`.

See the schema of `metadata.yaml` in [Metadata](yaml/metadata.md), and an
example below.

The `media` field may optionally contain metadata describing the package media.
Packages with `media` metadata are displayed by the filepack server with the
ability to browse and play the audio, image, or video items that they contain.

See [Media Formats](media-formats.md) for supported media item file formats.

The [unstable](unstable-features.md) `web` media type may be used for packages
containing web content, which is served from the `static/` directory of the
package, with the required entry point `static/index.html`.

```yaml
{{ self.example.yaml }}
```
%% for variant in &self.example.variants {

## `{{ variant.field }}: {{ variant.name }}`

```yaml
{{ variant.yaml }}
```
%% }

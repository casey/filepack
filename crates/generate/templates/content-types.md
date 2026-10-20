Content Types
=============

Routes that serve package files, such as `GET /file/{hash}/{*path}`, and the
package artwork, thumbnail, and media item routes, use a content type
determined by the file's extension.

The `Content-Security-Policy: sandbox` header is included with responses where
doing so does not break built-in browser preview pages.

Binary and unrecognized file extensions are served sandboxed with
`Content-Type: application/octet-stream` and `Content-Disposition: attachment`.

Files under `/mount/` are served with a content type guessed from their
extension, not using the following table, and are not sandboxed.

| extensions | content type | sandboxed |
|------------|--------------|-----------|
%% for content_type in self.content_types {
| {{ list(&content_type.extensions) }} | `{{ content_type.content_type }}` | {{ content_type.sandbox }} |
%% }

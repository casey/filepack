Schemas
=======

Filepack defines a variety of data formats, including `manifest.filepack`
manifests, `metadata.filemeta` metadata, and tagged hex strings.

These formats use *deco*, a minimal, schema-driven, canonical, evolvable binary
serialization format.

Deco has only one type, the byte string. The first byte, or *head*, of an
encoded byte string determines how to decode it:

- `00-7F`: literal single byte (0–127)
- `80-EF`: `value - 0x80` bytes follow (0–111 bytes)
- `F0-F7`: `value - 0xEF` byte little-endian length (1–8 bytes) followed by that
           many bytes (112–2<sup>64</sup>-1)
- `F8-FF`: reserved values

The shortest possible encoding must be used.

Since all native types are encoded as byte strings, an encoded deco value may
not be decoded without knowing its type.

Deco is mostly canonical. With some exceptions, there is only one way to
serialize a given value. Types which are designed to be extended are not
guaranteed to have a canonical serialization, since they must accept and ignore
data produced by newer versions of filepack.

Examples
--------

A single byte whose value is between `0x00` and `0x7F` is encoded as-is. For
example, the string `0x67` is encoded as `0x67`.

A byte string whose length is between 0 and 111 bytes are encoded as a head
byte equal to `0x80` plus the length of the string, followed by the byte
string. For example, the empty string is encoded as `0x80`, the string `0xFF`
is encoded as `0x81FF`, and the string `0xABCD` is encoded as `0x82ABCD`.

Byte strings whose length is greater than 111 bytes, up to 2<sup>64</sup>-1
bytes, are encoded as a head byte equal to `0xEF` plus the number of bytes
needed to encode the length, followed by that many length bytes encoded as a
little endian integer with trailing zero bytes removed, followed by that many
bytes. For example, the byte string containing 255 `0xAB` bytes is encoded as
`0xF001ABABAB…`.

The shortest possible encoding for a given byte string must be used. For
example, the byte string `0xAB` must be encoded as `0xAB` `0x81AB`, `0xF001AB`,
or `0xF10100AB`.

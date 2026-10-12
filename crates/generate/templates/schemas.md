Schemas
=======

Filepack defines a variety of data formats, including `manifest.filepack`
manifests, `metadata.filemeta` metadata, and tagged hex strings.

These formats use *deco*, a minimal, schema-driven, canonical, evolvable binary
serialization format.

Deco has only one type, the byte string. Since all native types are encoded as
byte strings, an encoded deco value may not be decoded without knowing its
type.

Deco is mostly canonical. With some exceptions, there is only one way to
serialize a given value. Types which are designed to be extended are not
guaranteed to have a canonical serialization, since they must accept and ignore
data produced by newer versions of filepack.

Strings
-------

The first byte, or *head*, of an encoded byte string determines how to decode
it:

- `00-7F`: literal single byte (0–127)
- `80-EF`: `value - 0x80` bytes follow (0–111 bytes)
- `F0-F7`: `value - 0xEF` byte little-endian length (1–8 bytes) followed by that
           many bytes (112–2<sup>64</sup>-1)
- `F8-FF`: reserved values

The shortest possible encoding must be used.

### Examples

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

Arrays
------

Arrays are encoded as a byte string whose payload is the concatenation of the
encoded array elements.

### Examples

An empty array is encoded as `0x80`.

An array containing one element, the byte string `0xAA`, is encoded as
`0x8281AA`. The element is encoded as `0x81AA`, which is two bytes, so the head
byte of the array is `0x82`.

An array containing two elements, the byte string `0xABCDEF` and `0x123456`, is
encoded as `0x8883ABCDEF83123456`.

Interestingly, an array containing a single element of length one whose value
is `0x7F` or less has the same encoding as the element. So the array containing
one element, the byte string `0x22`, is encoded as `0x22`.

Maps
----

Maps are encoded as a byte string whose payload is the concatenation of the
encoded keys and values, with each key being immediately followed by its value.
Keys must be unique and in ascending order.

### Examples

An empty map is encoded as `0x80`.

A map containing one entry, the key `0x01` with the value `0xAA`, is encoded as
`0x830181AA`. The key is encoded as `0x01` and the value as `0x81AA`, which is
three bytes in total, so the head byte of the map is `0x83`.

A map containing two entries, the key `0x01` with the value `0xABCDEF` and the
key `0x02` with the value `0x123456`, is encoded as `0x8A0183ABCDEF0283123456`.

Structs
-------

Structs are encoded as maps from [UnsignedInteger](schemas/unsigned-integer.md)
field number to field value. Absent optional fields are omitted.

Structs with `#[deco(strict)]` attribute reject unknown fields. Structs without
the attribute ignore unknown fields.

Non strict struct fields with type `Option<T>`, where `T` is an enum, are
treated as absent if the encoded value is an unrecognized variant.

Field 0 is reserved for versioning. If field 0 is present, decoding fails with
an unsupported version error.

### Examples

A struct whose optional fields are all absent is encoded as the empty map,
`0x80`.

A struct with one field, field 1 with the value 99, is encoded as `0x820163`.
The field number is encoded as `0x01` and the value as `0x63`, which is two
bytes in total, so the head byte of the struct is `0x82`.

A struct with two fields, field 1 with the value 1 and field 2 with the string
`foo`, is encoded as `0x8701010283666F6F`. If field 1 is optional and absent,
the same struct is encoded as `0x850283666F6F`.

Enums
-----

Enums are encoded as arrays whose first value is the
[UnsignedInteger](schemas/unsigned-integer.md) enum discriminant. The second
element, if present, is a struct containing the enum variant's fields.

If all of a variant's fields are optional and absent, the second element is
omitted entirely, rather than being encoded as an empty struct.

Unknown enum fields are ignored.

### Examples

A variant without fields is encoded as an array containing only its
discriminant, so the variant with discriminant 0 is encoded as `0x00` and the
variant with discriminant 1 is encoded as `0x01`.

A variant with discriminant 1 and one field, field 1 with the value 99, is
encoded as `0x8401820163`. The discriminant is encoded as `0x01` and the struct
containing the fields as `0x820163`, which is four bytes in total, so the head
byte of the array is `0x84`.

A variant with discriminant 0 and two fields, field 1 with the value 1 and
field 2 with the value 2, is encoded as `0x86008401010202`. If field 1 is
optional and absent, the same variant is encoded as `0x8400820202`. If field 2
were also optional and absent, the variant would be encoded as `0x00`.

Schemas
-------

| schema |
|--------|
%% for name in self.schemas.keys() {
| [{{ escape(name) }}](schemas/{{ name.to_kebab_case() }}.md) |
%% }

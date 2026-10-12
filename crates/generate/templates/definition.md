%% match self.schema {
%%   Schema::Array { element } => {
An array of {{ schema(element) }}.
%%   }
%%   Schema::Boolean => {
A boolean. Encoded as the byte string `0x00` for false and `0x01` for true.
%%   }
%%   Schema::Bytes { length } => {
%%     if let Some(length) = length {
A byte string of exactly {{ Count::new(*length, "byte") }}.
%%     } else {
An arbitrary byte string.
%%     }
%%   }
%%   Schema::Enum { variants } => {
An enum with {{ Count::new(variants.len(), "variant") }}:

| discriminant | variant |
|--------------|---------|
%%     for variant in variants {
%%       if variant.fields.is_empty() {
| {{ variant.n }} | {{ variant.name }} |
%%       } else {
| {{ variant.n }} | [{{ variant.name }}](#{{ variant.name.to_lowercase() }}) |
%%       }
%%     }
%%     for variant in variants {
%%       if !variant.fields.is_empty() {

## {{ variant.name }}

{{ FieldsMd { fields: &variant.fields } }}
%%       }
%%     }
%%   }
%%   Schema::Map { key, value } => {
A map from {{ schema(key) }} to {{ schema(value) }}.
%%   }
%%   Schema::SignedInteger => {
A signed 64-bit integer. First zig-zag encoded, then encoded as an
[UnsignedInteger](../schemas/unsigned-integer.md).
%%   }
%%   Schema::String => {
A string. Encoded as a byte string containing the UTF-8 encoding of the string.
%%   }
%%   Schema::Struct { fields, magic, strict } => {
A struct with {{ Count::new(fields.len(), "field") }}:

{{ FieldsMd { fields } }}
%%     if let Some(magic) = magic {

The value is encoded with magic bytes consisting of the deco encoded byte
string `filepack\0`, followed by the deco encoded byte string `{{ magic }}`,
followed by the encoded value.
%%     }
%%     if *strict {

Unknown fields are rejected.
%%     }
%%   }
%%   Schema::UnsignedInteger => {
An unsigned 64-bit integer. Encoded as little endian with trailing zero bytes
removed, with a minimum length of one byte.
%%   }
%%   Schema::Wrapper { inner } => {
A wrapper around {{ schema(inner) }}.
%%   }
%% }

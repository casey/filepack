%% match self.schema {
%%   deco::Schema::Array { element } => {
An array of {{ deco(element) }}.
%%   }
%%   deco::Schema::Boolean => {
A boolean. Encoded as the byte string `0x00` for false and `0x01` for true.
%%   }
%%   deco::Schema::Bytes { length } => {
%%     if let Some(length) = length {
A byte string of exactly {{ Count::new(*length, "byte") }}.
%%     } else {
An arbitrary byte string.
%%     }
%%   }
%%   deco::Schema::Enum { variants } => {
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

{{ DecoFieldsMd { fields: &variant.fields } }}
%%       }
%%     }
%%   }
%%   deco::Schema::Map { key, value } => {
A map from {{ deco(key) }} to {{ deco(value) }}.
%%   }
%%   deco::Schema::SignedInteger => {
A signed 64-bit integer. First zig-zag encoded, then encoded as an
[UnsignedInteger](../deco/unsigned-integer.md).
%%   }
%%   deco::Schema::String => {
A string. Encoded as a byte string containing the UTF-8 encoding of the string.
%%   }
%%   deco::Schema::Struct { fields, magic, strict } => {
A struct with {{ Count::new(fields.len(), "field") }}:

{{ DecoFieldsMd { fields } }}
%%     if let Some(magic) = magic {

The value is encoded with magic bytes consisting of the deco encoded byte
string `filepack\0`, followed by the deco encoded byte string `{{ magic }}`,
followed by the encoded value.
%%     }
%%     if *strict {

Unknown fields are rejected.
%%     }
%%   }
%%   deco::Schema::UnsignedInteger => {
An unsigned 64-bit integer. Encoded as little endian with trailing zero bytes
removed, with a minimum length of one byte.
%%   }
%%   deco::Schema::Wrapper { inner } => {
A wrapper around {{ deco(inner) }}.
%%   }
%% }

# {{ escape(self.name) }}

%% match self.schema {
%%   Schema::Array { element } => {
An array of {{ link(element) }}.
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
A map from {{ link(key) }} to {{ link(value) }}.
%%   }
%%   Schema::SignedInteger => {
A signed 64-bit integer. First zig-zag encoded, then encoded as an
[UnsignedInteger](unsigned-integer.md).
%%   }
%%   Schema::String => {
A string. Encoded as a byte string containing the UTF-8 encoding of the string.
%%   }
%%   Schema::Struct { fields } => {
A struct with {{ Count::new(fields.len(), "field") }}:

{{ FieldsMd { fields } }}
%%   }
%%   Schema::UnsignedInteger => {
An unsigned 64-bit integer. Encoded as little endian with trailing zero bytes
removed, with a minimum length of one byte.
%%   }
%%   Schema::Wrapper { inner } => {
A wrapper around {{ link(inner) }}.
%%   }
%% }

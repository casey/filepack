# {{ self.name }}

%% match self.schema {
%%   Schema::Boolean => {
A boolean. Encoded as the byte string `0x00` for false and `0x01` for true.
%%   }
%%   Schema::Bytes => {
An arbitrary byte string.
%%   }
%%   Schema::Enum { variants } => {
An enum with {{ Count::new(variants.len(), "variant") }}:

| discriminant | variant |
|--------------|---------|
%%     for variant in variants {
| {{ variant.n }} | {{ variant.name }} |
%%     }
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

| n | field | type | optional |
|---|-------|------|----------|
%%     for field in fields {
| {{ field.n }} | {{ field.name }} | {{ link(&field.schema) }} | {{ if field.optional { "yes" } else { "no" } }} |
%%     }
%%   }
%%   Schema::UnsignedInteger => {
An unsigned 64-bit integer. Encoded as little endian with trailing zero bytes
removed, with a minimum length of one byte.
%%   }
%% }

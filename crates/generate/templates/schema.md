# {{ self.name }}

%% match self.schema {
%%   Schema::Boolean => {
A boolean. Encoded as the byte string `0x00` for false and `0x01` for true.
%%   }
%%   Schema::Bytes => {
An arbitrary byte string.
%%   }
%%   Schema::Enum { variants } => {
An enum. Encoded as an array whose only element is the discriminant of the
variant, encoded as an [UnsignedInteger](unsigned-integer.md).

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
%%   Schema::UnsignedInteger => {
An unsigned 64-bit integer. Encoded as little endian with trailing zero bytes
removed, with a minimum length of one byte.
%%   }
%% }

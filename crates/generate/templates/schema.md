# {{ self.schema.name() }}

%% match self.schema {
%%   Schema::Boolean => {
A boolean. Encoded as the byte string `0x00` for false and `0x01` for true.
%%   }
%%   Schema::Bytes => {
An arbitrary byte string.
%%   }
%%   Schema::U64 => {
An unsigned 64-bit integer. Encoded as little endian with trailing zero bytes
removed, with a minimum length of one byte.
%%   }
%%   Schema::I64 => {
A signed 64-bit integer. First zig-zag encoded, then encoded as a
[u64](u64.md).
%%   }
%%   Schema::String => {
A string. Encoded as a byte string containing the UTF-8 encoding of the string.
%%   }
%% }

use super::*;

// todo:
// - test all error variants
// - encoding test for directory which tests exact byte string
// - any test coverage gaps?
// - anything which is pub(crate) but can be made more private?
// - add a `cbor: &[u8]` argument to the assert_encoding test case generator,
//   and assert that the generated cbor matches a specific byte slice
// - replace the `case` functions in src/cbor/encode.rs with `assert_encoding`
//
// later:
// - test against an existing cbor library

pub(crate) use self::{
  directory::Directory, encode::Encode, encoder::Encoder, entry::Entry, entry_type::EntryType,
  head::Head, major_type::MajorType, map_encoder::MapEncoder, version::Version,
};

#[cfg(test)]
pub(crate) use self::{
  decode::Decode, decode_error::DecodeError, decoder::Decoder, map_decoder::MapDecoder,
};

#[cfg(test)]
mod decode;
#[cfg(test)]
pub(crate) mod decode_error;
#[cfg(test)]
mod decoder;
mod directory;
mod encode;
mod encoder;
mod entry;
mod entry_type;
mod head;
mod major_type;
#[cfg(test)]
mod map_decoder;
mod map_encoder;
mod version;

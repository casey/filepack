use super::*;

pub(crate) trait DecodeOwned: for<'a> Decode<'a> {}

impl<T: for<'a> Decode<'a>> DecodeOwned for T {}

use super::*;

pub(crate) struct ParsedField<'a> {
  pub(crate) decode_with: Option<Path>,
  pub(crate) encode_with: Option<Path>,
  pub(crate) ident: &'a Ident,
  pub(crate) n: u64,
  pub(crate) optional: bool,
}

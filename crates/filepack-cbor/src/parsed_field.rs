use super::*;

pub(crate) struct ParsedField<'a> {
  pub(crate) ident: &'a Ident,
  pub(crate) n: u64,
  pub(crate) optional: bool,
}

use super::*;

pub(crate) struct ParsedVariant<'a> {
  pub(crate) ident: &'a Ident,
  pub(crate) n: u64,
}

use super::*;

pub(crate) struct ParsedVariant<'a> {
  pub(crate) description: String,
  pub(crate) fields: Vec<ParsedField<'a>>,
  pub(crate) ident: &'a Ident,
  pub(crate) n: u64,
}

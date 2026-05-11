#![expect(clippy::needless_continue)]

use super::*;

#[derive(FromVariant)]
#[darling(forward_attrs(n))]
pub(crate) struct Variant {
  attrs: Vec<Attribute>,
  ident: Ident,
}

impl Variant {
  pub(crate) fn parse(&self) -> Result<ParsedVariant> {
    Ok(ParsedVariant {
      ident: &self.ident,
      n: n(&self.ident, &self.attrs)?,
    })
  }
}

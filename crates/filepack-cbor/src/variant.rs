#![allow(clippy::needless_continue)]

use super::*;

#[derive(FromVariant)]
#[darling(forward_attrs(n))]
pub(crate) struct Variant {
  attrs: Vec<Attribute>,
  ident: Ident,
}

impl Variant {
  fn n(&self) -> Result<u64> {
    let mut n = None;

    for attribute in &self.attrs {
      if attribute.path().is_ident("n") {
        if n.is_some() {
          return Err(Error::new_spanned(attribute, "duplicate #[n] attribute"));
        }
        n = Some(attribute.parse_args::<LitInt>()?.base10_parse::<u64>()?);
      }
    }

    n.ok_or_else(|| Error::new_spanned(&self.ident, "missing #[n(N)] attribute"))
  }

  pub(crate) fn parse(&self) -> Result<ParsedVariant> {
    Ok(ParsedVariant {
      ident: &self.ident,
      n: self.n()?,
    })
  }
}

#![allow(clippy::needless_continue)]

use super::*;

#[derive(FromField)]
#[darling(forward_attrs(n))]
pub(crate) struct Field {
  attrs: Vec<Attribute>,
  ident: Option<Ident>,
  ty: Type,
}

impl Field {
  pub(crate) fn ident(&self) -> Option<&Ident> {
    self.ident.as_ref()
  }

  fn is_option(&self) -> bool {
    if let Type::Path(TypePath { qself: None, path }) = &self.ty {
      path.leading_colon.is_none() && path.segments.len() == 1 && path.segments[0].ident == "Option"
    } else {
      false
    }
  }

  fn n(&self) -> Result<u64> {
    let ident = self.ident.as_ref().unwrap();

    let mut n = None;

    for attribute in &self.attrs {
      if attribute.path().is_ident("n") {
        if n.is_some() {
          return Err(Error::new_spanned(attribute, "duplicate #[n] attribute"));
        }
        n = Some(attribute.parse_args::<LitInt>()?.base10_parse::<u64>()?);
      }
    }

    n.ok_or_else(|| Error::new_spanned(ident, "missing #[n(N)] attribute"))
  }

  pub(crate) fn parse(&self) -> Result<ParsedField> {
    Ok(ParsedField {
      ident: self.ident.as_ref().unwrap(),
      n: self.n()?,
      optional: self.is_option(),
    })
  }
}

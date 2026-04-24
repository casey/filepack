use super::*;

#[derive(FromField)]
#[darling(forward_attrs(n))]
pub(crate) struct Field {
  attrs: Vec<Attribute>,
  ident: Option<Ident>,
  ty: Type,
}

impl Field {
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

    for attr in &self.attrs {
      if attr.path().is_ident("n") {
        if n.is_some() {
          return Err(Error::new_spanned(attr, "duplicate #[n] attribute"));
        }
        let lit = attr.parse_args::<LitInt>()?;
        n = Some(lit.base10_parse::<u64>()?);
      }
    }

    n.ok_or_else(|| Error::new_spanned(ident, "missing #[n(N)] attribute"))
  }

  pub(crate) fn parse(&self) -> Result<ParsedField> {
    let ident = self.ident.as_ref().unwrap();
    let n = self.n()?;
    let optional = self.is_option();
    Ok(ParsedField { ident, n, optional })
  }
}

use super::*;

#[derive(FromField)]
#[darling(forward_attrs(deco, n))]
pub(crate) struct Field {
  attrs: Vec<Attribute>,
  ident: Option<Ident>,
  ty: Type,
}

impl Field {
  pub(crate) fn deco_attribute(&self) -> Option<&Attribute> {
    self.attrs.iter().find(|attr| attr.path().is_ident("deco"))
  }

  pub(crate) fn ident(&self) -> Option<&Ident> {
    self.ident.as_ref()
  }

  fn is_option(&self) -> bool {
    if let Type::Path(TypePath {
      attrs: _,
      path,
      qself: None,
    }) = &self.ty
    {
      path.leading_colon.is_none() && path.segments.len() == 1 && path.segments[0].ident == "Option"
    } else {
      false
    }
  }

  pub(crate) fn n_attribute(&self) -> Option<&Attribute> {
    self.attrs.iter().find(|attr| attr.path().is_ident("n"))
  }

  pub(crate) fn parse(&self) -> Result<ParsedField> {
    if let Some(attribute) = self.deco_attribute() {
      return Err(Error::new_spanned(
        attribute,
        "`#[deco(...)]` attributes cannot be used on fields",
      ));
    }

    let ident = self.ident.as_ref().unwrap();

    let n = number(ident, &self.attrs)?;

    if n == 0 {
      return Err(Error::new_spanned(
        self.n_attribute().unwrap(),
        "`#[n(0)]` is reserved for the version key",
      ));
    }

    Ok(ParsedField {
      ident,
      n,
      optional: self.is_option(),
    })
  }
}

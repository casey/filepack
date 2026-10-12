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

  pub(crate) fn n_attribute(&self) -> Option<&Attribute> {
    self.attrs.iter().find(|attr| attr.path().is_ident("n"))
  }

  fn option_inner(&self) -> Option<&Type> {
    if let Type::Path(TypePath {
      attrs: _,
      path,
      qself: None,
    }) = &self.ty
      && path.leading_colon.is_none()
      && path.segments.len() == 1
      && path.segments[0].ident == "Option"
      && let PathArguments::AngleBracketed(arguments) = &path.segments[0].arguments
      && arguments.args.len() == 1
      && let GenericArgument::Type(ty) = &arguments.args[0]
    {
      Some(ty)
    } else {
      None
    }
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

    let ty = self.option_inner();

    Ok(ParsedField {
      ident,
      n,
      optional: ty.is_some(),
      ty: ty.unwrap_or(&self.ty),
    })
  }
}

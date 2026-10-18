use super::*;

#[derive(FromField)]
#[darling(forward_attrs(deco, doc, n))]
pub(crate) struct Field {
  attrs: Vec<Attribute>,
  ident: Option<Ident>,
  ty: Type,
}

impl Field {
  pub(crate) fn deco_attribute(&self) -> Option<&Attribute> {
    self.attrs.iter().find(|attr| attr.path().is_ident("deco"))
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

    let mut doc = Vec::new();

    for attribute in &self.attrs {
      if let Some(line) = doc_line(attribute)? {
        doc.push(line);
      }
    }

    let Some(description) = description(doc) else {
      return Err(Error::new_spanned(ident, "fields must have doc comments"));
    };

    let ty = type_argument(&self.ty, "Option");

    Ok(ParsedField {
      description,
      ident,
      n,
      optional: ty.is_some(),
      ty: ty.unwrap_or(&self.ty),
    })
  }

  pub(crate) fn ty(&self) -> &Type {
    &self.ty
  }
}

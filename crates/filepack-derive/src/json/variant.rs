use super::*;

#[derive(FromVariant)]
#[darling(forward_attrs(serde))]
pub(crate) struct Variant {
  attrs: Vec<Attribute>,
  fields: Fields<Field>,
  ident: Ident,
}

impl Variant {
  fn check_attributes(&self) -> Result {
    if let Some(attribute) = self.attrs.first() {
      return Err(Error::new_spanned(
        attribute,
        "`#[serde]` attributes are not supported on variants",
      ));
    }

    Ok(())
  }

  pub(crate) fn is_unit(&self) -> bool {
    self.fields.is_unit()
  }

  pub(crate) fn name(&self, rename_all: Option<RenameAll>) -> Result<String> {
    self.check_attributes()?;

    let ident = self.ident.to_string();

    Ok(rename_all.map_or(ident.clone(), |rename_all| rename_all.apply(&ident)))
  }

  pub(crate) fn schema(&self, rename_all: Option<RenameAll>) -> Result<TokenStream> {
    let name = self.name(rename_all)?;

    let fields = self
      .fields
      .iter()
      .map(|field| field.schema(None))
      .collect::<Result<Vec<TokenStream>>>()?;

    Ok(quote! {
      json::Variant {
        fields: vec![#(#fields),*],
        name: #name.into(),
      }
    })
  }

  pub(crate) fn ty(&self) -> Result<TokenStream> {
    self.check_attributes()?;

    if !(self.fields.is_tuple() && self.fields.len() == 1) {
      return Err(Error::new_spanned(
        &self.ident,
        "untagged enum variants must be newtypes",
      ));
    }

    self.fields.iter().next().unwrap().ty()
  }
}

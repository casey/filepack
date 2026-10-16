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

  pub(crate) fn schema(&self, rename_all: Option<RenameAll>) -> Result<TokenStream> {
    self.check_attributes()?;

    let ident = self.ident.to_string();

    let name = rename_all.map_or_else(|| ident.clone(), |rename_all| rename_all.apply(&ident));

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

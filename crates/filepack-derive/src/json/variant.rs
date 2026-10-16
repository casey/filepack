use super::*;

#[derive(FromVariant)]
#[darling(forward_attrs(serde))]
pub(crate) struct Variant {
  attrs: Vec<Attribute>,
  fields: Fields<Field>,
  ident: Ident,
}

impl Variant {
  pub(crate) fn schema(&self, rename_all: Option<RenameAll>) -> Result<proc_macro2::TokenStream> {
    if let Some(attribute) = self.attrs.first() {
      return Err(Error::new_spanned(
        attribute,
        "`#[serde]` attributes are not supported on variants",
      ));
    }

    let ident = self.ident.to_string();

    let name = rename_all.map_or_else(|| ident.clone(), |rename_all| rename_all.apply(&ident));

    let fields = self
      .fields
      .iter()
      .map(|field| field.schema(None))
      .collect::<Result<Vec<proc_macro2::TokenStream>>>()?;

    Ok(quote! {
      json::Variant {
        fields: vec![#(#fields),*],
        name: #name.into(),
      }
    })
  }
}

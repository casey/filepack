use super::*;

#[derive(FromField)]
#[darling(forward_attrs(serde))]
pub(crate) struct Field {
  attrs: Vec<Attribute>,
  ident: Option<Ident>,
  ty: Type,
}

impl Field {
  pub(crate) fn schema(&self, rename_all: Option<RenameAll>) -> Result<proc_macro2::TokenStream> {
    if let Some(attribute) = self.attrs.first() {
      return Err(Error::new_spanned(
        attribute,
        "`#[serde]` attributes are not supported on fields",
      ));
    }

    let ident = self.ident.as_ref().unwrap().to_string();

    let name = rename_all.map_or_else(|| ident.clone(), |rename_all| rename_all.apply(&ident));

    let inner = option_inner(&self.ty);

    let optional = inner.is_some();

    let ty = inner.unwrap_or(&self.ty);

    Ok(quote! {
      yaml::Field {
        name: #name.into(),
        optional: #optional,
        schema: <#ty>::yaml_schema(registry),
      }
    })
  }
}

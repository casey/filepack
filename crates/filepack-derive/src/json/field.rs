use super::*;

const WITH: &[&str] = &[
  "::serde_with::As::<MapPreventDuplicates<::serde_with::Same,::serde_with::Same>>",
  "::serde_with::As::<SetPreventDuplicates<serde_with::Same>>",
];

#[derive(FromField)]
#[darling(forward_attrs(serde))]
pub(crate) struct Field {
  attrs: Vec<Attribute>,
  ident: Option<Ident>,
  ty: Type,
}

impl Field {
  fn json_schema(&self) -> TokenStream {
    let ty = option_inner(&self.ty).unwrap_or(&self.ty);
    quote! { <#ty>::json_schema(registry) }
  }

  fn parse(&self) -> Result<bool> {
    let option = option_inner(&self.ty).is_some();

    let mut default = false;

    let mut skip = false;

    for attribute in &self.attrs {
      attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("default") {
          default = true;
        } else if meta.path.is_ident("skip_serializing_if") {
          meta.value()?.parse::<LitStr>()?;
          skip = true;
        } else if meta.path.is_ident("with") {
          let with = meta
            .value()?
            .parse::<LitStr>()?
            .value()
            .split_whitespace()
            .collect::<String>();
          if !WITH.contains(&with.as_str()) {
            return Err(meta.error(format!(
              "unsupported `#[serde(with = \"{with}\")]` attribute"
            )));
          }
        } else {
          let ident = meta.path.require_ident()?;
          return Err(meta.error(format!("unsupported `#[serde({ident})]` attribute")));
        }

        Ok(())
      })?;
    }

    if option && !skip {
      return Err(Error::new_spanned(
        &self.ty,
        "`Option` fields must have `#[serde(skip_serializing_if)]` or the type must have `#[skip_serializing_none]`",
      ));
    }

    Ok(option || default || skip)
  }

  pub(crate) fn schema(&self, rename_all: Option<RenameAll>) -> Result<TokenStream> {
    let optional = self.parse()?;

    let ident = self.ident.as_ref().unwrap().to_string();

    let name = rename_all.map_or_else(|| ident.clone(), |rename_all| rename_all.apply(&ident));

    let ty = self.json_schema();

    Ok(quote! {
      json::Field {
        name: #name.into(),
        optional: #optional,
        schema: #ty,
      }
    })
  }

  pub(crate) fn ty(&self) -> Result<TokenStream> {
    if option_inner(&self.ty).is_some() || self.parse()? {
      return Err(Error::new_spanned(
        &self.ty,
        "transparent struct fields and untagged enum variants cannot be optional",
      ));
    }
    Ok(self.json_schema())
  }
}

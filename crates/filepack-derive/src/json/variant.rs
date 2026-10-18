use super::*;

#[derive(FromVariant)]
#[darling(forward_attrs(doc, serde))]
pub(crate) struct Variant {
  attrs: Vec<Attribute>,
  fields: Fields<Field>,
  ident: Ident,
}

impl Variant {
  fn check_attributes(&self) -> Result<ParsedVariant> {
    let mut doc = Vec::new();

    let mut rename = None;

    for attribute in &self.attrs {
      if let Some(line) = doc_line(attribute)? {
        doc.push(line);
        continue;
      }

      attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("rename") {
          rename = Some(meta.value()?.parse::<LitStr>()?.value());
        } else {
          let ident = meta.path.require_ident()?;
          return Err(meta.error(format!("unsupported `#[serde({ident})]` attribute")));
        }

        Ok(())
      })?;
    }

    Ok(ParsedVariant {
      description: description(doc),
      rename,
    })
  }

  pub(crate) fn is_unit(&self) -> bool {
    self.fields.is_unit()
  }

  pub(crate) fn name(&self, rename_all: Option<RenameAll>) -> Result<String> {
    if let Some(rename) = self.check_attributes()?.rename {
      return Ok(rename);
    }

    let ident = self.ident.to_string();

    Ok(rename_all.map_or(ident.clone(), |rename_all| rename_all.apply(&ident)))
  }

  pub(crate) fn schema(&self, rename_all: Option<RenameAll>) -> Result<TokenStream> {
    let name = self.name(rename_all)?;

    let Some(description) = self.check_attributes()?.description else {
      return Err(Error::new_spanned(
        &self.ident,
        "variants must have doc comments",
      ));
    };

    let fields = self
      .fields
      .iter()
      .map(|field| field.schema(None))
      .collect::<Result<Vec<TokenStream>>>()?;

    Ok(quote! {
      json::Variant {
        description: #description.into(),
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

use super::*;

const WITH: &[&str] = &[
  "::serde_with::As::<MapPreventDuplicates<::serde_with::Same,::serde_with::Same>>",
  "::serde_with::As::<SetPreventDuplicates<serde_with::Same>>",
];

#[derive(FromField)]
#[darling(forward_attrs(doc, json, serde))]
pub(crate) struct Field {
  attrs: Vec<Attribute>,
  ident: Option<Ident>,
  ty: Type,
}

impl Field {
  fn json_schema(&self) -> TokenStream {
    let ty = type_argument(&self.ty, "Option").unwrap_or(&self.ty);
    quote! { <#ty>::json_schema(registry) }
  }

  fn parse(&self) -> Result<ParsedField> {
    let option = type_argument(&self.ty, "Option").is_some();

    let mut default = false;

    let mut doc = Vec::new();

    let mut example = None;

    let mut skip = false;

    for attribute in &self.attrs {
      if let Some(line) = doc_line(attribute)? {
        doc.push(line);
        continue;
      }

      if attribute.path().is_ident("json") {
        attribute.parse_nested_meta(|meta| {
          if meta.path.is_ident("example") {
            example = Some(meta.value()?.parse::<LitStr>()?.value());
          } else {
            let ident = meta.path.require_ident()?;
            return Err(meta.error(format!("unsupported `#[json({ident})]` attribute")));
          }

          Ok(())
        })?;

        continue;
      }

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

    Ok(ParsedField {
      description: description(doc),
      example,
      optional: option || default || skip,
    })
  }

  pub(crate) fn schema(&self, rename_all: Option<RenameAll>) -> Result<TokenStream> {
    let ParsedField {
      description,
      example,
      optional,
    } = self.parse()?;

    let ident = self.ident.as_ref().unwrap();

    let Some(description) = description else {
      return Err(Error::new_spanned(ident, "fields must have doc comments"));
    };

    let example = option(example);

    let ident = ident.to_string();

    let name = rename_all.map_or_else(|| ident.clone(), |rename_all| rename_all.apply(&ident));

    let ty = self.json_schema();

    Ok(quote! {
      json::Field {
        description: #description.into(),
        example: #example,
        name: #name.into(),
        optional: #optional,
        schema: #ty,
      }
    })
  }

  pub(crate) fn ty(&self) -> Result<TokenStream> {
    if type_argument(&self.ty, "Option").is_some() || self.parse()?.optional {
      return Err(Error::new_spanned(
        &self.ty,
        "transparent struct fields and untagged enum variants cannot be optional",
      ));
    }
    Ok(self.json_schema())
  }
}

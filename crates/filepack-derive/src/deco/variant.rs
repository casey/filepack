use super::*;

#[derive(FromVariant)]
#[darling(forward_attrs(deco, doc, n))]
pub(crate) struct Variant {
  attrs: Vec<Attribute>,
  fields: Fields<Field>,
  ident: Ident,
}

impl Variant {
  pub(crate) fn parse(&self) -> Result<ParsedVariant> {
    if let Some(attr) = self.attrs.iter().find(|attr| attr.path().is_ident("deco")) {
      return Err(Error::new_spanned(
        attr,
        "`#[deco(...)]` attributes cannot be used on enum variants",
      ));
    }

    let fields = self
      .fields
      .iter()
      .map(Field::parse)
      .collect::<Result<Vec<ParsedField>>>()?;

    validate_numbers(1, fields.iter().map(|field| (field.ident, field.n)))?;

    let n = number(&self.ident, &self.attrs)?;

    let mut doc = Vec::new();

    for attribute in &self.attrs {
      if let Some(line) = doc_line(attribute)? {
        doc.push(line);
      }
    }

    let Some(description) = description(doc) else {
      return Err(Error::new_spanned(
        &self.ident,
        "variants must have doc comments",
      ));
    };

    Ok(ParsedVariant {
      description,
      fields,
      ident: &self.ident,
      n,
    })
  }
}

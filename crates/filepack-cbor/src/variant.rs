#![expect(clippy::needless_continue)]

use super::*;

#[derive(FromVariant)]
#[darling(forward_attrs(n))]
pub(crate) struct Variant {
  attrs: Vec<Attribute>,
  fields: Fields<Field>,
  ident: Ident,
}

impl Variant {
  pub(crate) fn parse(&self) -> Result<ParsedVariant> {
    let fields = self
      .fields
      .iter()
      .map(Field::parse)
      .collect::<Result<Vec<ParsedField>>>()?;

    validate_numbers(fields.iter().map(|field| (field.ident, field.n)))?;

    Ok(ParsedVariant {
      fields,
      ident: &self.ident,
      n: number(&self.ident, &self.attrs)?,
    })
  }
}

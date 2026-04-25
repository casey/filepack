#![allow(clippy::needless_continue)]

use super::*;

#[derive(FromField)]
#[darling(forward_attrs(cbor, n))]
pub(crate) struct Field {
  attrs: Vec<Attribute>,
  ident: Option<Ident>,
  ty: Type,
}

impl Field {
  pub(crate) fn ident(&self) -> Option<&Ident> {
    self.ident.as_ref()
  }

  fn is_option(&self) -> bool {
    if let Type::Path(TypePath { qself: None, path }) = &self.ty {
      path.leading_colon.is_none() && path.segments.len() == 1 && path.segments[0].ident == "Option"
    } else {
      false
    }
  }

  fn n(&self) -> Result<u64> {
    let ident = self.ident.as_ref().unwrap();

    let mut n = None;

    for attribute in &self.attrs {
      if attribute.path().is_ident("n") {
        if n.is_some() {
          return Err(Error::new_spanned(attribute, "duplicate #[n] attribute"));
        }
        n = Some(attribute.parse_args::<LitInt>()?.base10_parse::<u64>()?);
      }
    }

    n.ok_or_else(|| Error::new_spanned(ident, "missing #[n(N)] attribute"))
  }

  pub(crate) fn parse(&self) -> Result<ParsedField> {
    let (decode_with, encode_with) = self.parse_attributes()?;

    Ok(ParsedField {
      decode_with,
      encode_with,
      ident: self.ident.as_ref().unwrap(),
      n: self.n()?,
      optional: self.is_option(),
    })
  }

  fn parse_attributes(&self) -> Result<(Option<Path>, Option<Path>)> {
    let mut decode_with = None;
    let mut encode_with = None;

    for attribute in &self.attrs {
      if !attribute.path().is_ident("cbor") {
        continue;
      }

      attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("decode_with") {
          if decode_with.is_some() {
            return Err(meta.error("duplicate `decode_with` attribute"));
          }
          decode_with = Some(meta.value()?.parse::<Path>()?);
          Ok(())
        } else if meta.path.is_ident("encode_with") {
          if encode_with.is_some() {
            return Err(meta.error("duplicate `encode_with` attribute"));
          }
          encode_with = Some(meta.value()?.parse::<Path>()?);
          Ok(())
        } else {
          Err(meta.error("unknown cbor attribute"))
        }
      })?;
    }

    Ok((decode_with, encode_with))
  }
}

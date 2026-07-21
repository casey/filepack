use super::*;

pub(crate) struct Attributes {
  pub(crate) transparent: bool,
  pub(crate) validate: bool,
}

impl Attributes {
  pub(crate) fn parse(attributes: &[Attribute]) -> Result<Self> {
    let mut transparent = false;
    let mut validate = false;

    for attribute in attributes {
      if !attribute.path().is_ident("cbor") {
        continue;
      }

      attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("transparent") {
          if transparent {
            return Err(meta.error("duplicate `transparent` attribute"));
          }
          transparent = true;
          Ok(())
        } else if meta.path.is_ident("validate") {
          if validate {
            return Err(meta.error("duplicate `validate` attribute"));
          }
          validate = true;
          Ok(())
        } else {
          Err(meta.error("unknown cbor attribute"))
        }
      })?;
    }

    Ok(Self {
      transparent,
      validate,
    })
  }
}

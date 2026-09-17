use super::*;

pub(crate) struct Attributes {
  pub(crate) transparent: bool,
  pub(crate) validate: bool,
}

impl Attributes {
  pub(crate) fn parse(attributes: &[Attribute]) -> Result<Self> {
    let mut container_attributes = HashSet::new();

    for attribute in attributes {
      if !attribute.path().is_ident("deco") {
        continue;
      }

      attribute.parse_nested_meta(|meta| {
        let attribute = meta
          .path
          .require_ident()?
          .to_string()
          .parse::<ContainerAttribute>()
          .map_err(|_| meta.error("unknown deco attribute"))?;

        if !container_attributes.insert(attribute) {
          return Err(meta.error("duplicate `{attribute}` attribute"));
        }

        Ok(())
      })?;
    }

    Ok(Self {
      transparent: container_attributes.contains(&ContainerAttribute::Transparent),
      validate: container_attributes.contains(&ContainerAttribute::Validate),
    })
  }
}

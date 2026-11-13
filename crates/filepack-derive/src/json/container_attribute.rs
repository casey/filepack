use super::*;

#[derive(IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum ContainerAttribute {
  DenyUnknownFields,
  RenameAll(RenameAll),
  Tag(String),
  Transparent,
  Untagged,
}

impl ContainerAttribute {
  pub(crate) fn parse(meta: &ParseNestedMeta) -> Result<Self> {
    let ident = meta.path.require_ident()?;

    match ident.to_string().as_str() {
      "deny_unknown_fields" => Ok(Self::DenyUnknownFields),
      "rename_all" => {
        let value = meta.value()?.parse::<LitStr>()?.value();

        value
          .parse()
          .map(Self::RenameAll)
          .map_err(|_| meta.error(format!("unsupported `rename_all` value `{value}`")))
      }
      "tag" => Ok(Self::Tag(meta.value()?.parse::<LitStr>()?.value())),
      "transparent" => Ok(Self::Transparent),
      "untagged" => Ok(Self::Untagged),
      _ => Err(meta.error(format!("unsupported `#[serde({ident})]` attribute"))),
    }
  }
}

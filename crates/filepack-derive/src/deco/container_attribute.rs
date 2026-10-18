use super::*;

#[derive(IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum ContainerAttribute {
  Magic(Path),
  NonEmptyOrderedSet,
  SortedSet,
  Strict,
  Transparent,
  Undocumented,
  Validate,
}

impl ContainerAttribute {
  pub(crate) fn name(&self) -> &'static str {
    self.into()
  }

  pub(crate) fn parse(meta: &ParseNestedMeta) -> Result<Self> {
    let ident = meta.path.require_ident()?;

    let attribute = match ident.to_string().as_str() {
      "magic" => return Ok(Self::Magic(path_value(meta, "MagicType::Variant")?)),
      "non_empty_ordered_set" => Self::NonEmptyOrderedSet,
      "sorted_set" => Self::SortedSet,
      "strict" => Self::Strict,
      "transparent" => Self::Transparent,
      "undocumented" => Self::Undocumented,
      "validate" => Self::Validate,
      _ => {
        return Err(meta.error(format!("unknown container attribute `#[deco({ident})]`")));
      }
    };

    if !meta.input.is_empty() && !meta.input.peek(Token![,]) {
      return Err(meta.error(format!("`#[deco({ident})]` does not take a value")));
    }

    Ok(attribute)
  }
}

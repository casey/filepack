use super::*;

pub(crate) fn display_name(ident: &Ident) -> String {
  ident.to_string().to_snake_case().replace('_', " ")
}

pub(crate) fn number(ident: &Ident, attributes: &[Attribute]) -> Result<u64> {
  let mut n = None;

  for attribute in attributes {
    if attribute.path().is_ident("n") {
      if n.is_some() {
        return Err(Error::new_spanned(attribute, "duplicate `#[n]` attribute"));
      }
      n = Some(attribute.parse_args::<LitInt>()?.base10_parse::<u64>()?);
    }
  }

  n.ok_or_else(|| Error::new_spanned(ident, "missing `#[n(N)]` attribute"))
}

pub(crate) fn option(value: Option<String>) -> TokenStream {
  if let Some(value) = value {
    quote! { Some(#value.into()) }
  } else {
    quote! { None }
  }
}

pub(crate) fn path_value(meta: &ParseNestedMeta, placeholder: &str) -> Result<Path> {
  let name = meta.path.require_ident()?;
  meta.value().and_then(ParseBuffer::parse).map_err(|_| {
    meta.error(format!(
      "`#[deco({name})]` must be of the form `#[deco({name} = {placeholder})]`"
    ))
  })
}

pub(crate) fn type_argument<'a>(ty: &'a Type, name: &str) -> Option<&'a Type> {
  if let Type::Path(TypePath {
    attrs: _,
    path,
    qself: None,
  }) = ty
    && path.leading_colon.is_none()
    && path.segments.len() == 1
    && path.segments[0].ident == name
    && let PathArguments::AngleBracketed(arguments) = &path.segments[0].arguments
    && arguments.args.len() == 1
    && let GenericArgument::Type(ty) = &arguments.args[0]
  {
    Some(ty)
  } else {
    None
  }
}

pub(crate) fn validate_numbers<'a>(
  start: u64,
  numbers: impl IntoIterator<Item = (&'a Ident, u64)>,
) -> Result {
  let mut seen = HashSet::new();

  for (i, (ident, n)) in numbers.into_iter().enumerate() {
    if !seen.insert(n) {
      return Err(Error::new_spanned(
        ident,
        format!("duplicate `#[n({n})]` attribute"),
      ));
    }

    let expected = start + i.into_u64();

    if n != expected {
      return Err(Error::new_spanned(
        ident,
        format!(
          "`#[n]` attributes must be contiguous starting from {start}: expected {expected}, found {n}"
        ),
      ));
    }
  }

  Ok(())
}

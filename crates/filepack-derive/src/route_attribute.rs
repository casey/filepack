use super::*;

pub(crate) struct RouteAttribute {
  pub(crate) method: Ident,
  pub(crate) path: LitStr,
}

impl Parse for RouteAttribute {
  fn parse(input: ParseStream) -> Result<Self> {
    let method = input.parse()?;
    input.parse::<Token![,]>()?;
    let path = input.parse()?;
    Ok(Self { method, path })
  }
}

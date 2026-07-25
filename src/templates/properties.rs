use super::*;

#[derive(Boilerplate)]
pub(crate) struct PropertiesHtml<'a> {
  pub(crate) properties: &'a [(String, Value)],
}

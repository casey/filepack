use super::*;

#[derive(Boilerplate)]
pub struct PageHtml<T: Page> {
  pub(crate) content: T,
}

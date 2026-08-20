use super::*;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum PathError {
  #[snafu(display("path contains invalid component `{component}`"))]
  Component {
    component: String,
    source: ComponentError,
  },
  #[snafu(display("paths may not contain double slashes"))]
  DoubleSlash,
  #[snafu(display("paths may not be empty"))]
  Empty,
  #[snafu(display(
    "path must end in {}",
    Or::new(extensions.iter().map(|extension| format!("`.{extension}`"))),
  ))]
  Extension { extensions: &'static [&'static str] },
  #[snafu(display("paths may not begin with slash character"))]
  LeadingSlash,
  #[snafu(display("path may not contain non-normal component `{component}`"))]
  Normal { component: String },
  #[snafu(display("paths may not end with slash character"))]
  TrailingSlash,
}

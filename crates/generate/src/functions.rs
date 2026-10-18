use super::*;

pub(crate) fn escape(name: &str) -> String {
  name
    .replace('*', "\\*")
    .replace('<', "\\<")
    .replace('>', "\\>")
    .replace('{', "\\{")
    .replace('}', "\\}")
}

pub(crate) fn list(items: impl IntoIterator<Item = impl Display>) -> String {
  items
    .into_iter()
    .map(|item| format!("`{item}`"))
    .collect::<Vec<String>>()
    .join(", ")
}

pub(crate) fn parameter(parameter: Parameter) -> &'static str {
  match parameter {
    Parameter::Fingerprint => "[package fingerprint](../hex/package.md)",
    Parameter::Hash => "[hash](../hex/hash.md)",
    Parameter::Integer => "unsigned integer",
    Parameter::Ordinal => "one-based index",
    Parameter::PackageIdentifier => {
      "package [fingerprint](../hex/package.md), [revision](../hex/revision.md), or number"
    }
    Parameter::RelativePath => "relative path",
    Parameter::Revision => "[revision](../hex/revision.md)",
    Parameter::String => "string",
  }
}

pub(crate) fn route_link(path: &str, page: &Utf8Path) -> String {
  format!("[<code>{}</code>]({page})", route_path(path))
}

pub(crate) fn route_path(path: &str) -> String {
  let (first, rest) = path.split_at(1);
  format!("{first}{}", escape(rest).replace('/', "<wbr>/"))
}

pub(crate) fn write(path: impl AsRef<Utf8Path>, contents: impl Display) {
  let path = Utf8Path::new("book/src").join(path);
  fs::create_dir_all(path.parent().unwrap()).unwrap();
  fs::write(path, contents.to_string()).unwrap();
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn route_link() {
    assert_eq!(
      super::route_link("/foo/{bar}/{*baz}", Utf8Path::new("routes/foo.md")),
      "[<code>/foo<wbr>/\\{bar\\}<wbr>/\\{\\*baz\\}</code>](routes/foo.md)",
    );
  }
}

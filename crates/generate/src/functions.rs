use super::*;

pub(crate) fn deco(name: &str) -> String {
  format!("[{}](../deco/{}.md)", escape(name), name.to_kebab_case())
}

pub(crate) fn escape(name: &str) -> String {
  name
    .replace('*', "\\*")
    .replace('<', "\\<")
    .replace('>', "\\>")
    .replace('{', "\\{")
    .replace('}', "\\}")
}

pub(crate) fn link(chapter: Chapter, name: &str) -> String {
  format!(
    "[{}](../{chapter}/{}.md)",
    escape(name),
    name.to_kebab_case()
  )
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

pub(crate) fn write(path: impl AsRef<Utf8Path>, contents: impl Display) {
  let path = path.as_ref();
  fs::create_dir_all(path.parent().unwrap()).unwrap();
  let mut file = BufWriter::new(File::create(path).unwrap());
  write!(file, "{contents}").unwrap();
}

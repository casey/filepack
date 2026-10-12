use super::*;

pub(crate) fn escape(name: &str) -> String {
  name
    .replace('*', "\\*")
    .replace('<', "\\<")
    .replace('>', "\\>")
    .replace('{', "\\{")
    .replace('}', "\\}")
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

pub(crate) fn schema(name: &str) -> String {
  format!("[{}](../schemas/{}.md)", escape(name), name.to_kebab_case())
}

pub(crate) fn write(path: impl AsRef<Utf8Path>, contents: impl Display) {
  let path = path.as_ref();
  fs::create_dir_all(path.parent().unwrap()).unwrap();
  let mut file = BufWriter::new(File::create(path).unwrap());
  write!(file, "{contents}").unwrap();
}

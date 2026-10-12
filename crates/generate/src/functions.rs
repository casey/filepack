use super::*;

pub(crate) fn escape(name: &str) -> String {
  name
    .replace('<', "\\<")
    .replace('>', "\\>")
    .replace('*', "\\*")
}

pub(crate) fn link(name: &str) -> String {
  format!("[{}]({}.md)", escape(name), name.to_kebab_case())
}

pub(crate) fn write(path: impl AsRef<Utf8Path>, contents: impl Display) {
  let path = path.as_ref();
  fs::create_dir_all(path.parent().unwrap()).unwrap();
  let mut file = BufWriter::new(File::create(path).unwrap());
  write!(file, "{contents}").unwrap();
}

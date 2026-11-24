use super::*;

pub(crate) trait Format: Copy + Display + PartialEq {
  const EXTENSIONS: &'static [&'static str];

  fn extension(self) -> &'static str;

  fn filename(self, name: &str) -> String {
    let extension = self.extension();

    if name
      .strip_suffix(extension)
      .is_some_and(|prefix| prefix.ends_with('.'))
    {
      name.into()
    } else {
      format!("{name}.{extension}")
    }
  }

  fn from_extension(extension: &str) -> Option<Self>;

  fn from_path(path: &RelativePath) -> Result<Self, PathError> {
    path
      .extension()
      .and_then(Self::from_extension)
      .ok_or(PathError::Extension {
        extensions: Self::EXTENSIONS,
      })
  }

  fn resource_type(self) -> ResourceType;
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn filename() {
    #[track_caller]
    fn case(name: &str, expected: &str) {
      assert_eq!(TextFormat::Markdown.filename(name), expected);
    }

    case("foo", "foo.md");
    case("foo.md", "foo.md");
    case("foo.txt", "foo.txt.md");
    case("md", "md.md");
  }
}

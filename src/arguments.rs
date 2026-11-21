use {
  super::*,
  clap::builder::{
    Styles,
    styling::{AnsiColor, Effects},
  },
};

#[derive(Parser)]
#[command(
  version,
  about = "Content-addressed packages",
  long_about = "Content-addressed packages - https://github.com/casey/filepack",
  color = Style::color_choice(),
  styles = Styles::styled()
    .header(AnsiColor::Green.on_default() | Effects::BOLD)
    .usage(AnsiColor::Green.on_default() | Effects::BOLD)
    .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
    .placeholder(AnsiColor::Cyan.on_default()))
]
pub(crate) struct Arguments {
  #[command(flatten)]
  options: Options,
  #[command(subcommand)]
  subcommand: Subcommand,
}

impl Arguments {
  pub(crate) fn run(self) -> Result {
    self.subcommand.run(self.options)
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    clap::error::ErrorKind,
    pulldown_cmark::{CodeBlockKind, Event, TagEnd},
  };

  #[test]
  fn book_commands_are_valid() {
    let root = Utf8Path::new(env!("CARGO_MANIFEST_DIR"));

    let mut paths = root
      .join("book/src")
      .read_dir_utf8()
      .unwrap()
      .map(|entry| entry.unwrap().into_path())
      .filter(|path| path.extension() == Some("md"))
      .collect::<Vec<Utf8PathBuf>>();

    paths.push(root.join("README.md"));

    let mut commands = Vec::new();

    for path in paths {
      let markdown = fs::read_to_string(&path).unwrap();

      let mut block = None::<String>;

      for event in pulldown_cmark::Parser::new(&markdown) {
        match event {
          Event::Start(pulldown_cmark::Tag::CodeBlock(CodeBlockKind::Fenced(language)))
            if &*language == "sh" =>
          {
            block = Some(String::new());
          }
          Event::Text(text) => {
            if let Some(block) = &mut block {
              block.push_str(&text);
            }
          }
          Event::End(TagEnd::CodeBlock) => {
            for line in block.take().unwrap_or_default().lines() {
              if line == "filepack" || line.starts_with("filepack ") {
                commands.push((path.clone(), line.to_owned()));
              }
            }
          }
          _ => {}
        }
      }
    }

    assert!(!commands.is_empty());

    for (path, command) in commands {
      let arguments = command
        .split(' ')
        .map(|word| match word {
          "<DIRECTORY>" | "<KEY>" | "<SERVER>" => "foo",
          "<FINGERPRINT>" | "<PACKAGE>" => Fingerprint::EXAMPLE,
          "<PACKAGE_NUMBER>" => "0",
          "<PUBLIC_KEY>" => PublicKey::EXAMPLE,
          "<REVISION>" => RevisionHash::EXAMPLE,
          _ => {
            assert!(
              word
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "-.:".contains(c)),
              "unsupported word `{word}` in `{command}` in `{path}`",
            );
            word
          }
        })
        .collect::<Vec<&str>>();

      if let Err(err) = Arguments::try_parse_from(arguments)
        && err.kind() != ErrorKind::DisplayHelp
      {
        panic!("invalid command `{command}` in `{path}`: {err}");
      }
    }
  }
}

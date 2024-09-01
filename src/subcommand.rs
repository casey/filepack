use {
  super::*,
  clap::builder::{
    styling::{AnsiColor, Effects},
    Styles,
  },
};

#[derive(Parser)]
#[command(
  version,
  styles = Styles::styled()
    .header(AnsiColor::Green.on_default() | Effects::BOLD)
    .usage(AnsiColor::Green.on_default() | Effects::BOLD)
    .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
    .placeholder(AnsiColor::Cyan.on_default()))
]
pub(crate) enum Subcommand {
  Create { root: Utf8PathBuf },
}

impl Subcommand {
  pub(crate) fn run(self) -> Result {
    match self {
      Self::Create { root } => Self::create(&root),
    }
  }

  fn create(root: &Utf8Path) -> Result {
    let mut files = HashMap::new();

    for entry in WalkDir::new(&root) {
      let entry = entry?;

      if entry.file_type().is_dir() {
        continue;
      }

      let path = entry.path();

      let path = Utf8Path::from_path(path).context(error::Path { path })?;

      if entry.file_type().is_symlink() {
        return Err(error::Symlink { path }.build());
      }

      let file = File::open(path).context(error::Io { path })?;

      let mut hasher = Hasher::new();

      hasher.update_reader(file).context(error::Io { path })?;

      files.insert(
        path.strip_prefix(&root).unwrap().into(),
        hasher.finalize().into(),
      );
    }

    let filepack = Filepack { files };

    eprintln!("{}", serde_json::to_string_pretty(&filepack).unwrap());

    Ok(())
  }
}

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
  Verify { root: Utf8PathBuf },
}

impl Subcommand {
  pub(crate) fn run(self) -> Result {
    match self {
      Self::Create { root } => Self::create(&root),
      Self::Verify { root } => Self::verify(&root),
    }
  }

  fn create(root: &Utf8Path) -> Result {
    let mut files = HashMap::new();

    for entry in WalkDir::new(root) {
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
        path.strip_prefix(root).unwrap().into(),
        hasher.finalize().into(),
      );
    }

    let filepack = Filepack { files };

    let json = serde_json::to_string(&filepack).unwrap();

    let destination = root.join("filepack.json");

    fs::write(&destination, &json).context(error::Io { path: destination })?;

    Ok(())
  }

  fn verify(root: &Utf8Path) -> Result {
    let source = root.join("filepack.json");

    let json = fs::read_to_string(&source).context(error::Io { path: &source })?;

    let filepack =
      serde_json::from_str::<Filepack>(&json).context(error::Deserialize { path: &source })?;

    for (path, &expected) in &filepack.files {
      let file = File::open(path).context(error::Io { path })?;

      let mut hasher = Hasher::new();

      hasher.update_reader(file).context(error::Io { path })?;

      let actual = Hash::from(hasher.finalize());

      if actual != expected {
        return Err(Error::HashMismatch {
          path: path.into(),
          actual,
          expected,
        });
      }
    }

    for entry in WalkDir::new(root) {
      let entry = entry?;

      let path = entry.path();

      let path = Utf8Path::from_path(path).context(error::Path { path })?;

      let relative = path.strip_prefix(root).unwrap();

      if !filepack.files.contains_key(relative) {
        return Err(Error::ExtraneousFile { path: path.into() });
      }
    }

    Ok(())
  }
}

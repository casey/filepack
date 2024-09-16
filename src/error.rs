use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub(crate) enum Error {
  #[snafu(display("failed to get current directory"))]
  CurrentDir { source: io::Error },
  #[snafu(display("failed to deserialize manifest at `{path}`"))]
  DeserializeManifest {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
    source: serde_json::Error,
  },
  #[snafu(display("failed to deserialize metadata at `{path}`"))]
  DeserializeMetadata {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
    source: serde_yaml::Error,
  },
  #[snafu(
    display(
      "empty director{} {}",
      if paths.len() == 1 { "y" } else { "ies" },
      List::and_ticked(paths),
    )
  )]
  EmptyDirectory { paths: Vec<DisplayPath> },
  #[snafu(display("{count} mismatched file{}", if *count == 1 { "" } else { "s" }))]
  EntryMismatch {
    backtrace: Option<Backtrace>,
    count: usize,
  },
  #[snafu(display("extraneous file not in manifest: `{path}`"))]
  ExtraneousFile {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
  },
  #[snafu(display("manifest `{path}` already exists"))]
  ManifestAlreadyExists {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
  },
  #[snafu(display("manifest `{path}` not found"))]
  ManifestNotFound {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
  },
  #[snafu(display("metadata template `{path}` should not be included in package"))]
  MetadataTemplateIncluded {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
  },
  #[snafu(display("file missing: `{path}`"))]
  MissingFile {
    backtrace: Option<Backtrace>,
    path: RelativePath,
  },
  #[snafu(display("I/O error at `{path}`"))]
  Io {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
    source: io::Error,
  },
  #[snafu(display("{count} lint error{}", if *count == 1 { "" } else { "s" }))]
  Lint {
    backtrace: Option<Backtrace>,
    count: u64,
  },
  #[snafu(display("invalid path `{path}`"))]
  Path {
    path: DisplayPath,
    source: relative_path::Error,
  },
  #[snafu(display("path not valid unicode: `{}`", path.display()))]
  PathUnicode {
    backtrace: Option<Backtrace>,
    path: PathBuf,
  },
  #[snafu(display("I/O error reading standard input"))]
  StandardInputIo {
    backtrace: Option<Backtrace>,
    source: io::Error,
  },
  #[snafu(display("symlink at `{path}`"))]
  Symlink {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
  },
  #[snafu(transparent)]
  WalkDir {
    backtrace: Option<Backtrace>,
    source: walkdir::Error,
  },
}

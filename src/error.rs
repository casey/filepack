use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub(crate) enum Error {
  #[snafu(display("failed to get current directory"))]
  CurrentDir { source: io::Error },
  #[snafu(display("failed to deserialize manifest at `{path}`"))]
  Deserialize {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
    source: serde_json::Error,
  },
  #[snafu(
    display(
      "empty director{} {}",
      if paths.len() == 1 { "y" } else { "ies" },
      List::and_ticked(paths),
    )
  )]
  EmptyDirectory { paths: Vec<DisplayPath> },
  #[snafu(display("extraneous file not in manifest at `{path}`"))]
  ExtraneousFile {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
  },
  #[snafu(display("hash mismatch for `{path}`, expected {expected} but got {actual}"))]
  HashMismatch {
    actual: Hash,
    backtrace: Option<Backtrace>,
    expected: Hash,
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
  #[snafu(display("non-portable path `{path}`"))]
  PathLint {
    backtrace: Option<Backtrace>,
    path: RelativePath,
    source: Lint,
  },
  #[snafu(display("invalid path `{path}`"))]
  Path {
    path: Utf8PathBuf,
    source: relative_path::Error,
  },
  #[snafu(display("path `{}` not valid unicode", path.display()))]
  PathUnicode {
    backtrace: Option<Backtrace>,
    path: PathBuf,
  },
  #[snafu(display("size mismatch for `{path}`, expected {expected} but got {actual}"))]
  SizeMismatch {
    actual: u64,
    backtrace: Option<Backtrace>,
    expected: u64,
    path: DisplayPath,
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
  #[snafu(context(false))]
  WalkDir {
    backtrace: Option<Backtrace>,
    source: walkdir::Error,
  },
}

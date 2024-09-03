use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub(crate) enum Error {
  #[snafu(display("failed to deserialize manifest at `{path}`"))]
  Deserialize {
    backtrace: Option<Backtrace>,
    path: Utf8PathBuf,
    source: serde_json::Error,
  },
  #[snafu(display("extraneous file not in manifest at `{path}`"))]
  ExtraneousFile {
    backtrace: Option<Backtrace>,
    path: Utf8PathBuf,
  },
  #[snafu(display("hash mismatch for `{path}`, expected {expected} but got {actual}"))]
  HashMismatch {
    actual: Hash,
    backtrace: Option<Backtrace>,
    expected: Hash,
    path: Utf8PathBuf,
  },
  #[snafu(display("manifest `{path}` already exists"))]
  ManifestAlreadyExists {
    backtrace: Option<Backtrace>,
    path: Utf8PathBuf,
  },
  #[snafu(display("I/O error at `{path}`"))]
  Io {
    backtrace: Option<Backtrace>,
    path: Utf8PathBuf,
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
  #[snafu(display("symlink at `{path}`"))]
  Symlink {
    backtrace: Option<Backtrace>,
    path: Utf8PathBuf,
  },
  #[snafu(context(false))]
  WalkDir {
    backtrace: Option<Backtrace>,
    source: walkdir::Error,
  },
}

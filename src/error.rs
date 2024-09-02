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
  #[snafu(display(
    "internal error, this may indicate a bug in filepack: `{message}` \
     consider filing an issue: https://github.com/casey/filepack/issues/new"
  ))]
  Internal {
    backtrace: Option<Backtrace>,
    message: String,
  },
  #[snafu(display("I/O error at `{path}`"))]
  Io {
    backtrace: Option<Backtrace>,
    path: Utf8PathBuf,
    source: io::Error,
  },
  #[snafu(display("path `{path}` contains backslash"))]
  PathBackslash {
    backtrace: Option<Backtrace>,
    path: Utf8PathBuf,
  },
  #[snafu(display("path `{path}` contains invalid component {component}"))]
  PathComponent {
    backtrace: Option<Backtrace>,
    component: String,
    path: Utf8PathBuf,
  },
  #[snafu(display("path `{path}` contains double slash"))]
  PathDoubleSlash {
    backtrace: Option<Backtrace>,
    path: Utf8PathBuf,
  },
  #[snafu(display("path `{path}` has trailing slash"))]
  PathTrailingSlash {
    backtrace: Option<Backtrace>,
    path: Utf8PathBuf,
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

use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub(crate) enum Error {
  #[snafu(display("failed to deserialize filepack at `{path}`"))]
  Deserialize {
    path: Utf8PathBuf,
    source: serde_json::Error,
  },
  #[snafu(display("path `{path}` contains double slash"))]
  DoubleSlash { path: Utf8PathBuf },
  #[snafu(display("extraneous file not in filepack at `{path}`"))]
  ExtraneousFile { path: Utf8PathBuf },
  #[snafu(display("filepack `{path}` already exists"))]
  FilepackExists { path: Utf8PathBuf },
  #[snafu(display("hash mismatch for `{path}`, expected {expected} but got {actual}"))]
  HashMismatch {
    path: Utf8PathBuf,
    expected: Hash,
    actual: Hash,
  },
  #[snafu(display("I/O error at `{path}`"))]
  Io {
    path: Utf8PathBuf,
    source: io::Error,
  },
  #[snafu(display("path `{}` not valid unicode", path.display()))]
  Path { path: PathBuf },
  #[snafu(display("path `{path}` contains invalid component {component}"))]
  PathComponent {
    path: Utf8PathBuf,
    component: String,
  },
  #[snafu(display("symlink at `{path}`"))]
  Symlink { path: Utf8PathBuf },
  #[snafu(display("path `{path}` has trailing slash"))]
  TrailingSlash { path: Utf8PathBuf },
  #[snafu(context(false))]
  WalkDir { source: walkdir::Error },
}

use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub(crate) enum Error {
  #[snafu(display("I/O error at `{path}`"))]
  Io {
    path: Utf8PathBuf,
    source: io::Error,
  },
  #[snafu(display("symlink at `{path}`"))]
  Symlink {
    path: Utf8PathBuf,
  },
  #[snafu(context(false))]
  WalkDir {
    source: walkdir::Error,
  },
  Path {
    path: PathBuf,
  },
}

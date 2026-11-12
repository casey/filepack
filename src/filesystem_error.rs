use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum FilesystemError {
  #[snafu(display("expected `{expected}` but found case mismatch `{actual}`"))]
  FilenameCase {
    actual: String,
    expected: &'static str,
  },
  #[snafu(display("I/O error at `{path}`"))]
  Io {
    path: Utf8PathBuf,
    source: io::Error,
  },
}

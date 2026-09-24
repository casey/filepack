use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum FilesystemError {
  #[snafu(display("I/O error at `{path}`"))]
  Io {
    path: Utf8PathBuf,
    source: io::Error,
  },
}

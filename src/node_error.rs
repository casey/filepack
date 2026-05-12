use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum NodeError {
  #[snafu(display("I/O error on connection"))]
  ConnectionIo { source: io::Error },
  #[snafu(display("failed to decode message"))]
  DecodeMessage { source: DecodeError },
  #[snafu(display("I/O error at {path}"))]
  FilesystemIo {
    path: Utf8PathBuf,
    source: io::Error,
  },
  #[snafu(display("message size {size} above maximum of {}", u32::MAX))]
  MessageSize {
    size: usize,
    source: TryFromIntError,
  },
  #[snafu(display("received unexpected message: {message}"))]
  UnexpectedMessage { message: &'static str },
}

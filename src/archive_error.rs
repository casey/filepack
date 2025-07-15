use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub(crate) enum ArchiveError {
  Signature {
    backtrace: Option<Backtrace>,
    signature: Option<[u8; Archive::FILE_SIGNATURE.len()]>,
  },
  Truncated {
    backtrace: Option<Backtrace>,
  },
  FilesystemIo {
    backtrace: Option<Backtrace>,
    source: io::Error,
  },
}

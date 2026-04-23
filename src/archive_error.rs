use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum ArchiveError {
  #[snafu(display("failed to decode archive"))]
  Decode { source: DecodeError },
  #[snafu(display("archive file hash mismatch: expected {expected} but got {actual}"))]
  FileHashMismatch { actual: Hash, expected: Hash },
  #[snafu(display("archive missing entry for hash {hash}"))]
  FileMissing { hash: Hash },
  #[snafu(display("archive missing package directory"))]
  PackageMissing,
  #[snafu(display("failed to parse signature"))]
  SignatureParse { source: SignatureError },
  #[snafu(display("archive missing signatures directory"))]
  SignaturesMissing,
  #[snafu(display("archive contains unreferenced files: {hashes}"))]
  UnreferencedFiles { hashes: Ticked<Hash> },
}

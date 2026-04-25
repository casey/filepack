use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum ArchiveError {
  #[snafu(display("failed to decode directory"))]
  DirectoryDecode { source: DecodeError },
  #[snafu(display("archive file hash mismatch: expected {expected} but got {actual}"))]
  FileHashMismatch { actual: Hash, expected: Hash },
  #[snafu(display("archive missing entry for hash {hash}"))]
  FileMissing { hash: Hash },
  #[snafu(display("archive contains loose files: {hashes}"))]
  LooseFiles { hashes: Ticked<Hash> },
  #[snafu(display("archive missing package directory"))]
  PackageMissing,
  #[snafu(display("failed to decode signature"))]
  SignatureDecode { source: DecodeError },
  #[snafu(display("archive missing signatures directory"))]
  SignaturesMissing,
  #[snafu(display("archive contains unexpected embedded files: {paths}"))]
  UnexpectedEmbeddedFiles { paths: Ticked<RelativePath> },
}

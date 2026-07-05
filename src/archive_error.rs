use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum ArchiveError {
  #[snafu(display("failed to decode directory"))]
  DirectoryDecode { source: DecodeError },
  #[snafu(display("found directory `{hash}` total file size {actual} but expected {expected}"))]
  DirectoryTotalFileSizeMismatch {
    actual: u64,
    expected: u64,
    hash: Hash,
  },
  #[snafu(display("directory `{hash}` total file size overflows 64-bit integer"))]
  DirectoryTotalFileSizeOverflow { hash: Hash },
  #[snafu(display("archive file hash mismatch: expected {expected} but got {actual}"))]
  FileHashMismatch { actual: Hash, expected: Hash },
  #[snafu(display("archive missing entry for hash {hash}"))]
  FileMissing { hash: Hash },
  #[snafu(display("archive contains loose files: {hashes}"))]
  LooseFiles { hashes: Ticked<Hash> },
  #[snafu(display("archive missing package directory"))]
  PackageMissing,
  #[snafu(display("expected archive `package` entry to be directory but found {ty}"))]
  PackageType { ty: EntryType },
  #[snafu(display("failed to decode signature"))]
  SignatureDecode { source: DecodeError },
  #[snafu(display("found subdirectory in `signatures` directory"))]
  SignaturesDirectory,
  #[snafu(display("archive missing signatures directory"))]
  SignaturesMissing,
  #[snafu(display("expected archive `signatures` entry to be directory but found {ty}"))]
  SignaturesType { ty: EntryType },
  #[snafu(display("archive contains unexpected embedded files: {paths}"))]
  UnexpectedEmbeddedFiles { paths: Ticked<RelativePath> },
  #[snafu(display("archive contains unexpected entries: {names}"))]
  UnexpectedEntries { names: Ticked<ComponentBuf> },
}

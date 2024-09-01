use {
  self::{error::Error, hash::Hash, subcommand::Subcommand},
  blake3::Hasher,
  camino::{Utf8Path, Utf8PathBuf},
  clap::Parser,
  serde::{Serialize, Serializer},
  snafu::{ErrorCompat, OptionExt, ResultExt, Snafu},
  std::{backtrace::BacktraceStatus, collections::HashMap, fs::File, io, path::PathBuf},
  walkdir::WalkDir,
};

mod error;
mod hash;
mod subcommand;

// todo:
// - add scary warning to readme
// - add description to readme
// - tests
// - always use / path separators on windows
// - test display of walkdir error
// - proper errors
// - read files, hash them, write to filepack.json
// - this is clearly the way
// - canonicalization?
// - design decisions:
//   - nesting to avoid path component duplication
//   - CBOR or JSON
//     - JSON is human readable
//     - JSON can be manipulated with many tools
//       - if CBOR, `filepack json` can return a JSON representation, *only* after verification
//     - CBOR is more efficient, especially amenable to large amounts of data
//     - CBOR allows for encoding sub-objects as byte arrays, which can be easily hashed
//   - blake3 vs sha2
//     - blake3 is faster
//     - sha2 suffers from length extension attacks
// - get feedback from bitcoiners
// - later:
//   - signatures
//     - sign the "root hash", which covers:
//       - file hashes
//       - metadata hash (metadata could just be another file)
//
// - how to add metadata?
//   - metadata.json in-tree
//     - just write the metadata as another file
//     - metadata.json gets authored and checked
//     - metadata hash is included in file hashes
//     - you need another file
//
// - how many files might we have?
//   - filepack.json - file hashes
//   - metadata.json - package metadata
//   - hashes        - blake3 hash tiers for damage detection
//   - parity        - parity data for damage recovery
//   - signatures    - some signatures over the hash of filepack.json
//
// - what happens if we include metadata in filepack.json:
//   - it gets copied into place
//   - you can no longer refer to the metadata separately
//
// - how do i transition to zero loose files?
//
// {
//   "title": "This is the title",
//   "slug": "some-name"
// }

#[derive(Debug, Serialize)]
struct Filepack {
  files: HashMap<Utf8PathBuf, Hash>,
}

struct Metadata {
  title: String,
}

type Result<T = (), E = Error> = std::result::Result<T, E>;

fn main() {
  if let Err(err) = Subcommand::parse().run() {
    eprintln!("error: {err}");

    for (i, err) in err.iter_chain().skip(1).enumerate() {
      if i == 0 {
        eprintln!();
        eprintln!("because:");
      }

      eprintln!("- {err}");
    }

    if let Some(backtrace) = err.backtrace() {
      if backtrace.status() == BacktraceStatus::Captured {
        eprintln!();
        eprintln!("backtrace:");
        eprintln!("{backtrace}");
      }
    }
  }
}

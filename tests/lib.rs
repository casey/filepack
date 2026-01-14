use {
  self::{expected::Expected, test::Test},
  camino::{Utf8Path, Utf8PathBuf},
  executable_path::executable_path,
  filepack::{Manifest, PrivateKey, PublicKey, Signature},
  regex::Regex,
  std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{MAIN_SEPARATOR_STR, Path},
    process::{Command, Stdio},
    str,
  },
};

macro_rules! assert_matches {
  ($expression:expr, $( $pattern:pat_param )|+ $( if $guard:expr )? $(,)?) => {
    match $expression {
      $( $pattern )|+ $( if $guard )? => {}
      left => panic!(
        "assertion failed: (left ~= right)\n  left: `{:?}`\n right: `{}`",
        left,
        stringify!($($pattern)|+ $(if $guard)?)
      ),
    }
  }
}

mod create;
mod expected;
mod files;
mod fingerprint;
mod hash;
mod json;
mod key;
mod keygen;
mod man;
mod misc;
mod sign;
mod size;
mod test;
mod verify;

const EMPTY_HASH: &str = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";

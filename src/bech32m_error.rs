use {super::*, bech32::primitives::decode::CheckedHrpstringError};

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum Bech32mError {
  #[snafu(display("failed to decode bech32m {ty}"))]
  Decode {
    ty: &'static str,
    source: CheckedHrpstringError,
  },
  #[snafu(display(
    "expected bech32m human-readable prefix `{expected}1...` but found `{actual}1...`",
  ))]
  Hrp {
    expected: crate::Hrp,
    actual: crate::Hrp,
  },
  #[snafu(display("expected {expected} bytes but found {actual}"))]
  Length { expected: usize, actual: usize },
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    #[track_caller]
    fn case(s: &str, expected: &str) {
      assert_eq!(
        PublicKey::decode_bech32m(s).unwrap_err().to_string(),
        expected
      );
    }

    case("foo", "failed to decode bech32m public key");

    case(
      test::PRIVATE_KEY,
      "expected bech32m human-readable prefix `public1...` but found `private1...`",
    );

    case("public134jkgz", "expected 32 bytes but found 0");

    case(
      "public1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq6s7wps",
      "expected 32 bytes but found 33",
    );
  }
}

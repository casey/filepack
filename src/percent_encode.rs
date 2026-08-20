use {
  super::*,
  percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode},
};

const PATH: AsciiSet = CONTROLS
  .add(b' ')
  .add(b'"')
  .add(b'#')
  .add(b'%')
  .add(b'<')
  .add(b'>')
  .add(b'?')
  .add(b'`')
  .add(b'{')
  .add(b'}');

const SEGMENT: AsciiSet = PATH.add(b'/');

pub(crate) trait PercentEncode {
  fn percent_encode_path(&self) -> percent_encoding::PercentEncode;
  fn percent_encode_segment(&self) -> percent_encoding::PercentEncode;
}

impl<T: AsRef<str>> PercentEncode for T {
  fn percent_encode_path(&self) -> percent_encoding::PercentEncode {
    utf8_percent_encode(self.as_ref(), &PATH)
  }

  fn percent_encode_segment(&self) -> percent_encoding::PercentEncode {
    utf8_percent_encode(self.as_ref(), &SEGMENT)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_eq!("foo bar".percent_encode_segment().to_string(), "foo%20bar");
    assert_eq!("foo/bar".percent_encode_segment().to_string(), "foo%2Fbar");
    assert_eq!("foo bar".percent_encode_path().to_string(), "foo%20bar");
    assert_eq!(
      "foo/bar baz".percent_encode_path().to_string(),
      "foo/bar%20baz",
    );
  }
}

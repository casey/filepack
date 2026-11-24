use {
  super::*,
  percent_encoding::{AsciiSet, CONTROLS, NON_ALPHANUMERIC, utf8_percent_encode},
};

const PARAMETER: AsciiSet = NON_ALPHANUMERIC
  .remove(b'!')
  .remove(b'#')
  .remove(b'$')
  .remove(b'&')
  .remove(b'+')
  .remove(b'-')
  .remove(b'.')
  .remove(b'^')
  .remove(b'_')
  .remove(b'`')
  .remove(b'|')
  .remove(b'~');

const SEGMENT: AsciiSet = CONTROLS
  .add(b' ')
  .add(b'"')
  .add(b'#')
  .add(b'%')
  .add(b'/')
  .add(b'<')
  .add(b'>')
  .add(b'?')
  .add(b'`')
  .add(b'{')
  .add(b'}');

pub(crate) trait PercentEncode {
  fn percent_encode_parameter(&self) -> percent_encoding::PercentEncode;

  fn percent_encode_segment(&self) -> percent_encoding::PercentEncode;
}

impl<T: AsRef<str>> PercentEncode for T {
  fn percent_encode_parameter(&self) -> percent_encoding::PercentEncode {
    utf8_percent_encode(self.as_ref(), &PARAMETER)
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
    assert_eq!(
      "!#$&+-.^_`|~".percent_encode_parameter().to_string(),
      "!#$&+-.^_`|~",
    );
    assert_eq!(
      " \"%'()*,/:;<=>?@[\\]{}"
        .percent_encode_parameter()
        .to_string(),
      "%20%22%25%27%28%29%2A%2C%2F%3A%3B%3C%3D%3E%3F%40%5B%5C%5D%7B%7D",
    );
    assert_eq!("foo bar".percent_encode_segment().to_string(), "foo%20bar");
    assert_eq!("foo/bar".percent_encode_segment().to_string(), "foo%2Fbar");
  }
}

use super::*;

pub(crate) struct Vp9FrameHeader {
  pub(crate) bit_depth: u64,
}

impl Vp9FrameHeader {
  pub(crate) fn parse(data: &[u8]) -> Option<Self> {
    let mut reader = BitReader::new(data);

    // frame_marker
    if reader.bits(2)? != 2 {
      return None;
    }

    let profile_low_bit = reader.bit()?;
    let profile_high_bit = reader.bit()?;
    let profile = profile_high_bit << 1 | profile_low_bit;

    // reserved zero bit
    if profile == 3 && reader.bit()? != 0 {
      return None;
    }

    // show_existing_frame
    if reader.bit()? != 0 {
      return None;
    }

    // frame_type
    if reader.bit()? != 0 {
      return None;
    }

    // show_frame and error_resilient_mode
    reader.bits(2)?;

    // frame_sync_code
    if reader.bits(24)? != 0x0049_8342 {
      return None;
    }

    let bit_depth = if profile >= 2 {
      if reader.bit()? == 0 { 10 } else { 12 }
    } else {
      8
    };

    Some(Self { bit_depth })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parsing() {
    #[track_caller]
    fn case(data: &[u8], expected: Option<u64>) {
      assert_eq!(
        Vp9FrameHeader::parse(data).map(|header| header.bit_depth),
        expected,
      );
    }

    case(&[0x82, 0x49, 0x83, 0x42], Some(8));
    case(&[0x92, 0x49, 0x83, 0x42, 0x00], Some(10));
    case(&[0x92, 0x49, 0x83, 0x42, 0x80], Some(12));
    case(&[0x84, 0x49, 0x83, 0x42], None);
    case(&[0x88, 0x49, 0x83, 0x42], None);
    case(&[0x82, 0x49, 0x83, 0x43], None);
    case(&[0x82], None);
    case(b"foo", None);
    case(&[], None);
  }
}

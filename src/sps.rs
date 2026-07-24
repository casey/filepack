use super::*;

pub(crate) struct Sps {
  pub(crate) bit_depth: u64,
}

impl Sps {
  pub(crate) fn high_profile(profile_idc: u64) -> bool {
    matches!(
      profile_idc,
      44 | 83 | 86 | 100 | 110 | 118 | 122 | 128 | 134 | 135 | 138 | 139 | 244
    )
  }

  pub(crate) fn parse(sps: &[u8]) -> Option<Self> {
    let mut rbsp = Vec::new();

    for &byte in sps.get(1..)? {
      if byte == 3 && rbsp.ends_with(&[0, 0]) {
        continue;
      }

      rbsp.push(byte);
    }

    let mut reader = BitReader::new(&rbsp);

    let profile_idc = reader.bits(8)?;

    reader.bits(16)?;

    reader.ue()?;

    let bit_depth = if Self::high_profile(profile_idc) {
      if reader.ue()? == 3 {
        reader.bit()?;
      }

      8 + reader.ue()?
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
    fn case(sps: &[u8], expected: Option<u64>) {
      assert_eq!(Sps::parse(sps).map(|sps| sps.bit_depth), expected);
    }

    case(&[0x67, 66, 0, 30, 0x80], Some(8));
    case(&[0x67, 100, 0, 31, 0xA6], Some(10));
    case(&[0x67, 100, 0, 31, 0x91], Some(8));
    case(&[0x67, 100, 0, 0, 0x03, 0xA6], Some(10));
    case(&[0x67, 100, 0, 31], None);
    case(&[0x67], None);
    case(&[], None);
  }
}

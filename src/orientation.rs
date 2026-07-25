use super::*;

#[derive(Clone, Copy, Debug, Decode, Default, Encode, PartialEq, Serialize)]
pub(crate) struct Orientation {
  #[n(0)]
  pub(crate) mirrored: bool,
  #[n(1)]
  pub(crate) rotation: Rotation,
}

impl Orientation {
  pub(crate) fn dimensions(self, dimensions: Dimensions) -> Dimensions {
    match self.rotation {
      Rotation::R0 | Rotation::R180 => dimensions,
      Rotation::R90 | Rotation::R270 => Dimensions {
        height: dimensions.width,
        width: dimensions.height,
      },
    }
  }

  pub(crate) fn from_exif(data: &[u8]) -> Result<Self, ExifError> {
    let big_endian = match data.get(0..2).context(exif_error::Truncated)? {
      b"II" => false,
      b"MM" => true,
      _ => return Err(exif_error::ByteOrder.build()),
    };

    let decoder = ExifDecoder { big_endian, data };

    let magic = decoder.u16(2)?;

    ensure!(magic == 42, exif_error::Magic { magic });

    let ifd = usize::try_from(decoder.u32(4)?).unwrap();

    let entries = decoder.u16(ifd)?;

    for i in 0..usize::from(entries) {
      let entry = ifd + 2 + i * 12;

      if decoder.u16(entry)? != 0x0112 {
        continue;
      }

      let ty = decoder.u16(entry + 2)?;

      ensure!(ty == 3, exif_error::OrientationType { ty });

      let count = decoder.u32(entry + 4)?;

      ensure!(count == 1, exif_error::OrientationCount { count });

      let (mirrored, rotation) = match decoder.u16(entry + 8)? {
        1 => (false, Rotation::R0),
        2 => (true, Rotation::R0),
        3 => (false, Rotation::R180),
        4 => (true, Rotation::R180),
        5 => (true, Rotation::R90),
        6 => (false, Rotation::R90),
        7 => (true, Rotation::R270),
        8 => (false, Rotation::R270),
        value => return Err(exif_error::OrientationValue { value }.build()),
      };

      return Ok(Self { mirrored, rotation });
    }

    Ok(Self::new())
  }

  pub(crate) fn new() -> Self {
    Self::default()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dimensions() {
    #[track_caller]
    fn case(rotation: Rotation, expected: Dimensions) {
      assert_eq!(
        Orientation {
          mirrored: false,
          rotation,
        }
        .dimensions(Dimensions {
          height: 1,
          width: 2,
        }),
        expected,
      );
    }

    case(
      Rotation::R0,
      Dimensions {
        height: 1,
        width: 2,
      },
    );
    case(
      Rotation::R90,
      Dimensions {
        height: 2,
        width: 1,
      },
    );
    case(
      Rotation::R180,
      Dimensions {
        height: 1,
        width: 2,
      },
    );
    case(
      Rotation::R270,
      Dimensions {
        height: 2,
        width: 1,
      },
    );
  }

  #[test]
  fn encoding() {
    assert_cbor(Orientation::new(), "a200f40100");

    assert_cbor(
      Orientation {
        mirrored: true,
        rotation: Rotation::R90,
      },
      "a200f50101",
    );
  }

  #[test]
  fn from_exif() {
    #[track_caller]
    fn case(value: u16, mirrored: bool, rotation: Rotation) {
      assert_eq!(
        Orientation::from_exif(&exif(value)).unwrap(),
        Orientation { mirrored, rotation },
      );
    }

    case(1, false, Rotation::R0);
    case(2, true, Rotation::R0);
    case(3, false, Rotation::R180);
    case(4, true, Rotation::R180);
    case(5, true, Rotation::R90);
    case(6, false, Rotation::R90);
    case(7, true, Rotation::R270);
    case(8, false, Rotation::R270);
  }

  #[test]
  fn from_exif_big_endian() {
    assert_eq!(
      Orientation::from_exif(&[
        0x4D, 0x4D, 0x00, 0x2A, 0x00, 0x00, 0x00, 0x08, 0x00, 0x01, 0x01, 0x12, 0x00, 0x03, 0x00,
        0x00, 0x00, 0x01, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
      ])
      .unwrap(),
      Orientation {
        mirrored: false,
        rotation: Rotation::R90,
      },
    );
  }

  #[test]
  fn from_exif_errors() {
    #[track_caller]
    fn case(data: &[u8], expected: &str) {
      assert_eq!(
        Orientation::from_exif(data).unwrap_err().to_string(),
        expected,
      );
    }

    case(&[], "truncated EXIF data");
    case(b"I", "truncated EXIF data");
    case(b"foo", "invalid byte order");
    case(b"II", "truncated EXIF data");
    case(
      &[0x49, 0x49, 0x2B, 0x00, 0x08, 0x00, 0x00, 0x00],
      "expected magic 42 but found 43",
    );
    case(
      &[0x49, 0x49, 0x2A, 0x00, 0x08, 0x00, 0x00, 0x00],
      "truncated EXIF data",
    );
    case(
      &[0x49, 0x49, 0x2A, 0x00, 0x08, 0x00, 0x00, 0x00, 0x01, 0x00],
      "truncated EXIF data",
    );
    case(&exif(0), "invalid orientation value 0");
    case(&exif(9), "invalid orientation value 9");

    let mut ty = exif(1);
    ty[12] = 4;
    case(&ty, "invalid orientation type 4");

    let mut count = exif(1);
    count[14] = 2;
    case(&count, "invalid orientation count 2");
  }

  #[test]
  fn from_exif_missing_orientation() {
    let mut data = exif(6);
    data[10] = 0;
    data[11] = 1;
    assert_eq!(Orientation::from_exif(&data).unwrap(), Orientation::new(),);

    assert_eq!(
      Orientation::from_exif(&[
        0x49, 0x49, 0x2A, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
      ])
      .unwrap(),
      Orientation::new(),
    );
  }
}

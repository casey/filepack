use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Clone, Copy, Debug, Decode, Default, Encode, PartialEq)]
pub(crate) enum Rotation {
  /// No rotation.
  #[default]
  #[n(0)]
  R0,
  /// Rotate 90 degrees.
  #[n(1)]
  R90,
  /// Rotate 180 degrees.
  #[n(2)]
  R180,
  /// Rotate 270 degrees.
  #[n(3)]
  R270,
}

impl Rotation {
  pub(crate) fn degrees(self) -> u64 {
    match self {
      Self::R0 => 0,
      Self::R90 => 90,
      Self::R180 => 180,
      Self::R270 => 270,
    }
  }
}

impl Serialize for Rotation {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    self.degrees().serialize(serializer)
  }
}

impl json::Json for Rotation {
  fn json_schema(registry: &mut json::Registry) -> String {
    json::Schema::Wrapper {
      inner: u64::json_schema(registry),
    }
    .register("Rotation", registry)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn serialize() {
    #[track_caller]
    fn case(rotation: Rotation, expected: &str) {
      assert_eq!(serde_json::to_string(&rotation).unwrap(), expected);
    }

    case(Rotation::R0, "0");
    case(Rotation::R90, "90");
    case(Rotation::R180, "180");
    case(Rotation::R270, "270");
  }
}

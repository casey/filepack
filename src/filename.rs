use super::*;

macro_rules! filename {
  { $alias:ident, $extension:ident, $($literal:literal),+ } => {
    #[derive(Clone, Debug, PartialEq)]
    pub(crate) struct $extension;

    impl Extension for $extension {
      const EXTENSIONS: &[&str] = &[$($literal),+];
    }

    pub(crate) type $alias = Filename<$extension>;
  }
}

filename! { Flac, FlacExtension, "flac" }
filename! { Image, ImageExtension, "jpg", "png" }
filename! { Md, MdExtension, "md" }
filename! { Nfo, NfoExtension, "nfo" }

pub(crate) trait Extension {
  const EXTENSIONS: &[&str];
}

#[derive(Clone, Debug, DeserializeFromStr, PartialEq)]
pub(crate) struct Filename<T: Extension> {
  component: ComponentBuf,
  phantom: PhantomData<T>,
}

impl<T: Extension> Filename<T> {
  pub(crate) fn as_path(&self) -> RelativePath {
    self.component.as_path()
  }

  pub(crate) fn extension(&self) -> Option<&str> {
    self.component.extension()
  }
}

impl<T: Extension> From<Filename<T>> for RelativePath {
  fn from(filename: Filename<T>) -> Self {
    filename.as_path()
  }
}

impl<T: Extension> Serialize for Filename<T> {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    self.component.serialize(serializer)
  }
}

impl<T: Extension> Decode for Filename<T> {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    decoder.text()?.parse().context(decode_error::Component)
  }
}

impl<T: Extension> Encode for Filename<T> {
  fn encode(&self, encoder: &mut Encoder) {
    self.component.encode(encoder);
  }
}

impl<T: Extension> FromStr for Filename<T> {
  type Err = ComponentError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let component = s.parse::<ComponentBuf>()?;

    if component
      .extension()
      .is_none_or(|extension| !T::EXTENSIONS.contains(&extension))
    {
      return Err(ComponentError::Extension {
        extensions: T::EXTENSIONS,
      });
    }

    Ok(Self {
      component,
      phantom: PhantomData,
    })
  }
}

impl Image {
  pub(crate) fn resource_type(&self) -> ResourceType {
    match self.ty() {
      ImageType::Jpeg => ResourceType::Jpeg,
      ImageType::Png => ResourceType::Png,
    }
  }

  pub(crate) fn ty(&self) -> ImageType {
    match self.extension().unwrap() {
      "jpg" => ImageType::Jpeg,
      "png" => ImageType::Png,
      _ => unreachable!(),
    }
  }
}

impl Flac {
  #[expect(clippy::unused_self)]
  pub(crate) fn resource_type(&self) -> ResourceType {
    ResourceType::Flac
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn decode_error() {
    assert_matches!(
      Image::decode(&mut Decoder::new(&"cover.svg".encode_to_vec())),
      Err(DecodeError::Component {
        source: ComponentError::Extension {
          extensions: &["jpg", "png"]
        },
      }),
    );
  }

  #[test]
  fn encoding() {
    assert_cbor_eq("cover.md".parse::<Md>().unwrap(), "cover.md");
  }

  #[test]
  fn invalid() {
    #[track_caller]
    fn case<T: FromStr<Err = ComponentError> + std::fmt::Debug>(
      input: &str,
      expected: ComponentError,
    ) {
      assert_eq!(input.parse::<T>().unwrap_err(), expected);
    }

    case::<Image>(
      "cover.svg",
      ComponentError::Extension {
        extensions: &["jpg", "png"],
      },
    );
    case::<Md>(
      "README.txt",
      ComponentError::Extension {
        extensions: &["md"],
      },
    );
    case::<Nfo>(
      "info.txt",
      ComponentError::Extension {
        extensions: &["nfo"],
      },
    );
    case::<Flac>(
      "track.mp3",
      ComponentError::Extension {
        extensions: &["flac"],
      },
    );
    case::<Md>("", ComponentError::Empty);
    case::<Md>("foo/bar.md", ComponentError::Separator { character: '/' });
  }

  #[test]
  fn valid() {
    #[track_caller]
    fn case<T: FromStr<Err = ComponentError>>(input: &str) {
      input.parse::<T>().unwrap();
    }

    case::<Image>("cover.jpg");
    case::<Image>("cover.png");
    case::<Flac>("track.flac");
    case::<Md>("README.md");
    case::<Nfo>("info.nfo");
  }
}

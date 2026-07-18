use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ResourceType {
  Binary,
  Flac,
  Jpeg,
  Mp4,
  Png,
}

impl ResourceType {
  pub(crate) fn content_disposition(self) -> Option<HeaderValue> {
    match self {
      Self::Binary => Some(HeaderValue::from_static("attachment")),
      Self::Flac | Self::Jpeg | Self::Mp4 | Self::Png => None,
    }
  }

  pub(crate) fn content_type(self) -> Mime {
    match self {
      Self::Binary => mime::APPLICATION_OCTET_STREAM,
      Self::Flac => "audio/flac".parse().unwrap(),
      Self::Jpeg => mime::IMAGE_JPEG,
      Self::Mp4 => "video/mp4".parse().unwrap(),
      Self::Png => mime::IMAGE_PNG,
    }
  }

  pub(crate) fn from_filename(component: &Component) -> Option<Self> {
    match component.extension()? {
      "flac" => Some(Self::Flac),
      "jpeg" | "jpg" => Some(Self::Jpeg),
      "mp4" => Some(Self::Mp4),
      "png" => Some(Self::Png),
      _ => None,
    }
  }

  pub(crate) fn sandbox(self) -> bool {
    match self {
      Self::Binary | Self::Jpeg | Self::Png => true,
      Self::Flac | Self::Mp4 => false,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn from_filename() {
    #[track_caller]
    fn case(filename: &str, expected: Option<ResourceType>) {
      assert_eq!(
        ResourceType::from_filename(Component::new(filename).unwrap()),
        expected,
      );
    }

    // supported extensions
    case("foo.flac", Some(ResourceType::Flac));
    case("foo.jpeg", Some(ResourceType::Jpeg));
    case("foo.jpg", Some(ResourceType::Jpeg));
    case("foo.png", Some(ResourceType::Png));

    // unsupported extensions
    case("foo.PNG", None);
    case("foo.txt", None);

    // no extension
    case("foo", None);
    case(".png", None);

    // scriptable extensions
    case("foo.htm", None);
    case("foo.html", None);
    case("foo.svg", None);
    case("foo.svgz", None);
    case("foo.xhtml", None);
    case("foo.xml", None);
    case("foo.xsl", None);
    case("foo.xslt", None);
  }
}

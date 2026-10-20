use super::*;

#[derive(Clone, Copy, Debug, EnumIter, PartialEq)]
pub(crate) enum ResourceType {
  Binary,
  Flac,
  Jpeg,
  M4a,
  Mp3,
  Mp4,
  Png,
  Text,
  Webm,
}

impl ResourceType {
  pub(crate) fn content_disposition(self) -> Option<HeaderValue> {
    match self {
      Self::Binary => Some(HeaderValue::from_static("attachment")),
      Self::Flac
      | Self::Jpeg
      | Self::M4a
      | Self::Mp3
      | Self::Mp4
      | Self::Png
      | Self::Text
      | Self::Webm => None,
    }
  }

  pub(crate) fn content_type(self) -> &'static str {
    match self {
      Self::Binary => "application/octet-stream",
      Self::Flac => "audio/flac",
      Self::Jpeg => "image/jpeg",
      Self::M4a => "audio/mp4",
      Self::Mp3 => "audio/mpeg",
      Self::Mp4 => "video/mp4",
      Self::Png => "image/png",
      Self::Text => "text/plain;charset=utf-8",
      Self::Webm => "video/webm",
    }
  }

  pub(crate) fn extensions(self) -> &'static [&'static str] {
    match self {
      Self::Binary => &[],
      Self::Flac => &["flac"],
      Self::Jpeg => &["jpeg", "jpg"],
      Self::M4a => &["m4a"],
      Self::Mp3 => &["mp3"],
      Self::Mp4 => &["mp4"],
      Self::Png => &["png"],
      Self::Text => &[
        "css", "csv", "html", "js", "json", "md", "svg", "toml", "tsv", "txt", "xml", "yaml",
      ],
      Self::Webm => &["webm"],
    }
  }

  pub(crate) fn from_filename(component: &Component) -> Option<Self> {
    let extension = component.extension()?;
    Self::iter().find(|ty| ty.extensions().contains(&extension))
  }

  pub(crate) fn sandbox(self) -> bool {
    match self {
      Self::Binary | Self::Jpeg | Self::Png | Self::Text => true,
      Self::Flac | Self::M4a | Self::Mp3 | Self::Mp4 | Self::Webm => false,
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
    case("foo.css", Some(ResourceType::Text));
    case("foo.csv", Some(ResourceType::Text));
    case("foo.flac", Some(ResourceType::Flac));
    case("foo.html", Some(ResourceType::Text));
    case("foo.jpeg", Some(ResourceType::Jpeg));
    case("foo.jpg", Some(ResourceType::Jpeg));
    case("foo.js", Some(ResourceType::Text));
    case("foo.json", Some(ResourceType::Text));
    case("foo.m4a", Some(ResourceType::M4a));
    case("foo.md", Some(ResourceType::Text));
    case("foo.mp3", Some(ResourceType::Mp3));
    case("foo.mp4", Some(ResourceType::Mp4));
    case("foo.png", Some(ResourceType::Png));
    case("foo.svg", Some(ResourceType::Text));
    case("foo.toml", Some(ResourceType::Text));
    case("foo.tsv", Some(ResourceType::Text));
    case("foo.txt", Some(ResourceType::Text));
    case("foo.webm", Some(ResourceType::Webm));
    case("foo.xml", Some(ResourceType::Text));
    case("foo.yaml", Some(ResourceType::Text));

    // unsupported extensions
    case("foo.PNG", None);
    case("foo.rtf", None);

    // no extension
    case("foo", None);
    case(".png", None);

    // scriptable extensions
    case("foo.htm", None);
    case("foo.svgz", None);
    case("foo.xhtml", None);
    case("foo.xsl", None);
    case("foo.xslt", None);
  }
}

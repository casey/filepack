use super::*;

#[derive(Boilerplate)]
pub(crate) struct HomeHtml {
  pub(crate) packages: Vec<PackageSummary>,
}

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
enum Section {
  Media(MediaType),
  None,
}

impl Display for Section {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Media(MediaType::Audio) => write!(f, "Audio"),
      Self::Media(MediaType::Image) => write!(f, "Image"),
      Self::Media(MediaType::Video) => write!(f, "Video"),
      Self::Media(MediaType::Web) => write!(f, "Web"),
      Self::None => write!(f, "None"),
    }
  }
}

struct Package<'a> {
  artwork: bool,
  creator: Option<&'a str>,
  fingerprint: Fingerprint,
  number: u64,
  title: Option<&'a str>,
}

impl HomeHtml {
  fn sections(&self) -> BTreeMap<Section, Vec<Package<'_>>> {
    let mut sections = BTreeMap::<Section, Vec<Package>>::new();

    for summary in &self.packages {
      let section = summary
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.media.as_ref())
        .map_or(Section::None, |media| Section::Media(media.ty()));

      sections.entry(section).or_default().push(Package {
        artwork: summary
          .metadata
          .as_ref()
          .is_some_and(|metadata| metadata.artwork.is_some()),
        creator: summary
          .metadata
          .as_ref()
          .and_then(|metadata| metadata.creator.as_deref()),
        fingerprint: summary.fingerprint,
        number: summary.number,
        title: summary
          .metadata
          .as_ref()
          .and_then(|metadata| metadata.title.as_deref()),
      });
    }

    sections
  }
}

impl Page for HomeHtml {
  fn stylesheet(&self) -> Option<&'static str> {
    Some("/static/home.css")
  }

  fn title(&self) -> String {
    "Filepack".into()
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  #[test]
  fn empty() {
    assert_eq!(
      HomeHtml {
        packages: Vec::new()
      }
      .to_string(),
      "<p>No packages yet. Upload one with <code>filepack upload</code>.</p>\n",
    );
  }

  #[test]
  fn sections() {
    let fingerprint = test::FINGERPRINT.parse::<Fingerprint>().unwrap();

    let audio = Metadata {
      artwork: Some(Image::test("foo.png")),
      creator: Some("foo".parse().unwrap()),
      media: Some(Media::Audio { items: Vec::new() }),
      title: Some("bar".parse().unwrap()),
      ..default()
    };

    let web = Metadata {
      media: Some(Media::Web),
      title: Some("baz".parse().unwrap()),
      ..default()
    };

    assert_eq!(
      HomeHtml {
        packages: vec![
          PackageSummary {
            fingerprint,
            metadata: None,
            number: 1,
            totals: Totals::default(),
          },
          PackageSummary {
            fingerprint,
            metadata: Some(web),
            number: 2,
            totals: Totals::default(),
          },
          PackageSummary {
            fingerprint,
            metadata: Some(audio),
            number: 3,
            totals: Totals::default(),
          },
        ],
      }
      .to_string(),
      unindent(&format!(
        "
          <section>
            <h2>Audio</h2>
            <ul class=thumbnails>
              <li>
                <a href=/package/3>
                  <img loading=lazy src=/artwork/{fingerprint}/thumbnail>
                </a>
                <div class=title>bar</div>
                <div class=creator>foo</div>
              </li>
            </ul>
          </section>
          <section>
            <h2>Web</h2>
            <ul class=thumbnails>
              <li>
                <a href=/package/2>
                </a>
                <div class=title>baz</div>
              </li>
            </ul>
          </section>
          <section>
            <h2>None</h2>
            <ul class=thumbnails>
              <li>
                <a href=/package/1>
                </a>
              </li>
            </ul>
          </section>
        ",
        fingerprint = test::FINGERPRINT,
      )),
    );
  }
}

use super::*;

#[derive(Boilerplate)]
pub(crate) struct PackagesHtml {
  pub(crate) packages: Vec<(Fingerprint, Option<Metadata>, Totals)>,
  pub(crate) view: View,
}

struct Package<'a> {
  artwork: bool,
  creator: Option<&'a Component>,
  file_size: u64,
  files: u64,
  fingerprint: Fingerprint,
  media: Option<MediaType>,
  title: Option<&'a Component>,
  year: Option<i64>,
}

impl PackagesHtml {
  fn packages(&self) -> impl Iterator<Item = Package<'_>> {
    self
      .packages
      .iter()
      .map(|(fingerprint, metadata, totals)| Package {
        artwork: metadata
          .as_ref()
          .is_some_and(|metadata| metadata.artwork.is_some()),
        creator: metadata
          .as_ref()
          .and_then(|metadata| metadata.creator.as_deref()),
        file_size: totals.file_size,
        files: totals.files,
        fingerprint: *fingerprint,
        media: metadata
          .as_ref()
          .and_then(|metadata| metadata.media.as_ref())
          .map(Media::ty),
        title: metadata
          .as_ref()
          .and_then(|metadata| metadata.title.as_deref()),
        year: metadata
          .as_ref()
          .and_then(|metadata| metadata.time.as_ref())
          .map(Time::year),
      })
  }
}

impl Page for PackagesHtml {
  fn stylesheet(&self) -> Option<&'static str> {
    Some("/static/packages.css")
  }

  fn title(&self) -> String {
    "packages · filepack".into()
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  #[test]
  fn grid() {
    let fingerprint = test::FINGERPRINT.parse::<Fingerprint>().unwrap();

    let metadata = Metadata {
      artwork: Some("foo.png".parse().unwrap()),
      creator: Some("foo".parse().unwrap()),
      title: Some("bar".parse().unwrap()),
      ..default()
    };

    assert_eq!(
      PackagesHtml {
        packages: vec![
          (fingerprint, Some(metadata), Totals::default()),
          (fingerprint, None, Totals::default()),
        ],
        view: View::Grid,
      }
      .to_string(),
      unindent(&format!(
        "
          <header>
            <h1>Packages</h1>
            <nav>
              <a href=/packages>List</a> | <a>Grid</a>
            </nav>
          </header>
          <ul class=grid>
            <li>
              <a href=/package/{fingerprint}>
                <img loading=lazy src=/artwork/{fingerprint} width=150 height=150>
              </a>
              <span class=title>bar</span>
              <span class=creator>foo</span>
            </li>
            <li>
              <a href=/package/{fingerprint}></a>
            </li>
          </ul>
        ",
        fingerprint = test::FINGERPRINT,
      )),
    );
  }

  #[test]
  fn list() {
    let fingerprint = test::FINGERPRINT.parse::<Fingerprint>().unwrap();

    let metadata = Metadata {
      creator: Some("foo".parse().unwrap()),
      media: Some(Media::Audio { items: Vec::new() }),
      time: Some("2024".parse().unwrap()),
      title: Some("bar".parse().unwrap()),
      ..default()
    };

    let totals = Totals {
      directories: 0,
      directory_size: 0,
      file_size: 1536,
      files: 2,
    };

    assert_eq!(
      PackagesHtml {
        packages: vec![
          (fingerprint, Some(metadata), totals),
          (fingerprint, None, Totals::default()),
        ],
        view: View::List,
      }
      .to_string(),
      unindent(&format!(
        "
          <header>
            <h1>Packages</h1>
            <nav>
              <a>List</a> | <a href=/packages?view=grid>Grid</a>
            </nav>
          </header>
          <table>
            <thead>
              <tr>
                <th>title</th>
                <th>creator</th>
                <th>year</th>
                <th>media</th>
                <th class=right>files</th>
                <th class=right>size</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td><a href=/package/{fingerprint}>bar</a></td>
                <td>foo</td>
                <td>2024</td>
                <td>audio</td>
                <td class=right><a href=/directory/{hash}>2</a></td>
                <td class=right>1.5 KiB</td>
              </tr>
              <tr>
                <td><a href=/package/{fingerprint}><code>{fingerprint}</code></a></td>
                <td></td>
                <td></td>
                <td></td>
                <td class=right><a href=/directory/{hash}>0</a></td>
                <td class=right>0 B</td>
              </tr>
            </tbody>
          </table>
        ",
        fingerprint = test::FINGERPRINT,
        hash = Hash::from(fingerprint),
      )),
    );
  }
}

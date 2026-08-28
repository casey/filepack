use super::*;

#[derive(Boilerplate)]
pub(crate) struct PackagesHtml {
  pub(crate) order: Order,
  pub(crate) packages: Vec<(Fingerprint, Option<Metadata>, Totals)>,
  pub(crate) sort: Sort,
  pub(crate) view: View,
}

struct Package<'a> {
  artwork: bool,
  creator: Option<&'a str>,
  file_size: u64,
  files: u64,
  fingerprint: Fingerprint,
  media: Option<MediaType>,
  title: Option<&'a str>,
  year: Option<i64>,
}

impl PackagesHtml {
  fn header(&self, sort: Sort) -> Trusted<String> {
    let order = if sort == self.sort {
      self.order.toggle()
    } else {
      Order::default()
    };

    let class = match sort {
      Sort::Files | Sort::Size => " class=right",
      Sort::Creator | Sort::Media | Sort::Title | Sort::Year => "",
    };

    let path = Self::path(self.view, sort, order);

    Trusted(format!("<th{class}><a href={path}>{sort}</a></th>"))
  }

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

  fn path(view: View, sort: Sort, order: Order) -> String {
    let mut params = Vec::new();

    if view != View::default() {
      params.push(format!("view={view}"));
    }

    if sort != Sort::default() {
      params.push(format!("sort={sort}"));
    }

    if order != Order::default() {
      params.push(format!("order={order}"));
    }

    if params.is_empty() {
      "/packages".into()
    } else {
      format!("/packages?{}", params.join("&"))
    }
  }

  fn view(&self, view: View) -> String {
    Self::path(view, self.sort, self.order)
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
      artwork: Some(Image::test("foo.png")),
      creator: Some("foo".parse().unwrap()),
      title: Some("bar".parse().unwrap()),
      ..default()
    };

    assert_eq!(
      PackagesHtml {
        order: Order::default(),
        packages: vec![
          (fingerprint, Some(metadata), Totals::default()),
          (fingerprint, None, Totals::default()),
        ],
        sort: Sort::default(),
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
                <img loading=lazy src=/artwork/{fingerprint}/thumbnail>
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
        order: Order::default(),
        packages: vec![
          (fingerprint, Some(metadata), totals),
          (fingerprint, None, Totals::default()),
        ],
        sort: Sort::default(),
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
                <th><a href=/packages?order=descending>title</a></th>
                <th><a href=/packages?sort=creator>creator</a></th>
                <th><a href=/packages?sort=year>year</a></th>
                <th><a href=/packages?sort=media>media</a></th>
                <th class=right><a href=/packages?sort=files>files</a></th>
                <th class=right><a href=/packages?sort=size>size</a></th>
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

  #[test]
  fn sorted() {
    assert_eq!(
      PackagesHtml {
        order: Order::Descending,
        packages: Vec::new(),
        sort: Sort::Size,
        view: View::List,
      }
      .to_string(),
      unindent(
        "
          <header>
            <h1>Packages</h1>
            <nav>
              <a>List</a> | <a href=/packages?view=grid&amp;sort=size&amp;order=descending>Grid</a>
            </nav>
          </header>
          <table>
            <thead>
              <tr>
                <th><a href=/packages>title</a></th>
                <th><a href=/packages?sort=creator>creator</a></th>
                <th><a href=/packages?sort=year>year</a></th>
                <th><a href=/packages?sort=media>media</a></th>
                <th class=right><a href=/packages?sort=files>files</a></th>
                <th class=right><a href=/packages?sort=size>size</a></th>
              </tr>
            </thead>
            <tbody>
            </tbody>
          </table>
        ",
      ),
    );
  }
}

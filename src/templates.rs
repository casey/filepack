use super::*;

#[derive(Boilerplate)]
pub struct DirectoryHtml {
  pub directory: Directory,
  pub hash: Hash,
}

impl Page for DirectoryHtml {
  fn title(&self) -> String {
    format!("directory {} · filepack", self.hash)
  }
}

#[derive(Boilerplate)]
pub(crate) struct FilesHtml {
  pub(crate) files: Vec<Hash>,
}

impl Page for FilesHtml {
  fn title(&self) -> String {
    "files · filepack".into()
  }
}

#[derive(Boilerplate)]
pub(crate) struct PackagesHtml {
  pub(crate) packages: Vec<(Fingerprint, Option<ComponentBuf>)>,
}

impl Page for PackagesHtml {
  fn title(&self) -> String {
    "packages · filepack".into()
  }
}

#[derive(Boilerplate)]
pub struct PackageHtml {
  pub fingerprint: Fingerprint,
  pub metadata: Option<Metadata>,
}

impl PackageHtml {
  fn title(&self) -> Option<&Component> {
    self.metadata.as_ref()?.title.as_deref()
  }
}

impl Page for PackageHtml {
  fn title(&self) -> String {
    if let Some(title) = self.title() {
      format!("{title} · filepack")
    } else {
      format!("{} · filepack", self.fingerprint)
    }
  }
}

#[derive(Boilerplate)]
pub struct PageHtml<T: Page> {
  content: T,
}

pub trait Page: Display {
  fn title(&self) -> String;
}

impl<T: Page> From<T> for PageHtml<T> {
  fn from(page: T) -> PageHtml<T> {
    PageHtml { content: page }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn directory_listing() {
    let directory = Directory {
      version: Version::Zero,
      entries: BTreeMap::from([
        (
          "bar".parse().unwrap(),
          Entry {
            ty: EntryType::File,
            hash: Hash::bytes(b"bar"),
            size: 1500,
          },
        ),
        (
          "foo".parse().unwrap(),
          Entry {
            ty: EntryType::Directory,
            hash: Hash::bytes(b"foo"),
            size: 2_500_000,
          },
        ),
      ]),
    };

    assert_matches_regex!(
      DirectoryHtml {
        hash: Hash::bytes(b"baz"),
        directory,
      }
      .to_string(),
      unindent(
        r#"
          <h1>Directory [[:xdigit:]]{64}</h1>
          <table>
            <thead>
              <tr>
                <th>name</th>
                <th class=size>size</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td><a href=/file/[[:xdigit:]]{64} download="bar">bar</a></td>
                <td class=size>1\.5 KiB</td>
              </tr>
              <tr>
                <td><a href=/directory/[[:xdigit:]]{64}>foo/</a></td>
                <td class=size>2\.4 MiB</td>
              </tr>
            </tbody>
          </table>
        "#,
      ),
    );
  }
}

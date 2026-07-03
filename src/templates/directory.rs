use super::*;

#[derive(Boilerplate)]
pub struct DirectoryHtml {
  pub directory: Directory,
  pub hash: Hash,
}

impl Page for DirectoryHtml {
  fn stylesheet(&self) -> Option<&'static str> {
    Some("/static/directory.css")
  }

  fn title(&self) -> String {
    format!("directory {} · filepack", self.hash)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn directory() {
    let directory = Directory {
      version: Version::Zero,
      entries: BTreeMap::from([
        (
          "bar".parse().unwrap(),
          Entry {
            ty: EntryType::File,
            hash: Hash::bytes(b"bar"),
            size: 1500,
            totals: None,
          },
        ),
        (
          "baz.png".parse().unwrap(),
          Entry {
            ty: EntryType::File,
            hash: Hash::bytes(b"baz"),
            size: 1500,
            totals: None,
          },
        ),
        (
          "foo".parse().unwrap(),
          Entry {
            ty: EntryType::Directory,
            hash: Hash::bytes(b"foo"),
            size: 100,
            totals: Some(Totals {
              directories: 2,
              directory_size: 500_000,
              file_size: 2_000_000,
              files: 3,
            }),
          },
        ),
        (
          "qux quux.png".parse().unwrap(),
          Entry {
            ty: EntryType::File,
            hash: Hash::bytes(b"qux"),
            size: 1500,
            totals: None,
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
                <td><a href=/file/[[:xdigit:]]{64}/baz\.png>baz\.png</a></td>
                <td class=size>1\.5 KiB</td>
              </tr>
              <tr>
                <td><a href=/directory/[[:xdigit:]]{64}>foo/</a></td>
                <td class=size>2\.4 MiB</td>
              </tr>
              <tr>
                <td><a href=/file/[[:xdigit:]]{64}/qux%20quux\.png>qux quux\.png</a></td>
                <td class=size>1\.5 KiB</td>
              </tr>
            </tbody>
          </table>
        "#,
      ),
    );
  }
}

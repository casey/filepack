use super::*;

#[derive(Boilerplate)]
pub struct DirectoryHtml {
  pub directory: Directory,
  pub hash: Hash,
}

impl Page for DirectoryHtml {
  fn stylesheet(&self) -> Option<&str> {
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
          },
        ),
        (
          "baz.png".parse().unwrap(),
          Entry {
            ty: EntryType::File,
            hash: Hash::bytes(b"baz"),
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
        (
          "qux quux.png".parse().unwrap(),
          Entry {
            ty: EntryType::File,
            hash: Hash::bytes(b"qux"),
            size: 1500,
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

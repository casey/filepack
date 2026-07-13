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
    let mut directory = Directory::new();
    directory
      .insert_entry("bar", Entry::file(Hash::bytes(b"bar"), 1500))
      .insert_entry("baz.png", Entry::file(Hash::bytes(b"baz"), 1500))
      .insert_entry(
        "foo",
        Entry::Directory {
          hash: Hash::bytes(b"foo"),
          size: 2_500_000,
          totals: Totals {
            directories: 1,
            directory_size: 1_000,
            file_size: 10_000_000,
            files: 3,
          },
        },
      )
      .insert_entry("qux quux.png", Entry::file(Hash::bytes(b"qux"), 1500));

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

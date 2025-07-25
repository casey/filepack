use super::*;

#[derive(Boilerplate)]
pub(crate) struct IndexHtml {
  pub(crate) packages: BTreeMap<Hash, Package>,
}

impl PageContent for IndexHtml {
  fn title(&self) -> String {
    "filepack server".into()
  }
}

#[derive(Boilerplate)]
pub(crate) struct PackageHtml {
  pub(crate) package: Package,
}

impl PageContent for PackageHtml {
  fn title(&self) -> String {
    format!("filepack package {}", self.package.fingerprint)
  }
}

#[derive(Boilerplate)]
pub(crate) struct PageHtml<T: PageContent>(pub(crate) T);

pub trait PageContent: Display + 'static {
  fn page(self) -> PageHtml<Self>
  where
    Self: Sized,
  {
    PageHtml(self)
  }

  fn title(&self) -> String;
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn index_empty() {
    assert_eq!(
      IndexHtml {
        packages: BTreeMap::default()
      }
      .to_string(),
      "<ul>\n</ul>\n",
    );
  }

  #[test]
  fn index_with_packages() {
    let fingerprint = Hash::from([0; 32]);

    assert_eq!(
      IndexHtml {
        packages: [(
          fingerprint,
          Package {
            fingerprint,
            manifest: Manifest {
              files: BTreeMap::default(),
              signatures: BTreeMap::default(),
            },
            metadata: None,
          }
        )]
        .into(),
      }
      .to_string(),
      format!(
        "\
<ul>
  <li class=monospace><a href=/package/{fingerprint}>{fingerprint}</a></li>
</ul>
"
      ),
    );
  }

  #[test]
  fn package() {
    let manifest = Manifest {
      files: [
        (
          "hello.txt".parse().unwrap(),
          Entry {
            hash: Hash::bytes(&[]),
            size: 10,
          },
        ),
        (
          "goodbye.txt".parse().unwrap(),
          Entry {
            hash: Hash::bytes(&[]),
            size: 20,
          },
        ),
      ]
      .into(),
      signatures: [(
        "1".repeat(64).parse().unwrap(),
        "1".repeat(128).parse().unwrap(),
      )]
      .into(),
    };

    let fingerprint = manifest.fingerprint();

    pretty_assertions::assert_eq!(
      PackageHtml {
        package: Package {
          fingerprint,
          manifest,
          metadata: Some(Metadata {
            title: "foo".into()
          }),
        }
      }
      .to_string(),
      format!(
        "\
<h1>foo</h1>
<dl>
  <dt>file count</dt>
  <dd>2</dd>
  <dt>total size</dt>
  <dd>30 bytes</dd>
  <dt>fingerprint</dt>
  <dd class=monospace>{fingerprint}</dd>
  <dt>signatures</dt>
  <dd class=monospace>1111111111111111111111111111111111111111111111111111111111111111</dd>
  <dt>files</dt>
  <dd>
    <table>
      <tr>
        <td class=monospace>goodbye.txt</td>
        <td>20 bytes</td>
      </tr>
      <tr>
        <td class=monospace>hello.txt</td>
        <td>10 bytes</td>
      </tr>
    </table>
  </dd>
</dl>
"
      ),
    );
  }
}

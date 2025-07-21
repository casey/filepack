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
        packages: Default::default()
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
              files: Default::default(),
              signatures: Default::default(),
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
}

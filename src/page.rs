use super::*;

#[derive(Boilerplate)]
#[boilerplate(filename = "page.html")]
pub(crate) struct Page {
  pub(crate) manifest: Manifest,
  pub(crate) metadata: Option<Metadata>,
  pub(crate) present: HashSet<RelativePath>,
}

#[cfg(test)]
mod tests {
  use super::*;

  fn hash() -> Hash {
    Hash::bytes(&[])
  }

  fn private_key() -> PrivateKey {
    "0".repeat(64).parse().unwrap()
  }

  fn public_key() -> PublicKey {
    private_key().into()
  }

  fn signature() -> Signature {
    private_key().sign(&[])
  }

  #[test]
  fn display() {
    let page = Page {
      manifest: Manifest {
        files: [(
          "foo".parse().unwrap(),
          Entry {
            hash: hash(),
            size: 1024,
          },
        )]
        .into(),
        signatures: [(public_key(), signature())].into(),
      },
      metadata: Some(Metadata {
        title: "foo".into(),
      }),
      present: ["foo".parse().unwrap()].into(),
    };

    pretty_assertions::assert_eq!(
      page.to_string(),
      r#"<!doctype html>
<html lang=en>
  <head>
    <meta charset=utf-8>
    <meta name=viewport content='width=device-width,initial-scale=1.0'>
    <title>foo</title>
    <style>
      .monospace {
        font-family: monospace;
      }
    </style>
  </head>
  <body>
    <h1>foo</h1>
    <dl>
      <dt>file count</dt>
      <dd>1</dd>
      <dt>total size</dt>
      <dd>1 KiB</dd>
      <dt>fingerprint</dt>
      <dd class=monospace>2e2f6ca534371afe8783a9bcace2237a7611e2e5aa87eb272782b563f70d14ac</dd>
      <dt>signatures</dt>
      <dd class=monospace>3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29</dd>
      <dt>files</dt>
      <dd>
        <table>
          <tr>
            <td class=monospace><a href="foo">foo</a></td>
            <td>1 KiB</td>
          </tr>
        </table>
      </dd>
    </dl>
  </body>
</html>
"#,
    );
  }
}

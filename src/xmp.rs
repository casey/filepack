use {
  super::*,
  roxmltree::{Document, Node},
};

const DC: &str = "http://purl.org/dc/elements/1.1/";
const RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
const X: &str = "adobe:ns:meta/";

fn child<'a, 'input>(
  node: Node<'a, 'input>,
  namespace: &'static str,
  local: &'static str,
  name: &'static str,
) -> Result<Node<'a, 'input>, XmpError> {
  let mut children = children(node);

  let child = children
    .next()
    .context(xmp_error::MissingElement { name })?;

  expect(child, namespace, local, name)?;

  if let Some(extra) = children.next() {
    return Err(unexpected(extra));
  }

  Ok(child)
}

fn children<'a, 'input>(node: Node<'a, 'input>) -> impl Iterator<Item = Node<'a, 'input>> {
  node.children().filter(Node::is_element)
}

fn expect(
  node: Node,
  namespace: &'static str,
  local: &'static str,
  name: &'static str,
) -> Result<(), XmpError> {
  if node.has_tag_name((namespace, local)) {
    Ok(())
  } else if node.tag_name().name() == local {
    Err(xmp_error::MissingElement { name }.build())
  } else {
    Err(unexpected(node))
  }
}

#[cfg(test)]
pub(crate) fn packet(items: &[(&str, &str)]) -> Vec<u8> {
  use fmt::Write;

  let mut list = String::new();

  for (lang, text) in items {
    write!(list, r#"<rdf:li xml:lang="{lang}">{text}</rdf:li>"#).unwrap();
  }

  format!(
    r#"
      <x:xmpmeta xmlns:x="{X}">
        <rdf:RDF xmlns:rdf="{RDF}">
          <rdf:Description xmlns:dc="{DC}">
            <dc:title>
              <rdf:Alt>{list}</rdf:Alt>
            </dc:title>
          </rdf:Description>
        </rdf:RDF>
      </x:xmpmeta>
    "#
  )
  .into_bytes()
}

pub(crate) fn title(data: &[u8]) -> Result<Option<String>, XmpError> {
  let xml = str::from_utf8(data).context(xmp_error::Utf8)?;

  let document = Document::parse(xml).context(xmp_error::Parse)?;

  let root = document.root_element();

  expect(root, X, "xmpmeta", "x:xmpmeta")?;

  let rdf = child(root, RDF, "RDF", "rdf:RDF")?;

  let mut title = None;

  for description in children(rdf) {
    expect(description, RDF, "Description", "rdf:Description")?;

    for property in children(description) {
      if property.has_tag_name((DC, "title")) {
        ensure!(title.is_none(), xmp_error::TitleMultiple);
        title = Some(property);
      }
    }
  }

  let Some(title) = title else {
    return Ok(None);
  };

  let alt = child(title, RDF, "Alt", "rdf:Alt")?;

  let mut item = None;

  for child in children(alt) {
    expect(child, RDF, "li", "rdf:li")?;
    ensure!(item.is_none(), xmp_error::TitleMultiple);
    item = Some(child);
  }

  let item = item.context(xmp_error::MissingElement { name: "rdf:li" })?;

  if let Some(child) = children(item).next() {
    return Err(unexpected(child));
  }

  Ok(Some(item.text().unwrap_or_default().into()))
}

fn unexpected(node: Node) -> XmpError {
  let tag = node.tag_name();

  let name = match node.lookup_prefix(tag.namespace().unwrap_or_default()) {
    Some(prefix) if !prefix.is_empty() => format!("{prefix}:{}", tag.name()),
    _ => tag.name().into(),
  };

  xmp_error::UnexpectedElement { name }.build()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn title() {
    #[track_caller]
    fn error(xml: &str, expected: &str) {
      assert_eq!(
        xmp::title(xml.as_bytes()).unwrap_err().to_string(),
        expected
      );
    }

    #[track_caller]
    fn case(items: &[(&str, &str)], expected: Result<Option<&str>, &str>) {
      assert_eq!(
        xmp::title(&xmp::packet(items)).map_err(|err| err.to_string()),
        expected
          .map(|title| title.map(String::from))
          .map_err(String::from),
      );
    }

    case(&[], Err("missing element `rdf:li`"));
    case(&[("x-default", "foo")], Ok(Some("foo")));
    case(&[("en", "foo")], Ok(Some("foo")));
    case(
      &[("x-default", "foo"), ("en", "bar")],
      Err("multiple `dc:title` values"),
    );

    error(
      r#"<x:xmpmeta xmlns:x="adobe:ns:meta/"/>"#,
      "missing element `rdf:RDF`",
    );

    assert_eq!(
      xmp::title(
        format!(
          r#"
            <x:xmpmeta xmlns:x="{X}">
              <rdf:RDF xmlns:rdf="{RDF}">
                <rdf:Description xmlns:dc="{DC}">
                  <dc:creator>
                    <rdf:Seq>
                      <rdf:li>foo</rdf:li>
                    </rdf:Seq>
                  </dc:creator>
                </rdf:Description>
              </rdf:RDF>
            </x:xmpmeta>
          "#
        )
        .as_bytes()
      )
      .unwrap(),
      None,
    );

    error(
      r#"<x:xmpmeta xmlns:x="adobe:ns:meta/"><foo/></x:xmpmeta>"#,
      "unexpected element `foo`",
    );

    error(
      r#"<x:xmpmeta xmlns:x="adobe:ns:meta/"><rdf:RDF xmlns:rdf="foo"/></x:xmpmeta>"#,
      "missing element `rdf:RDF`",
    );

    error(
      &format!(r#"<rdf:RDF xmlns:rdf="{RDF}"/>"#),
      "unexpected element `rdf:RDF`",
    );

    error(
      &format!(
        r#"
          <x:xmpmeta xmlns:x="{X}">
            <rdf:RDF xmlns:rdf="{RDF}"/>
            <rdf:RDF xmlns:rdf="{RDF}"/>
          </x:xmpmeta>
        "#
      ),
      "unexpected element `rdf:RDF`",
    );

    error(
      &format!(
        r#"
          <x:xmpmeta xmlns:x="{X}">
            <rdf:RDF xmlns:rdf="{RDF}">
              <rdf:Seq/>
            </rdf:RDF>
          </x:xmpmeta>
        "#
      ),
      "unexpected element `rdf:Seq`",
    );

    assert_eq!(
      xmp::title(
        format!(
          r#"
            <x:xmpmeta xmlns:x="{X}">
              <rdf:RDF xmlns:rdf="{RDF}">
                <rdf:Description xmlns:dc="{DC}">
                  <dc:creator>
                    <rdf:Seq>
                      <rdf:li>foo</rdf:li>
                    </rdf:Seq>
                  </dc:creator>
                  <foo>
                    <dc:title>
                      <rdf:Alt>
                        <rdf:li>bar</rdf:li>
                      </rdf:Alt>
                    </dc:title>
                  </foo>
                </rdf:Description>
              </rdf:RDF>
            </x:xmpmeta>
          "#
        )
        .as_bytes()
      )
      .unwrap(),
      None,
    );

    error(
      &format!(
        r#"
          <x:xmpmeta xmlns:x="{X}">
            <rdf:RDF xmlns:rdf="{RDF}">
              <rdf:Description xmlns:dc="{DC}">
                <dc:title>
                  <rdf:Seq>
                    <rdf:li>foo</rdf:li>
                  </rdf:Seq>
                </dc:title>
              </rdf:Description>
            </rdf:RDF>
          </x:xmpmeta>
        "#
      ),
      "unexpected element `rdf:Seq`",
    );

    error(
      &format!(
        r#"
          <x:xmpmeta xmlns:x="{X}">
            <rdf:RDF xmlns:rdf="{RDF}">
              <rdf:Description xmlns:dc="{DC}">
                <dc:title>foo</dc:title>
              </rdf:Description>
            </rdf:RDF>
          </x:xmpmeta>
        "#
      ),
      "missing element `rdf:Alt`",
    );

    error(
      &format!(
        r#"
          <x:xmpmeta xmlns:x="{X}">
            <rdf:RDF xmlns:rdf="{RDF}">
              <rdf:Description xmlns:dc="{DC}">
                <dc:title>
                  <rdf:Alt>
                    <rdf:li>foo</rdf:li>
                    <rdf:Bag/>
                  </rdf:Alt>
                </dc:title>
              </rdf:Description>
            </rdf:RDF>
          </x:xmpmeta>
        "#
      ),
      "unexpected element `rdf:Bag`",
    );

    assert_eq!(
      xmp::title(
        format!(
          r#"
          <x:xmpmeta xmlns:x="{X}">
            <rdf:RDF xmlns:rdf="{RDF}">
              <rdf:Description xmlns:dc="{DC}">
                <dc:title>
                  <rdf:Alt>
                    <rdf:li/>
                  </rdf:Alt>
                </dc:title>
              </rdf:Description>
            </rdf:RDF>
          </x:xmpmeta>
        "#
        )
        .as_bytes()
      )
      .unwrap(),
      Some(String::new()),
    );

    error(
      &format!(
        r#"
          <x:xmpmeta xmlns:x="{X}">
            <rdf:RDF xmlns:rdf="{RDF}">
              <rdf:Description xmlns:dc="{DC}">
                <dc:title>
                  <rdf:Alt>
                    <rdf:li>foo<b/>bar</rdf:li>
                  </rdf:Alt>
                </dc:title>
              </rdf:Description>
            </rdf:RDF>
          </x:xmpmeta>
        "#
      ),
      "unexpected element `b`",
    );

    error(
      &format!(
        r#"
          <x:xmpmeta xmlns:x="{X}">
            <rdf:RDF xmlns:rdf="{RDF}">
              <rdf:Description xmlns:dc="{DC}">
                <dc:title>
                  <rdf:Alt>
                    <rdf:li>foo</rdf:li>
                  </rdf:Alt>
                </dc:title>
                <dc:title>
                  <rdf:Alt>
                    <rdf:li>bar</rdf:li>
                  </rdf:Alt>
                </dc:title>
              </rdf:Description>
            </rdf:RDF>
          </x:xmpmeta>
        "#
      ),
      "multiple `dc:title` values",
    );

    assert_eq!(
      xmp::title(b"<foo").unwrap_err().to_string(),
      "failed to parse XML",
    );

    assert_eq!(
      xmp::title(b"\xff").unwrap_err().to_string(),
      "invalid UTF-8",
    );
  }
}

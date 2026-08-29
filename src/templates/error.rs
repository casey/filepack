use super::*;

#[derive(Boilerplate)]
pub(crate) struct ErrorHtml {
  pub(crate) message: String,
  pub(crate) status: StatusCode,
}

impl Page for ErrorHtml {
  fn stylesheet(&self) -> Option<&'static str> {
    Some("/static/error.css")
  }

  fn title(&self) -> String {
    match self.status.canonical_reason() {
      Some(reason) => format!("{} · filepack", reason.to_lowercase()),
      None => format!("{} · filepack", self.status.as_u16()),
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  #[test]
  fn render() {
    assert_eq!(
      ErrorHtml {
        message: "foo".into(),
        status: StatusCode::NOT_FOUND,
      }
      .to_string(),
      unindent(
        "
          <div class=status>404</div>
          <div class=message>foo</div>
        "
      ),
    );
  }

  #[test]
  fn title() {
    #[track_caller]
    fn case(status: StatusCode, expected: &str) {
      assert_eq!(
        ErrorHtml {
          message: String::new(),
          status,
        }
        .title(),
        expected,
      );
    }

    case(StatusCode::NOT_FOUND, "not found · filepack");
    case(
      StatusCode::INTERNAL_SERVER_ERROR,
      "internal server error · filepack",
    );
    case(StatusCode::from_u16(599).unwrap(), "599 · filepack");
  }
}

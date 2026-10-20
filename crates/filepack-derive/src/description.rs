use super::*;

pub(crate) struct Description {
  rest: Option<String>,
  summary: String,
}

impl Description {
  pub(crate) fn check_summary(ident: &Ident, summary: &str, variant: bool) -> Result {
    if !summary.ends_with('.') {
      return Err(Error::new_spanned(
        ident,
        format!("description `{summary}` does not end with a period"),
      ));
    }

    if !summary.starts_with(|c: char| c.is_ascii_uppercase() || c == '`' || c == '[') {
      return Err(Error::new_spanned(
        ident,
        format!(
          "description `{summary}` does not start with an uppercase letter, backtick, or link"
        ),
      ));
    }

    if !variant {
      for article in ["A ", "An ", "The "] {
        if summary.starts_with(article) {
          return Err(Error::new_spanned(
            ident,
            format!("description `{summary}` starts with an article"),
          ));
        }
      }
    }

    for line in summary.lines() {
      if line.is_empty() || line != line.trim() || line.contains("  ") {
        return Err(Error::new_spanned(
          ident,
          format!("description `{summary}` has irregular whitespace"),
        ));
      }
    }

    Ok(())
  }

  pub(crate) fn new(
    ident: &Ident,
    attributes: &[Attribute],
    noun: &str,
    variant: bool,
    single: bool,
  ) -> Result<Self> {
    let mut lines = Vec::new();

    for attribute in attributes {
      if !attribute.path().is_ident("doc") {
        continue;
      }

      let Expr::Lit(ExprLit {
        lit: Lit::Str(line),
        ..
      }) = &attribute.meta.require_name_value()?.value
      else {
        return Err(Error::new_spanned(
          attribute,
          "`#[doc]` attribute value must be a string literal",
        ));
      };

      let line = line.value();
      lines.push(line.strip_prefix(' ').unwrap_or(&line).to_string());
    }

    if lines.is_empty() {
      return Err(Error::new_spanned(
        ident,
        format!("{noun}s must have doc comments"),
      ));
    }

    let (summary, rest) = if let Some(index) = lines.iter().position(String::is_empty) {
      (&lines[..index], Some(&lines[index + 1..]))
    } else {
      (&lines[..], None)
    };

    let summary = summary.join("\n");

    Self::check_summary(ident, &summary, variant)?;

    if single && rest.is_some() {
      return Err(Error::new_spanned(
        ident,
        format!("{noun} descriptions must be a single paragraph"),
      ));
    }

    let rest = rest.map(|rest| rest.join("\n"));

    let irregular = rest.as_deref().is_some_and(|rest| {
      rest.is_empty()
        || rest.starts_with('\n')
        || rest.ends_with('\n')
        || rest.contains("\n\n\n")
        || rest
          .lines()
          .any(|line| line.trim_end() != line || line.trim_start().contains("  "))
    });

    if irregular {
      return Err(Error::new_spanned(
        ident,
        format!(
          "description `{}` has irregular whitespace",
          lines.join("\n")
        ),
      ));
    }

    Ok(Self { rest, summary })
  }

  pub(crate) fn rest(&self) -> Option<&str> {
    self.rest.as_deref()
  }

  pub(crate) fn summary(&self) -> &str {
    &self.summary
  }
}

impl ToTokens for Description {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    if let Some(rest) = &self.rest {
      format!("{}\n\n{rest}", self.summary).to_tokens(tokens);
    } else {
      self.summary.to_tokens(tokens);
    }
  }
}

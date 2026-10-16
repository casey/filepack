use super::*;

#[derive(Clone, Copy, Display)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Chapter {
  Deco,
  Json,
  Yaml,
}

impl Chapter {
  pub(crate) fn link(self, name: &str) -> String {
    format!("[{}](../{self}/{}.md)", escape(name), name.to_kebab_case())
  }
}

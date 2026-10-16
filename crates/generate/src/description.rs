use super::*;

pub(crate) trait Description {
  fn description(&self, chapter: Chapter, name: &str) -> String;
}

impl Description for deco::Registry {
  fn description(&self, chapter: Chapter, name: &str) -> String {
    match &self[name] {
      deco::Schema::Array { element } => {
        format!("array of {}", self.description(chapter, element))
      }
      deco::Schema::Map { key, value } => format!(
        "map from {} to {}",
        self.description(chapter, key),
        self.description(chapter, value)
      ),
      deco::Schema::OrderedSet { element } => {
        format!("ordered set of {}", self.description(chapter, element))
      }
      deco::Schema::SortedSet { element } => {
        format!("sorted set of {}", self.description(chapter, element))
      }
      _ => chapter.link(name),
    }
  }
}

impl Description for json::Registry {
  fn description(&self, chapter: Chapter, name: &str) -> String {
    match &self[name] {
      json::Schema::Array { element } => {
        format!("array of {}", self.description(chapter, element))
      }
      json::Schema::Map { key, value } => format!(
        "map from {} to {}",
        self.description(chapter, key),
        self.description(chapter, value)
      ),
      json::Schema::Set { element } => {
        format!("sorted set of {}", self.description(chapter, element))
      }
      _ => chapter.link(name),
    }
  }
}

use super::*;

pub(crate) trait PathExtractor {
  fn parameters() -> Vec<reference::Parameter>;
}

impl<A: PathExtractor, B: PathExtractor> PathExtractor for (A, B) {
  fn parameters() -> Vec<reference::Parameter> {
    let mut parameters = A::parameters();
    parameters.extend(B::parameters());
    parameters
  }
}

impl PathExtractor for String {
  fn parameters() -> Vec<reference::Parameter> {
    vec![reference::Parameter::String]
  }
}

impl PathExtractor for u64 {
  fn parameters() -> Vec<reference::Parameter> {
    vec![reference::Parameter::Integer]
  }
}

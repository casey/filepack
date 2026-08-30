#[derive(Debug, PartialEq)]
pub(crate) enum Info {
  Code(String),
  Link { text: String, url: String },
  List(Vec<Info>),
  Map(Vec<(String, Info)>),
  Value(String),
}

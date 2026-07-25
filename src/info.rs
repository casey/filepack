#[derive(Debug, PartialEq)]
pub(crate) enum Info {
  Link { text: String, url: String },
  List(Vec<Info>),
  Map(Vec<(String, Info)>),
  Value(String),
}

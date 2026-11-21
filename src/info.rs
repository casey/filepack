#[derive(Debug, PartialEq)]
pub(crate) enum Info {
  Code(String),
  Link {
    code: bool,
    text: String,
    url: String,
  },
  List(Vec<Info>),
  Map(Vec<(String, Info)>),
  Value(String),
}

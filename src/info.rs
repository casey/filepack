#[derive(Debug, PartialEq)]
pub(crate) enum Info {
  List(Vec<Info>),
  Map(Vec<(String, Info)>),
  Value(String),
}

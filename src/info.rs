use super::*;

#[derive(Debug, PartialEq)]
pub(crate) enum Info {
  Link(Link),
  List(Vec<Info>),
  Map(Vec<(String, Info)>),
  Value(String),
}

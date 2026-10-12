use super::*;

#[derive(Boilerplate)]
pub(crate) struct CommandMd<'a> {
  pub(crate) help: &'a str,
  pub(crate) name: &'a str,
}

#[derive(Boilerplate)]
pub(crate) struct FieldsMd<'a> {
  pub(crate) fields: &'a [schema::Field],
}

#[derive(Boilerplate)]
pub(crate) struct SummaryMd {
  pub(crate) commands: BTreeMap<String, Utf8PathBuf>,
  pub(crate) schemas: BTreeMap<String, Utf8PathBuf>,
}

#[derive(Boilerplate)]
pub(crate) struct SchemaMd<'a> {
  pub(crate) name: &'a str,
  pub(crate) schema: &'a Schema,
}

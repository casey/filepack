use super::*;

#[derive(Boilerplate)]
pub(crate) struct CommandMd<'a> {
  pub(crate) help: &'a str,
  pub(crate) name: &'a str,
}

#[derive(Boilerplate)]
pub(crate) struct CommandsMd<'a> {
  pub(crate) pages: &'a BTreeMap<String, (&'a Command, Utf8PathBuf)>,
}

#[derive(Boilerplate)]
pub(crate) struct DefinitionMd<'a> {
  pub(crate) schema: &'a Schema,
}

#[derive(Boilerplate)]
pub(crate) struct FieldsMd<'a> {
  pub(crate) fields: &'a [Field],
}

#[derive(Boilerplate)]
pub(crate) struct HexMd<'a> {
  pub(crate) tags: &'a BTreeMap<Tag, Hex>,
}

#[derive(Boilerplate)]
pub(crate) struct LanguagesMd<'a> {
  pub(crate) languages: &'a BTreeSet<Language>,
}

#[derive(Boilerplate)]
pub(crate) struct LintsMd<'a> {
  pub(crate) groups: &'a BTreeMap<LintGroup, BTreeSet<Lint>>,
  pub(crate) lints: &'a BTreeSet<Lint>,
}

#[derive(Boilerplate)]
pub(crate) struct RouteMd<'a> {
  pub(crate) path: &'a str,
  pub(crate) route: &'a Route,
}

#[derive(Boilerplate)]
pub(crate) struct RoutesMd<'a> {
  pub(crate) pages: &'a BTreeMap<&'a str, Utf8PathBuf>,
  pub(crate) routes: &'a BTreeMap<String, Route>,
}

#[derive(Boilerplate)]
pub(crate) struct SchemasMd<'a> {
  pub(crate) schemas: &'a BTreeMap<String, Schema>,
}

#[derive(Boilerplate)]
pub(crate) struct SummaryMd<'a> {
  pub(crate) commands: BTreeMap<String, (&'a Command, Utf8PathBuf)>,
  pub(crate) routes: BTreeMap<&'a str, Utf8PathBuf>,
  pub(crate) schemas: BTreeMap<&'a str, Utf8PathBuf>,
  pub(crate) tags: BTreeMap<&'a str, Utf8PathBuf>,
}

#[derive(Boilerplate)]
pub(crate) struct SchemaMd<'a> {
  pub(crate) name: &'a str,
  pub(crate) schema: &'a Schema,
}

#[derive(Boilerplate)]
pub(crate) struct TagMd<'a> {
  pub(crate) hex: &'a Hex,
  pub(crate) tag: Tag,
}

#[derive(Boilerplate)]
pub(crate) struct UnstableFeaturesMd<'a> {
  pub(crate) features: &'a BTreeSet<UnstableFeature>,
}

use super::*;

#[derive(Boilerplate)]
pub(crate) struct ApiMd<'a> {
  pub(crate) pages: &'a BTreeMap<&'a str, Utf8PathBuf>,
  pub(crate) routes: &'a BTreeMap<String, Route>,
}

#[derive(Boilerplate)]
pub(crate) struct CommandMd<'a> {
  pub(crate) help: &'a str,
  pub(crate) input: Option<&'a str>,
  pub(crate) name: &'a str,
  pub(crate) output: Option<&'a str>,
  pub(crate) registry: &'a json::Registry,
}

#[derive(Boilerplate)]
pub(crate) struct CommandsMd<'a> {
  pub(crate) pages: &'a BTreeMap<String, (&'a Command, Utf8PathBuf)>,
}

#[derive(Boilerplate)]
pub(crate) struct ContentTypesMd<'a> {
  pub(crate) content_types: &'a [ContentType],
}

#[derive(Boilerplate)]
pub(crate) struct DecoDefinitionMd<'a> {
  pub(crate) registry: &'a deco::Registry,
  pub(crate) schema: &'a deco::Schema,
}

#[derive(Boilerplate)]
pub(crate) struct DecoFieldsMd<'a> {
  pub(crate) fields: &'a [deco::Field],
  pub(crate) registry: &'a deco::Registry,
}

#[derive(Boilerplate)]
pub(crate) struct DecoMd<'a> {
  pub(crate) deco: &'a deco::Registry,
}

#[derive(Boilerplate)]
pub(crate) struct DecoSchemaMd<'a> {
  pub(crate) name: &'a str,
  pub(crate) registry: &'a deco::Registry,
  pub(crate) schema: &'a deco::Schema,
}

#[derive(Boilerplate)]
pub(crate) struct HexMd<'a> {
  pub(crate) hex: &'a BTreeMap<Tag, Hex>,
}

#[derive(Boilerplate)]
pub(crate) struct JsonFieldsMd<'a> {
  pub(crate) chapter: Chapter,
  pub(crate) fields: &'a [json::Field],
  pub(crate) registry: &'a json::Registry,
}

#[derive(Boilerplate)]
pub(crate) struct JsonMd<'a> {
  pub(crate) json: &'a json::Registry,
}

#[derive(Boilerplate)]
pub(crate) struct JsonSchemaMd<'a> {
  pub(crate) chapter: Chapter,
  pub(crate) name: &'a str,
  pub(crate) registry: &'a json::Registry,
  pub(crate) schema: &'a json::Schema,
}

#[derive(Boilerplate)]
pub(crate) struct LanguagesMd<'a> {
  pub(crate) languages: &'a BTreeMap<Language, String>,
}

#[derive(Boilerplate)]
pub(crate) struct LintsMd<'a> {
  pub(crate) groups: &'a BTreeMap<LintGroup, reference::LintGroup>,
  pub(crate) junk_names: &'a [String],
  pub(crate) lints: &'a BTreeMap<Lint, String>,
  pub(crate) windows_reserved_characters: &'a [char],
  pub(crate) windows_reserved_names: &'a [String],
}

#[derive(Boilerplate)]
pub(crate) struct MediaFormatsMd<'a> {
  pub(crate) media_formats: &'a BTreeMap<MediaType, Vec<MediaFormat>>,
}

#[derive(Boilerplate)]
pub(crate) struct MetadataMd<'a> {
  pub(crate) example: &'a yaml::Example,
}

#[derive(Boilerplate)]
pub(crate) struct RouteMd<'a> {
  pub(crate) path: &'a str,
  pub(crate) registry: &'a deco::Registry,
  pub(crate) route: &'a Route,
}

#[derive(Boilerplate)]
pub(crate) struct SummaryMd<'a> {
  pub(crate) api: BTreeMap<&'a str, Utf8PathBuf>,
  pub(crate) commands: BTreeMap<String, (&'a Command, Utf8PathBuf)>,
  pub(crate) deco: BTreeMap<&'a str, Utf8PathBuf>,
  pub(crate) hex: BTreeMap<&'a str, Utf8PathBuf>,
  pub(crate) json: BTreeMap<&'a str, Utf8PathBuf>,
  pub(crate) web: BTreeMap<&'a str, Utf8PathBuf>,
  pub(crate) yaml: BTreeMap<&'a str, Utf8PathBuf>,
}

#[derive(Boilerplate)]
pub(crate) struct TagMd<'a> {
  pub(crate) hex: &'a Hex,
  pub(crate) tag: Tag,
}

#[derive(Boilerplate)]
pub(crate) struct UnstableFeaturesMd<'a> {
  pub(crate) features: &'a BTreeMap<UnstableFeature, String>,
}

#[derive(Boilerplate)]
pub(crate) struct WebMd<'a> {
  pub(crate) pages: &'a BTreeMap<&'a str, Utf8PathBuf>,
  pub(crate) routes: &'a BTreeMap<String, Route>,
}

#[derive(Boilerplate)]
pub(crate) struct YamlMd<'a> {
  pub(crate) yaml: &'a json::Registry,
}

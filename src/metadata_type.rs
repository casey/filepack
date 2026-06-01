use super::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields, tag = "type")]
enum MetadataType {
  Audio { tracks: Vec<ComponentBuf> },
  Video { main: ComponentBuf },
}

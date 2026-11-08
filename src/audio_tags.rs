use super::*;

pub(crate) struct AudioTags {
  pub(crate) album: &'static str,
  pub(crate) artist: &'static str,
  pub(crate) disc: Position,
  pub(crate) title: &'static str,
  pub(crate) track: Position,
}

impl AudioTags {
  pub(crate) fn map(&self) -> BTreeMap<String, String> {
    let Self {
      album,
      artist,
      disc,
      title,
      track,
    } = self;

    let (disc, disc_total) = disc.tuple();
    let (track, track_total) = track.tuple();

    [
      ("album", *album),
      ("artist", *artist),
      ("disc", disc),
      ("disc total", disc_total),
      ("title", *title),
      ("track", track),
      ("track total", track_total),
    ]
    .into_iter()
    .map(|(field, tag)| (field.into(), tag.into()))
    .collect()
  }
}

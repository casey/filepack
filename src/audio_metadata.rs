use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct AudioMetadata {
  pub(crate) album: Text,
  pub(crate) artist: Text,
  pub(crate) channels: u64,
  pub(crate) disc: u64,
  pub(crate) discs: u64,
  pub(crate) sample_bits: Option<u64>,
  pub(crate) sample_rate: u64,
  pub(crate) samples: u64,
  pub(crate) size: u64,
  pub(crate) title: Text,
  pub(crate) track: u64,
  pub(crate) tracks: u64,
}

impl AudioMetadata {
  pub(crate) fn check_positions(
    tracks: &[(RelativePath, Self, AudioType)],
  ) -> Result<(), AudioPositionError> {
    let Some((_, first, _ty)) = tracks.first() else {
      return Ok(());
    };

    let discs = first.discs;

    let mut expected_disc = 1;
    let mut expected_track = 1;
    let mut disc_tracks = 0;

    for (path, audio, _ty) in tracks {
      ensure! {
        audio.discs == discs,
        audio_position_error::DiscTotalMismatch {
          actual: audio.discs,
          expected: discs,
          path: path.clone(),
        },
      }

      ensure! {
        audio.disc == expected_disc && audio.track == expected_track,
        audio_position_error::PositionMismatch {
          disc: audio.disc,
          expected_disc,
          expected_track,
          path: path.clone(),
          track: audio.track,
        },
      }

      ensure! {
        audio.disc <= discs,
        audio_position_error::DiscNumberExceedsTotal {
          path: path.clone(),
          number: audio.disc,
          total: discs,
        },
      }

      if expected_track == 1 {
        disc_tracks = audio.tracks;
      } else {
        ensure! {
          audio.tracks == disc_tracks,
          audio_position_error::TotalMismatch {
            actual: audio.tracks,
            disc: expected_disc,
            expected: disc_tracks,
            path: path.clone(),
          },
        }
      }

      ensure! {
        audio.track <= disc_tracks,
        audio_position_error::NumberExceedsTotal {
          path: path.clone(),
          number: audio.track,
          total: disc_tracks,
        },
      }

      if expected_track == disc_tracks {
        expected_disc += 1;
        expected_track = 1;
      } else {
        expected_track += 1;
      }
    }

    ensure! {
      expected_disc == discs + 1,
      audio_position_error::Missing {
        disc: expected_disc,
        track: expected_track,
      },
    }

    Ok(())
  }

  pub(crate) fn check_tags(&self, path: &RelativePath, creator: &Text, title: &Text) -> Result {
    for (tag, field, actual, expected) in [
      ("artist", "creator", &self.artist, creator),
      ("album", "title", &self.album, title),
    ] {
      ensure! {
        actual == expected,
        error::AudioTagMismatch {
          actual: actual.clone(),
          expected: expected.clone(),
          field,
          path: path.clone(),
          tag,
        },
      }
    }

    Ok(())
  }

  pub(crate) fn into_item(self, path: RelativePath, ty: AudioType) -> Item<Audio> {
    Item {
      content: Audio {
        channels: self.channels,
        disc: self.disc,
        discs: self.discs,
        path,
        sample_bits: self.sample_bits,
        sample_rate: self.sample_rate,
        samples: self.samples,
        size: self.size,
        ty: Some(ty),
      },
      title: Some(self.title),
    }
  }

  pub(crate) fn load(root: &Utf8Path, path: &RelativePath) -> Result<(Self, AudioType)> {
    let ty = AudioType::from_path(path).context(error::Path { path })?;

    let metadata = match ty {
      AudioType::Flac => FlacDecoder::read(&root.join(path))?,
      AudioType::Mp3 => Mp3Decoder::read(&root.join(path))?,
    };

    Ok((metadata, ty))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn check_positions() {
    #[track_caller]
    fn case(positions: &[(u64, u64, u64, u64)], expected: Result<(), AudioPositionError>) {
      let tracks = positions
        .iter()
        .enumerate()
        .map(|(i, (disc, discs, track, tracks))| {
          (
            format!("{i}.flac").parse::<RelativePath>().unwrap(),
            AudioMetadata {
              album: "foo".parse().unwrap(),
              artist: "bar".parse().unwrap(),
              channels: 2,
              disc: *disc,
              discs: *discs,
              sample_bits: Some(16),
              sample_rate: 44100,
              samples: 44100,
              size: 1024,
              title: "baz".parse().unwrap(),
              track: *track,
              tracks: *tracks,
            },
            AudioType::Flac,
          )
        })
        .collect::<Vec<(RelativePath, AudioMetadata, AudioType)>>();

      assert_eq!(AudioMetadata::check_positions(&tracks), expected);
    }

    case(&[], Ok(()));

    case(&[(1, 1, 1, 1)], Ok(()));

    case(&[(1, 2, 1, 2), (1, 2, 2, 2), (2, 2, 1, 1)], Ok(()));

    case(
      &[(1, 1, 2, 2), (1, 1, 1, 2)],
      Err(AudioPositionError::PositionMismatch {
        disc: 1,
        expected_disc: 1,
        expected_track: 1,
        path: "0.flac".parse().unwrap(),
        track: 2,
      }),
    );

    case(
      &[(1, 1, 1, 2), (1, 1, 1, 2)],
      Err(AudioPositionError::PositionMismatch {
        disc: 1,
        expected_disc: 1,
        expected_track: 2,
        path: "1.flac".parse().unwrap(),
        track: 1,
      }),
    );

    case(
      &[(1, 1, 1, 3), (1, 1, 3, 3)],
      Err(AudioPositionError::PositionMismatch {
        disc: 1,
        expected_disc: 1,
        expected_track: 2,
        path: "1.flac".parse().unwrap(),
        track: 3,
      }),
    );

    case(
      &[(1, 1, 1, 2)],
      Err(AudioPositionError::Missing { disc: 1, track: 2 }),
    );

    case(
      &[(1, 2, 1, 1)],
      Err(AudioPositionError::Missing { disc: 2, track: 1 }),
    );

    case(
      &[(1, 2, 1, 1), (2, 1, 1, 1)],
      Err(AudioPositionError::DiscTotalMismatch {
        actual: 1,
        expected: 2,
        path: "1.flac".parse().unwrap(),
      }),
    );

    case(
      &[(1, 1, 1, 2), (1, 1, 2, 3)],
      Err(AudioPositionError::TotalMismatch {
        actual: 3,
        disc: 1,
        expected: 2,
        path: "1.flac".parse().unwrap(),
      }),
    );

    case(
      &[(1, 1, 1, 1), (2, 1, 1, 1)],
      Err(AudioPositionError::DiscNumberExceedsTotal {
        path: "1.flac".parse().unwrap(),
        number: 2,
        total: 1,
      }),
    );

    case(
      &[(1, 0, 1, 1)],
      Err(AudioPositionError::DiscNumberExceedsTotal {
        path: "0.flac".parse().unwrap(),
        number: 1,
        total: 0,
      }),
    );

    case(
      &[(1, 1, 1, 0)],
      Err(AudioPositionError::NumberExceedsTotal {
        path: "0.flac".parse().unwrap(),
        number: 1,
        total: 0,
      }),
    );

    case(
      &[(0, 1, 1, 1)],
      Err(AudioPositionError::PositionMismatch {
        disc: 0,
        expected_disc: 1,
        expected_track: 1,
        path: "0.flac".parse().unwrap(),
        track: 1,
      }),
    );

    case(
      &[(1, 1, 0, 1)],
      Err(AudioPositionError::PositionMismatch {
        disc: 1,
        expected_disc: 1,
        expected_track: 1,
        path: "0.flac".parse().unwrap(),
        track: 0,
      }),
    );
  }
}

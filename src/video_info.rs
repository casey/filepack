use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct VideoInfo {
  pub(crate) audio_codec: AudioCodec,
  pub(crate) dimensions: Dimensions,
  pub(crate) video_codec: VideoCodec,
}

impl VideoInfo {
  pub(crate) fn check(&self, video: &Video) -> Result<(), VideoError> {
    ensure! {
      self.dimensions == video.dimensions,
      video_error::DimensionsMismatch {
        actual: self.dimensions,
        expected: video.dimensions,
      },
    }

    ensure! {
      self.video_codec == video.video_codec,
      video_error::VideoCodecMismatch {
        actual: self.video_codec,
        expected: video.video_codec,
      },
    }

    ensure! {
      self.audio_codec == video.audio_codec,
      video_error::AudioCodecMismatch {
        actual: self.audio_codec,
        expected: video.audio_codec,
      },
    }

    Ok(())
  }

  pub(crate) fn decode<T: Read>(reader: &mut T) -> Result<Self, VideoError> {
    let context = mp4parse::read_mp4(reader).context(video_error::Decode)?;

    let mut audio = None;
    let mut video = None;

    for track in &context.tracks {
      match &track.track_type {
        mp4parse::TrackType::Audio => {
          ensure!(audio.is_none(), video_error::AudioTrackMultiple);
          audio = Some(track);
        }
        mp4parse::TrackType::Video => {
          ensure!(video.is_none(), video_error::VideoTrackMultiple);
          video = Some(track);
        }
        ty => {
          return video_error::TrackUnsupported {
            ty: format!("{ty:?}"),
          }
          .fail();
        }
      }
    }

    let video = video.context(video_error::VideoTrackMissing)?;
    let audio = audio.context(video_error::AudioTrackMissing)?;

    let mp4parse::SampleEntry::Video(video) = Self::description(video)? else {
      return video_error::VideoCodecUnsupported { codec: "unknown" }.fail();
    };

    let video_codec = match video.codec_type {
      mp4parse::CodecType::H263 => VideoCodec::H263,
      codec => {
        return video_error::VideoCodecUnsupported {
          codec: format!("{codec:?}"),
        }
        .fail();
      }
    };

    let dimensions = Dimensions {
      height: video.height.into(),
      width: video.width.into(),
    };

    let mp4parse::SampleEntry::Audio(audio) = Self::description(audio)? else {
      return video_error::AudioCodecUnsupported { codec: "unknown" }.fail();
    };

    let audio_codec = match audio.codec_type {
      mp4parse::CodecType::AAC => AudioCodec::Aac,
      mp4parse::CodecType::MP3 => AudioCodec::Mp3,
      codec => {
        return video_error::AudioCodecUnsupported {
          codec: format!("{codec:?}"),
        }
        .fail();
      }
    };

    Ok(Self {
      audio_codec,
      dimensions,
      video_codec,
    })
  }

  fn description(track: &mp4parse::Track) -> Result<&mp4parse::SampleEntry, VideoError> {
    let descriptions = track
      .stsd
      .as_ref()
      .map(|stsd| &*stsd.descriptions)
      .unwrap_or_default();

    ensure! {
      descriptions.len() == 1,
      video_error::SampleDescriptions {
        count: descriptions.len(),
      },
    }

    Ok(&descriptions[0])
  }
}

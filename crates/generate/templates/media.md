Media
=====
%% for ty in self.media_types {
%%   match ty {
%%     MediaType::Audio => {

Audio
-----

A collection of audio tracks.

### Formats

{{ FormatsMd { formats: &self.media_formats[ty] } }}

### Tags

| field | FLAC | MP3 | MP4 |
|-------|------|-----|-----|
| album | `ALBUM` | `TALB` | `©alb` |
| artist | `ARTIST` | `TPE1` | `©ART` |
| disc | `DISCNUMBER` | `TPOS` | `disk` |
| disc total | `DISCTOTAL` | `TPOS` | `disk` |
| title | `TITLE` | `TIT2` | `©nam` |
| track | `TRACKNUMBER` | `TRCK` | `trkn` |
| track total | `TRACKTOTAL` | `TRCK` | `trkn` |

A number and total with the same tag name means the tag contains both values.
ID3 frames contain the number and total separated by a slash, for example
`5/14`, and MP4 atoms contain a pair of integers.
%%     }
%%     MediaType::Image => {

Image
-----

A collection of images.

### Formats

{{ FormatsMd { formats: &self.media_formats[ty] } }}

### Tags

| field | JPEG | PNG |
|-------|------|-----|
| orientation | EXIF `Orientation` | EXIF `Orientation` |
| title | XMP `dc:title` | `Title` text chunk |
%%     }
%%     MediaType::Video => {

Video
-----

A collection of videos.

### Formats

{{ FormatsMd { formats: &self.media_formats[ty] } }}

### Tags

| field | MP4 | WebM |
|-------|-----|------|
| title | `©nam` | segment info `Title` |
%%     }
%%     MediaType::Web => {

Web
---

Arbitrary web content. Currently unstable. Content is served out of the `static`
directory under the package root with `static/index.html` as the entry point.
%%     }
%%   }
%% }

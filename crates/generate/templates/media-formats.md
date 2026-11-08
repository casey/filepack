Media Formats
=============

| media type | format | extension | description |
|------------|--------|-----------|-------------|
%% for (media, formats) in self.media_formats {
%%   for format in formats {
{{ Row((media, &format.name, format!("`{}`", format.extension), &format.description)) }}
%%   }
%% }

Tags
----

### Audio

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

### Image

| field | JPEG | PNG |
|-------|------|-----|
| orientation | EXIF `Orientation` | EXIF `Orientation` |
| title | XMP `dc:title` | `Title` text chunk |

### Video

| field | MP4 | WebM |
|-------|-----|------|
| title | `©nam` | segment info `Title` |

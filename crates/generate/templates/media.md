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

### Tagging

FLAC files can be tagged with [metaflac](https://xiph.org/flac/). If necessary,
first remove old artwork:

```sh
metaflac --remove --block-type=PICTURE foo.flac
```

Then tag and add new artwork:

```sh
metaflac \
  --remove-tag=ALBUM --set-tag=ALBUM=foo \
  --remove-tag=ARTIST --set-tag=ARTIST=bar \
  --remove-tag=DISCNUMBER --set-tag=DISCNUMBER=1 \
  --remove-tag=DISCTOTAL --set-tag=DISCTOTAL=2 \
  --remove-tag=TITLE --set-tag=TITLE=baz \
  --remove-tag=TRACKNUMBER --set-tag=TRACKNUMBER=3 \
  --remove-tag=TRACKTOTAL --set-tag=TRACKTOTAL=4 \
  --import-picture-from=cover.jpg \
  foo.flac
```

MP3s can be tagged with [eyeD3](https://eyed3.readthedocs.io/):

```sh
eyeD3 \
  --album foo \
  --artist bar \
  --disc-num 1 \
  --disc-total 2 \
  --title baz \
  --track 3 \
  --track-total 4 \
  --remove-all-images \
  --add-image cover.jpg:FRONT_COVER \
  foo.mp3
```

MP4s can be tagged with [ExifTool](https://exiftool.org/):

```sh
exiftool \
  -overwrite_original \
  -ItemList:Album=foo \
  -ItemList:Artist=bar \
  -ItemList:DiskNumber='1 of 2' \
  -ItemList:Title=baz \
  -ItemList:TrackNumber='3 of 4' \
  '-ItemList:CoverArt<=cover.jpg' \
  foo.m4a
```
%%     }
%%     MediaType::Document => {

Document
--------

A collection of documents.

### Formats

{{ FormatsMd { formats: &self.media_formats[ty] } }}
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

### Tagging

Images may be tagged with [ExifTool](https://exiftool.org/).

For JPEGs:

```sh
exiftool -overwrite_original -XMP-dc:Title=foo foo.jpg
```

For PNGs:

```sh
exiftool -overwrite_original -PNG:Title=foo foo.png
```

If necessary, orientation may be corrected:

```sh
exiftool -overwrite_original -Orientation='Rotate 90 CW' foo.jpg
```
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

### Tagging

MP4s may be tagged with [ExifTool](https://exiftool.org/):

```sh
exiftool -overwrite_original -ItemList:Title=foo foo.mp4
```

WebM videos may be tagged with
[mkvpropedit](https://mkvtoolnix.download/doc/mkvpropedit.html):

```sh
mkvpropedit foo.webm --edit info --set title=foo
```
%%     }
%%     MediaType::Web => {

Web
---

Arbitrary web content. Currently unstable. Content is served out of the `static`
directory under the package root with `static/index.html` as the entry point.
%%     }
%%   }
%% }

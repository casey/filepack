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

%% let formats = self.audio_tags.values().next().unwrap().keys().map(String::as_str);
{{ Row::new("field").extend(formats.clone()) }}
{{ Row(vec!["---"; formats.len() + 1]) }}
%% for (field, tags) in self.audio_tags {
{{ Row::new(field.clone()).extend(tags.values().map(|tag| format!("`{tag}`"))) }}
%% }

A number and total with the same tag name means the tag contains both values.
ID3 frames contain the number and total separated by a slash, for example
`5/14`, and MP4 atoms contain a pair of integers.

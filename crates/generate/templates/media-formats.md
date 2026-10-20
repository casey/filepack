Media Formats
=============

| media type | format | extension | description |
|------------|--------|-----------|-------------|
%% for (media, formats) in self.media_formats {
%%   for format in formats {
{{ Row((media, &format.name, format!("`{}`", format.extension), &format.description)) }}
%%   }
%% }

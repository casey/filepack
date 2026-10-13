Media Formats
=============

| media type | format | extension |
|------------|--------|-----------|
%% for (media, formats) in self.media_formats {
%%   for format in formats {
| {{ media }} | {{ format.name }} | `{{ format.extension }}` |
%%   }
%% }

Languages
=========

A [Language](schemas/language.md) is an [ISO
639-1](https://en.wikipedia.org/wiki/ISO_639-1) language code:


| code | language |
|------|----------|
%% for (language, name) in self.languages {
| `{{ language }}` | {{ name }} |
%% }

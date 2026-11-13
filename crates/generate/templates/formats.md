| format | extension | description |
|--------|-----------|-------------|
%% for format in self.formats {
{{ Row((&format.name, format!("`{}`", format.extension), &format.description)) }}
%% }

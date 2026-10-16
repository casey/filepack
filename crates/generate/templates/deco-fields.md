| n | field | type | optional |
|---|-------|------|----------|
%% for field in self.fields {
| {{ field.n }} | {{ field.name }} | {{ self.registry.description(Chapter::Deco, &field.schema) }} | {{ field.optional }} |
%% }

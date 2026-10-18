| n | field | type | optional | description |
|---|-------|------|----------|-------------|
%% for field in self.fields {
| {{ field.n }} | {{ field.name }} | {{ self.registry.description(Chapter::Deco, &field.schema) }} | {{ field.optional }} | {{ field.description.replace('\n', " ") }} |
%% }

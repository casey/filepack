| field | type | optional |
|-------|------|----------|
%% for field in self.fields {
| {{ field.name }} | {{ self.registry.description(self.chapter, &field.schema) }} | {{ field.optional }} |
%% }

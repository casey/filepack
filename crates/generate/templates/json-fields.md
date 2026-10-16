| field | type | optional |
|-------|------|----------|
%% for field in self.fields {
| {{ field.name }} | {{ description(self.chapter, self.registry, &field.schema) }} | {{ field.optional }} |
%% }

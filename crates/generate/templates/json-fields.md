| field | type | optional | description |
|-------|------|----------|-------------|
%% for field in self.fields {
| {{ field.name }} | {{ self.registry.description(self.chapter, &field.schema) }} | {{ field.optional }} | {{ field.description() }} |
%% }

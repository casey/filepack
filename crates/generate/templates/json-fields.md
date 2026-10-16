| field | type | optional |
|-------|------|----------|
%% for field in self.fields {
| {{ field.name }} | {{ link(self.chapter, &field.schema) }} | {{ field.optional }} |
%% }

| field | type | optional |
|-------|------|----------|
%% for field in self.fields {
| {{ field.name }} | {{ yaml(&field.schema) }} | {{ field.optional }} |
%% }

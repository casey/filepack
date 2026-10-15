| field | type | optional |
|-------|------|----------|
%% for field in self.fields {
| {{ field.name }} | {{ yaml(&field.schema) }} | {{ if field.optional { "yes" } else { "no" } }} |
%% }

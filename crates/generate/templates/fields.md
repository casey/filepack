| n | field | type | optional |
|---|-------|------|----------|
%% for field in self.fields {
| {{ field.n }} | {{ field.name }} | {{ link(&field.schema) }} | {{ if field.optional { "yes" } else { "no" } }} |
%% }

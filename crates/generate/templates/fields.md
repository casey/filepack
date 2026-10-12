| n | field | type | optional |
|---|-------|------|----------|
%% for field in self.fields {
| {{ field.n }} | {{ field.name }} | {{ schema(&field.schema) }} | {{ if field.optional { "yes" } else { "no" } }} |
%% }

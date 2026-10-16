| n | field | type | optional |
|---|-------|------|----------|
%% for field in self.fields {
| {{ field.n }} | {{ field.name }} | {{ deco(&field.schema) }} | {{ field.optional }} |
%% }

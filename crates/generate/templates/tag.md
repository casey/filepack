# {{ self.prefix }}

A tagged hex [{{ escape(&self.tag.schema) }}](../schemas/{{ self.tag.schema.to_kebab_case() }}.md).

Example:

```
{{ self.tag.example }}
```

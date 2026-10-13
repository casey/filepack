# {{ self.tag.prefix() }}

A tagged hex [{{ escape(&self.hex.schema) }}](../schemas/{{ self.hex.schema.to_kebab_case() }}.md).

Example:

```
{{ self.hex.example }}
```

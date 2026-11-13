# {{ self.tag.tag() }}

{{ self.hex.description }}

A tagged hex [{{ escape(&self.hex.schema) }}](../deco/{{ self.hex.schema.to_kebab_case() }}.md).

## Example

{{ self.hex.example_description }}

```
{{ self.hex.example }}
```

{%- set file_name = ctx.root_namespace | snake_case -%}
{{- template.set_file_name("span/" ~ file_name ~ ".md") -}}

## Namespace Span `{{ ctx.root_namespace }}`

{% for span in ctx.spans %}
## Span `{{ span.id }}`

{{ span.brief | trim }}
{{ span.note | trim }}
Prefix: {{ span.prefix }}
Kind: {{ span.span_kind }}

### Attributes

{% for attribute in span.attributes %}
#### Attribute `{{ attribute.name }}`

{{ attribute.brief }}

{% if attribute.note %}
{{ attribute.note | trim }}
{% endif %}

{%- if attribute.requirement_level == "required" %}
- Requirement Level: Required
  {%- elif attribute.requirement_level.conditionally_required %}
- Requirement Level: Conditionally Required - {{ attribute.requirement_level.conditionally_required }}
  {%- elif attribute.requirement_level == "recommended" %}
- Requirement Level: Recommended
  {%- else %}
- Requirement Level: Optional
  {%- endif %}
  {% if attribute.tag %}
- Tag: {{ attribute.tag }}
  {% endif %}
  {%- include "attribute_type.j2" %}
  {%- include "examples.j2" -%}
  {%- if attribute.sampling_relevant %}
- Sampling relevant: {{ attribute.sampling_relevant }}
  {%- endif %}
  {%- if attribute.deprecated %}
- Deprecated: {{ attribute.deprecated }}
  {%- endif %}
  {% if attribute.stability %}
- Stability: {{ attribute.stability | capitalize }}
  {% endif %}
{% endfor %}
{% endfor %} 
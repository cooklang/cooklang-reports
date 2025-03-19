{% set total = 0 %}
# Cost Report

{%- for ingredient in ingredients %}
{%- set price = db(ingredient.name ~ '/shopping.price_per_1') * ingredient.quantity %}
* {{ ingredient.name }}: {{ price | format_price(2) }}
{%- set total = total + price %}
{%- endfor %}

Total: {{ total | format_price(2) }}

# Qdrant REST API Filtering Specification

## Overview

Qdrant's filtering system allows you to apply conditions when searching or retrieving points based on payload fields and point IDs. Filters are essential for expressing features that cannot be captured in embeddings alone, such as price ranges, availability, categories, or geographic constraints.

## REST API Endpoints Supporting Filters

### Primary Endpoints

- **POST** `/collections/{collection_name}/points/search` - Vector search with filters
- **POST** `/collections/{collection_name}/points/scroll` - List/iterate points with filters
- **POST** `/collections/{collection_name}/points/delete` - Delete points matching filters
- **PUT** `/collections/{collection_name}/points/payload` - Update payload for filtered points
- **POST** `/collections/{collection_name}/points/payload/delete` - Delete payload fields for filtered points
- **POST** `/collections/{collection_name}/points/count` - Count points matching filters

## Filter Structure

All filters are specified in the `filter` object within the request body:

```json
{
  "filter": {
    // Filter clauses and conditions
  }
}
```

## Filtering Clauses

### 1. Must Clause (AND)

All conditions must be satisfied.

```json
{
  "filter": {
    "must": [
      { "key": "city", "match": { "value": "London" } },
      { "key": "color", "match": { "value": "red" } }
    ]
  }
}
```

### 2. Should Clause (OR)

At least one condition must be satisfied.

```json
{
  "filter": {
    "should": [
      { "key": "city", "match": { "value": "London" } },
      { "key": "color", "match": { "value": "red" } }
    ]
  }
}
```

### 3. Must Not Clause (NOT)

None of the conditions can be satisfied.

```json
{
  "filter": {
    "must_not": [
      { "key": "city", "match": { "value": "London" } },
      { "key": "color", "match": { "value": "red" } }
    ]
  }
}
```

### 4. Clause Combinations

Combine multiple clauses with AND logic:

```json
{
  "filter": {
    "must": [{ "key": "city", "match": { "value": "London" } }],
    "must_not": [{ "key": "color", "match": { "value": "red" } }]
  }
}
```

### 5. Nested Clauses

Recursively nest clauses for complex boolean expressions:

```json
{
  "filter": {
    "must_not": [
      {
        "must": [
          { "key": "city", "match": { "value": "London" } },
          { "key": "color", "match": { "value": "red" } }
        ]
      }
    ]
  }
}
```

## Filtering Conditions

### Match

Exact value matching for `keyword`, `integer`, and `bool` fields.

```json
{ "key": "color", "match": { "value": "red" } }
{ "key": "count", "match": { "value": 0 } }
{ "key": "active", "match": { "value": true } }
```

### Match Any

Match any value from a list (IN operator). Available for `keyword` and `integer` fields.

```json
{ "key": "color", "match": { "any": ["black", "yellow"] } }
{ "key": "size", "match": { "any": [1, 2, 3] } }
```

### Match Except

Exclude specific values (NOT IN operator). Available for `keyword` and `integer` fields.

```json
{ "key": "color", "match": { "except": ["black", "yellow"] } }
{ "key": "status", "match": { "except": ["deleted", "archived"] } }
```

### Range

Numeric range filtering for `float` and `integer` fields.

```json
{
  "key": "price",
  "range": {
    "gt": 100.0, // Greater than
    "gte": null, // Greater than or equal (optional)
    "lt": 500.0, // Less than
    "lte": null // Less than or equal (optional)
  }
}
```

### Datetime Range

RFC 3339 datetime filtering (automatically converted to UTC).

```json
{
  "key": "created_at",
  "range": {
    "gt": "2023-02-08T10:49:00Z",
    "gte": null,
    "lt": "2024-01-31T10:14:31Z",
    "lte": null
  }
}
```

### UUID Match

Efficient UUID value matching.

```json
{
  "key": "uuid",
  "match": {
    "value": "f47ac10b-58cc-4372-a567-0e02b2c3d479"
  }
}
```

### Full Text Match

Text search within fields (requires full-text index).

```json
{
  "key": "description",
  "match": {
    "text": "good cheap"
  }
}
```

### Nested Key Access

Access nested fields using dot notation and array projection.

```json
// Access nested field
{ "key": "country.name", "match": { "value": "Germany" } }

// Search through array values
{ "key": "country.cities[].population", "range": { "gte": 9.0 } }

// Match in nested arrays
{ "key": "country.cities[].sightseeing", "match": { "value": "Osaka Castle" } }
```

### Nested Object Filter

Apply conditions to array elements independently.

```json
{
  "filter": {
    "must": [
      {
        "nested": {
          "key": "diet",
          "filter": {
            "must": [
              { "key": "food", "match": { "value": "meat" } },
              { "key": "likes", "match": { "value": true } }
            ]
          }
        }
      }
    ]
  }
}
```

### Geo Filters

#### Geo Bounding Box

```json
{
  "key": "location",
  "geo_bounding_box": {
    "bottom_right": { "lon": 13.455868, "lat": 52.495862 },
    "top_left": { "lon": 13.403683, "lat": 52.520711 }
  }
}
```

#### Geo Radius

```json
{
  "key": "location",
  "geo_radius": {
    "center": { "lon": 13.403683, "lat": 52.520711 },
    "radius": 1000.0 // meters
  }
}
```

#### Geo Polygon

```json
{
  "key": "location",
  "geo_polygon": {
    "exterior": {
      "points": [
        { "lon": -70.0, "lat": -70.0 },
        { "lon": 60.0, "lat": -70.0 },
        { "lon": 60.0, "lat": 60.0 },
        { "lon": -70.0, "lat": 60.0 },
        { "lon": -70.0, "lat": -70.0 }
      ]
    },
    "interiors": [] // Optional interior rings
  }
}
```

### Values Count

Filter by the number of values in an array field.

```json
{
  "key": "comments",
  "values_count": {
    "gt": 2,
    "gte": null,
    "lt": 10,
    "lte": null
  }
}
```

### Is Empty

Match fields that are null, missing, or empty arrays.

```json
{ "is_empty": { "key": "reports" } }
```

### Is Null

Match fields that exist but have NULL value.

```json
{ "is_null": { "key": "reports" } }
```

### Has ID

Filter by specific point IDs.

```json
{ "has_id": [1, 3, 5, 7, 9, 11] }
```

### Has Vector

Filter by presence of a named vector (v1.13.0+).

```json
{ "has_vector": "image" }
// For unnamed vectors, use empty string
{ "has_vector": "" }
```

## Complete Examples

### Search with Multiple Filters

```json
POST /collections/products/points/search
{
  "vector": [0.2, 0.1, 0.9, 0.7],
  "filter": {
    "must": [
      { "key": "category", "match": { "value": "laptop" } },
      { "key": "price", "range": { "lte": 1000 } },
      { "key": "in_stock", "match": { "value": true } }
    ],
    "must_not": [
      { "key": "brand", "match": { "any": ["Unknown", "Generic"] } }
    ]
  },
  "limit": 10,
  "with_payload": true
}
```

### Scroll with Ordering

```json
POST /collections/products/points/scroll
{
  "filter": {
    "must": [
      { "key": "category", "match": { "value": "laptop" } }
    ]
  },
  "limit": 20,
  "order_by": [
    { "key": "price", "direction": "asc" }
  ],
  "with_payload": true
}
```

### Complex Nested Filtering

```json
POST /collections/users/points/scroll
{
  "filter": {
    "must": [
      {
        "nested": {
          "key": "purchases",
          "filter": {
            "must": [
              { "key": "product_type", "match": { "value": "electronics" } },
              { "key": "amount", "range": { "gte": 100 } },
              { "key": "date", "range": { "gte": "2024-01-01T00:00:00Z" } }
            ]
          }
        }
      }
    ]
  }
}
```

## Best Practices

### 1. Payload Indexing

Create indexes for frequently filtered fields to improve performance:

```json
PUT /collections/{collection_name}/index
{
  "field_name": "category",
  "field_schema": "keyword"
}
```

### 2. Float Filtering

Use range filters instead of exact matches for float values to avoid precision issues:

```json
// Instead of exact match
{ "key": "price", "match": { "value": 11.99 } }

// Use range
{ "key": "price", "range": { "gte": 11.99, "lte": 11.99 } }
```

### 3. Multi-tenant Filtering

Mark tenant fields for optimization:

```json
PUT /collections/{collection_name}/index
{
  "field_name": "user_id",
  "field_schema": {
    "type": "keyword",
    "is_tenant": true
  }
}
```

### 4. Query Optimization

- **Low cardinality filters**: Qdrant may switch from HNSW to payload index search
- **High cardinality filters**: HNSW with additional filterable links
- **Combine filters efficiently**: More restrictive conditions first
- **Use appropriate indexes**: Text index for full-text search, keyword for exact matches

### 5. Performance Considerations

- Filters are evaluated during search, not after
- Proper indexing dramatically improves filter performance
- Nested object filters have higher computational cost
- Geo filters require proper geo-point format in payload

## Error Handling

Common filter-related errors:

- **400 Bad Request**: Invalid filter syntax or unknown field
- **422 Unprocessable Entity**: Type mismatch (e.g., string value for numeric field)
- **404 Not Found**: Collection does not exist

Always validate filter syntax and ensure field types match the condition requirements.

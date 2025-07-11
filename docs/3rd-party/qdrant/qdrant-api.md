# Qdrant API Reference

A concise reference for Qdrant's essential APIs for semantic search applications.

## Base Configuration

**Base URL:** `http://localhost:6333` (default)  
**Authentication:** Optional API key via `api-key` header or Bearer token

## Service Endpoints

### Health Check

```http
GET /healthz
```

Returns: `200` with "healthz check passed"

### Version Info

```http
GET /
```

Returns version and build information.

## Collection Management

### List Collections

```http
GET /collections
```

**Response:**

```json
{
  "result": {
    "collections": [{ "name": "documents" }, { "name": "images" }]
  }
}
```

### Create Collection

```http
PUT /collections/{collection_name}
```

**Request Body:**

```json
{
  "vectors": {
    "size": 384,
    "distance": "Cosine"
  },
  "shard_number": 1,
  "replication_factor": 1
}
```

**Distance Options:** `Cosine`, `Euclid`, `Dot`, `Manhattan`

### Get Collection Info

```http
GET /collections/{collection_name}
```

Returns detailed collection configuration and statistics.

### Delete Collection

```http
DELETE /collections/{collection_name}
```

### Check Collection Exists

```http
GET /collections/{collection_name}/exists
```

**Response:**

```json
{
  "result": {
    "exists": true
  }
}
```

## Point Management

### Upsert Points

```http
PUT /collections/{collection_name}/points
```

**Request Body:**

```json
{
  "points": [
    {
      "id": 1,
      "vector": [0.1, 0.2, 0.3, ...],
      "payload": {
        "text": "Document content",
        "source": "file1.txt",
        "folder": "docs"
      }
    }
  ]
}
```

**Note**: The `id` field must be an unsigned integer or UUID. Strings like `"point-1"` will cause a 400 error.

**Query Parameters:**

- `wait` (boolean): Wait for operation to complete

### Get Points

```http
POST /collections/{collection_name}/points
```

**Request Body:**

```json
{
  "ids": [1, 2, 3],
  "with_payload": true,
  "with_vector": false
}
```

### Delete Points

```http
POST /collections/{collection_name}/points/delete
```

**Request Body (by IDs):**

```json
{
  "points": [1, 2, 3]
}
```

**Request Body (by Filter):**

```json
{
  "filter": {
    "must": [
      {
        "key": "folder",
        "match": { "value": "old_docs" }
      }
    ]
  }
}
```

## Search Operations

### Basic Vector Search

```http
POST /collections/{collection_name}/points/search
```

**Request Body:**

```json
{
  "vector": [0.1, 0.2, 0.3, ...],
  "limit": 10,
  "with_payload": true,
  "filter": {
    "must": [
      {
        "key": "folder",
        "match": {"value": "documents"}
      }
    ]
  },
  "score_threshold": 0.7
}
```

### Universal Query (Recommended)

```http
POST /collections/{collection_name}/points/query
```

**Request Body:**

```json
{
  "query": [0.1, 0.2, 0.3, ...],
  "limit": 10,
  "with_payload": true,
  "filter": {
    "must": [
      {
        "key": "source",
        "match": {"value": "important.pdf"}
      }
    ]
  }
}
```

### Scroll Through Points

```http
POST /collections/{collection_name}/points/scroll
```

**Request Body:**

```json
{
  "limit": 100,
  "with_payload": true,
  "filter": {
    "must": [
      {
        "key": "folder",
        "match": { "value": "documents" }
      }
    ]
  }
}
```

## Common Data Structures

### Point Structure

```json
{
  "id": "unsigned_integer_or_uuid",
  "vector": [float_array],
  "payload": {
    "key": "value"
  }
}
```

**Important**: Point IDs must be either:
- **Unsigned integers**: `1`, `2`, `12345` 
- **UUIDs**: `"550e8400-e29b-41d4-a716-446655440000"`

Arbitrary strings like `"my-custom-id"` are **not valid** and will result in a 400 Bad Request error.

### Filter Examples

**Exact Match:**

```json
{
  "key": "category",
  "match": { "value": "document" }
}
```

**Range Filter:**

```json
{
  "key": "timestamp",
  "range": {
    "gte": "2024-01-01T00:00:00Z",
    "lt": "2024-12-31T23:59:59Z"
  }
}
```

**Multiple Conditions:**

```json
{
  "must": [
    { "key": "folder", "match": { "value": "docs" } },
    { "key": "size", "range": { "gte": 1000 } }
  ],
  "should": [
    { "key": "type", "match": { "value": "pdf" } },
    { "key": "type", "match": { "value": "txt" } }
  ]
}
```

### Search Response

```json
{
  "result": [
    {
      "id": 1,
      "score": 0.95,
      "payload": {
        "text": "Document content",
        "source": "file1.txt"
      },
      "vector": [0.1, 0.2, ...]
    }
  ],
  "status": "ok",
  "time": 0.002
}
```

## Error Responses

All errors return:

```json
{
  "status": {
    "error": "Error description"
  },
  "time": 0.001
}
```

Common HTTP status codes:

- `400` - Bad Request (invalid parameters)
- `404` - Collection/Point not found
- `422` - Unprocessable Entity (validation errors)
- `500` - Internal Server Error

## Best Practices

1. **Use consistent vector dimensions** across your collection
2. **Choose appropriate distance metric** (`Cosine` for normalized vectors, `Euclid` for raw embeddings)
3. **Include meaningful payload** for filtering and metadata
4. **Use the universal query endpoint** for new applications
5. **Implement proper error handling** for network and API errors
6. **Batch operations** when possible for better performance

## Example Workflow

```bash
# 1. Create collection
curl -X PUT "http://localhost:6333/collections/documents" \
  -H "Content-Type: application/json" \
  -d '{"vectors": {"size": 384, "distance": "Cosine"}}'

# 2. Add documents
curl -X PUT "http://localhost:6333/collections/documents/points" \
  -H "Content-Type: application/json" \
  -d '{"points": [{"id": 1, "vector": [...], "payload": {"text": "..."}}]}'

# 3. Search
curl -X POST "http://localhost:6333/collections/documents/points/query" \
  -H "Content-Type: application/json" \
  -d '{"query": [...], "limit": 5, "with_payload": true}'
```

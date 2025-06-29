# OpenAI Embeddings API Specification

## Endpoint

```
POST https://api.openai.com/v1/embeddings
```

## Authentication

```
Authorization: Bearer $OPENAI_API_KEY
Content-Type: application/json
```

## Request Parameters

| Parameter         | Type            | Required | Description                                                                                                                                |
| ----------------- | --------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| `input`           | string \| array | ✓        | Text to embed. String or array of strings/tokens. Max 8192 tokens per input, 300k tokens total per request, 2048 dimensions max for arrays |
| `model`           | string          | ✓        | Model ID. Available: `text-embedding-3-small`, `text-embedding-3-large`, `text-embedding-ada-002`                                          |
| `dimensions`      | integer         | -        | Output embedding dimensions. Only for `text-embedding-3-*` models                                                                          |
| `encoding_format` | string          | -        | Response format: `float` (default) or `base64`                                                                                             |
| `user`            | string          | -        | End-user identifier for abuse monitoring                                                                                                   |

## Available Models

| Model                    | Default Dimensions | Max Input Tokens | Performance (MTEB) |
| ------------------------ | ------------------ | ---------------- | ------------------ |
| `text-embedding-3-small` | 1536               | 8192             | 62.3%              |
| `text-embedding-3-large` | 3072               | 8192             | 64.6%              |
| `text-embedding-ada-002` | 1536               | 8192             | 61.0%              |

## Response Format

```json
{
  "object": "list",
  "data": [
    {
      "object": "embedding",
      "embedding": [0.0023064255, -0.009327292, ...],
      "index": 0
    }
  ],
  "model": "text-embedding-3-small",
  "usage": {
    "prompt_tokens": 8,
    "total_tokens": 8
  }
}
```

### Embedding Object

| Field       | Type    | Description                      |
| ----------- | ------- | -------------------------------- |
| `object`    | string  | Always `"embedding"`             |
| `embedding` | array   | Vector of floating point numbers |
| `index`     | integer | Position in the input array      |

## Example Request

```bash
curl https://api.openai.com/v1/embeddings \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "input": "The food was delicious and the waiter...",
    "model": "text-embedding-3-small",
    "encoding_format": "float"
  }'
```

## Key Constraints

- Max input: 8192 tokens per string
- Max total: 300,000 tokens per request
- Max array dimensions: 2048
- Empty strings not allowed
- Dimensions parameter only works with `text-embedding-3-*` models

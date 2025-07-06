# Ollama API Reference

## Endpoints Overview

| Method | Endpoint             | Purpose                  |
| ------ | -------------------- | ------------------------ |
| POST   | `/api/generate`      | Generate completion      |
| POST   | `/api/chat`          | Generate chat completion |
| POST   | `/api/create`        | Create model             |
| GET    | `/api/tags`          | List local models        |
| POST   | `/api/show`          | Show model info          |
| POST   | `/api/copy`          | Copy model               |
| DELETE | `/api/delete`        | Delete model             |
| POST   | `/api/pull`          | Pull model from registry |
| POST   | `/api/push`          | Push model to registry   |
| POST   | `/api/embed`         | Generate embeddings      |
| GET    | `/api/ps`            | List running models      |
| HEAD   | `/api/blobs/:digest` | Check blob exists        |
| POST   | `/api/blobs/:digest` | Push blob                |
| GET    | `/api/version`       | Get version              |

## Generate Completion

```http
POST /api/generate
```

**Required:**

- `model`: Model name

**Optional:**

- `prompt`: Input prompt
- `suffix`: Text after response
- `images`: Base64 encoded images (multimodal)
- `format`: Response format (`json` or JSON schema)
- `options`: Model parameters (temperature, etc.)
- `system`: System message override
- `template`: Prompt template override
- `stream`: Boolean (default: true)
- `raw`: Boolean - bypass templating
- `keep_alive`: Duration (default: "5m")
- `think`: Boolean - enable thinking (for thinking models)

## Generate Chat Completion

```http
POST /api/chat
```

**Required:**

- `model`: Model name
- `messages`: Array of message objects

**Message Object:**

- `role`: "system" | "user" | "assistant" | "tool"
- `content`: Message content
- `images`: Base64 images (optional)
- `tool_calls`: Tool function calls (optional)
- `thinking`: Model's thinking process (thinking models)

**Optional:**

- `tools`: Available tools array
- `format`: Response format
- `options`: Model parameters
- `stream`: Boolean (default: true)
- `keep_alive`: Duration
- `think`: Boolean - enable thinking

## Create Model

```http
POST /api/create
```

**Required:**

- `model`: New model name

**Optional:**

- `from`: Existing model to copy from
- `files`: Dict of filename → SHA256 digest
- `adapters`: Dict of LORA adapter files
- `template`: Prompt template
- `license`: License string/array
- `system`: System prompt
- `parameters`: Model parameters dict
- `messages`: Conversation messages
- `stream`: Boolean (default: true)
- `quantize`: Quantization type (`q4_K_M`, `q4_K_S`, `q8_0`)

## List Local Models

```http
GET /api/tags
```

Returns array of models with: name, size, digest, format, family, parameter_size, quantization_level

## Show Model Information

```http
POST /api/show
```

**Required:**

- `model`: Model name

**Optional:**

- `verbose`: Boolean - include full tokenizer data

## Copy Model

```http
POST /api/copy
```

**Required:**

- `source`: Source model name
- `destination`: Destination model name

## Delete Model

```http
DELETE /api/delete
```

**Required:**

- `model`: Model name to delete

## Pull Model

```http
POST /api/pull
```

**Required:**

- `model`: Model name to pull

**Optional:**

- `insecure`: Boolean - allow insecure connections
- `stream`: Boolean (default: true)

## Push Model

```http
POST /api/push
```

**Required:**

- `model`: Model name (`namespace/model:tag` format)

**Optional:**

- `insecure`: Boolean
- `stream`: Boolean (default: true)

## Generate Embeddings

```http
POST /api/embed
```

**Required:**

- `model`: Model name
- `input`: String or array of strings

**Optional:**

- `truncate`: Boolean (default: true)
- `options`: Model parameters
- `keep_alive`: Duration

## List Running Models

```http
GET /api/ps
```

Returns array of loaded models with memory usage and expiration times.

## Blob Operations

### Check Blob Exists

```http
HEAD /api/blobs/:digest
```

Returns 200 if exists, 404 if not found.

### Push Blob

```http
POST /api/blobs/:digest
```

Upload file as blob. Returns 201 on success.

## Get Version

```http
GET /api/version
```

Returns: `{"version": "x.y.z"}`

## Common Parameters

**Model Names:** Format: `model:tag` (tag defaults to "latest")

**Durations:** Specified as strings: "5m", "1h", "30s"

**Streaming:** Most endpoints return JSON stream by default. Set `"stream": false` for single response.

**Options Object:** Common model parameters:

- `temperature`: 0.0-1.0
- `top_k`: Integer
- `top_p`: 0.0-1.0
- `num_predict`: Max tokens
- `seed`: Integer for reproducibility
- `stop`: Array of stop sequences

## Response Formats

**Streaming Response:**

```json
{"model": "...", "response": "...", "done": false}
{"model": "...", "response": "", "done": true, "total_duration": 123}
```

**Single Response:**

```json
{ "model": "...", "response": "...", "done": true, "total_duration": 123 }
```

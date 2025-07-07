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

Generate embeddings from a model.

**Required:**

- `model`: Model name
- `input`: String or array of strings to generate embeddings for

**Optional:**

- `truncate`: Boolean - truncates the end of each input to fit within context length. Returns error if false and context length is exceeded (default: true)
- `options`: Additional model parameters listed in the documentation for the Modelfile such as temperature
- `keep_alive`: Duration - controls how long the model will stay loaded into memory following the request (default: "5m")

**Examples:**

Single input:
```bash
curl http://localhost:11434/api/embed -d '{
  "model": "all-minilm",
  "input": "Why is the sky blue?"
}'
```

Response:
```json
{
  "model": "all-minilm",
  "embeddings": [[
    0.010071029, -0.0017594862, 0.05007221, 0.04692972, 0.054916814,
    0.008599704, 0.105441414, -0.025878139, 0.12958129, 0.031952348
  ]],
  "total_duration": 14143917,
  "load_duration": 1019500,
  "prompt_eval_count": 8
}
```

Multiple inputs:
```bash
curl http://localhost:11434/api/embed -d '{
  "model": "all-minilm",
  "input": ["Why is the sky blue?", "Why is the grass green?"]
}'
```

Response:
```json
{
  "model": "all-minilm",
  "embeddings": [[
    0.010071029, -0.0017594862, 0.05007221, 0.04692972, 0.054916814,
    0.008599704, 0.105441414, -0.025878139, 0.12958129, 0.031952348
  ],[
    -0.0098027075, 0.06042469, 0.025257962, -0.006364387, 0.07272725,
    0.017194884, 0.09032035, -0.051705178, 0.09951512, 0.09072481
  ]]
}
```

### Legacy Endpoint (Deprecated)

**Note:** The `/api/embeddings` endpoint has been deprecated in favor of `/api/embed`.

```http
POST /api/embeddings
```

**Required:**

- `model`: Model name  
- `prompt`: Text to generate embeddings for

**Optional:**

- `options`: Additional model parameters
- `keep_alive`: Duration (default: "5m")

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

# MCP Server Implementation

## JSON-RPC 2.0 Protocol

- Request ID MUST NOT be null or reused
- Error codes: `-32002` (not found), `-32601` (method not found), `-32602` (invalid params), `-32603` (internal)

## Initialization

```json
// Request
{"method": "initialize", "params": {"protocolVersion": "2025-06-18", "capabilities": {...}}}

// Response
{"result": {"protocolVersion": "2025-06-18", "capabilities": {...}, "serverInfo": {...}}}
```

## Server Capabilities

```json
{
  "prompts": { "listChanged": true },
  "resources": { "subscribe": true, "listChanged": true },
  "tools": { "listChanged": true },
  "completions": {},
  "logging": {}
}
```

## Resources

```json
// List: resources/list -> {resources: [{uri, name, mimeType}], nextCursor?}
// Read: resources/read {uri} -> {contents: [{uri, text|blob, mimeType}]}
// Templates: resources/templates/list -> {resourceTemplates: [{uriTemplate, name}]}
// Subscribe: resources/subscribe {uri}
// Notify: notifications/resources/updated {uri}
```

## Prompts

```json
// List: prompts/list -> {prompts: [{name, description, arguments: [{name, required}]}]}
// Get: prompts/get {name, arguments} -> {messages: [{role, content: {type, text}}]}
```

## Tools

```json
// List: tools/list -> {tools: [{name, description, inputSchema}]}
// Call: tools/call {name, arguments} -> {content: [{type, text}], isError}
```

## Content Types

- `{type: "text", text: "..."}`
- `{type: "image", data: "base64", mimeType: "image/png"}`
- `{type: "resource", resource: {uri, text|blob, mimeType}}`

## URI Schemes

- `file://` - filesystem-like
- `https://` - web resources
- `git://` - git repos

## Required Methods by Capability

- `resources`: `resources/list`, `resources/read`
- `prompts`: `prompts/list`, `prompts/get`
- `tools`: `tools/list`, `tools/call`

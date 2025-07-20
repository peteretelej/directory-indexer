# Feature Design: Document Format Support (#11)

Github issue: https://github.com/peteretelej/directory-indexer/issues/11

## Problem Statement

Directory-indexer only supports text files, excluding common office documents and PDFs from indexing.

**Current formats**: `.md`, `.txt`, `.py`, `.js`, `.json`, etc.
**Missing formats**: `.docx`, `.xlsx`, `.pptx`, `.pdf`, `.odt`

This limits the tool's usefulness for knowledge workers with mixed document types.

## Solution

Convert documents to text during indexing using a single library.

**Simple approach**:
1. Detect document format during file processing
2. Convert to text using `officeparser` 
3. Process converted text through existing indexing pipeline
4. Mark search results as converted documents

**Design principles**:
- Convert during indexing (embedding is the bottleneck, not conversion)
- Single dependency for all formats
- Fail gracefully - skip file if conversion fails
- No intermediate storage needed

## Implementation

### 1. Document Conversion

Add simple converter function in `src/converters.ts`:

```typescript
export async function convertDocument(filePath: string): Promise<string | null> {
  const ext = path.extname(filePath).toLowerCase();
  
  if (['.docx', '.xlsx', '.pptx', '.odt', '.ods', '.odp', '.pdf'].includes(ext)) {
    try {
      // Use selected library after research phase
      const result = await selectedParser.parse(filePath);
      return result;
    } catch (error) {
      console.warn(`Failed to convert ${filePath}:`, error.message);
      return null;
    }
  }
  
  return null; // Not a convertible format
}
```

### 2. Update File Processing

Modify `src/indexing.ts`:

```typescript
// Replace direct file read with conversion check
const convertedContent = await convertDocument(file.path);
const content = convertedContent ?? await fs.readFile(file.path, "utf-8");
```

### 3. Update Supported File Types

Extend `isSupportedFileType()` in `src/utils.ts` to include:
`.docx`, `.xlsx`, `.pptx`, `.pdf`, `.odt`, `.ods`, `.odp`

## Dependencies

**Research required**: Evaluate document parsing libraries

**Criteria for selection:**
- Format coverage (docx, xlsx, pptx, pdf, odt, ods, odp)
- TypeScript/Node.js compatibility
- Active maintenance and community
- Performance and memory usage
- AI-era parsing libraries vs traditional tools

**Candidate libraries to evaluate:**
- `officeparser` - Traditional option
- AI-powered document parsers
- Specialized libraries per format
- Modern alternatives

```bash
npm install [selected-library]
```

## Configuration

Optional environment variable:

```bash
ENABLE_DOCUMENT_CONVERSION=true  # Default: false
```

## Error Handling

- Skip file if conversion fails
- Log warning and continue indexing
- No fallback complexity needed

## Benefits

- Users can search office documents and PDFs
- Minimal code changes required  
- No performance impact on text file indexing
- Simple to maintain and debug

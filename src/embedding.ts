// Phase 2 - Embedding providers (not yet implemented)

export function createEmbeddingProvider(_provider: string, _config: any): any {
  throw new Error('createEmbeddingProvider not implemented');
}

export async function generateEmbedding(_text: string, _config: any): Promise<number[]> {
  throw new Error('generateEmbedding not implemented');
}
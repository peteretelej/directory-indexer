import { describe, it, expect, beforeAll } from 'vitest';
import { spawn } from 'child_process';
import { join } from 'path';
import { setupServicesCheck, getTestDataPath } from '../utils/test-helpers.js';
import { loadConfig } from '../../src/config.js';
import { startMcpServer } from '../../src/mcp.js';

describe.sequential('MCP Server Integration Tests', () => {
  beforeAll(async () => {
    await setupServicesCheck();
  });

  describe('MCP Server Startup', () => {
    it('should start MCP server without crashing', async () => {
      const child = spawn('node', ['dist/cli.js', 'serve'], {
        env: {
          ...process.env,
          DIRECTORY_INDEXER_QDRANT_COLLECTION: 'directory-indexer-mcp-test'
        }
      });

      await new Promise(resolve => setTimeout(resolve, 1000));

      expect(child.killed).toBe(false);

      child.kill();
      await new Promise(resolve => setTimeout(resolve, 100));
    });

    it('should test MCP server components directly', async () => {
      await loadConfig({ verbose: false });
      
      expect(typeof startMcpServer).toBe('function');
      
      console.log('✅ MCP server components loaded successfully');
    });
  });

  describe('MCP Handlers', () => {
    it('should test MCP handlers with workspace filtering', async () => {
      const { handleServerInfoTool, handleSearchTool } = await import('../../src/mcp-handlers.js');
      
      const originalEnv = process.env;
      const testDataPath = getTestDataPath();
      process.env.WORKSPACE_DOCS = join(testDataPath, 'docs');
      
      try {
        const serverInfo = await handleServerInfoTool('test-version');
        const content = JSON.parse(serverInfo.content[0].text as string);
        
        expect(content.name).toBe('directory-indexer');
        expect(content.version).toBe('test-version');
        expect(content.status.workspaceHealth).toBeDefined();
        expect(typeof content.status.workspaceHealth.healthy).toBe('number');
        
        const searchResult = await handleSearchTool({ query: 'test', workspace: 'docs' });
        expect(searchResult.content[0].text).toBeDefined();
        
        const invalidSearch = await handleSearchTool({ query: 'test', workspace: 'nonexistent' });
        expect(invalidSearch.content[0].text).toContain('not found');
        
      } finally {
        process.env = originalEnv;
      }
    });
  });
});
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { main } from '../src/cli.js';

describe('CLI Unit Tests', () => {
  let originalArgv: string[];
  let originalExit: typeof process.exit;
  let consoleLogSpy: any;
  let consoleErrorSpy: any;
  let exitCode: number | undefined;

  beforeEach(() => {
    originalArgv = process.argv;
    originalExit = process.exit;
    
    consoleLogSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    
    process.exit = vi.fn((code?: number) => {
      exitCode = code;
      throw new Error(`Process exited with code ${code}`);
    }) as never;
  });

  afterEach(() => {
    process.argv = originalArgv;
    process.exit = originalExit;
    exitCode = undefined;
    vi.restoreAllMocks();
  });


  describe('Command Validation', () => {
    it('should error when search command has no query', async () => {
      process.argv = ['node', 'cli.js', 'search'];
      
      try {
        await main();
      } catch (error) {
        // Expected to exit with error
      }
      
      expect(exitCode).toBe(1);
    });

    it('should error when similar command has no file argument', async () => {
      process.argv = ['node', 'cli.js', 'similar'];
      
      try {
        await main();
      } catch (error) {
        // Expected to exit with error
      }
      
      expect(exitCode).toBe(1);
    });

    it('should error when get command has no file argument', async () => {
      process.argv = ['node', 'cli.js', 'get'];
      
      try {
        await main();
      } catch (error) {
        // Expected to exit with error
      }
      
      expect(exitCode).toBe(1);
    });

    it('should error when index command has no paths', async () => {
      process.argv = ['node', 'cli.js', 'index'];
      
      try {
        await main();
      } catch (error) {
        // Expected to exit with error
      }
      
      expect(exitCode).toBe(1);
    });
  });

  describe('Option Parsing', () => {

    it('should parse chunks option for get command', async () => {
      process.argv = ['node', 'cli.js', 'get', '/test/file.txt', '--chunks', '1-3'];
      
      try {
        await main();
      } catch (error) {
        // Expected to fail due to missing file, but should parse correctly
      }
      
      expect(consoleErrorSpy).toHaveBeenCalledWith('Error getting file content:', expect.any(Error));
      expect(exitCode).toBe(1);
    });
  });

  describe('Error Handling', () => {
    it('should handle invalid commands gracefully', async () => {
      process.argv = ['node', 'cli.js', 'invalid-command'];
      
      try {
        await main();
      } catch (error) {
        // Expected to exit with error
      }
      
      expect(exitCode).toBe(1);
    });


    it('should handle similar files errors gracefully', async () => {
      process.argv = ['node', 'cli.js', 'similar', '/nonexistent/file.txt'];
      
      try {
        await main();
      } catch (error) {
        // Expected to fail due to missing file
      }
      
      expect(consoleErrorSpy).toHaveBeenCalledWith('Error finding similar files:', expect.any(Error));
      expect(exitCode).toBe(1);
    });

    it('should handle get content errors gracefully', async () => {
      process.argv = ['node', 'cli.js', 'get', '/nonexistent/file.txt'];
      
      try {
        await main();
      } catch (error) {
        // Expected to fail due to missing file
      }
      
      expect(consoleErrorSpy).toHaveBeenCalledWith('Error getting file content:', expect.any(Error));
      expect(exitCode).toBe(1);
    });

  });

});
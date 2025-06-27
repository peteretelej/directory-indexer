#!/usr/bin/env node

const packageJson = require('../package.json');

console.log(`${packageJson.name} v${packageJson.version}`);
console.log('This is a placeholder package. Full implementation coming soon!');
console.log('');
console.log('Planned commands:');
console.log('  directory-indexer index <paths...>     - Index directories');
console.log('  directory-indexer search <query>       - Search indexed content');
console.log('  directory-indexer similar <file>       - Find similar files');
console.log('  directory-indexer serve                 - Start MCP server');
console.log('  directory-indexer status                - Show indexing status');
console.log('');
console.log('GitHub: https://github.com/peteretelej/directory-indexer');

process.exit(0);
export type LogLevel = 'debug' | 'info' | 'warning' | 'error';

const LEVEL_ORDER: Record<LogLevel, number> = {
  debug: 0,
  info: 1,
  warning: 2,
  error: 3,
};

let currentLevel: LogLevel = 'info';

/**
 * Set the minimum log level. Messages below this level are discarded.
 */
export function setLogLevel(level: LogLevel): void {
  currentLevel = level;
}

/**
 * Get the current log level.
 */
export function getLogLevel(): LogLevel {
  return currentLevel;
}

/**
 * Initialize the log level from the LOG_LEVEL environment variable.
 * Invalid values default to 'info' with a warning on stderr.
 */
export function initLogLevel(): void {
  const envLevel = process.env.LOG_LEVEL?.toLowerCase();
  if (!envLevel) return;

  if (envLevel in LEVEL_ORDER) {
    currentLevel = envLevel as LogLevel;
  } else {
    process.stderr.write(
      `Warning: invalid LOG_LEVEL "${process.env.LOG_LEVEL}", defaulting to "info"\n`
    );
    currentLevel = 'info';
  }
}

/**
 * Write a structured JSON log line to stderr.
 * Never writes to stdout (reserved for the MCP protocol channel).
 */
export function log(level: LogLevel, message: string, data?: Record<string, unknown>): void {
  if (LEVEL_ORDER[level] < LEVEL_ORDER[currentLevel]) return;

  const entry: Record<string, unknown> = {
    timestamp: new Date().toISOString(),
    level,
    message,
    ...data,
  };

  process.stderr.write(JSON.stringify(entry) + '\n');
}

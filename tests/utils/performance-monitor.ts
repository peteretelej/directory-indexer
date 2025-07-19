interface TestMetrics {
  testName: string;
  duration: number;
  status: 'passed' | 'failed' | 'skipped';
  timestamp: Date;
}

class TestPerformanceMonitor {
  private metrics: TestMetrics[] = [];
  private testStartTimes: Map<string, number> = new Map();

  startTest(testName: string): void {
    this.testStartTimes.set(testName, Date.now());
  }

  endTest(testName: string, status: 'passed' | 'failed' | 'skipped' = 'passed'): void {
    const startTime = this.testStartTimes.get(testName);
    if (startTime) {
      const duration = Date.now() - startTime;
      this.metrics.push({
        testName,
        duration,
        status,
        timestamp: new Date()
      });
      this.testStartTimes.delete(testName);
    }
  }

  getMetrics(): TestMetrics[] {
    return [...this.metrics];
  }

  getSlowestTests(limit = 5): TestMetrics[] {
    return [...this.metrics]
      .sort((a, b) => b.duration - a.duration)
      .slice(0, limit);
  }

  getTotalDuration(): number {
    return this.metrics.reduce((total, metric) => total + metric.duration, 0);
  }

  getAverageDuration(): number {
    return this.metrics.length > 0 ? this.getTotalDuration() / this.metrics.length : 0;
  }

  generateReport(): string {
    const total = this.getTotalDuration();
    const average = this.getAverageDuration();
    const slowest = this.getSlowestTests(5);
    
    let report = `\n📊 Test Performance Report\n`;
    report += `═══════════════════════════\n`;
    report += `Total tests: ${this.metrics.length}\n`;
    report += `Total duration: ${total}ms (${(total / 1000).toFixed(2)}s)\n`;
    report += `Average duration: ${average.toFixed(2)}ms\n\n`;
    
    if (slowest.length > 0) {
      report += `🐌 Slowest tests:\n`;
      slowest.forEach((metric, index) => {
        report += `${index + 1}. ${metric.testName}: ${metric.duration}ms\n`;
      });
    }
    
    return report;
  }

  clear(): void {
    this.metrics = [];
    this.testStartTimes.clear();
  }
}

export const performanceMonitor = new TestPerformanceMonitor();

export function withPerformanceMonitoring<T>(testName: string, testFn: () => T): T {
  performanceMonitor.startTest(testName);
  try {
    const result = testFn();
    performanceMonitor.endTest(testName, 'passed');
    return result;
  } catch (error) {
    performanceMonitor.endTest(testName, 'failed');
    throw error;
  }
}

export async function withPerformanceMonitoringAsync<T>(testName: string, testFn: () => Promise<T>): Promise<T> {
  performanceMonitor.startTest(testName);
  try {
    const result = await testFn();
    performanceMonitor.endTest(testName, 'passed');
    return result;
  } catch (error) {
    performanceMonitor.endTest(testName, 'failed');
    throw error;
  }
}
// TestPanel.tsx - Test Generation & Execution UI
// PRD Phase 3: Test generation & execution atoms

import { useState, useEffect } from 'react';
import {
  Play,
  FileCode,
  CheckCircle2,
  XCircle,
  AlertCircle,
  Loader2,
  RefreshCw,
  Sparkles,
  ChevronDown,
  Clock
} from 'lucide-react';
import {
  detectTestFramework,
  runTests,
  generateTests,
  TestFrameworkInfo,
  TestExecutionResult,
  GeneratedTest
} from '../tauri-api';
import { useMakerStore } from '../store/makerStore';

interface TestPanelProps {
  className?: string; // Optional if needed
}

export const TestPanel: React.FC<TestPanelProps> = () => {
  const { workspacePath } = useMakerStore();
  const [framework, setFramework] = useState<TestFrameworkInfo | null>(null);
  const [testResult, setTestResult] = useState<TestExecutionResult | null>(null);
  const [generatedTest, setGeneratedTest] = useState<GeneratedTest | null>(null);
  const [isRunning, setIsRunning] = useState(false);
  const [isGenerating, setIsGenerating] = useState(false);
  const [isDetecting, setIsDetecting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [testPattern, setTestPattern] = useState('');
  const [sourceFile, setSourceFile] = useState('');
  const [testType, setTestType] = useState<'unit' | 'integration' | 'property'>('unit');
  const [activeTab, setActiveTab] = useState<'run' | 'generate'>('run');

  useEffect(() => {
    if (workspacePath) {
      handleDetectFramework();
    }
  }, [workspacePath]);

  const handleDetectFramework = async () => {
    if (!workspacePath) return;
    setIsDetecting(true);
    setError(null);
    try {
      const info = await detectTestFramework(workspacePath);
      setFramework(info);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsDetecting(false);
    }
  };

  const handleRunTests = async () => {
    if (!workspacePath) return;
    setIsRunning(true);
    setError(null);
    setTestResult(null);
    try {
      const result = await runTests(workspacePath, testPattern || undefined);
      setTestResult(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsRunning(false);
    }
  };

  const handleGenerateTests = async () => {
    if (!workspacePath || !sourceFile) return;
    setIsGenerating(true);
    setError(null);
    setGeneratedTest(null);
    try {
      const result = await generateTests(workspacePath, sourceFile, testType);
      setGeneratedTest(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsGenerating(false);
    }
  };

  return (
    <div className="p-4 h-full overflow-y-auto bg-zinc-900">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-white flex items-center gap-2">
          <FileCode size={20} className="text-green-400" />
          Test Runner
        </h3>
        <button
          onClick={handleDetectFramework}
          disabled={isDetecting}
          className="p-2 text-zinc-400 hover:text-white hover:bg-zinc-800 rounded-lg"
          title="Re-detect test framework"
        >
          {isDetecting ? <Loader2 size={16} className="animate-spin" /> : <RefreshCw size={16} />}
        </button>
      </div>

      {/* Framework Info */}
      {framework && (
        <div className="mb-4 p-3 bg-zinc-800/50 border border-zinc-700 rounded-lg">
          <div className="flex items-center gap-2 text-sm">
            <span className="text-zinc-400">Framework:</span>
            <span className="text-indigo-400 font-medium">{framework.framework}</span>
          </div>
          <div className="text-xs text-zinc-500 mt-1">
            Command: <code className="text-zinc-400">{framework.test_command}</code>
          </div>
        </div>
      )}

      {/* Tab Switcher */}
      <div className="flex mb-4 border-b border-zinc-700">
        <button
          onClick={() => setActiveTab('run')}
          className={`px-4 py-2 text-sm font-medium transition-colors ${activeTab === 'run'
            ? 'text-indigo-400 border-b-2 border-indigo-400'
            : 'text-zinc-400 hover:text-white'
            }`}
        >
          <Play size={14} className="inline mr-1" /> Run Tests
        </button>
        <button
          onClick={() => setActiveTab('generate')}
          className={`px-4 py-2 text-sm font-medium transition-colors ${activeTab === 'generate'
            ? 'text-indigo-400 border-b-2 border-indigo-400'
            : 'text-zinc-400 hover:text-white'
            }`}
        >
          <Sparkles size={14} className="inline mr-1" /> Generate Tests
        </button>
      </div>

      {activeTab === 'run' && (
        <>
          {/* Test Pattern Filter */}
          <div className="mb-4">
            <label className="block text-xs text-zinc-500 mb-1">Test Pattern (optional)</label>
            <input
              type="text"
              value={testPattern}
              onChange={(e) => setTestPattern(e.target.value)}
              placeholder="Filter tests by pattern..."
              className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 text-white text-sm"
            />
          </div>

          {/* Run Button */}
          <button
            onClick={handleRunTests}
            disabled={isRunning || !framework}
            className="w-full py-2 mb-4 bg-green-600 hover:bg-green-500 disabled:bg-zinc-700 text-white rounded-lg font-medium flex items-center justify-center gap-2"
          >
            {isRunning ? <Loader2 size={16} className="animate-spin" /> : <Play size={16} />}
            {isRunning ? 'Running Tests...' : 'Run Tests'}
          </button>

          {/* Test Results */}
          {testResult && (
            <div className="space-y-3">
              {/* Summary */}
              <div className={`p-4 rounded-lg border ${testResult.success ? 'bg-green-500/10 border-green-500/30' : 'bg-red-500/10 border-red-500/30'}`}>
                <div className="flex items-center gap-2 mb-2">
                  {testResult.success ? (
                    <CheckCircle2 size={20} className="text-green-400" />
                  ) : (
                    <XCircle size={20} className="text-red-400" />
                  )}
                  <span className={`font-medium ${testResult.success ? 'text-green-300' : 'text-red-300'}`}>
                    {testResult.success ? 'All Tests Passed' : 'Some Tests Failed'}
                  </span>
                </div>
                <div className="grid grid-cols-4 gap-2 text-center text-sm">
                  <div className="bg-black/30 rounded p-2">
                    <div className="text-lg font-bold text-white">{testResult.total_tests}</div>
                    <div className="text-zinc-500 text-xs">Total</div>
                  </div>
                  <div className="bg-black/30 rounded p-2">
                    <div className="text-lg font-bold text-green-400">{testResult.passed}</div>
                    <div className="text-zinc-500 text-xs">Passed</div>
                  </div>
                  <div className="bg-black/30 rounded p-2">
                    <div className="text-lg font-bold text-red-400">{testResult.failed}</div>
                    <div className="text-zinc-500 text-xs">Failed</div>
                  </div>
                  <div className="bg-black/30 rounded p-2">
                    <div className="text-lg font-bold text-yellow-400">{testResult.skipped}</div>
                    <div className="text-zinc-500 text-xs">Skipped</div>
                  </div>
                </div>
                <div className="mt-2 text-xs text-zinc-400 flex items-center gap-1">
                  <Clock size={12} /> {testResult.duration_ms}ms
                </div>
              </div>

              {/* Failed Tests List */}
              {testResult.failed_tests.length > 0 && (
                <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-3">
                  <h4 className="text-sm font-medium text-red-300 mb-2">Failed Tests</h4>
                  {testResult.failed_tests.map((test, i) => (
                    <div key={i} className="text-xs text-red-200 mb-1">
                      • {test.name}
                    </div>
                  ))}
                </div>
              )}

              {/* Output */}
              <details className="bg-black border border-zinc-800 rounded-lg">
                <summary className="px-3 py-2 text-sm text-zinc-400 cursor-pointer hover:text-white">
                  View Output
                </summary>
                <pre className="p-3 text-xs text-zinc-300 overflow-x-auto max-h-60 overflow-y-auto">
                  {testResult.output}
                </pre>
              </details>
            </div>
          )}
        </>
      )}

      {activeTab === 'generate' && (
        <>
          {/* Source File Input */}
          <div className="mb-3">
            <label className="block text-xs text-zinc-500 mb-1">Source File Path</label>
            <input
              type="text"
              value={sourceFile}
              onChange={(e) => setSourceFile(e.target.value)}
              placeholder="src/components/MyComponent.tsx"
              className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 text-white text-sm"
            />
          </div>

          {/* Test Type Select */}
          <div className="mb-4 relative">
            <label className="block text-xs text-zinc-500 mb-1">Test Type</label>
            <select
              value={testType}
              onChange={(e) => setTestType(e.target.value as 'unit' | 'integration' | 'property')}
              className="w-full appearance-none bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 pr-8 text-white text-sm"
            >
              <option value="unit">Unit Tests</option>
              <option value="integration">Integration Tests</option>
              <option value="property">Property-based Tests</option>
            </select>
            <ChevronDown size={14} className="absolute right-3 top-8 text-zinc-500 pointer-events-none" />
          </div>

          {/* Generate Button */}
          <button
            onClick={handleGenerateTests}
            disabled={isGenerating || !sourceFile}
            className="w-full py-2 mb-4 bg-indigo-600 hover:bg-indigo-500 disabled:bg-zinc-700 text-white rounded-lg font-medium flex items-center justify-center gap-2"
          >
            {isGenerating ? <Loader2 size={16} className="animate-spin" /> : <Sparkles size={16} />}
            {isGenerating ? 'Generating...' : 'Generate Tests'}
          </button>

          {/* Generated Test Output */}
          {generatedTest && (
            <div className="bg-black border border-zinc-800 rounded-lg overflow-hidden">
              <div className="px-3 py-2 bg-zinc-800/50 border-b border-zinc-700 flex items-center justify-between">
                <span className="text-sm text-zinc-300">{generatedTest.suggested_file}</span>
                <span className="text-xs text-zinc-500">{generatedTest.language}</span>
              </div>
              <pre className="p-3 text-xs text-zinc-300 overflow-x-auto max-h-80 overflow-y-auto">
                {generatedTest.test_code}
              </pre>
            </div>
          )}
        </>
      )}

      {/* Error Display */}
      {error && (
        <div className="mt-4 p-3 bg-red-500/10 border border-red-500/30 rounded-lg flex items-start gap-2">
          <AlertCircle size={16} className="text-red-400 mt-0.5 shrink-0" />
          <span className="text-red-300 text-sm">{error}</span>
        </div>
      )}
    </div>
  );
};

export default TestPanel;

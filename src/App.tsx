import { useState, useEffect } from "react";
import "./App.css";
import Navbar from "./components/Navbar";

interface ExecutionResult {
  success: boolean;
  stack_outputs?: string[];
  program_hash?: string;
  cycles?: number;
  error?: string;
  compilation_time_ms?: number;
  execution_time_ms?: number;
  total_time_ms?: number;
}

interface ProofResult {
  success: boolean;
  proof_bytes?: number[];
  program_hash?: string;
  stack_outputs?: string[];
  error?: string;
  compilation_time_ms?: number;
  proving_time_ms?: number;
  total_time_ms?: number;
}

// Environment detection
const isTauri = () => {
  return typeof window !== 'undefined' && 
         typeof (window as any).__TAURI__ !== 'undefined';
};

// API configuration
const getApiBaseUrl = () => {
  if (typeof window !== 'undefined') {
    // In development, use the current origin (Vite will proxy /api to port 3001)
    if (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1') {
      return window.location.origin;
    }
    // In production, use the same origin
    return window.location.origin;
  }
  return 'http://localhost:3000';
};

class WebAPIClient {
  private baseUrl: string;

  constructor() {
    this.baseUrl = getApiBaseUrl();
  }

  async executeProgram(program: string, inputs?: any): Promise<ExecutionResult> {
    try {
      const response = await fetch(`${this.baseUrl}/api/execute`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          program,
          inputs,
        }),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      return await response.json();
    } catch (error) {
      return {
        success: false,
        error: `API Error: ${error instanceof Error ? error.message : String(error)}`,
      };
    }
  }

  async generateProof(program: string, inputs?: any): Promise<ProofResult> {
    try {
      const response = await fetch(`${this.baseUrl}/api/prove`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          program,
          inputs,
        }),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      return await response.json();
    } catch (error) {
      return {
        success: false,
        error: `API Error: ${error instanceof Error ? error.message : String(error)}`,
      };
    }
  }

  async getExamples(): Promise<[string, string][]> {
    try {
      const response = await fetch(`${this.baseUrl}/api/examples`);
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data = await response.json();
      console.log('new data', data);
      return data;
    } catch (error) {
      console.error('Failed to load examples from API:', error);
      return this.getFallbackExamples();
    }
  }

  private getFallbackExamples(): [string, string][] {
    return [
      [
        "Basic Addition",
        `# Adds 3 + 5 and leaves 8 on the stack
  begin
      push.3
      push.5
      add
      swap 
      drop
  end`
      ],
      [
        "Fibonacci (8th)",
        `# Computes 8th Fibonacci number using repeat loop
  begin
      push.1    # fib(1)
      push.1    # fib(2)
      push.6    # loop 6 times (8-2)
      repeat.6
          dup.1
          add
          swap
      end
      drop      # clean extra copy
  end`
      ],
      [
        "Simple Loop Sum",
        `# Sums 0 to 9 -> result = 45
  begin
      push.0    # acc
      push.10   # counter
      repeat.10
          dup add.1
      end
      drop
  end`
      ],
      [
        "Conditional Example",
        `# Push 5 and 3, swap if 5 > 3 (which is true)
  begin
      push.5
      push.3
      dup.1 gt
      if.true
          swap
      end
  end`
      ],
      [
        "Stack Manipulation",
        `# Demonstrates swap.2 and dup
  begin
      push.1
      push.2
      push.3
      push.4
      swap.2
      drop
      dup
  end`
      ],
      [
        "Memory Operations",
        `# Stores 42 in memory at index 0 and loads it back
  begin
      push.42
      push.0
      mem_store
      push.0
      mem_load
  end`
      ],
      [
        "Prime Generator (nprime)",
        `# Outputs the first n primes (n from input)
  # Provide input like: { "operand_stack": ["10"] }
  begin
      nprime
  end`
      ],
      [
        "Input Stack Add",
        `# Adds two input values
  # Input: { "operand_stack": ["10", "20"] }
  begin
      add
  end`
      ]
    ];
  }

}

function App() {
  const [program, setProgram] = useState<string>(`# Simple addition: 3 + 5 = 8
begin
    push.3
    push.5
    add
    swap
    drop
end`);
  
  const [inputs, setInputs] = useState<string>('{\n  "operand_stack": []\n}');
  const [result, setResult] = useState<ExecutionResult | null>(null);
  const [proofResult, setProofResult] = useState<ProofResult | null>(null);
  const [isRunning, setIsRunning] = useState(false);
  const [isProving, setIsProving] = useState(false);
  const [examples, setExamples] = useState<[string, string][]>([]);
  const [activeTab, setActiveTab] = useState<'execution' | 'proof'>('execution');
  const [environment, setEnvironment] = useState<'web' | 'tauri'>('web');
  const [isConnected, setIsConnected] = useState(false);

  const webAPI = new WebAPIClient();

  useEffect(() => {
    // Detect environment
    if (isTauri()) {
      setEnvironment('tauri');
      loadTauriExamples();
    } else {
      setEnvironment('web');
      loadWebExamples();
      checkAPIConnection();
    }
  }, []);

  async function checkAPIConnection() {
    try {
      const response = await fetch(`${getApiBaseUrl()}/health`);
      setIsConnected(response.ok);
    } catch (error) {
      setIsConnected(false);
    }
  }

  async function loadTauriExamples() {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const examplesJson = await invoke("get_example_programs") as string;
      const examplesList = JSON.parse(examplesJson) as [string, string][];
      setExamples(examplesList);
    } catch (err) {
      console.error("Failed to load Tauri examples:", err);
      loadWebExamples();
    }
  }

  async function loadWebExamples() {
    const examples = await webAPI.getExamples();
    setExamples(examples);
  }

  async function runProgram() {
    setIsRunning(true);
    setResult(null);
    
    try {
      let parsedInputs: any = null;
      
      // Parse inputs
      if (inputs.trim() && inputs.trim() !== '{\n  "operand_stack": []\n}') {
        try {
          parsedInputs = JSON.parse(inputs);
        } catch (e) {
          throw new Error("Invalid JSON input format");
        }
      }

      let executionResult: ExecutionResult;

      if (environment === 'tauri') {
        // Use Tauri backend
        const { invoke } = await import("@tauri-apps/api/core");
        const inputsToSend = inputs.trim() === '' || inputs.trim() === '{\n  "operand_stack": []\n}' 
          ? null 
          : inputs;
        
        const res = await invoke("exec_program_with_inputs", { 
          program, 
          inputsJson: inputsToSend 
        }) as string;
        
        executionResult = JSON.parse(res) as ExecutionResult;
      } else {
        // Use web API
        executionResult = await webAPI.executeProgram(program, parsedInputs);
      }
      
      setResult(executionResult);
    } catch (err: any) {
      console.error("Error:", err);
      setResult({ 
        success: false, 
        error: err.toString() 
      });
    } finally {
      setIsRunning(false);
    }
  }

  async function generateProof() {
    setIsProving(true);
    setProofResult(null);
    
    try {
      let parsedInputs: any = null;
      
      // Parse inputs
      if (inputs.trim() && inputs.trim() !== '{\n  "operand_stack": []\n}') {
        try {
          parsedInputs = JSON.parse(inputs);
        } catch (e) {
          throw new Error("Invalid JSON input format");
        }
      }

      let proofResult: ProofResult;

      if (environment === 'tauri') {
        // Use Tauri backend
        const { invoke } = await import("@tauri-apps/api/core");
        const inputsToSend = inputs.trim() === '' || inputs.trim() === '{\n  "operand_stack": []\n}' 
          ? null 
          : inputs;
        
        const res = await invoke("generate_proof_with_inputs", { 
          program, 
          inputsJson: inputsToSend 
        }) as string;
        
        proofResult = JSON.parse(res) as ProofResult;
      } else {
        // Use web API
        proofResult = await webAPI.generateProof(program, parsedInputs);
      }
      
      setProofResult(proofResult);
    } catch (err: any) {
      console.error("Error:", err);
      setProofResult({ 
        success: false, 
        error: err.toString() 
      });
    } finally {
      setIsProving(false);
    }
  }

  function loadExample(exampleProgram: string, exampleName: string) {
    setProgram(exampleProgram);
    setResult(null);
    setProofResult(null);
    
    // Auto-populate inputs based on example type
    if (exampleName.includes('Input Stack Demo')) {
      setInputs('{\n  "operand_stack": ["10", "20"]\n}');
    } else if (exampleName.includes('Counter with Input')) {
      setInputs('{\n  "operand_stack": ["7"]\n}');
    } else if (exampleName.includes('Prime Generator')) {
      setInputs('{\n  "operand_stack": ["10"]\n}');
    } else {
      setInputs('{\n  "operand_stack": []\n}');
    }
  }

  function formatStackOutput(outputs: string[]) {
    return outputs.map((val, idx) => (
      <div key={idx} className="stack-item">
        <span className="stack-index">[{idx}]</span>
        <span className="stack-value">{val}</span>
      </div>
    ));
  }

  function formatProofBytes(bytes: number[]) {
    const hex = bytes.map(b => b.toString(16).padStart(2, '0')).join('');
    return hex.substring(0, 64) + (hex.length > 64 ? '...' : '');
  }

  return (
    <div className="playground-container">
      <Navbar environment={environment} isConnected={isConnected} />

      <div className="main-content">
        <div className="left-panel">
          <div className="section">
            <div className="section-header">
              <h3>📝 Assembly Code</h3>
              <select 
                className="example-selector"
                onChange={(e) => {
                  const selectedExample = examples.find(ex => ex[0] === e.target.value);
                  if (selectedExample) loadExample(selectedExample[1], selectedExample[0]);
                }}
                value=""
              >
                <option value="">Load Example...</option>
                {examples.map(([name, _], idx) => (
                  <option key={`${name}-${idx}`} value={name}>{name}</option>
                ))}
              </select>
            </div>
            <div className="section-content">
              <textarea
                className="code-editor"
                value={program}
                onChange={(e) => setProgram(e.target.value)}
                rows={16}
                spellCheck={false}
                placeholder="Enter your Miden assembly code here..."
              />
            </div>
          </div>

          <div className="section">
            <div className="section-header">
              <h3>📥 Stack Inputs</h3>
              <button 
                className="clear-btn"
                onClick={() => setInputs('{\n  "operand_stack": []\n}')}
              >
                Clear
              </button>
            </div>
            <div className="section-content">
              <textarea
                className="inputs-editor"
                value={inputs}
                onChange={(e) => setInputs(e.target.value)}
                rows={4}
                spellCheck={false}
                placeholder='{"operand_stack": ["10", "20"]}'
              />
              <p style={{ fontSize: '0.875rem', color: 'var(--text-muted)', marginTop: '0.5rem' }}>
                Provide initial stack values in JSON format. Values are pushed in reverse order.
              </p>
            </div>
          </div>

          <div className="controls">
            <button 
              className="btn btn-primary"
              onClick={runProgram} 
              disabled={isRunning || isProving || (environment === 'web' && !isConnected)}
            >
              {isRunning ? (
                <>
                  <span className="loading-spinner"></span>
                  Running...
                </>
              ) : (
                <>
                  ▶️ Execute Program
                </>
              )}
            </button>
            
            <button 
              className="btn btn-secondary"
              onClick={generateProof} 
              disabled={isRunning || isProving || (environment === 'web' && !isConnected)}
            >
              {isProving ? (
                <>
                  <span className="loading-spinner"></span>
                  Proving...
                </>
              ) : (
                <>
                  🔐 Generate Proof
                </>
              )}
            </button>
          </div>
        </div>

        <div className="right-panel">
          <div className="tabs">
            <button 
              className={`tab ${activeTab === 'execution' ? 'active' : ''}`}
              onClick={() => setActiveTab('execution')}
            >
              📊 Execution Results
            </button>
            <button 
              className={`tab ${activeTab === 'proof' ? 'active' : ''}`}
              onClick={() => setActiveTab('proof')}
            >
              🔐 Proof Results
            </button>
          </div>

          <div className="results-content">
            {activeTab === 'execution' && (
              <div className="execution-results">
                {result ? (
                  <div className={`result-panel ${result.success ? 'success' : 'error'}`}>
                    {result.success ? (
                      <>
                        <div className="result-section">
                          <h4>📦 Stack Outputs</h4>
                          <div className="stack-display">
                            {result.stack_outputs && result.stack_outputs.length > 0 ? (
                              formatStackOutput(result.stack_outputs)
                            ) : (
                              <div className="empty-stack">Stack is empty</div>
                            )}
                          </div>
                        </div>

                        {result.program_hash && (
                          <div className="result-section">
                            <h4>🏷️ Program Hash</h4>
                            <div className="hash-display">{result.program_hash}</div>
                          </div>
                        )}

                        {result.cycles && (
                          <div className="result-section">
                            <h4>⚡ Execution Cycles</h4>
                            <div className="cycles-display">{result.cycles.toLocaleString()}</div>
                          </div>
                        )}

                        {(result.compilation_time_ms || result.execution_time_ms || result.total_time_ms) && (
                          <div className="result-section">
                            <h4>⏱️ Performance Metrics</h4>
                            <div style={{ 
                              background: 'var(--bg-tertiary)', 
                              borderRadius: 'var(--radius-md)', 
                              padding: '1rem',
                              display: 'grid',
                              gridTemplateColumns: 'repeat(auto-fit, minmax(120px, 1fr))',
                              gap: '0.75rem'
                            }}>
                              {result.compilation_time_ms && (
                                <div style={{ textAlign: 'center' }}>
                                  <div style={{ 
                                    fontSize: '0.875rem', 
                                    color: 'var(--text-muted)', 
                                    marginBottom: '0.25rem' 
                                  }}>
                                    Compilation
                                  </div>
                                  <div style={{ 
                                    fontWeight: '600', 
                                    color: 'var(--primary-orange)' 
                                  }}>
                                    {result.compilation_time_ms.toFixed(2)}ms
                                  </div>
                                </div>
                              )}
                              {result.execution_time_ms && (
                                <div style={{ textAlign: 'center' }}>
                                  <div style={{ 
                                    fontSize: '0.875rem', 
                                    color: 'var(--text-muted)', 
                                    marginBottom: '0.25rem' 
                                  }}>
                                    Execution
                                  </div>
                                  <div style={{ 
                                    fontWeight: '600', 
                                    color: 'var(--primary-orange)' 
                                  }}>
                                    {result.execution_time_ms.toFixed(2)}ms
                                  </div>
                                </div>
                              )}
                              {result.total_time_ms && (
                                <div style={{ textAlign: 'center' }}>
                                  <div style={{ 
                                    fontSize: '0.875rem', 
                                    color: 'var(--text-muted)', 
                                    marginBottom: '0.25rem' 
                                  }}>
                                    Total
                                  </div>
                                  <div style={{ 
                                    fontWeight: '600', 
                                    color: 'var(--text-primary)' 
                                  }}>
                                    {result.total_time_ms.toFixed(2)}ms
                                  </div>
                                </div>
                              )}
                            </div>
                          </div>
                        )}
                      </>
                    ) : (
                      <div className="result-section">
                        <h4 style={{ color: '#ef4444' }}>❌ Execution Error</h4>
                        <div style={{ 
                          background: '#1e293b', 
                          color: '#ef4444', 
                          padding: '1rem', 
                          borderRadius: 'var(--radius-md)', 
                          fontFamily: 'Fira Code, monospace',
                          fontSize: '0.875rem',
                          whiteSpace: 'pre-wrap',
                          wordBreak: 'break-word'
                        }}>
                          {result.error}
                        </div>
                      </div>
                    )}
                  </div>
                ) : (
                  <div className="placeholder">
                    Click "Execute Program" to see results
                  </div>
                )}
              </div>
            )}

            {activeTab === 'proof' && (
              <div className="proof-results">
                {proofResult ? (
                  <div className={`result-panel ${proofResult.success ? 'success' : 'error'}`}>
                    {proofResult.success ? (
                      <>
                        <div className="result-section">
                          <h4>📦 Stack Outputs</h4>
                          <div className="stack-display">
                            {proofResult.stack_outputs && proofResult.stack_outputs.length > 0 ? (
                              formatStackOutput(proofResult.stack_outputs)
                            ) : (
                              <div className="empty-stack">Stack is empty</div>
                            )}
                          </div>
                        </div>

                        {proofResult.proof_bytes && (
                          <div className="result-section">
                            <h4>🔐 Proof Data</h4>
                            <div style={{ 
                              background: '#1e293b', 
                              borderRadius: 'var(--radius-md)', 
                              padding: '1rem',
                              border: '1px solid var(--border-light)'
                            }}>
                              <div style={{ 
                                color: 'var(--primary-orange)', 
                                fontWeight: '600', 
                                marginBottom: '0.5rem' 
                              }}>
                                Size: {proofResult.proof_bytes.length} bytes
                              </div>
                              <div style={{ 
                                fontFamily: 'Fira Code, monospace',
                                color: '#e2e8f0',
                                fontSize: '0.85rem',
                                wordBreak: 'break-all',
                                lineHeight: '1.4'
                              }}>
                                {formatProofBytes(proofResult.proof_bytes)}
                              </div>
                            </div>
                          </div>
                        )}

                        {proofResult.program_hash && (
                          <div className="result-section">
                            <h4>🏷️ Program Hash</h4>
                            <div className="hash-display">{proofResult.program_hash}</div>
                          </div>
                        )}

                        {(proofResult.compilation_time_ms || proofResult.proving_time_ms || proofResult.total_time_ms) && (
                          <div className="result-section">
                            <h4>⏱️ Performance Metrics</h4>
                            <div style={{ 
                              background: 'var(--bg-tertiary)', 
                              borderRadius: 'var(--radius-md)', 
                              padding: '1rem',
                              display: 'grid',
                              gridTemplateColumns: 'repeat(auto-fit, minmax(120px, 1fr))',
                              gap: '0.75rem'
                            }}>
                              {proofResult.compilation_time_ms && (
                                <div style={{ textAlign: 'center' }}>
                                  <div style={{ 
                                    fontSize: '0.875rem', 
                                    color: 'var(--text-muted)', 
                                    marginBottom: '0.25rem' 
                                  }}>
                                    Compilation
                                  </div>
                                  <div style={{ 
                                    fontWeight: '600', 
                                    color: 'var(--primary-orange)' 
                                  }}>
                                    {proofResult.compilation_time_ms.toFixed(2)}ms
                                  </div>
                                </div>
                              )}
                              {proofResult.proving_time_ms && (
                                <div style={{ textAlign: 'center' }}>
                                  <div style={{ 
                                    fontSize: '0.875rem', 
                                    color: 'var(--text-muted)', 
                                    marginBottom: '0.25rem' 
                                  }}>
                                    Proving
                                  </div>
                                  <div style={{ 
                                    fontWeight: '600', 
                                    color: 'var(--primary-orange)' 
                                  }}>
                                    {proofResult.proving_time_ms.toFixed(2)}ms
                                  </div>
                                </div>
                              )}
                              {proofResult.total_time_ms && (
                                <div style={{ textAlign: 'center' }}>
                                  <div style={{ 
                                    fontSize: '0.875rem', 
                                    color: 'var(--text-muted)', 
                                    marginBottom: '0.25rem' 
                                  }}>
                                    Total
                                  </div>
                                  <div style={{ 
                                    fontWeight: '600', 
                                    color: 'var(--text-primary)' 
                                  }}>
                                    {proofResult.total_time_ms.toFixed(2)}ms
                                  </div>
                                </div>
                              )}
                            </div>
                          </div>
                        )}
                      </>
                    ) : (
                      <div className="result-section">
                        <h4 style={{ color: '#ef4444' }}>❌ Proof Generation Error</h4>
                        <div style={{ 
                          background: '#1e293b', 
                          color: '#ef4444', 
                          padding: '1rem', 
                          borderRadius: 'var(--radius-md)', 
                          fontFamily: 'Fira Code, monospace',
                          fontSize: '0.875rem',
                          whiteSpace: 'pre-wrap',
                          wordBreak: 'break-word'
                        }}>
                          {proofResult.error}
                        </div>
                      </div>
                    )}
                  </div>
                ) : (
                  <div className="placeholder">
                    Click "Generate Proof" to create a proof
                  </div>
                )}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
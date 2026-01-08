import React, { useState, useEffect } from 'react';
import init, { query } from 'claude-agent-sdk-rs';

function App() {
  const [prompt, setPrompt] = useState('What is 2 + 2?');
  const [response, setResponse] = useState('');
  const [loading, setLoading] = useState(false);
  const [initialized, setInitialized] = useState(false);
  const [error, setError] = useState(null);

  // Initialize WASM on mount
  useEffect(() => {
    let mounted = true;

    init()
      .then(() => {
        if (mounted) {
          setInitialized(true);
          console.log('WASM initialized successfully');
        }
      })
      .catch((err) => {
        if (mounted) {
          console.error('Failed to initialize WASM:', err);
          setError(err.message);
        }
      });

    return () => {
      mounted = false;
    };
  }, []);

  const handleQuery = async () => {
    if (!initialized) {
      setError('WASM not initialized yet');
      return;
    }

    setLoading(true);
    setError(null);
    setResponse('');

    try {
      console.log('Sending query:', prompt);
      const result = await query(prompt, null);
      console.log('Received result:', result);

      // Format the response
      if (typeof result === 'string') {
        setResponse(result);
      } else if (Array.isArray(result)) {
        const formatted = result
          .filter(msg => msg.type === 'assistant')
          .map(msg => msg.content?.join('\n') || '')
          .join('\n\n');
        setResponse(formatted);
      } else {
        setResponse(JSON.stringify(result, null, 2));
      }
    } catch (err) {
      console.error('Query failed:', err);
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{ maxWidth: '800px', margin: '0 auto', padding: '20px' }}>
      <h1>🤖 Claude Agent SDK - React + WASM</h1>

      {!initialized && !error && (
        <div style={{ padding: '20px', background: '#f0f0f0', borderRadius: '4px' }}>
          Initializing WASM...
        </div>
      )}

      {error && (
        <div style={{ padding: '20px', background: '#ffeeee', borderRadius: '4px', marginBottom: '20px' }}>
          <strong>Error:</strong> {error}
        </div>
      )}

      {initialized && (
        <>
          <div style={{ marginBottom: '20px' }}>
            <label style={{ display: 'block', marginBottom: '8px', fontWeight: 'bold' }}>
              Your Query:
            </label>
            <textarea
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              rows={4}
              style={{
                width: '100%',
                padding: '10px',
                border: '1px solid #ddd',
                borderRadius: '4px',
                fontFamily: 'monospace',
                fontSize: '14px'
              }}
              placeholder="Ask Claude something..."
            />
          </div>

          <button
            onClick={handleQuery}
            disabled={loading || !prompt.trim()}
            style={{
              background: loading || !prompt.trim() ? '#ccc' : '#0066cc',
              color: 'white',
              border: 'none',
              padding: '12px 24px',
              borderRadius: '4px',
              cursor: loading || !prompt.trim() ? 'not-allowed' : 'pointer',
              fontSize: '16px',
              marginRight: '10px'
            }}
          >
            {loading ? 'Processing...' : 'Query Claude'}
          </button>

          <button
            onClick={() => setResponse('')}
            style={{
              background: '#f0f0f0',
              color: '#333',
              border: '1px solid #ccc',
              padding: '12px 24px',
              borderRadius: '4px',
              cursor: 'pointer',
              fontSize: '16px'
            }}
          >
            Clear
          </button>

          {response && (
            <div style={{ marginTop: '20px' }}>
              <h3 style={{ marginBottom: '10px' }}>Response:</h3>
              <pre style={{
                background: '#f9f9f9',
                padding: '15px',
                borderLeft: '4px solid #0066cc',
                borderRadius: '4px',
                overflow: 'auto',
                whiteSpace: 'pre-wrap',
                wordBreak: 'break-word'
              }}>
                {response}
              </pre>
            </div>
          )}

          <div style={{ marginTop: '30px', padding: '15px', background: '#f0f7ff', borderRadius: '4px' }}>
            <h4 style={{ marginTop: 0 }}>✅ WASM Status</h4>
            <ul style={{ marginBottom: 0 }}>
              <li>Initialized: <strong>Yes</strong></li>
              <li>Platform: <strong>WebAssembly</strong></li>
              <li>Performance: <strong>Excellent</strong></li>
              <li>Bundle Size: <strong>~4MB (optimized)</strong></li>
            </ul>
          </div>
        </>
      )}
    </div>
  );
}

export default App;

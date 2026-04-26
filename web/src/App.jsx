import { useState, useEffect, useCallback } from 'react'
import ForceGraph2D from 'react-force-graph-2d'
import './index.css'

const KIND_COLORS = {
  function: '#58a6ff',
  method: '#3fb950',
  class: '#f0883e',
  struct: '#bc8cff',
  interface: '#79c0ff',
  enum: '#ffa657',
  trait: '#d2a8ff',
  module: '#8b949e',
  variable: '#7ee787',
  constant: '#79c0ff',
  macro: '#ffa657',
  other: '#484f58',
}

const LANGUAGES = [
  { value: '', label: 'All languages' },
  { value: 'rust', label: 'Rust' },
  { value: 'python', label: 'Python' },
  { value: 'javascript', label: 'JavaScript' },
  { value: 'typescript', label: 'TypeScript' },
  { value: 'java', label: 'Java' },
  { value: 'go', label: 'Go' },
  { value: 'c', label: 'C' },
  { value: 'cpp', label: 'C++' },
  { value: 'csharp', label: 'C#' },
  { value: 'swift', label: 'Swift' },
  { value: 'kotlin', label: 'Kotlin' },
  { value: 'php', label: 'PHP' },
  { value: 'ruby', label: 'Ruby' },
  { value: 'shell', label: 'Shell' },
]

async function apiSearch(query, language, limit) {
  const res = await fetch('/api/search', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ query, language: language || undefined, limit, local: true }),
  })
  return res.json()
}

async function apiSearchGraph(query, language, limit) {
  const res = await fetch('/api/search/graph', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ query, language: language || undefined, limit, local: true }),
  })
  return res.json()
}

async function apiIndex() {
  const res = await fetch('/api/index', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ local: true }),
  })
  return res.json()
}

async function apiIndexStatus() {
  const res = await fetch('/api/index/status')
  return res.json()
}

async function apiRepoInfo() {
  const res = await fetch('/api/repo/info')
  return res.json()
}

export default function App() {
  const [query, setQuery] = useState('')
  const [language, setLanguage] = useState('')
  const [results, setResults] = useState([])
  const [graphData, setGraphData] = useState(null)
  const [total, setTotal] = useState(0)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const [viewMode, setViewMode] = useState('list') // 'list' | 'graph'
  const [repoInfo, setRepoInfo] = useState(null)
  const [indexing, setIndexing] = useState(false)
  const [indexProgress, setIndexProgress] = useState(null)
  const [selectedNode, setSelectedNode] = useState(null)

  // Graph interaction state
  const fgRef = useState(null)
  const [hoveredNodeId, setHoveredNodeId] = useState(null)
  const [highlight, setHighlight] = useState({ nodes: new Set(), links: new Set() })

  const fetchRepoInfo = useCallback(async () => {
    try {
      const info = await apiRepoInfo()
      if (info.path) setRepoInfo(info)
    } catch { /* server may not be running */ }
  }, [])

  useEffect(() => { fetchRepoInfo() }, [fetchRepoInfo])

  const checkIndex = useCallback(async () => {
    try {
      const status = await apiIndexStatus()
      setIndexing(status.is_indexing)
      if (status.progress) setIndexProgress(status.progress)
      if (!status.is_indexing) setIndexProgress(null)
    } catch { /* ignore */ }
  }, [])

  useEffect(() => {
    const timer = setInterval(checkIndex, 2000)
    return () => clearInterval(timer)
  }, [checkIndex])

  const doSearch = async (mode) => {
    if (!query.trim()) return
    setLoading(true)
    setError('')
    setGraphData(null)
    setSelectedNode(null)
    try {
      if (mode === 'graph') {
        const data = await apiSearchGraph(query, language, 50)
        setResults([])
        setGraphData(data)
        setTotal(data.total || 0)
        setViewMode('graph')
      } else {
        const data = await apiSearch(query, language, 20)
        setResults(data.results || [])
        setTotal(data.total || 0)
        setViewMode('list')
      }
    } catch (err) {
      setError(err.message || 'Search failed')
      setResults([])
      setGraphData(null)
      setTotal(0)
    } finally {
      setLoading(false)
    }
  }

  const handleSearch = (e) => { e.preventDefault(); doSearch('list') }
  const handleGraphSearch = (e) => { e.preventDefault(); doSearch('graph') }

  const handleIndex = async () => {
    setIndexing(true)
    setIndexProgress(null)
    try {
      await apiIndex()
    } catch (err) {
      setError(err.message)
      setIndexing(false)
    }
  }

  // Graph hover logic
  const handleNodeHover = useCallback((node) => {
    if (!node || !graphData) {
      setHighlight({ nodes: new Set(), links: new Set() })
      setHoveredNodeId(null)
      return
    }
    setHoveredNodeId(node.id)
    const linkSet = new Set()
    const nodeSet = new Set([node.id])
    ;(graphData.links || []).forEach((l, i) => {
      const srcId = typeof l.source === 'object' ? l.source.id : l.source
      const tgtId = typeof l.target === 'object' ? l.target.id : l.target
      if (srcId === node.id || tgtId === node.id) {
        linkSet.add(i)
        nodeSet.add(srcId)
        nodeSet.add(tgtId)
      }
    })
    setHighlight({ nodes: nodeSet, links: linkSet })
  }, [graphData])

  const handleNodeClick = useCallback((node) => {
    if (node) {
      setSelectedNode(node)
      if (fgRef[0]) {
        fgRef[0].centerAt(node.x, node.y, 500)
        fgRef[0].zoom(3, 500)
      }
    } else {
      setSelectedNode(null)
    }
  }, [fgRef])

  const hasData = results.length > 0 || (graphData && graphData.nodes && graphData.nodes.length > 0)

  return (
    <div className="container">
      <h1>CodeRAG</h1>
      <p className="subtitle">AST-based code search with vector embeddings</p>

      <form className="search-bar" onSubmit={handleSearch}>
        <input
          type="text"
          placeholder="Search code... e.g. &quot;authentication middleware&quot;"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        <select value={language} onChange={(e) => setLanguage(e.target.value)}>
          {LANGUAGES.map((l) => (
            <option key={l.value} value={l.value}>{l.label}</option>
          ))}
        </select>
        <button type="submit" disabled={loading || !query.trim()}>
          {loading ? 'Searching...' : 'List'}
        </button>
        <button type="button" onClick={handleGraphSearch} disabled={loading || !query.trim()}
          style={{ background: '#1f6feb' }}>
          {loading ? 'Searching...' : 'Graph'}
        </button>
      </form>

      <div style={{ display: 'flex', gap: '1rem', alignItems: 'center', marginBottom: '1rem' }}>
        <button className="index-btn" onClick={handleIndex} disabled={indexing}>
          {indexing ? 'Indexing...' : 'Reindex'}
        </button>
        {indexing && indexProgress && (
          <span style={{ fontSize: '0.8rem', color: '#8b949e' }}>
            {indexProgress.files_processed} files, {indexProgress.symbols_extracted} symbols
          </span>
        )}
      </div>

      {error && <div className="error-msg">{error}</div>}
      {loading && <div className="loading">Searching...</div>}

      {/* Graph View */}
      {!loading && viewMode === 'graph' && graphData && (
        <div style={{ position: 'relative', height: '70vh', border: '1px solid #30363d', borderRadius: 8, overflow: 'hidden' }}>
          <div style={{
            position: 'absolute', top: 8, left: 8, zIndex: 10,
            background: '#161b22ee', padding: '0.4rem 0.75rem', borderRadius: 6,
            fontSize: '0.75rem', color: '#8b949e', pointerEvents: 'none',
          }}>
            {graphData.nodes?.length || 0} nodes, {graphData.links?.length || 0} edges &middot; scroll to zoom, drag nodes
          </div>

          {/* Legend */}
          <div style={{
            position: 'absolute', top: 8, right: 8, zIndex: 10,
            background: '#161b22ee', padding: '0.5rem 0.75rem', borderRadius: 6,
            fontSize: '0.7rem', color: '#8b949e',
          }}>
            {Object.entries(KIND_COLORS).slice(0, 8).map(([k, v]) => (
              <div key={k} style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
                <span style={{ width: 8, height: 8, borderRadius: '50%', background: v, display: 'inline-block' }} />
                {k}
              </div>
            ))}
          </div>

          {hoveredNodeId && graphData.nodes && (
            <div style={{
              position: 'absolute', bottom: 8, left: 8, right: 8, zIndex: 10,
              background: '#161b22f0', padding: '0.75rem', borderRadius: 6,
              border: '1px solid #30363d',
            }}>
              {(() => {
                const node = graphData.nodes.find(n => n.id === hoveredNodeId)
                if (!node) return null
                return (
                  <>
                    <div style={{ fontWeight: 600, fontSize: '0.95rem' }}>{node.name}</div>
                    <div style={{ fontSize: '0.8rem', color: '#8b949e' }}>{node.file} : {node.start_line}</div>
                    <div style={{ fontSize: '0.8rem', color: '#8b949e', marginTop: '0.25rem' }}>
                      Score: {node.score.toFixed(4)} &middot; {node.kind}
                    </div>
                    {node.doc && (
                      <div style={{ fontSize: '0.8rem', color: '#8b949e', marginTop: '0.25rem' }}>
                        {node.doc.length > 120 ? node.doc.slice(0, 120) + '...' : node.doc}
                      </div>
                    )}
                  </>
                )
              })()}
            </div>
          )}

          {selectedNode && (
            <div style={{
              position: 'absolute', top: 8, left: '50%', transform: 'translateX(-50%)', zIndex: 10,
              background: '#161b22f0', padding: '0.75rem 1rem', borderRadius: 6,
              border: '1px solid #58a6ff', maxWidth: 500, maxHeight: 200, overflow: 'auto',
            }}>
              <pre style={{ fontSize: '0.8rem', fontFamily: 'monospace', color: '#e6edf3', margin: 0, whiteSpace: 'pre-wrap' }}>
                {selectedNode.code}
              </pre>
            </div>
          )}

          <ForceGraph2D
            ref={(el) => {
              // Update ref without triggering re-render issues
              if (el && !fgRef[0]) fgRef[0] = el
            }}
            graphData={{
              nodes: (graphData.nodes || []).map(n => ({ ...n, color: KIND_COLORS[n.kind] || KIND_COLORS.other })),
              links: (graphData.links || []).map(l => ({ ...l })),
            }}
            nodeLabel="name"
            nodeColor="color"
            nodeVal={(node) => Math.max(3, node.val || 5)}
            nodeCanvasObjectMode={() => 'after'}
            nodeCanvasObject={(node, ctx, globalScale) => {
              const isSelected = highlight.nodes.has(node.id)
              const isDimmed = highlight.nodes.size > 0 && !isSelected
              const label = node.name
              const fontSize = Math.max(8, 12 / globalScale)
              const radius = Math.max(3, node.val || 5)

              ctx.beginPath()
              ctx.arc(node.x, node.y, radius, 0, 2 * Math.PI)
              ctx.fillStyle = isDimmed ? '#484f5844' : (node.color || '#484f58')
              ctx.fill()

              if (isSelected || !highlight.nodes.size) {
                ctx.font = `${fontSize} Sans-Serif`
                ctx.fillStyle = isDimmed ? '#484f5844' : '#e6edf3'
                ctx.fillText(label, node.x + radius + 2, node.y + fontSize / 2)
              }
            }}
            linkWidth={(link) => {
              const idx = (graphData.links || []).indexOf(link)
              if (highlight.links.size > 0 && !highlight.links.has(idx)) return 0.2
              return 1
            }}
            linkColor={(link) => {
              const idx = (graphData.links || []).indexOf(link)
              if (highlight.links.size > 0 && !highlight.links.has(idx)) return '#30363d22'
              return '#58a6ff88'
            }}
            linkDirectionalParticles={(link) => {
              const idx = (graphData.links || []).indexOf(link)
              return highlight.links.has(idx) ? 2 : 0
            }}
            linkDirectionalParticleWidth={1}
            onNodeHover={handleNodeHover}
            onNodeClick={handleNodeClick}
            backgroundColor="#0d1117"
          />
        </div>
      )}

      {/* List View */}
      {!loading && viewMode === 'list' && results.length > 0 && (
        <>
          <div className="results-header">{total} result{total !== 1 ? 's' : ''}</div>
          {results.map((r, i) => (
            <div className="result-card" key={r.id || i}>
              <div className="result-header">
                <span className="result-kind">{r.kind}</span>
                <span className="result-score">{r.score.toFixed(4)}</span>
              </div>
              <div className="result-symbol">{r.symbol || r.signature}</div>
              <div className="result-file">{r.file}</div>
              {r.doc && (
                <div style={{ fontSize: '0.85rem', color: '#8b949e', marginBottom: '0.5rem' }}>
                  {r.doc.length > 150 ? r.doc.slice(0, 150) + '...' : r.doc}
                </div>
              )}
              <pre className="result-code">{r.code}</pre>
              {r.start_line > 0 && (
                <div className="result-lines">
                  Lines {r.start_line}{r.end_line !== r.start_line ? `-${r.end_line}` : ''}
                </div>
              )}
            </div>
          ))}
        </>
      )}

      {!loading && query && !hasData && (
        <div className="empty-state">No results found. Try a different query or reindex.</div>
      )}

      {repoInfo && (
        <div className="repo-status">
          <span>{repoInfo.path} &middot; {repoInfo.branch} &middot; {repoInfo.file_count} files</span>
        </div>
      )}
    </div>
  )
}

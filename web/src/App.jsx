import { useState, useEffect, useCallback } from 'react'
import './index.css'

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

async function apiIndex(full) {
  const res = await fetch('/api/index', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ full, local: true }),
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
  const [total, setTotal] = useState(0)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const [repoInfo, setRepoInfo] = useState(null)
  const [indexing, setIndexing] = useState(false)
  const [indexProgress, setIndexProgress] = useState(null)

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

  const handleSearch = async (e) => {
    e.preventDefault()
    if (!query.trim()) return

    setLoading(true)
    setError('')
    try {
      const data = await apiSearch(query, language, 20)
      setResults(data.results || [])
      setTotal(data.total || 0)
    } catch (err) {
      setError(err.message || 'Search failed')
      setResults([])
      setTotal(0)
    } finally {
      setLoading(false)
    }
  }

  const handleIndex = async () => {
    setIndexing(true)
    setIndexProgress(null)
    try {
      await apiIndex(true)
    } catch (err) {
      setError(err.message)
      setIndexing(false)
    }
  }

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
          {loading ? 'Searching...' : 'Search'}
        </button>
      </form>

      <div style={{ display: 'flex', gap: '1rem', alignItems: 'center', marginBottom: '1rem' }}>
        <button className="index-btn" onClick={handleIndex} disabled={indexing}>
          {indexing ? 'Indexing...' : 'Reindex'}
        </button>
        {indexing && indexProgress && (
          <span style={{ fontSize: '0.8rem', color: '#8b949e' }}>
            {indexProgress.files_processed} files, {indexProgress.symbols_extracted} symbols, {indexProgress.chunks_created} chunks
          </span>
        )}
      </div>

      {error && <div className="error-msg">{error}</div>}

      {loading && <div className="loading">Searching...</div>}

      {!loading && results.length > 0 && (
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

      {!loading && query && results.length === 0 && (
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

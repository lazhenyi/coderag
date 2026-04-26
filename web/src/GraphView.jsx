import { useRef, useCallback, useState } from 'react'
import ForceGraph2D from 'react-force-graph-2d'

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

const LINK_COLORS = {
  same_file: '#30363d',
  same_module: '#58a6ff44',
  parent: '#f0883e',
}

export default function GraphView({ data, onSelect }) {
  const fgRef = useRef()
  const [hoveredNode, setHoveredNode] = useState(null)
  const [highlight, setHighlight] = useState({ nodes: new Set(), links: new Set() })

  const nodeCount = data.nodes?.length || 0
  const linkCount = data.links?.length || 0

  const handleNodeHover = useCallback((node) => {
    if (!node) {
      setHighlight({ nodes: new Set(), links: new Set() })
      setHoveredNode(null)
      return
    }
    setHoveredNode(node)
    const linkSet = new Set()
    const nodeSet = new Set([node.id])
    ;(data.links || []).forEach((l, i) => {
      const srcId = typeof l.source === 'object' ? l.source.id : l.source
      const tgtId = typeof l.target === 'object' ? l.target.id : l.target
      if (srcId === node.id || tgtId === node.id) {
        linkSet.add(i)
        nodeSet.add(srcId)
        nodeSet.add(tgtId)
      }
    })
    setHighlight({ nodes: nodeSet, links: linkSet })
  }, [data.links])

  const handleNodeClick = useCallback((node) => {
    if (onSelect) onSelect(node)
    // zoom to node
    if (fgRef.current) {
      fgRef.current.centerAt(node.x, node.y, 500)
      fgRef.current.zoom(3, 500)
    }
  }, [onSelect])

  if (!data.nodes || data.nodes.length === 0) {
    return <div className="empty-state">No graph data. Try a different query.</div>
  }

  const graphData = {
    nodes: data.nodes.map(n => ({
      ...n,
      color: KIND_COLORS[n.kind] || KIND_COLORS.other,
    })),
    links: (data.links || []).map(l => ({
      ...l,
      color: LINK_COLORS[l.relation] || '#30363d',
    })),
  }

  return (
    <div style={{ position: 'relative' }}>
      <div style={{
        position: 'absolute', top: 8, left: 8, zIndex: 10,
        background: '#161b22ee', padding: '0.5rem 0.75rem', borderRadius: 6,
        fontSize: '0.75rem', color: '#8b949e', pointerEvents: 'none',
      }}>
        {nodeCount} nodes, {linkCount} edges &middot; scroll to zoom, drag nodes
      </div>

      {hoveredNode && (
        <div style={{
          position: 'absolute', bottom: 8, left: 8, right: 8, zIndex: 10,
          background: '#161b22f0', padding: '0.75rem', borderRadius: 6,
          border: '1px solid #30363d',
        }}>
          <div style={{ fontWeight: 600, fontSize: '0.95rem' }}>{hoveredNode.name}</div>
          <div style={{ fontSize: '0.8rem', color: '#8b949e' }}>{hoveredNode.file} : {hoveredNode.start_line}</div>
          <div style={{ fontSize: '0.8rem', color: '#8b949e', marginTop: '0.25rem' }}>
            Score: {hoveredNode.score.toFixed(4)} &middot; {hoveredNode.kind}
          </div>
          {hoveredNode.doc && (
            <div style={{ fontSize: '0.8rem', color: '#8b949e', marginTop: '0.25rem' }}>
              {hoveredNode.doc.length > 120 ? hoveredNode.doc.slice(0, 120) + '...' : hoveredNode.doc}
            </div>
          )}
        </div>
      )}

      <ForceGraph2D
        ref={fgRef}
        graphData={graphData}
        nodeLabel="name"
        nodeColor="color"
        nodeVal="val"
        nodeCanvasObjectMode={() => 'after'}
        nodeCanvasObject={(node, ctx, globalScale) => {
          const isSelected = highlight.nodes.has(node.id)
          const isDimmed = highlight.nodes.size > 0 && !highlight.nodes.has(node.id)
          const label = node.name
          const fontSize = Math.max(8, 12 / globalScale)
          const radius = Math.max(3, node.val || 5)

          ctx.beginPath()
          ctx.arc(node.x, node.y, radius, 0, 2 * Math.PI)
          ctx.fillStyle = isDimmed ? '#484f5844' : (node.color || '#484f58')
          ctx.fill()

          if (isSelected || (!highlight.nodes.size)) {
            ctx.font = `${fontSize} Sans-Serif`
            ctx.fillStyle = isDimmed ? '#484f5844' : '#e6edf3'
            ctx.fillText(label, node.x + radius + 2, node.y + fontSize / 2)
          }
        }}
        linkColor={link => {
          const idx = graphData.links.indexOf(link)
          if (highlight.links.size > 0 && !highlight.links.has(idx)) return '#30363d22'
          return link.color || '#30363d'
        }}
        linkWidth={link => {
          const idx = graphData.links.indexOf(link)
          if (highlight.links.size > 0 && !highlight.links.has(idx)) return 0.2
          return 1
        }}
        linkDirectionalParticles={link => {
          const idx = graphData.links.indexOf(link)
          return highlight.links.has(idx) ? 1 : 0
        }}
        onNodeHover={handleNodeHover}
        onNodeClick={handleNodeClick}
        backgroundColor="#0d1117"
      />
    </div>
  )
}

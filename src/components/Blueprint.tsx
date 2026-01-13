// Cerebras-MAKER: The Blueprint Component
// PRD Section 6.2: 3D interactive visualization of the grits-core SymbolGraph

import { useRef, useMemo } from 'react';
import { Canvas, useFrame, ThreeEvent } from '@react-three/fiber';
import { OrbitControls, Text, Line } from '@react-three/drei';
import { useMakerStore, SymbolNode, SymbolEdge } from '../store/makerStore';
import * as THREE from 'three';
import './Blueprint.css';

// Color scheme for different symbol kinds
const KIND_COLORS: Record<string, string> = {
  function: '#4ade80',
  struct: '#60a5fa',
  class: '#a78bfa',
  interface: '#f472b6',
  module: '#fbbf24',
  const: '#22d3ee',
  type: '#fb923c',
  default: '#9ca3af',
};

interface NodeMeshProps {
  node: SymbolNode;
  selected: boolean;
  onClick: () => void;
}

function NodeMesh({ node, selected, onClick }: NodeMeshProps) {
  const meshRef = useRef<THREE.Mesh>(null);
  const color = KIND_COLORS[node.kind] || KIND_COLORS.default;
  
  useFrame((state) => {
    if (meshRef.current && selected) {
      meshRef.current.scale.setScalar(1 + Math.sin(state.clock.elapsedTime * 4) * 0.1);
    }
  });
  
  return (
    <group position={node.position}>
      <mesh
        ref={meshRef}
        onClick={(e: ThreeEvent<MouseEvent>) => {
          e.stopPropagation();
          onClick();
        }}
      >
        <sphereGeometry args={[0.3, 16, 16]} />
        <meshStandardMaterial 
          color={color} 
          emissive={selected ? color : '#000000'}
          emissiveIntensity={selected ? 0.5 : 0}
        />
      </mesh>
      <Text
        position={[0, 0.5, 0]}
        fontSize={0.2}
        color="white"
        anchorX="center"
        anchorY="middle"
      >
        {node.name}
      </Text>
    </group>
  );
}

interface EdgeLineProps {
  edge: SymbolEdge;
  nodes: SymbolNode[];
}

function EdgeLine({ edge, nodes }: EdgeLineProps) {
  const sourceNode = nodes.find(n => n.id === edge.source);
  const targetNode = nodes.find(n => n.id === edge.target);
  
  if (!sourceNode || !targetNode) return null;
  
  const points = useMemo(() => [
    new THREE.Vector3(...sourceNode.position),
    new THREE.Vector3(...targetNode.position),
  ], [sourceNode.position, targetNode.position]);
  
  return (
    <Line
      points={points}
      color="#4b5563"
      lineWidth={1}
      opacity={0.5}
      transparent
    />
  );
}

function Graph() {
  const { symbolNodes, symbolEdges, selectedNode, setSelectedNode } = useMakerStore();
  
  return (
    <>
      {/* Render edges first (behind nodes) */}
      {symbolEdges.map((edge, idx) => (
        <EdgeLine key={`edge-${idx}`} edge={edge} nodes={symbolNodes} />
      ))}
      
      {/* Render nodes */}
      {symbolNodes.map((node) => (
        <NodeMesh
          key={node.id}
          node={node}
          selected={selectedNode === node.id}
          onClick={() => setSelectedNode(selectedNode === node.id ? null : node.id)}
        />
      ))}
    </>
  );
}

export function Blueprint() {
  const { symbolNodes, selectedNode, redFlagResult } = useMakerStore();
  const selectedNodeData = symbolNodes.find(n => n.id === selectedNode);
  
  return (
    <div className="blueprint">
      <div className="blueprint-header">
        <h2>🏗️ Blueprint</h2>
        <div className="blueprint-stats">
          <span>Nodes: {symbolNodes.length}</span>
          {redFlagResult && (
            <span className={redFlagResult.introduced_cycle ? 'red-flag' : 'green-flag'}>
              {redFlagResult.introduced_cycle ? '🚩 Cycle Detected' : '✅ Clean'}
            </span>
          )}
        </div>
      </div>
      
      <div className="blueprint-canvas">
        <Canvas camera={{ position: [5, 5, 5], fov: 60 }}>
          <ambientLight intensity={0.5} />
          <pointLight position={[10, 10, 10]} />
          <Graph />
          <OrbitControls enableDamping dampingFactor={0.05} />
          <gridHelper args={[20, 20, '#333', '#222']} />
        </Canvas>
      </div>
      
      {selectedNodeData && (
        <div className="node-details">
          <h3>{selectedNodeData.name}</h3>
          <p><strong>Kind:</strong> {selectedNodeData.kind}</p>
          <p><strong>File:</strong> {selectedNodeData.file}</p>
        </div>
      )}
    </div>
  );
}


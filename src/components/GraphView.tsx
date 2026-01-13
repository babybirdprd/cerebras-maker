import React, { useEffect, useRef, useState } from 'react';
import * as d3 from 'd3';
import { GraphNode, GraphLink } from '../types';

interface GraphViewProps {
  nodes: GraphNode[];
  links: GraphLink[];
}

const GraphView: React.FC<GraphViewProps> = ({ nodes, links }) => {
  const svgRef = useRef<SVGSVGElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [dimensions, setDimensions] = useState({ width: 0, height: 0 });

  useEffect(() => {
    const handleResize = () => {
      if (containerRef.current) {
        setDimensions({
          width: containerRef.current.clientWidth,
          height: containerRef.current.clientHeight
        });
      }
    };

    window.addEventListener('resize', handleResize);
    handleResize();

    return () => window.removeEventListener('resize', handleResize);
  }, []);

  useEffect(() => {
    if (!svgRef.current || !containerRef.current || dimensions.width === 0) return;

    const { width, height } = dimensions;

    d3.select(svgRef.current).selectAll("*").remove();

    const svg = d3.select(svgRef.current)
      .attr("viewBox", [0, 0, width, height]);

    const color = d3.scaleOrdinal<string>()
      .domain(["1", "2", "3", "4"])
      .range(["#6366f1", "#10b981", "#f59e0b", "#ec4899"]);

    const simulation = d3.forceSimulation(nodes as d3.SimulationNodeDatum[])
      .force("link", d3.forceLink(links).id((d: any) => d.id).distance(100))
      .force("charge", d3.forceManyBody().strength(-300))
      .force("center", d3.forceCenter(width / 2, height / 2));

    const link = svg.append("g")
      .attr("stroke", "#3f3f46")
      .attr("stroke-opacity", 0.6)
      .selectAll("line")
      .data(links)
      .join("line")
      .attr("stroke-width", d => Math.sqrt(d.value));

    const node = svg.append("g")
      .attr("stroke", "#fff")
      .attr("stroke-width", 1.5)
      .selectAll("circle")
      .data(nodes)
      .join("circle")
      .attr("r", d => Math.max(5, d.val))
      .attr("fill", (d) => color(String(d.group)))
      .call(drag(simulation) as any);

    const label = svg.append("g")
      .selectAll("text")
      .data(nodes)
      .join("text")
      .attr("dx", 12)
      .attr("dy", ".35em")
      .text(d => d.label)
      .attr("fill", "#e4e4e7")
      .style("font-size", "10px")
      .style("font-family", "monospace")
      .style("pointer-events", "none");

    simulation.on("tick", () => {
      link
        .attr("x1", (d: any) => d.source.x)
        .attr("y1", (d: any) => d.source.y)
        .attr("x2", (d: any) => d.target.x)
        .attr("y2", (d: any) => d.target.y);

      node
        .attr("cx", (d: any) => d.x)
        .attr("cy", (d: any) => d.y);

      label
        .attr("x", (d: any) => d.x)
        .attr("y", (d: any) => d.y);
    });

    function drag(simulation: d3.Simulation<d3.SimulationNodeDatum, undefined>) {
      function dragstarted(event: any) {
        if (!event.active) simulation.alphaTarget(0.3).restart();
        event.subject.fx = event.subject.x;
        event.subject.fy = event.subject.y;
      }

      function dragged(event: any) {
        event.subject.fx = event.x;
        event.subject.fy = event.y;
      }

      function dragended(event: any) {
        if (!event.active) simulation.alphaTarget(0);
        event.subject.fx = null;
        event.subject.fy = null;
      }

      return d3.drag()
        .on("start", dragstarted)
        .on("drag", dragged)
        .on("end", dragended);
    }

    return () => {
      simulation.stop();
    };
  }, [nodes, links, dimensions]);

  return (
    <div ref={containerRef} className="w-full h-full bg-zinc-900 rounded-lg overflow-hidden border border-zinc-800 relative">
        <div className="absolute top-4 left-4 z-10 bg-zinc-900/80 p-2 rounded backdrop-blur-sm border border-zinc-700 pointer-events-none">
            <h3 className="text-xs font-bold text-zinc-400 uppercase tracking-wider mb-1">Topology Analysis (Grits)</h3>
            <div className="flex flex-wrap gap-2 text-xs">
                <span className="flex items-center gap-1"><span className="w-2 h-2 rounded-full bg-indigo-500"></span> Logic</span>
                <span className="flex items-center gap-1"><span className="w-2 h-2 rounded-full bg-emerald-500"></span> Data</span>
                <span className="flex items-center gap-1"><span className="w-2 h-2 rounded-full bg-amber-500"></span> UI</span>
                <span className="flex items-center gap-1"><span className="w-2 h-2 rounded-full bg-pink-500"></span> Ext</span>
            </div>
        </div>
      <svg ref={svgRef} className="w-full h-full"></svg>
    </div>
  );
};

export default GraphView;


# Render Graph (Host-Defined)

This document describes the host-facing render graph format. The host builds a graph using logical IDs; the core validates, maps, and executes it. If the graph is missing or invalid, the core executes a safe fallback graph.

## Goals

- **Host control**: The host defines the render sequence and dependencies.
- **Logical IDs only**: The host never sees GPU handles or internal IDs.
- **Performance**: Minimal per-frame overhead, cacheable plan, and reusable resources.
- **Robustness**: Invalid or missing graphs fall back to a default graph.

## High-Level Flow

1. The host constructs a render graph using logical IDs.
2. The core validates the graph and compiles an execution plan.
3. On failure (or if no graph is supplied), the core uses a fallback graph.

## Graph Structure

### Graph

| Field       | Type        | Description |
|------------|-------------|-------------|
| graph_id   | LogicalId    | Logical graph identifier (cache key) |
| nodes      | Node[]       | Render nodes |
| edges      | Edge[]       | Dependencies between nodes |
| resources  | Resource[]   | Declared resources |
| fallback   | bool         | If true, use fallback when invalid or missing |

### Node

| Field     | Type        | Description |
|----------|-------------|-------------|
| node_id  | LogicalId    | Logical node identifier |
| pass_id  | LogicalId    | Logical pass type (e.g., "forward") |
| inputs   | LogicalId[]  | Resource IDs read by this node |
| outputs  | LogicalId[]  | Resource IDs written by this node |
| params   | Map          | Optional parameters (clear, flags, etc.) |

### Resource

| Field       | Type      | Description |
|------------|-----------|-------------|
| res_id     | LogicalId  | Logical resource identifier |
| kind       | string     | "texture", "buffer", "attachment" |
| desc       | Map        | Logical descriptor (format, size, usage) |
| lifetime   | string     | "frame" or "persistent" |
| alias_group| LogicalId? | Optional alias group for memory reuse |

### Edge

| Field        | Type     | Description |
|-------------|----------|-------------|
| from_node_id| LogicalId| Dependency source |
| to_node_id  | LogicalId| Dependency target |
| reason      | string?  | Optional: "read_after_write", "write_after_read" |

### LogicalId

Logical IDs can be strings or numeric values. The core maps them to internal IDs once per `graph_id` and caches the result to avoid per-frame cost.

## Minimal Example

```json
{
  "graph_id": "main_render",
  "nodes": [
    { "node_id": "shadow_pass", "pass_id": "shadow", "inputs": [], "outputs": ["shadow_atlas"] },
    { "node_id": "forward_pass", "pass_id": "forward", "inputs": ["shadow_atlas"], "outputs": ["hdr_color", "depth"] },
    { "node_id": "compose_pass", "pass_id": "compose", "inputs": ["hdr_color"], "outputs": ["swapchain"] }
  ],
  "edges": [
    { "from_node_id": "shadow_pass", "to_node_id": "forward_pass" },
    { "from_node_id": "forward_pass", "to_node_id": "compose_pass" }
  ],
  "resources": [
    { "res_id": "shadow_atlas", "kind": "texture", "desc": { "format": "depth24", "size": "shadow_res" }, "lifetime": "frame" },
    { "res_id": "hdr_color", "kind": "texture", "desc": { "format": "rgba16f", "size": "screen" }, "lifetime": "frame" },
    { "res_id": "depth", "kind": "texture", "desc": { "format": "depth24", "size": "screen" }, "lifetime": "frame" },
    { "res_id": "swapchain", "kind": "attachment", "desc": { "format": "swapchain" }, "lifetime": "frame" }
  ],
  "fallback": true
}
```

## Validation Rules (Core)

- **DAG only**: No cycles.
- **Resources exist**: Every input/output refers to a declared resource.
- **Write ordering**: A resource must be produced before it is read.
- **Pass compatibility**: Each `pass_id` must be a known core pass type.
- **Format compatibility**: Pass requirements must match declared formats/usages.

If validation fails, the core switches to the fallback graph.

## Fallback Graph

The fallback graph represents the default rendering pipeline that always works. It is used when:

- The host provides no graph.
- The provided graph fails validation.

Example fallback:

```
shadow -> forward -> compose
```

## Performance Notes

- **Cache per graph_id**: Compile once, reuse execution plan and resource layout.
- **Alias groups**: Allow the core to reuse memory for non-overlapping resources.
- **Frame lifetime**: `lifetime = "frame"` resources are recycled automatically.
- **Minimal validation on hot path**: Validate only when the graph changes.


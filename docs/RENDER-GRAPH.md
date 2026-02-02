# Render Graph (Host-Defined)

This document describes the host-facing render graph format. The host builds a graph using logical IDs; the core validates, maps, and executes it. If the graph is missing or invalid, the core executes a safe fallback graph.

The core infers any missing resources from node inputs/outputs using default values (texture + frame lifetime).

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
| graphId    | LogicalId    | Logical graph identifier (cache key) |
| nodes      | Node[]       | Render nodes |
| edges      | Edge[]       | Dependencies between nodes |
| resources  | Resource[]   | Declared resources |
| fallback   | bool         | If true, use fallback when invalid or missing |

### Node

| Field     | Type        | Description |
|----------|-------------|-------------|
| nodeId   | LogicalId    | Logical node identifier |
| passId   | LogicalId    | Logical pass type (e.g., "forward") |
| inputs   | LogicalId[]  | Resource IDs read by this node |
| outputs  | LogicalId[]  | Resource IDs written by this node |
| params   | Map          | Optional parameters (clear, flags, etc.) |

### Resource

| Field       | Type      | Description |
|------------|-----------|-------------|
| resId      | LogicalId  | Logical resource identifier |
| kind       | string     | "texture", "buffer", "attachment" (defaults to "texture") |
| lifetime   | string     | "frame" or "persistent" (defaults to "frame") |
| aliasGroup | LogicalId? | Optional alias group for memory reuse |

### Edge

| Field        | Type     | Description |
|-------------|----------|-------------|
| fromNodeId | LogicalId| Dependency source |
| toNodeId   | LogicalId| Dependency target |
| reason      | string?  | Optional: "read_after_write", "write_after_read" |

### LogicalId

Logical IDs can be strings or numeric values. The core maps them to internal IDs once per `graphId` and caches the result to avoid per-frame cost.

## Known Pass IDs

- `shadow`
- `light-cull`
- `skybox`
- `forward`
- `outline`
- `post`
- `compose`

## Minimal Example

```json
{
  "graphId": "main_render",
  "nodes": [
    { "nodeId": "shadow_pass", "passId": "shadow", "inputs": [], "outputs": ["shadow_atlas"] },
    { "nodeId": "forward_pass", "passId": "forward", "inputs": ["shadow_atlas"], "outputs": ["hdr_color", "depth"] },
    { "nodeId": "outline_pass", "passId": "outline", "inputs": ["depth"], "outputs": ["outline_color"] },
    { "nodeId": "post_pass", "passId": "post", "inputs": ["hdr_color", "outline_color"], "outputs": ["post_color"] },
    { "nodeId": "compose_pass", "passId": "compose", "inputs": ["post_color"], "outputs": ["swapchain"] }
  ],
  "edges": [
    { "fromNodeId": "shadow_pass", "toNodeId": "forward_pass" },
    { "fromNodeId": "forward_pass", "toNodeId": "outline_pass" },
    { "fromNodeId": "outline_pass", "toNodeId": "post_pass" },
    { "fromNodeId": "post_pass", "toNodeId": "compose_pass" }
  ],
  "resources": [
    { "resId": "shadow_atlas" },
    { "resId": "hdr_color" },
    { "resId": "depth" },
    { "resId": "outline_color" },
    { "resId": "post_color" },
    { "resId": "swapchain", "kind": "attachment" }
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
shadow -> forward -> outline -> post -> compose
```

## Performance Notes

- **Cache per graphId**: Compile once, reuse execution plan and resource layout.
- **Alias groups**: Allow the core to reuse memory for non-overlapping resources.
- **Frame lifetime**: `lifetime = "frame"` resources are recycled automatically.
- **Minimal validation on hot path**: Validate only when the graph changes.

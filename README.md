# ast-visitor

**Arena-backed AST storage with the Visitor pattern for traversal, analysis, and transformation of syntax trees.**

The Visitor pattern decouples algorithms (type checking, constant folding, code generation) from the data structures they operate on (AST nodes). Instead of embedding behavior in each node type via virtual methods, a single `Visitor` trait exposes `visit_*` methods for each node kind, and a free `walk()` function dispatches based on `NodeData` discriminants. This crate stores all nodes in a flat `Vec<NodeData>` arena indexed by `NodeId(usize)`, following the design used by `rustc`'s `TyCtxt` and Roslyn's `SyntaxTree`.

## Why It Matters

The Visitor pattern is the backbone of compiler engineering. Every compiler pass — name resolution, type inference, optimization, linting, pretty-printing, code generation — is implemented as a visitor that walks the AST. Separating traversal from data has critical advantages:

- **Open/closed principle**: Add new passes without modifying node definitions
- **No lifetime gymnastics**: Arena allocation means `NodeId` is `Copy`, `Clone`, `Eq`, `Hash` — no `&'a Node` references threaded through every signature
- **Cache locality**: Nodes stored contiguously in `Vec<NodeData>` are prefetcher-friendly during depth-first traversal
- **Immutable sharing**: Multiple passes can share the same `&Ast` without synchronization

The arena design also enables **interning**: identical subtrees can be deduplicated by hashing `NodeId` sequences, and structural equality reduces to index comparison.

## How It Works

### Arena Allocation

```rust
pub struct Ast { nodes: Vec<NodeData> }
```

Nodes are never moved after allocation. `alloc(data)` appends to the vector and returns `NodeId(self.nodes.len())`. This is a **monotonic arena** — nodes are added but never removed during a compilation phase. `get(id)` returns `&NodeData` by index lookup in O(1).

This mirrors the approach in `rustc` where `TyCtxt<'tcx>` owns all HIR/MIR nodes and references are stable indices, not pointers. The benefit: no `Rc<RefCell<>>` overhead, no `unsafe`, no lifetime annotations on every struct.

### Node Types

```
NodeData
├── Literal(LiteralKind, String)    // Int, Float, String, Bool constants
├── Ident(String)                    // Variable / function references
├── BinaryOp(op, NodeId, NodeId)    // Left ⊕ Right
├── UnaryOp(op, NodeId)             // Prefix operator
├── Block(Vec<NodeId>)              // Statement sequence
├── FuncDef { name, params, body }  // Function declaration
└── Call { callee, args }           // Function application
```

### Walk Dispatch

The `walk()` free function matches on `ast.get(id)` and calls the appropriate `visit_*` method:

```rust
fn walk<V: Visitor>(v: &mut V, ast: &Ast, id: NodeId) -> V::Result {
    match ast.get(id) {
        NodeData::Literal(kind, val) => v.visit_literal(ast, *kind, val),
        NodeData::BinaryOp(op, lhs, rhs) => v.visit_binary(ast, op, *lhs, *rhs),
        // ...
    }
}
```

Each `visit_*` method receives `&Ast` so it can recursively descend into children via `self.visit(ast, child_id)`. This gives the visitor full control over traversal order (pre-order, post-order, or custom).

### PrintVisitor

The built-in `PrintVisitor` demonstrates the pattern with depth-tracked indentation:

```
binary(+)
  Int(42)
  Int(99)
```

### Complexity

| Operation | Time | Notes |
|-----------|------|-------|
| `alloc()` | O(1) amortized | Vec push |
| `get(id)` | O(1) | Direct index |
| Full traversal | O(V + E) | V = nodes, E = edges |
| `PrintVisitor` output | O(V) | One line per node |

Space: O(V) for the arena, O(1) per `NodeId`.

## Quick Start

```rust
use ast_visitor::{Ast, NodeData, LiteralKind, Visitor, PrintVisitor};

// Build: 1 + 2
let mut ast = Ast::new();
let lhs = ast.alloc(NodeData::Literal(LiteralKind::Int, "1".into()));
let rhs = ast.alloc(NodeData::Literal(LiteralKind::Int, "2".into()));
let add = ast.alloc(NodeData::BinaryOp("+".into(), lhs, rhs));

// Traverse
let mut printer = PrintVisitor::new();
printer.visit(&ast, add);
print!("{}", printer.output);
// binary(+)
//   Int(1)
//   Int(2)
```

### Custom Visitor

```rust
use ast_visitor::{Ast, NodeId, Visitor, NodeData};

struct ConstantFinder { count: usize }

impl Visitor for ConstantFinder {
    type Result = ();
    fn visit_literal(&mut self, _ast: &Ast, _kind: LiteralKind, _value: &str) {
        self.count += 1;
    }
    // Inherit default walk() for all other node types
}
```

## API

- **`Ast`** — Arena store: `new()`, `alloc(data) → NodeId`, `get(id) → &NodeData`, `get_mut(id) → &mut NodeData`
- **`NodeId(usize)`** — `Copy`, `Clone`, `Debug`, `Eq`, `Hash` — opaque node handle
- **`NodeData`** — 7-variant enum of all node kinds
- **`Visitor` trait** — Override `visit_literal`, `visit_ident`, `visit_binary`, `visit_unary`, `visit_block`, `visit_func`, `visit_call`
- **`walk()`** — Free dispatch function
- **`PrintVisitor`** — Ready-made pretty-printer with indentation
- **`LiteralKind`** — Int, Float, String, Bool

## Architecture Notes

This crate is the **traversal layer** (η — evaluative depth) of the SuperInstance compiler infrastructure. It pairs with `ast-builder` (γ — generative construction). The γ+η=C identity: the builder determines what AST shapes are constructible (γ), the visitor determines what analyses are expressible (η), and their composition C is the compiler's total capability on the AST IR.

## References

1. Gamma, E. et al. (1994). *Design Patterns: Elements of Reusable Object-Oriented Software*. — Original Visitor pattern formulation.
2. Matsakis, N. & Klock, F. (2014). "The Rust Language." *ACM SIGAda Ada Letters*. — Arena patterns in Rust.
3. Rust Compiler Team. `rustc_hir::HirId` and `TyCtxt`. — Production arena-backed AST design.
4. Roslyn Team (MSR). "The Roslyn Project" architecture documents. — C# compiler's pooled syntax tree design.

## License

MIT

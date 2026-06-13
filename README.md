# AST Visitor

**A Rust library implementing the Visitor pattern for traversing and transforming Abstract Syntax Trees.** Provides an arena-backed AST representation, a `Visitor` trait with walk dispatch, and a ready-made `PrintVisitor` for pretty-printing.

## Why It Matters

The Visitor pattern is the workhorse of compiler engineering. It lets you separate algorithms (type checking, optimization, code generation) from the data structures they operate on (AST nodes). Instead of adding a method to every node type, you define a visitor with methods for each node kind — the `walk` function dispatches to the right method based on node data.

This is essential for:

- **Type checkers** — traverse the AST validating types
- **Constant folders** — rewrite `2 + 3` to `5` during traversal
- **Pretty printers** — produce readable source from internal ASTs
- **Linters** — flag patterns as they're visited
- **Code generators** — emit bytecode/IR from AST nodes

## How It Works

**Arena allocation**: Nodes are stored in a flat `Vec<NodeData>` inside the `Ast` struct. Node references are `NodeId(usize)` indices into this vector, not pointers. This is the same technique used by the Rust compiler (`rustc`'s `TyCtxt`) and roslyn (C#'s compiler). Benefits: cache-friendly traversal, no lifetime gymnastics, easy sharing between threads.

**Node types** (`NodeData` enum):
- `Literal(Int|Float|String|Bool, value)` — Constant values
- `Ident(name)` — Variable references
- `BinaryOp(op, lhs, rhs)` — Binary operations with two child nodes
- `UnaryOp(op, expr)` — Unary operations (negation, not)
- `Block(stmts)` — Statement sequences
- `FuncDef(name, params, body)` — Function definitions
- `Call(callee, args)` — Function calls

**Walk dispatch**: The `walk()` function matches on `NodeData` and calls the appropriate `visit_*` method. Visitors override only the methods they care about. `PrintVisitor` demonstrates the pattern — it indents based on depth and formats each node type differently.

## Quick Start

```rust
use ast_visitor::{Ast, NodeData, LiteralKind, Visitor, PrintVisitor};

let mut ast = Ast::new();
let lhs = ast.alloc(NodeData::Literal(LiteralKind::Int, "42".into()));
let rhs = ast.alloc(NodeData::Literal(LiteralKind::Int, "99".into()));
let add = ast.alloc(NodeData::BinaryOp("+".into(), lhs, rhs));

let mut printer = PrintVisitor::new();
printer.visit(&ast, add);

print!("{}", printer.output);
// Output:
//   binary(+)
//     Int(42)
//     Int(99)
```

## API

- **`Ast`** — Arena-backed node store: `alloc()`, `get()`, `get_mut()`
- **`NodeId(usize)`** — Opaque node handle
- **`NodeData`** — Enum of all node kinds (literals, idents, binary/unary ops, blocks, funcs, calls)
- **`Visitor` trait** — Override `visit_literal`, `visit_binary`, `visit_func`, etc.
- **`walk()`** — Free function that dispatches to the right visitor method
- **`PrintVisitor`** — Built-in pretty-printer with indentation

## Architecture Notes

This is the traversal layer of the SuperInstance compiler infrastructure. It pairs with `ast-builder` (construction) and `ast-diff` (comparison). The arena design enables efficient batch processing of ASTs during compilation passes. See the [architecture overview](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## License

MIT

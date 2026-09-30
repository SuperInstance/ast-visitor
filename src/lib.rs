//! AST visitor pattern for tree traversal and transformation.


/// A typed AST node identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

/// Node data stored in the AST.
#[derive(Debug, Clone)]
pub enum NodeData {
    Literal(LiteralKind, String),
    Ident(String),
    BinaryOp(String, NodeId, NodeId),
    UnaryOp(String, NodeId),
    Block(Vec<NodeId>),
    FuncDef {
        name: String,
        params: Vec<String>,
        body: NodeId,
    },
    Call {
        callee: NodeId,
        args: Vec<NodeId>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteralKind {
    Int,
    Float,
    String,
    Bool,
}

/// A simple AST storage arena.
pub struct Ast {
    nodes: Vec<NodeData>,
}

impl Ast {
    pub fn new() -> Self {
        Ast { nodes: Vec::new() }
    }

    /// Allocate a node and return its id.
    pub fn alloc(&mut self, data: NodeData) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(data);
        id
    }

    /// Get a reference to node data.
    pub fn get(&self, id: NodeId) -> &NodeData {
        &self.nodes[id.0]
    }

    /// Get a mutable reference to node data.
    pub fn get_mut(&mut self, id: NodeId) -> &mut NodeData {
        &mut self.nodes[id.0]
    }
}

/// Trait for AST visitors. Override methods as needed.
pub trait Visitor {
    type Result;
    fn visit(&mut self, ast: &Ast, id: NodeId) -> Self::Result where Self: Sized {
        walk(self, ast, id)
    }
    fn visit_literal(&mut self, _ast: &Ast, _kind: LiteralKind, _value: &str) -> Self::Result;
    fn visit_ident(&mut self, _ast: &Ast, _name: &str) -> Self::Result;
    fn visit_binary(&mut self, ast: &Ast, op: &str, lhs: NodeId, rhs: NodeId) -> Self::Result;
    fn visit_unary(&mut self, ast: &Ast, op: &str, expr: NodeId) -> Self::Result;
    fn visit_block(&mut self, ast: &Ast, stmts: &[NodeId]) -> Self::Result;
    fn visit_func(&mut self, ast: &Ast, name: &str, params: &[String], body: NodeId) -> Self::Result;
    fn visit_call(&mut self, ast: &Ast, callee: NodeId, args: &[NodeId]) -> Self::Result;
}

/// Default walk that dispatches to the appropriate visit_* method.
pub fn walk<V: Visitor>(v: &mut V, ast: &Ast, id: NodeId) -> V::Result {
    match ast.get(id) {
        NodeData::Literal(kind, val) => v.visit_literal(ast, *kind, val),
        NodeData::Ident(name) => v.visit_ident(ast, name),
        NodeData::BinaryOp(op, lhs, rhs) => v.visit_binary(ast, op, *lhs, *rhs),
        NodeData::UnaryOp(op, expr) => v.visit_unary(ast, op, *expr),
        NodeData::Block(stmts) => v.visit_block(ast, stmts),
        NodeData::FuncDef { name, params, body } => v.visit_func(ast, name, params, *body),
        NodeData::Call { callee, args } => v.visit_call(ast, *callee, args),
    }
}

/// A visitor that pretty-prints the AST.
pub struct PrintVisitor {
    pub output: String,
    depth: usize,
}

impl PrintVisitor {
    pub fn new() -> Self {
        PrintVisitor { output: String::new(), depth: 0 }
    }
    fn indent(&self) -> String {
        "  ".repeat(self.depth)
    }
}

impl Visitor for PrintVisitor {
    type Result = ();
    fn visit_literal(&mut self, _ast: &Ast, kind: LiteralKind, value: &str) {
        self.output.push_str(&format!("{}{:?}({})\n", self.indent(), kind, value));
    }
    fn visit_ident(&mut self, _ast: &Ast, name: &str) {
        self.output.push_str(&format!("{}ident({})\n", self.indent(), name));
    }
    fn visit_binary(&mut self, ast: &Ast, op: &str, lhs: NodeId, rhs: NodeId) {
        self.output.push_str(&format!("{}binary({})\n", self.indent(), op));
        self.depth += 1;
        self.visit(ast, lhs);
        self.visit(ast, rhs);
        self.depth -= 1;
    }
    fn visit_unary(&mut self, ast: &Ast, op: &str, expr: NodeId) {
        self.output.push_str(&format!("{}unary({})\n", self.indent(), op));
        self.depth += 1;
        self.visit(ast, expr);
        self.depth -= 1;
    }
    fn visit_block(&mut self, ast: &Ast, stmts: &[NodeId]) {
        self.output.push_str(&format!("{}block\n", self.indent()));
        self.depth += 1;
        for &s in stmts { self.visit(ast, s); }
        self.depth -= 1;
    }
    fn visit_func(&mut self, ast: &Ast, name: &str, params: &[String], body: NodeId) {
        self.output.push_str(&format!("{}func({}) params={:?}\n", self.indent(), name, params));
        self.depth += 1;
        self.visit(ast, body);
        self.depth -= 1;
    }
    fn visit_call(&mut self, ast: &Ast, callee: NodeId, args: &[NodeId]) {
        self.output.push_str(&format!("{}call\n", self.indent()));
        self.depth += 1;
        self.visit(ast, callee);
        for &a in args { self.visit(ast, a); }
        self.depth -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_and_print() {
        let mut ast = Ast::new();
        let lhs = ast.alloc(NodeData::Literal(LiteralKind::Int, "1".into()));
        let rhs = ast.alloc(NodeData::Literal(LiteralKind::Int, "2".into()));
        let add = ast.alloc(NodeData::BinaryOp("+".into(), lhs, rhs));
        let mut pv = PrintVisitor::new();
        pv.visit(&ast, add);
        assert!(pv.output.contains("binary(+)"));
    }
}

/// FNV-1a 64 — the digest every substrate in the SuperInstance fleet agrees on.
pub const FNV_OFFSET: u64 = 0xcbf29ce484222325;
pub const FNV_PRIME: u64 = 0x100000001b3;

#[inline]
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h = FNV_OFFSET;
    for &b in bytes {
        h = (h ^ b as u64).wrapping_mul(FNV_PRIME);
    }
    h
}

/// True if this crate's FNV-1a still agrees with the rest of the fleet.
pub fn canary_holds() -> bool {
    fnv1a64("café Δ 日本語".as_bytes()) == 0x024a555471370b18d
}

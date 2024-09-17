#[derive(Debug)]
pub struct Ast {
    stmts: Vec<Node>
}

impl Ast {
    pub fn new() -> Self {
        Self {
            stmts: vec![]
        }
    }

    pub fn push_stmt(&mut self, stmt: Node) {
        self.stmts.push(stmt);
    }
}

#[derive(Debug)]
pub struct Node {
    
}

use lexer::SrcPos;

use std::fmt;
use std::fmt::Display;

/// Values that originated in source code, IR trees and suchlike.
trait Sourced {
    /// Return the source position.
    fn pos(&self) -> SrcPos;
}

/// A type for variable names.
pub type Name = String;

/// Abstract Syntax Tree
#[derive(Debug)]
pub enum AST {
    Block(Block),
    Fn { pos: SrcPos, clauses: Vec<Clause> },
    App { pos: SrcPos, op: Box<AST>, args: Vec<AST> },

    Var { pos: SrcPos, name: String },
    Const { pos: SrcPos, val: Const }
}

impl AST {
    pub fn new_if(pos: SrcPos, cond: AST, then: AST, els: AST) -> AST {
        use self::AST::*;

        App {
            pos: pos,
            op: Box::new(Fn {
                pos: pos,
                clauses: vec![
                    Clause {
                        params: String::from("_"), // HACK
                        cond: Var { pos: pos, name: String::from("_") }, // HACK
                        body: self::Block {
                            pos: then.pos(),
                            stmts: vec![Stmt::Expr(then)]
                        }
                    },
                    Clause {
                        params: String::from("_"), // HACK
                        cond: Const { pos: pos, val: self::Const::Bool(true) },
                        body: self::Block {
                            pos: els.pos(),
                            stmts: vec![Stmt::Expr(els)]
                        }
                    }
                ]
            }),
            args: vec![cond]
        }
    }
}

impl Sourced for AST {
    fn pos(&self) -> SrcPos {
        match self {
            &AST::Block(ref block) => block.pos(),
            &AST::Fn { pos, .. } => pos,
            &AST::App { pos, .. } => pos,
            &AST::Var { pos, .. } => pos,
            &AST::Const { pos, .. } => pos
        }
    }
}

impl Display for AST {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &AST::Block(ref block) => block.fmt(f),
            &AST::Fn { ref clauses, .. } => {
                try!(write!(f, "{{"));
                let mut it = clauses.iter();
                if let Some(arg) = it.next() {
                    try!(write!(f, "{}", arg));
                }
                for arg in it {
                    try!(write!(f, "; {}", arg));
                }
                write!(f, "}}")
            },
            &AST::App { ref op, ref args, .. } => {
                try!(write!(f, "({} ", op));
                let mut it = args.iter();
                if let Some(arg) = it.next() {
                    try!(write!(f, "{}", arg));
                }
                for arg in it {
                    try!(write!(f, " {}", arg));
                }
                write!(f, ")")
            },
            &AST::Var { ref name, .. } => write!(f, "{}", name),
            &AST::Const { ref val, .. } => write!(f, "{}", val)
        }
    }
}

/// A block.
#[derive(Debug)]
pub struct Block {
    pub pos: SrcPos,
    //decls: Vec<Name>
    pub stmts: Vec<Stmt>
}

impl Sourced for Block {
    fn pos(&self) -> SrcPos {
        self.pos
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(write!(f, "{{"));
        let mut it = self.stmts.iter();
        if let Some(arg) = it.next() {
            try!(write!(f, "{}", arg));
        }
        for arg in it {
            try!(write!(f, "; {}", arg));
        }
        write!(f, "}}")
    }
}

/// Statement (for `Block`s).
#[derive(Debug)]
pub enum Stmt {
    Def { name: String, val: AST },
    Expr(AST)
}

impl Sourced for Stmt {
    fn pos(&self) -> SrcPos {
        match self {
            &Stmt::Def { ref val, .. } => val.pos(),
            &Stmt::Expr(ref expr) => expr.pos()
        }
    }
}

impl Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &Stmt::Def { ref name, ref val, ..} => write!(f, "{} = {}", name, val),
            &Stmt::Expr(ref expr) => write!(f, "{}", expr)
        }
    }
}

/// Function clause.
#[derive(Debug)]
pub struct Clause {
    pub params: Name, // TODO: Vec<AST>
    pub cond: AST,
    pub body: Block
}

impl Clause {
    fn push(&mut self, stmt: Stmt) {
        self.body.stmts.push(stmt);
    }
}

impl Sourced for Clause {
    fn pos(&self) -> SrcPos { self.body.pos() }
}

impl Display for Clause {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(write!(f, "{} | {} => ", self.params, self.cond));
        let mut it = self.body.stmts.iter();
        if let Some(stmt) = it.next() {
            try!(write!(f, "{}", stmt));
        }
        for stmt in it {
            try!(write!(f, "; {}", stmt));
        }
        Ok(())
    }
}

/// Source constants.
#[derive(Debug)]
pub enum Const {
    Int(isize),
    Float(f64),
    Char(char),
    String(String),
    Bool(bool)
}

impl Display for Const {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::Const::*;

        match self {
            &Int(i) => write!(f, "{}", i),
            &Float(n) => write!(f, "{}", n),
            &Char(c) => write!(f, "{:?}", c),
            &String(ref cs) => write!(f, "\"{}\"", cs),
            &Bool(true) => write!(f, "True"),
            &Bool(false) => write!(f, "False")
        }
    }
}

/// A block item. This only exists for `parse_block`.
#[derive(Debug)]
pub enum BlockItem {
    Stmt(Stmt),
    Clause(Clause)
}

impl BlockItem {
    fn stmt(self) -> Option<Stmt> {
        match self {
            BlockItem::Stmt(stmt) => Some(stmt),
            BlockItem::Clause(_) => None
        }
    }
}

// TODO: Option -> Result for better error reporting
/// Try to convert a sequence of `BlockItem`s into either an `AST::Block` or an `AST::Fn`.
pub fn parse_block(pos: SrcPos, items: Vec<BlockItem>) -> Option<AST> {
    fn parse_stmt_block(pos: SrcPos, items: Vec<BlockItem>) -> Option<AST> {
        let mut stmts = Vec::new();

        for item in items.into_iter() {
            match item {
                BlockItem::Clause(_) => return None,
                BlockItem::Stmt(stmt) => stmts.push(stmt)
            }
        }

        Some(AST::Block(Block {
            pos: pos,
            stmts: stmts
        }))
    }

    fn parse_fn_block(pos: SrcPos, items: Vec<BlockItem>) -> Option<AST> {
        let mut it = items.into_iter().peekable();
        let mut clauses = Vec::new();

        loop {
            match it.next() {
                Some(BlockItem::Clause(mut clause)) => {
                    while let Some(&BlockItem::Stmt(_)) = it.peek() {
                        clause.push(it.next().unwrap().stmt().unwrap());
                    }
                    clauses.push(clause);
                },
                Some(BlockItem::Stmt(_)) => return None,
                None => return Some(AST::Fn { pos: pos, clauses: clauses})
            }
        }
    }

    match items.first() {
        Some(&BlockItem::Clause(_)) => parse_fn_block(pos, items),
        Some(&BlockItem::Stmt(_)) => parse_stmt_block(pos, items),
        None => Some(AST::Block(Block { pos: pos, stmts: vec![] }))
    }
}

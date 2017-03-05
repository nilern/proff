use std::collections::HashMap;
use std::rc::Rc;

use util::{Name, IndexSrc};
use ast;
use ast::{AST, App, Var, VarRef, Const, Block, Stmt, Clause, CtxMapping};

// FIXME: Remove this since it is actually unused.
/// An error to signal when resolution fails.
#[derive(Debug)]
pub struct ResolveError;

struct Env {
    bindings: HashMap<Name, Name>,
    parent: Option<Rc<Env>>
}

impl Env {
    fn new(parent: Option<Rc<Env>>, bindings: HashMap<Name, Name>) -> Env {
        Env {
            bindings: bindings,
            parent: parent
        }
    }

    fn resolve(&self, name: &Name) -> VarRef {
        self.bindings.get(name)
                     .map(|name| VarRef::Local(name.clone()))
                     .or_else(||
                         self.parent.clone()
                                    .and_then(|parent| parent.resolve_str(name))
                                    .map(|name| VarRef::Clover(name.clone())))
                     .unwrap_or(VarRef::Global(name.clone()))
    }

    fn resolve_str(&self, name: &Name) -> Option<Name> {
        self.bindings.get(name)
                     .map(Clone::clone)
                     .or_else(||
                         self.parent.clone().and_then(|parent| parent.resolve_str(name)))
    }
}

struct Resolve {
    counter: IndexSrc
}

impl Resolve {
    fn new(counter: IndexSrc) -> Resolve {
        Resolve { counter: counter }
    }

    fn rename(&mut self, name: &Name) -> Name {
        name.as_unique(&mut self.counter)
    }

    fn block_bindings<'a, I>(&mut self, bindings: &mut HashMap<Name, Name>, stmts: I)
        where I: Iterator<Item=&'a Stmt>
    {
        for stmt in stmts {
            match stmt {
                &Stmt::Def { ref name, .. } => {
                    bindings.insert(name.clone(), self.rename(name));
                },
                &Stmt::Expr(..) => ()
            }
        }
    }

    fn param_bindings<'a>(&mut self, bindings: &mut HashMap<Name, Name>, params: &Name) {
        bindings.insert(params.clone(), self.rename(params));
    }
}

impl CtxMapping for Resolve {
    type Ctx = Option<Rc<Env>>;
    type ASTRes = Result<AST, ResolveError>;
    type StmtRes = Result<Stmt, ResolveError>;
    type ClauseRes = Result<Clause, ResolveError>;

    fn map_block(&mut self, node: Block, env: Option<Rc<Env>>) -> Result<AST, ResolveError> {
        let mut bindings = HashMap::new();
        self.block_bindings(&mut bindings, node.stmts.iter());
        let env = Some(Rc::new(Env::new(env, bindings)));
        Ok(AST::Block(Block {
            pos: node.pos,
            stmts: node.stmts.into_iter()
                             .map(|stmt| self.map_stmt(stmt, env.clone()))
                             .collect::<Result<Vec<Stmt>, ResolveError>>()?
        }))
    }

    fn map_fn(&mut self, node: ast::Fn, env: Option<Rc<Env>>) -> Result<AST, ResolveError> {
        Ok(AST::Fn(ast::Fn {
            pos: node.pos,
            clauses: node.clauses.into_iter()
                                 .map(|clause| self.map_clause(clause, env.clone()))
                                 .collect::<Result<Vec<Clause>, ResolveError>>()?
        }))
    }

    fn map_app(&mut self, node: App, env: Option<Rc<Env>>) -> Result<AST, ResolveError> {
        Ok(AST::App(App {
            pos: node.pos,
            op: Box::new(node.op.accept_ctx(self, env.clone())?),
            args: node.args.into_iter()
                           .map(|arg| arg.accept_ctx(self, env.clone()))
                           .collect::<Result<Vec<AST>, ResolveError>>()?
        }))
    }

    fn map_var(&mut self, node: Var, env: Option<Rc<Env>>) -> Result<AST, ResolveError> {
        Ok(env.map(|env| AST::Var(Var { pos: node.pos, name: env.resolve(&node.name()) }))
              .unwrap_or(AST::Var(node)))
    }

    fn map_const(&mut self, c: Const, _: Option<Rc<Env>>) -> Result<AST, ResolveError> {
        Ok(AST::Const(c))
    }

    fn map_stmt(&mut self, node: Stmt, env: Option<Rc<Env>>) -> Result<Stmt, ResolveError> {
        match node {
            Stmt::Def { name, val } => {
                Ok(Stmt::Def {
                    name: env.clone().and_then(|env| env.resolve_str(&name)).unwrap_or(name),
                    val: val.accept_ctx(self, env.clone())?
                })
            },
            Stmt::Expr(e) => Ok(Stmt::Expr(e.accept_ctx(self, env)?))
         }
    }

    fn map_clause(&mut self, node: Clause, env: Option<Rc<Env>>) -> Result<Clause, ResolveError> {
        let mut param_bindings = HashMap::new();
        self.param_bindings(&mut param_bindings, &node.params);
        let param_env = Some(Rc::new(Env::new(env.clone(), param_bindings.clone())));

        let mut bindings = param_bindings;
        self.block_bindings(&mut bindings, node.body.iter());
        let env = Some(Rc::new(Env::new(env.clone(), bindings)));

        Ok(Clause {
            pos: node.pos,
            params: param_env.clone()
                             .and_then(|env| env.resolve_str(&node.params))
                             .unwrap(), // we just put it there with param_bindings
            cond: node.cond.accept_ctx(self, param_env)?,
            body: node.body.into_iter()
                           .map(|stmt| self.map_stmt(stmt, env.clone()))
                           .collect::<Result<Vec<Stmt>, ResolveError>>()?
        })
    }
}

impl AST {
    /// Resolve variables to be local, closed over or global and alphatize their names.
    pub fn resolve(self, counter: IndexSrc) -> Result<AST, ResolveError> {
        self.accept_ctx(&mut Resolve::new(counter), None)
    }
}

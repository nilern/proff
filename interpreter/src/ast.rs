use std::fmt::{self, Formatter};

use object_model::{HeapValueSub, DynHeapValueSub, DynamicDebug, HeapValue, DynHeapValue,
                   ValueRef, TypedValueRef};
use value::{TypeRegistry, TypeIndex, Symbol};

/// Function AST node
pub struct Function {
    pub base: DynHeapValue
}

impl Function {
    pub fn methods(&self) -> &[TypedValueRef<Method>] { self.tail() }
}

impl HeapValueSub for Function {
    const TYPE_INDEX: TypeIndex = TypeIndex::Function;

    const UNIFORM_REF_LEN: usize = 0;
}

impl DynHeapValueSub for Function {
    type TailItem = TypedValueRef<Method>;
}

impl DynamicDebug for Function {
    fn fmt<R: TypeRegistry>(&self, f: &mut Formatter, types: &R) -> Result<(), fmt::Error> {
        f.debug_struct("Function")
         .field("base", &self.base.fmt_wrap(types))
         .field("methods", &self.methods().fmt_wrap(types))
         .finish()
    }
}

/// Method AST (function sub)node
pub struct Method {
    pub base: HeapValue,
    pub pattern: ValueRef,
    pub guard: ValueRef,
    pub body: ValueRef
}

impl HeapValueSub for Method {
    const TYPE_INDEX: TypeIndex = TypeIndex::Method;

    const UNIFORM_REF_LEN: usize = 3;
}

impl DynamicDebug for Method {
    fn fmt<T: TypeRegistry>(&self, f: &mut Formatter, types: &T) -> Result<(), fmt::Error> {
        f.debug_struct("Method")
         .field("base", &self.base.fmt_wrap(types))
         .field("pattern", &self.pattern.fmt_wrap(types))
         .field("guard", &self.guard.fmt_wrap(types))
         .field("body", &self.body.fmt_wrap(types))
         .finish()
    }
}

/// Block AST node
pub struct Block {
    pub base: DynHeapValue,
    pub expr: ValueRef,
}

impl Block {
    pub fn stmts(&self) -> &[ValueRef] { self.tail() }
}

impl HeapValueSub for Block {
    const TYPE_INDEX: TypeIndex = TypeIndex::Block;

    const UNIFORM_REF_LEN: usize = 1;
}

impl DynHeapValueSub for Block {
    type TailItem = ValueRef;
}

impl DynamicDebug for Block {
    fn fmt<T: TypeRegistry>(&self, f: &mut Formatter, types: &T) -> Result<(), fmt::Error> {
        f.debug_struct("Block")
         .field("base", &self.base.fmt_wrap(types))
         .field("expr", &self.expr.fmt_wrap(types))
         .field("stmts", &self.stmts().fmt_wrap(types))
         .finish()
    }
}

/// Call AST node
pub struct Call {
    pub base: DynHeapValue,
    pub callee: ValueRef,
}

impl Call {
    pub fn args(&self) -> &[ValueRef] { self.tail() }
}

impl HeapValueSub for Call {
    const TYPE_INDEX: TypeIndex = TypeIndex::Call;

    const UNIFORM_REF_LEN: usize = 1;
}

impl DynHeapValueSub for Call {
    type TailItem = ValueRef;
}

impl DynamicDebug for Call {
    fn fmt<T: TypeRegistry>(&self, f: &mut Formatter, types: &T) -> Result<(), fmt::Error> {
        f.debug_struct("Call")
         .field("base", &self.base.fmt_wrap(types))
         .field("callee", &self.callee.fmt_wrap(types))
         .field("args", &self.args().fmt_wrap(types))
         .finish()
    }
}

/// An AST node for definitions.
#[repr(C)]
pub struct Def {
    pub base: HeapValue,
    pub pattern: ValueRef,
    pub expr: ValueRef
}

impl HeapValueSub for Def {
    const TYPE_INDEX: TypeIndex = TypeIndex::Def;

    const UNIFORM_REF_LEN: usize = 2;
}

impl DynamicDebug for Def {
    fn fmt<T: TypeRegistry>(&self, f: &mut Formatter, types: &T) -> Result<(), fmt::Error> {
        f.debug_struct("Def")
         .field("base", &self.base.fmt_wrap(types))
         .field("pattern", &self.pattern.fmt_wrap(types))
         .field("expr", &self.expr.fmt_wrap(types))
         .finish()
    }
}

/// An AST node for constants.
#[repr(C)]
pub struct Const {
    pub heap_value: HeapValue,
    /// The value of the constant
    pub value: ValueRef
}

impl HeapValueSub for Const {
    const TYPE_INDEX: TypeIndex = TypeIndex::Const;

    const UNIFORM_REF_LEN: usize = 1;
}

impl DynamicDebug for Const {
    fn fmt<T: TypeRegistry>(&self, f: &mut Formatter, types: &T) -> Result<(), fmt::Error> {
        f.debug_struct("Const")
         .field("heap_value", &self.heap_value.fmt_wrap(types))
         .field("value", &self.value.fmt_wrap(types))
         .finish()
    }
}

/// An AST node for lexical variable names
#[repr(C)]
pub struct Lex {
    pub base: HeapValue,
    pub name: TypedValueRef<Symbol>
}

impl HeapValueSub for Lex {
    const TYPE_INDEX: TypeIndex = TypeIndex::Lex;

    const UNIFORM_REF_LEN: usize = 1;
}

impl DynamicDebug for Lex {
    fn fmt<T: TypeRegistry>(&self, f: &mut Formatter, types: &T) -> Result<(), fmt::Error> {
        f.debug_struct("Lex")
         .field("base", &self.base.fmt_wrap(types))
         .field("name", &self.name.fmt_wrap(types))
         .finish()
    }
}

/*use std::convert::TryFrom;
use std::fmt::Debug;
use std::iter::Peekable;
use gc::{Gc, Trace, Finalize};

use value::Value;

#[derive(Debug)]
pub struct PatternError;

#[derive(Debug)]
pub struct ClauseError;

#[derive(Debug, Clone, Trace, Finalize)]
pub enum Pattern {
    Call(Call<Pattern>),
    Name(Name),
    Const(Gc<Value>)
}

#[derive(Debug, Clone, Trace, Finalize)]
pub enum Expr {
    Block(Block),
    Function(Gc<Function>),
    Call(Call<Expr>),
    PrimCall(PrimCall),

    NameRef(Name),
    Const(Gc<Value>)
}

#[derive(Debug, Clone, Trace, Finalize)]
pub enum Stmt {
    Def(Def),
    Aug(Aug),
    Expr(Expr)
}

#[derive(Debug, Clone, Trace, Finalize)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub expr: Gc<Expr>
}

#[derive(Debug, Clone, Trace, Finalize)]
pub struct Function(pub Vec<Method>);

#[derive(Debug, Clone, Trace, Finalize)]
pub struct Method {
    pub pattern: Pattern,
    pub guard: Expr,
    pub body: Expr
}

#[derive(Debug, Clone, Trace, Finalize)]
pub struct Call<T> where T: Debug + Trace + Finalize {
    pub callee: Gc<Expr>,
    pub args: Vec<T>
}

#[derive(Debug, Clone, Trace, Finalize)]
pub struct PrimCall {
    pub op: Primop,
    pub args: Vec<Expr>
}

#[derive(Debug, Clone, Trace, Finalize)]
pub enum Name {
    Lex(String),
    Dyn(String)
}

#[derive(Debug, Clone, Trace, Finalize)]
pub struct Def {
    pub pattern: Pattern,
    pub guard: Expr,
    pub value: Expr
}

#[derive(Debug, Clone, Trace, Finalize)]
pub struct Aug {
    pub pattern: Pattern,
    pub guard: Expr,
    pub value: Expr
}

#[derive(Debug, Clone, Trace, Finalize)]
pub enum Primop {
    IntAdd, IntSub, IntMul, IntDiv, IntRem
}

// -------------------------------------------------------------------------------------------------

impl<'a> TryFrom<&'a Expr> for Pattern {
    type Error = PatternError;

    fn try_from(expr: &Expr) -> Result<Pattern, PatternError> {
        match expr {
            &Expr::Call(Call { ref callee, ref args }) => Ok(Pattern::Call(Call {
                callee: callee.clone(),
                args: args.iter().map(Pattern::try_from).collect::<Result<Vec<_>, _>>()?
            })),
            &Expr::NameRef(ref name) => Ok(Pattern::Name(name.clone())),
            &Expr::Const(ref c) => Ok(Pattern::Const(c.clone())),
            &Expr::Block(_) | &Expr::Function(_) | &Expr::PrimCall(_) => Err(PatternError)
        }
    }
}

// -------------------------------------------------------------------------------------------------

pub enum Clause {
    Method {
        pattern: Pattern,
        guard: Expr,
        body: Stmt
    },
    Stmt(Stmt)
}

pub fn analyze_clauses(clauses: Vec<Clause>) -> Result<Expr, ClauseError> {
    fn cparse_body<I>(mut stmts: Vec<Stmt>, clauses: &mut Peekable<I>) -> Result<Block, ClauseError>
        where I: Iterator<Item=Clause>
    {
        while let Some(&Clause::Stmt(_)) = clauses.peek() {
            if let Some(Clause::Stmt(ref stmt)) = clauses.next() {
                stmts.push(stmt.clone());
            } else {
                unreachable!()
            }
        }

        if let Some(Stmt::Expr(ref expr)) = stmts.pop() {
            Ok(Block { stmts, expr: Gc::new(expr.clone()) })
        } else {
            Err(ClauseError)
        }
    }

    fn cparse_function<I>(clauses: &mut Peekable<I>) -> Result<Function, ClauseError>
        where I: Iterator<Item=Clause>
    {
        let mut methods = Vec::new();

        loop {
            let (pattern, guard, stmt) = match clauses.next() {
                Some(Clause::Method { pattern, guard, body }) => (pattern, guard, body),
                Some(_) => return Err(ClauseError),
                None => break
            };
            let body = cparse_body(vec![stmt], clauses)?;
            methods.push(Method { pattern, guard, body: Expr::Block(body) });
        }

        if methods.len() > 0 {
            Ok(Function(methods))
        } else {
            Err(ClauseError)
        }
    }

    let mut clauses = clauses.into_iter().peekable();
    match clauses.peek() {
        Some(&Clause::Method { .. }) =>
            cparse_function(&mut clauses).map(|f| Expr::Function(Gc::new(f))),
        Some(&Clause::Stmt(_)) => {
            let body = cparse_body(Vec::new(), &mut clauses)?;
            if clauses.peek().is_none() {
                Ok(Expr::Block(body))
            } else {
                Err(ClauseError)
            }
        },
        None => Err(ClauseError)
    }
} */

use crate::{token::Token, value::Val};

#[derive(Debug)]
pub struct Ast {
    stmts: Vec<Box<dyn Stmt>>
}

impl Ast {
    pub fn new() -> Self {
        Self {
            stmts: vec![]
        }
    }

    pub fn push_stmt(&mut self, stmt: Box<dyn Stmt>) {
        self.stmts.push(stmt);
    }

    pub fn stmts(&self) -> &[Box<dyn Stmt>] {
        &self.stmts
    }
}

macro_rules! create_structs {
    (
        impl $trait:ident for $( 
            $name:ident ( $( 
                $tp:ty 
            ),* ) 
        ),* 
    ) => {
        $(
            #[derive(Debug, Clone)]
            pub struct $name($( pub $tp ),* );

            impl $trait for $name {
                fn as_any(&self) -> &dyn Any {
                    self
                }
            }
        )*
    };
}

pub mod stmt {
    use std::any::Any;
    use std::fmt::Debug;

    use super::expr::Expr;

    pub trait Stmt : Any + Debug {
        fn as_any(&self) -> &dyn Any;
    }
    
    impl dyn Stmt {
        pub fn downcast_ref<T: Stmt>(&self) -> Option<&T> {
            self.as_any().downcast_ref::<T>()
        }

    }

    create_structs!(
        impl Stmt for
            ExprStmt(Box<dyn Expr>, bool) // bool is whether or not to log the resultant value.
    );
}

pub use stmt::Stmt;

pub mod expr {
    use std::any::Any;
    use std::fmt;
    use std::fmt::Debug;

    use super::Val;
    use super::Token;

    pub trait Expr : Any + Debug + CloneExpr {
        fn as_any(&self) -> &dyn Any;
    }
    
    impl dyn Expr {
        pub fn downcast_ref<T: Expr>(&self) -> Option<&T> {
            self.as_any().downcast_ref::<T>()
        }
    }

    impl fmt::Display for dyn Expr {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if let Some(Literal(val)) = self.downcast_ref() {
                write!(f, "{}", val)
            } else if let Some(Symbol(name)) = self.downcast_ref() {
                write!(f, "{}", name)
            } else if let Some(Group(expr)) = self.downcast_ref() {
                write!(f, "({})", expr)
            } else if let Some(Unary(op, expr)) = self.downcast_ref() {
                write!(f, "{}{}", op.lexeme(), expr)
            } else if let Some(Binary(left, op, right)) = self.downcast_ref() {
                write!(f, "{} {} {}", left, op.lexeme(), right)
            } else if let Some(Call(caller, args)) = self.downcast_ref() {
                write!(f, "{}(", caller)?;

                for (i, arg) in args.iter().enumerate() {
                    if let Some(actual) = arg {
                        write!(f, "{actual}")?
                    } else {
                        write!(f, "")?
                    }

                    if i < args.len() - 1 {
                        write!(f, ", ")?
                    }
                }

                write!(f, ")")
            } else if let Some(Func(args, expr)) = self.downcast_ref() {
                if args.len() == 1 {
                    write!(f, "{} -> ", args[0].0)
                } else {
                    let mut s = String::from("(");
        
                    for (i, a) in args.iter().enumerate() {
                        if i == args.len() - 1 {
                            s.push_str(&format!("{}", a.0));
                        } else {
                            s.push_str(&format!("{}, ", a.0));
                        }
                    }
        
                    s.push(')');
        
                    write!(f, "{s} -> ")
                }?;
        
                write!(f, "{}", expr)
            } else if let Some(Tuple(exprs)) = self.downcast_ref() {
                write!(f, "[")?;

                for expr in exprs {
                    write!(f, "{}", expr)?;
                }

                write!(f, "]")
            } else if let Some(Matrix(rows)) = self.downcast_ref() {
                write!(f, "[ ")?;

                for (i, row) in rows.iter().enumerate() {
                    for (j, expr) in row.iter().enumerate() {
                        write!(f, "{}{}", expr, if j == row.len() - 1 {
                            if i != rows.len() - 1 {
                                "; "
                            } else {
                                " "
                            }
                        } else {
                            ", "
                        })?;
                    }
                }

                write!(f, "]")
            } else if let Some(Set(exprs)) = self.downcast_ref() {
                write!(f, "{{")?;

                for expr in exprs {
                    write!(f, "{}", expr)?;
                }

                write!(f, "}}")
            } else {
                todo!()
            }
        }
    }

    pub trait CloneExpr {
        fn clone_expr(&self) -> Box<dyn Expr>;
    }
    
    impl<T> CloneExpr for T
    where 
        T: 'static + Expr + Clone
    {
        fn clone_expr(&self) -> Box<dyn Expr> {
            Box::new(self.clone())
        }
    }
    
    impl Clone for Box<dyn Expr> {
        fn clone(&self) -> Self {
            self.clone_expr()
        }
    }
    
    create_structs!(
        impl Expr for 
            Literal(Box<dyn Val>),
            Symbol(String),
            Group(Box<dyn Expr>),
            Unary(Token, Box<dyn Expr>),
            Binary(Box<dyn Expr>, Token, Box<dyn Expr>),
            Call(Box<dyn Expr>, Vec<Option<Box<dyn Expr>>>),
            Assign(Symbol, Box<dyn Expr>),
            TypedAssign(Symbol, Box<dyn Expr>, Box<dyn Expr>), // name, type, value (x : Int = 5; y : {1, 2, 3} = 0)
            Func(Vec<Symbol>, Box<dyn Expr>),
            Tuple(Vec<Box<dyn Expr>>),
            Matrix(Vec<Vec<Box<dyn Expr>>>),
            Set(Vec<Box<dyn Expr>>), // store exprs in a vector, and turn into set when resolving values
            TypeExpr(Box<dyn Expr>, Box<dyn Expr>), // value, type (2 : Int; msg : Str)
            FuncTypeExpr(Box<dyn Expr>, Vec<Box<dyn Expr>>, Box<dyn Expr>) // value, arg types, outtype
    );
}

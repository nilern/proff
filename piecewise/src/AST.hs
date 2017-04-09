module AST (Exp(..), Stmt(..), BlockItem(..)) where

data Exp = Fn [([Exp], [Stmt])]
         | Block [Stmt]
         | Call Exp [Exp]
         | Var String
         | Int Int
         | Set [Exp]
         deriving Show

data Stmt = Def Exp Exp
          | AugDef Exp Exp
          | Expr Exp
          deriving Show

data BlockItem = Clause [Exp] Stmt
               | Stmt Stmt
               deriving Show

> {-# LANGUAGE FlexibleContexts, RankNTypes, GADTs #-}
> {-# LANGUAGE ViewPatterns, OverloadedStrings #-}

> module PatExpand (expandExpr, expandStmt, expandStmtList,
>                   runExpansion, PatError) where
> import Data.Text (Text, pack)
> import Control.Monad (foldM)
> import Control.Eff
> import Control.Eff.Exception
> import Control.Eff.State.Lazy
> import Control.Eff.Reader.Lazy
> import qualified Parsing.CST as CST
> import Parsing.CST (Const(..), Var(..))
> import AST (Stmt(..), Expr(..), Jump(..))
> import Util (Name(..), position)

> data PatError = InvalidPat CST.Expr deriving Show

> type Expansion a =
>     forall r . (Member (Exc PatError) r,
>                 Member (Reader Jump) r, Member (State Int) r)
>              => Eff r a

> runExpansion :: Expansion a -> Jump -> Int -> Either PatError (Int, a)
> runExpansion m jmp counter = run (runExc (runReader (runState counter m) jmp))

> freshName :: (Member (State Int) r) => Text -> Eff r Name
> freshName chars = do res <- UniqueName chars <$> get
>                      modify (+ (1::Int))
>                      return res

> expandExpr :: CST.Expr -> Expansion Expr
> expandExpr (CST.Fn pos cases) = Fn pos <$> traverse expandCase cases
>     where expandCase (pats, cond, body) =
>               local (const NextMethod)
>                   (do formals <- traverse freshArg [0..length pats]
>                       patStmts <- expandPatList Def pats
>                                       [CST.Var (LexVar pos formal)
>                                        | formal <- formals]
>                       cond' <- Expr <$> expandExpr cond
>                       body' <- Expr <$> expandExpr body
>                       return (formals, Block pos (patStmts ++ [cond', body'])))
>           freshArg i = freshName (pack ("arg" ++ show i))
> expandExpr (CST.Block pos stmts) =
>     local (const ThrowBindErr) (Block pos <$> expandStmtList stmts)
> expandExpr (CST.App pos f args) =
>     App pos <$> expandExpr f <*> traverse expandExpr args
> expandExpr (CST.PrimApp pos op args) =
>     PrimApp pos op <$> traverse expandExpr args
> expandExpr (CST.Var v) = return (Var v)
> expandExpr (CST.Const c) = return (Const c)

> expandStmt :: CST.Stmt -> Expansion [Stmt]
> expandStmt (CST.Def pat val) = expandPat Def pat val
> expandStmt (CST.AugDef pat val) = expandPat AugDef pat val
> expandStmt (CST.Expr e) = (:[]) . Expr <$> expandExpr e

> expandStmtList :: [CST.Stmt] -> Expansion [Stmt]
> expandStmtList stmts = foldM (\stmts' stmt -> (stmts' ++) <$> expandStmt stmt)
>                              mempty stmts

TODO: CST.PrimApp

> expandPat :: (Var -> Expr -> Stmt) -> CST.Expr -> CST.Expr -> Expansion [Stmt]
> expandPat _ pat @ (CST.Fn _ _) _ = throwExc $ InvalidPat pat
> expandPat _ pat @ (CST.Block _ _) _ = throwExc $ InvalidPat pat
> expandPat mkDef (CST.App pos f args) val =
>     do f' <- expandExpr f
>        val' <- expandExpr val
>        view <- LexVar pos <$> freshName "view"
>        jmp <- ask
>        (viewStmts view f' val' jmp ++) <$>
>            expandPatList mkDef args (map (field (CST.Var view)) [0..])
>     where viewStmts view f' val' jmp =
>               [Def view (App pos (Var unapply) [f', val']),
>                Guard (isJust (Var view)) jmp,
>                Guard (hasLen (Const (Int pos argc)) (Var view)) jmp]
>           field v i = CST.App pos ref [v, CST.Const (Int pos i)]
>           unapply = LexVar pos (PlainName "unapply")
>           isJust v = App pos (Var (LexVar pos (PlainName "isJust"))) [v]
>           hasLen l e = App pos eq [l, (App pos count [e])]
>           eq = Var (LexVar pos (PlainName "=="))
>           count = Var (LexVar pos (PlainName "count"))
>           ref = CST.Var (LexVar pos (PlainName "get"))
>           argc = length args
> expandPat mkDef (CST.Var var) val = (:[]) . mkDef var <$> expandExpr val
> expandPat _ (CST.Const (c @ (position -> pos))) val =
>     do jmp <- ask
>        val' <- expandExpr val
>        return [Guard (App pos eq [val', Const c]) jmp]
>     where eq = Var (LexVar pos (PlainName "=="))

> expandPatList :: (Var -> Expr -> Stmt) -> [CST.Expr] -> [CST.Expr]
>               -> Expansion [Stmt]
> expandPatList mkDef pats vals =
>     foldM (\stmts (pat, val) -> (stmts ++) <$> expandPat mkDef pat val)
>           mempty (zip pats vals)

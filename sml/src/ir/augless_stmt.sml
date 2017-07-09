signature AUGLESS_STMT = sig
    structure Var : TO_DOC

    datatype 'expr t = Def of Pos.t * Var.t * 'expr
                     | Guard of Pos.t * 'expr DNF.t
                     | Expr of 'expr

    val pos : ('e -> Pos.t) -> 'e t -> Pos.t

    val toDoc : ('e -> PPrint.doc) -> 'e t -> PPrint.doc
end

functor AuglessStmt(V : TO_DOC) :> AUGLESS_STMT where type Var.t = V.t = struct
    structure PP = PPrint
    val op<+> = PP.<+>

    structure Var = V

    datatype 'expr t = Def of Pos.t * Var.t * 'expr
                     | Guard of Pos.t * 'expr DNF.t
                     | Expr of 'expr

    fun pos _ (Def (pos, _, _)) = pos
      | pos _ (Guard (pos, _)) = pos
      | pos exprPos (Expr expr) = exprPos expr

    fun toDoc exprToDoc (Def (_, var, expr)) =
        Var.toDoc var <+> PP.text "=" <+> exprToDoc expr
      | toDoc exprToDoc (Guard (_, dnf)) = PP.text "@guard" <+> DNF.toDoc exprToDoc dnf
      | toDoc exprToDoc (Expr expr) = exprToDoc expr
end

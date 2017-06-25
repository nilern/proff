structure Stmt0 = struct
    structure PP = PPrint
    val op<+> = PP.<+>

    datatype 'expr t = Def of 'expr * 'expr
                     | AugDef of 'expr * 'expr
                     | Expr of 'expr

    fun pos exprPos (Def (pat, _)) = exprPos pat
      | pos exprPos (AugDef (pat, _)) = exprPos pat
      | pos exprPos (Expr expr) = exprPos expr

    fun toString exprToString (Def (pat, expr)) =
        exprToString pat ^ " = " ^ exprToString expr
      | toString exprToString (AugDef (pat, expr)) =
        exprToString pat ^ " += " ^ exprToString expr
      | toString exprToString (Expr expr) = exprToString expr

    fun toDoc exprToDoc (Def (pat, expr)) =
        exprToDoc pat <+> PP.text "=" <+> exprToDoc expr
      | toDoc exprToDoc (AugDef (pat, expr)) =
        exprToDoc pat <+> PP.text "+=" <+> exprToDoc expr
      | toDoc exprToDoc (Expr expr) = exprToDoc expr
end

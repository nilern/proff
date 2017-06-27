structure Expr0 :> sig
    structure Var : VAR

    datatype ('expr, 'stmt, 'bind) t =
      Fn of Pos.t * ('bind * 'expr) vector
    | Block of Pos.t * 'stmt vector
    | App of Pos.t * 'expr * 'expr vector
    | PrimApp of Pos.t * Primop.t * 'expr vector
    | Var of Pos.t * Var.t
    | Const of Pos.t * Const.t

    val pos : ('expr, 'stmt, 'bind) t -> Pos.t

    val toDoc : ('e -> PPrint.doc) -> ('s -> PPrint.doc) -> ('b -> PPrint.doc)
              -> ('e, 's, 'b) t -> PPrint.doc
end where type Var.Name.t = StringName.t = struct
    structure PP = PPrint
    val op^^ = PP.^^
    val op<+> = PP.<+>
    val op<$> = PP.<$>

    structure Var = Var(StringName)

    datatype ('expr, 'stmt, 'bind) t =
      Fn of Pos.t * ('bind * 'expr) vector
    | Block of Pos.t * 'stmt vector
    | App of Pos.t * 'expr * 'expr vector
    | PrimApp of Pos.t * Primop.t * 'expr vector
    | Var of Pos.t * Var.t
    | Const of Pos.t * Const.t

    fun pos (Fn (pos, _)) = pos
      | pos (Block (pos, _)) = pos
      | pos (App (pos, _, _)) = pos
      | pos (PrimApp (pos, _, _)) = pos
      | pos (Var (pos, _)) = pos
      | pos (Const (pos, _)) = pos

    fun toDoc toDoc' _ bindToDoc (Fn (_, cases)) =
        let fun caseToDoc (bind, body) =
                bindToDoc bind <+> PP.text "=>" <+> toDoc' body
        in case Vector.length cases
            of 1 => PP.braces (caseToDoc (Vector.sub (cases, 0)))
             | _ => let fun step (cs, acc) = acc ^^ PP.semi <$> caseToDoc cs
                        val caseDoc = caseToDoc (Vector.sub (cases, 0))
                        val rcases = VectorSlice.slice(cases, 1, NONE)
                        val caseDocs = VectorSlice.foldl step caseDoc rcases
                    in
                        PP.lBrace ^^ PP.align caseDocs ^^ PP.rBrace
                    end
        end
      | toDoc _ stmtToDoc _ (Block (_, stmts)) =
        (case Vector.length stmts
         of 1 => PP.braces (stmtToDoc (Vector.sub (stmts, 0)))
          | _ => let fun step (stmt, acc) = acc ^^ PP.semi <$> stmtToDoc stmt
                     val stmtDoc = stmtToDoc (Vector.sub (stmts, 0))
                     val rstmts = VectorSlice.slice(stmts, 1, NONE)
                     val stmtDocs = VectorSlice.foldl step stmtDoc rstmts
                 in
                     PP.lBrace ^^
                         PP.nest 4 (PP.line ^^ stmtDocs) ^^
                             PP.line ^^ PP.rBrace
                 end)
      | toDoc toDoc' _ _ (App (_, f, args)) =
        let fun step (arg, acc) = acc <+> toDoc' arg
            val argDocs = Vector.foldl step (toDoc' f) args
        in PP.parens (PP.align argDocs)
        end
      | toDoc toDoc' _ _ (PrimApp (_, po, args)) =
        let fun step (arg, acc) = acc <+> toDoc' arg
            val argDocs = Vector.foldl step (Primop.toDoc po) args
        in PP.parens (PP.align argDocs)
        end
      | toDoc _ _ _ (Var (_, v)) = Var.toDoc v
      | toDoc _ _ _ (Const (_, c)) = Const.toDoc c
end

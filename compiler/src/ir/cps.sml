structure Cps : sig
    structure Expr : ANF_EXPR

    structure Stmt : sig
        datatype t = Def of Pos.t * Name.t * Expr.t
                   | Expr of Expr.t
    end

    structure Transfer : sig
        datatype t = Continue of Label.t * Expr.Triv.t vector
                   | Branch of Expr.Triv.t * Label.t * Label.t
    end

    structure Argv : TO_DOC

    structure Cont : sig
        type t = { args: Name.t vector
                 , block: (Transfer.t, Stmt.t) Block.t }
    end

    structure Cfg : sig
        type t = { entry: Label.t
                 , conts: Cont.t LabelMap.map }

        structure Builder : sig
            type builder

            val empty : unit -> builder
            val insert : builder * Label.t * Cont.t -> unit
            val build : builder -> Label.t option -> t
        end
    end

    type proc = { name: Name.t
                , clovers: Name.t vector
                , args: Argv.t
                , cfg: Cfg.t }

    type program = { procs: proc NameMap.map
                   , main: Cfg.t }

    val toDoc : program -> PPrint.doc
end = struct
    structure PP = PPrint
    val op^^ = PP.^^
    val op<+> = PP.<+>
    val op<$> = PP.<$>

    structure Expr = AnfExpr

    structure Stmt = struct
        datatype t = Def of Pos.t * Name.t * Expr.t
                   | Expr of Expr.t

        val toDoc =
            fn Def (_, name, expr) => Name.toDoc name <+> PP.text "=" <+> Expr.toDoc expr
             | Expr expr => Expr.toDoc expr
    end

    structure Transfer = struct
        datatype t = Continue of Label.t * Expr.Triv.t vector
                   | Branch of Expr.Triv.t * Label.t * Label.t

        val toDoc =
            fn Continue (label, args) =>
               PP.text "__continue" <+> Label.toDoc label <+>
                   PP.punctuate PP.space (Vector.map Expr.Triv.toDoc args)
             | Branch (cond, conseq, alt) =>
               PP.text "@if" <+> Expr.Triv.toDoc cond <+>
                   Label.toDoc conseq <+> PP.text "|" <+> Label.toDoc alt
    end

    structure Argv = struct
        val op^^ = PPrint.^^
        val op<+> = PPrint.<+>

        type t = {self: Name.t, params: Name.t, denv: Name.t, ret: Label.t}

        fun toDoc {self = self, params = params, denv = denv, ret = ret} =
            PPrint.parens (PPrint.text "self =" <+> Name.toDoc self ^^ PPrint.text "," <+>
                               PPrint.text "params =" <+> Name.toDoc params ^^ PPrint.text "," <+>
                                   PPrint.text "denv =" <+> Name.toDoc denv ^^ PPrint.text "," <+>
                                       PPrint.text "ret =" <+> Label.toDoc ret)
    end

    structure Cont = struct
        type t = { args: Name.t vector
                 , block: (Transfer.t, Stmt.t) Block.t }

        fun toDoc { args = args, block = block } =
            PP.punctuate PP.space (Vector.map Name.toDoc args) <+> PP.lBrace <$>
                PP.nest 4 (Block.toDoc Transfer.toDoc Stmt.toDoc block) <$> PP.rBrace
    end

    structure Cfg = struct
        type t = { entry: Label.t
                 , conts: Cont.t LabelMap.map }

        fun toDoc { entry = entry, conts = conts } =
            let fun pairToDoc (label, cont) =
                    (if label = entry
                     then PP.text "-> " ^^ Label.toDoc label
                     else Label.toDoc label) ^^ PP.text ": " ^^ PP.align (Cont.toDoc cont)
            in PP.punctuate (PP.line ^^ PP.line)
                            (Vector.map pairToDoc (Vector.fromList (LabelMap.listItemsi conts)))
            end

        structure Builder = struct
            type builder = { entry: Label.t option, conts: Cont.t LabelMap.map } ref

            fun empty () = ref { entry = NONE, conts = LabelMap.empty }

            fun insert (builder, label, cont) =
                let val { entry = entry, conts = conts } = !builder
                in builder := { entry = entry, conts = LabelMap.insert (conts, label, cont) }
                end

            fun build builder defaultEntry =
                let val { entry = entry, conts = conts } = !builder
                in { entry = valOf (OptionExt.or entry defaultEntry), conts = conts }
                end
        end
    end

    type proc = { name: Name.t
                , clovers: Name.t vector
                , args: Argv.t
                , cfg: Cfg.t }

    type program = { procs: proc NameMap.map
                   , main: Cfg.t }

    fun procToDoc { name = name, clovers = clovers, args = args, cfg = cfg } =
        let val nameDoc = Name.toDoc name
            val cloversDoc = PP.braces (PP.punctuate (PP.text ", ") (Vector.map Name.toDoc clovers))
            val argsDoc = Argv.toDoc args
            val cfgDoc = Cfg.toDoc cfg
        in nameDoc ^^ cloversDoc ^^ argsDoc <+> PP.text "=" <+> PP.lBrace <$>
               PP.nest 4 cfgDoc <$> PP.rBrace
        end

    fun toDoc { procs = procs, main = main } =
        let fun step (proc, acc) = procToDoc proc ^^ PP.line <$> acc
        in NameMap.foldl step PP.empty procs <$> Cfg.toDoc main
        end
end
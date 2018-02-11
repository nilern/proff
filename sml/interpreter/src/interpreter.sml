structure Interpreter :> sig
    exception Panic of Value.value

    val interpret : Value.expr -> Value.value
end = struct
    structure Prim = Primops

    type value = Value.value
    type expr = Value.expr
    type stmt = Value.stmt

    val wrap = Value.wrap
    val force = Value.force
    val forceExn = Value.forceExn

    exception Panic of Value.value

    type envs = { lex: value Env.t, dyn: value Env.t }

    datatype callable = Value of value
                      | Opcode of string

    datatype frame = Callee of envs * expr vector
                   | Arg of envs * expr VectorSlice.slice * callable * value VectorExt.builder
                   | Stmt of envs * stmt VectorSlice.slice * expr
                   | Def of envs * expr * expr option
                   | Match of envs * (* HACK: *) envs ref * expr VectorSlice.slice
                   | OuterMatch of value
                   | Commit of envs * (* HACK: *) envs ref
                   | Body of envs * expr

    fun lookup dump { lex = lenv, dyn = denv } =
        fn Value.Lex name => Env.lookup lenv name
         | Value.Dyn name => valOf (OptionExt.orElse (Env.find denv name)
                                                     (fn () => Dump.find dump name))

    fun insert { lex = lenv, dyn = denv } var value =
        case var
        of Value.Lex name => { lex = Env.insert lenv name value, dyn = denv }
         | Value.Dyn name => { lex = lenv, dyn = Env.insert denv name value }

    fun declare { lex = lenv, dyn = denv } stmts =
        { lex = Env.declare lenv (Value.blockBinders Value.lexName stmts) Value.uninitialized
        , dyn = Env.declare denv (Value.blockBinders Value.dynName stmts) Value.uninitialized }

    fun declareParams { lex = lenv, dyn = denv } pats =
        { lex = Env.declare lenv (VectorExt.flatMap (Value.patBinders Value.lexName) pats)
                            Value.uninitialized
        , dyn = Env.declare denv (VectorExt.flatMap (Value.patBinders Value.dynName) pats)
                            Value.uninitialized }

    fun define { lex = lenv, dyn = denv } { lex = lDelta, dyn = dDelta } =
        ( Env.define lenv lDelta Value.initialize
        ; Env.define denv dDelta Value.initialize )

    fun eval dump cont envs =
        fn Value.Fn (pos, methods) =>
            continue pos (wrap (Value.Closure (methods, #lex envs))) dump cont
         | Value.Call (_, callee, args) =>
            eval dump (Callee (envs, args) :: cont) envs callee
         | Value.PrimCall (pos, opcode, args) => evalArgs pos dump cont envs (Opcode opcode) args
         | Value.Block (_, stmts, expr) =>
            (case VectorExt.uncons stmts
             of SOME (stmt, remStmts) =>
                 let val envs = declare envs stmts
                     val cont = Stmt (envs, remStmts, expr) :: cont
                 in exec dump cont envs stmt
                 end
              | NONE => eval dump cont envs expr)
         | Value.Var (pos, var) => continue pos (lookup dump envs var) dump cont
         | Value.Const (pos, v) => continue pos v dump cont

    and evalArgs pos dump cont envs callee argExprs =
        case VectorExt.uncons argExprs
        of SOME (argExpr, remArgExprs) =>
            let val argsBuilder = VectorExt.builder (Vector.length argExprs)
                val cont = Arg (envs, remArgExprs, callee, argsBuilder) :: cont
            in eval dump cont envs argExpr
            end
         | NONE => applyAny pos dump cont envs callee #[]

    and exec dump cont envs =
        fn Value.Def (pat, guard, expr) =>
            let val cont = Def (envs, pat, guard) :: cont
            in eval dump cont envs expr
            end
         | Value.Expr expr => eval dump cont envs expr

    and continue pos value dump =
        fn Callee (envs, argExprs) :: cont => evalArgs pos dump cont envs (Value value) argExprs
         | Arg (envs, argExprs, callee, argsBuilder) :: cont =>
            ( #update argsBuilder (#2 (VectorSlice.base argExprs) - 1, value)
            ; case VectorSliceExt.uncons argExprs
              of SOME (argExpr, remArgExprs) =>
                  let val cont = Arg (envs, remArgExprs, callee, argsBuilder) :: cont
                  in eval dump cont envs argExpr
                  end
               | NONE => applyAny pos dump cont envs callee (#done argsBuilder ()))
         | Stmt (envs, stmts, expr) :: cont =>
            (case VectorSliceExt.uncons stmts
             of SOME (stmt, remStmts) =>
                 let val cont = Stmt (envs, remStmts, expr) :: cont
                 in exec dump cont envs stmt
                 end
              | NONE => eval dump cont envs expr)
         | Def (envs, pat, (* TODO: *) guard) :: cont =>
            let val envDeltas = ref { lex = Env.empty, dyn = Env.empty }
                val seq = wrap (Value.Slice (wrap (Value.Tuple #[value]), 0))
                val cont = Commit (envs, envDeltas) :: cont
            in match dump cont envs envDeltas pat seq
            end
         | Match (envs, envDeltas, pats) :: cont =>
            (case VectorSliceExt.uncons pats
             of SOME (pat, remPats) =>
                 let val cont = Match (envs, envDeltas, remPats) :: cont
                 in match dump cont envs envDeltas pat value
                 end
              | NONE => continue pos value dump cont)
         | OuterMatch argSeq :: cont =>
            (* TODO: Check that `value` is an empty seq *)
            continue pos argSeq dump cont
         | Commit (envs, envDeltas) :: cont =>
            (* TODO: Check that `value` is an empty seq *)
            ( define envs (!envDeltas)
            ; continue pos value dump cont )
         | Body (envs, body) :: cont => eval dump cont envs body
         | [] =>
            (case Dump.pop dump
             of SOME (cont', dump') => continue pos value dump' cont'
              | NONE => value)

    and applyAny pos dump cont envs callee args =
        case callee
        of Value f => apply pos dump cont envs f (wrap (Value.Tuple args))
         | Opcode opcode => primApply pos dump cont envs opcode args

    and apply pos dump cont envs callee args =
        case forceExn callee
        of Value.Closure (methods, lenv) =>
            let val Value.Method (pats, _, body) = Vector.sub (methods, 0)
            in case VectorExt.uncons pats
               of SOME (pat, remPats) =>
                   let val envs = declareParams { lex = lenv, dyn = #dyn envs } pats
                       val envDeltas = ref { lex = Env.empty, dyn = Env.empty }
                       val argSlice = wrap (Value.Slice (args, 0))
                       val cont = Match (envs, envDeltas, remPats)
                                  :: Commit (envs, envDeltas) :: Body (envs, body) :: cont
                   in match dump cont envs envDeltas pat argSlice
                   end
                | NONE => raise Fail "unimplemented"
            end
         | calleeContent => signal dump cont envs (wrap (Value.String "InApplicable")) pos

    and primApply pos dump cont envs opcode argv =
        case opcode
        of "panic" =>
           (case argv
            of #[exn] => raise Panic exn
             | _ => raise Fail "__panic: argc")
         | _ => continue pos (Prim.applyPure opcode argv) dump cont

    and match dump cont envs envDeltas pattern argSeq =
        (* MAYBE: Make it possible to resume the match when Argc is signaled? *)
        case pattern
        of Value.Fn _ => raise Fail "Illegal pattern (function literal)"
         | Value.Call (pos, callee, innerPats) => raise Fail "unimplemented"
         | Value.PrimCall (pos, opcode, innerPats) =>
            (case Prim.unApply opcode argSeq (Vector.length innerPats)
             of SOME (innerArgSeq, outerArgSeq) =>
                 (case VectorExt.uncons innerPats
                  of SOME (pat, remPats) =>
                      let val cont = Match (envs, envDeltas, remPats)
                                     :: OuterMatch outerArgSeq :: cont
                      in match dump cont envs envDeltas pat innerArgSeq
                      end
                   | NONE => raise Fail "unimplemented")
              | NONE => signal dump cont envs (wrap (Value.String "Argc")) pos)
         | Value.Block _ => raise Fail "Illegal pattern (block)"
         | Value.Var (pos, var) =>
            (case Prim.slicePopFront argSeq
             of SOME (arg, remArgs) =>
                 ( envDeltas := insert (!envDeltas) var arg
                 ; continue pos remArgs dump cont )
              | NONE => signal dump cont envs (wrap (Value.String "Argc")) pos)
         | Value.Const (pos, c) => raise Fail "unimplemented"

    and signal dump cont envs tag pos =
        let val handler = lookup dump envs (Value.Dyn "handleException")
            val exn = wrap (Value.Tuple #[tag,
                                          wrap (Value.String (#file pos)),
                                          wrap (Value.Int (#index pos)),
                                          wrap (Value.Int (#line pos)),
                                          wrap (Value.Int (#col pos))])
            val args = wrap (Value.Tuple #[exn])
        in apply pos dump cont envs handler args
        end

    fun interpret expr = eval Dump.empty [] { lex = Env.empty, dyn = Env.empty } expr
end

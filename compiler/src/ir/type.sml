structure Type = struct
    datatype t = Any
               | Int | Float | Bool | Char
               | Tuple
               | Fn
               | Closure | DynEnv
               | Label of t vector

    val methodLabel = Label (Vector.fromList [Closure, DynEnv, Fn, Int, Any])

    local structure PP = PPrint
          val op^^ = PP.^^
    in val rec toDoc = fn Any => PP.text "Any"
                        | Int => PP.text "Int"
                        | Float => PP.text "Float"
                        | Bool => PP.text "Bool"
                        | Char => PP.text "Char"
                        | Tuple => PP.text "Tuple"
                        | Fn => PP.text "Fn"
                        | Closure => PP.text "Closure"
                        | DynEnv => PP.text "DynEnv"
                        | Label argTypes =>
                          PP.text "Label" ^^
                              PP.parens (PP.punctuate (PP.text "," ^^ PP.space)
                                                      (Vector.map toDoc argTypes))
    end
end

signature TRIV = sig
    structure Var: TO_DOC
    structure Const: TO_DOC

    datatype t = Var of Var.t
               | Const of Const.t

    val toDoc : t -> PPrint.doc
end

functor TrivFn(structure V: TO_DOC structure C: TO_DOC)
:> TRIV where type Var.t = V.t and type Const.t = C.t = struct
    structure Var = V
    structure Const = C

    datatype t = Var of Var.t
               | Const of Const.t

    val toDoc = fn Var v => Var.toDoc v
                 | Const c => Const.toDoc c
end

structure CTriv = TrivFn(structure V = CVar
                         structure C = Const)
structure ATriv = TrivFn(structure V = AVar
                         structure C = Const)
structure FlatTriv0 = TrivFn(structure V = FlatVar0
                             structure C = Const)
structure FlatTriv1 = TrivFn(structure V = FlatVar1
                             structure C = Const)
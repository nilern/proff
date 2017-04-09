{
module Lexer (Tok(..), Delimiter(..), Side(..), Precedence(..), lexer) where
import Data.Char (isAlpha, isSpace, isDigit)
}

%wrapper "basic"

$digit = 0-9
$idchar = [a-zA-Z\$@_]
$opchar = [!\%&\*\+\-\/\<=>\?\\\^\|\~]

tokens :-
    $white+  ;
    "=>"     { const TokArrow }
    "+="     { const TokPlusEq }
    "="      { const TokEq }
    $digit+  { TokInt . read }
    $idchar+ { TokId }
    $opchar+ { \s -> TokOp s (precedence s) }
    "{"      { const $ TokDelim Brace L }
    "}"      { const $ TokDelim Brace R }
    ";"      { const TokSemiColon }
    ","      { const TokComma }

{
data Delimiter = Paren | Bracket | Brace deriving Show

data Side = L | R deriving Show

data Precedence = Zero | One | Two | Three | Four | Five | Six | Seven
                deriving Show

data Tok = TokId String
         | TokOp String Precedence
         | TokInt Int
         | TokEq
         | TokPlusEq
         | TokArrow
         | TokDelim Delimiter Side
         | TokSemiColon
         | TokComma
         deriving Show

precedence :: String -> Precedence
precedence ('|':cs) = One
precedence ('^':cs) = Two
precedence ('&':cs) = Three
precedence ('=':cs) = Four
precedence ('!':cs) = Four
precedence ('<':cs) = Five
precedence ('>':cs) = Five
precedence ('+':cs) = Six
precedence ('-':cs) = Six
precedence ('*':cs) = Seven
precedence ('/':cs) = Seven
precedence ('%':cs) = Seven

lexer :: String -> [Tok]
lexer = alexScanTokens
}

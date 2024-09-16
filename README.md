# Math-Based Language

## Base Spec
```
Functionality Supported:

== Computation
1 + 3 // Prints 4

    -- Math Operators
    
    a+b Addition
    a-b Subtraction
    a*b Multiplication
    a/b Division
    a^b Exponentiation
    ~a  Conjugate

== Boolean
true // Prints true

    -- Boolean Operators

    == Equality
    != Not Equal
    <  Less Than
    <= Less Than or Equal
    >  Greater Than
    >= Greater Than or Equal

== Compound Expression
1 < 2 <= 3 // Prints true

== Strings
"hello world" // prints "hello world"

== Chars
'h' // prints 'h'

== Tuples
(1, "hi", true) // prints (1, "hi", true)
(4, 5) // prints (4, 5)

== Lists (and Matrix Literal)
[1, 2, 3] // prints [1, 2, 3]
[1, 2, 3; 4, 5, 6; 7, 8, 9] // prints
// [ 1, 2, 3; 
//   4, 5, 6;
//   7, 8, 9 ]

== Print
value // prints value
value; // doesn't print

== Variables
x = 1 // creates x (immutable)
y = x // creates y equal to x

== Functions
f(x) = x^2 + 1 // creates f
f(1) // prints 2

    -- Builtin Functions (aka auto imported)
    
    sqrt
    nrt
    floor
    ceil
    round
    size
    print
    ...

    -- Function Symbolic Notation (without args)

    f(x) = x + 1 // regular function
    g = 1 + f^2 // no args

== Derivative
f(x) = x^2 - 2x + 1
g = f' // computes symbolic derivative

== Types
x : Int // x exists and is of type Int
msg : [Char] = "Hello World"

    -- Numerical Types

    Whole   1,2,3,...
    Nat     0,1,2,...
    Int     ...,-1,0,1,...
    Real    <real numbers>
    Complex <complex numbers>

    -- String Types

    Str    String
    Char   Unicode-Encoded Character
    [Char] Array of Chars, equivalent to a String

    -- Other Types
    Bool  Boolean
    Univ  Universal-Set, contains every possible value
    Empty Empty-Set, contains nothing // Empty = { }
    None  Member of Empty, not an actual type, but a value

    -- Typing Functions

        -- Inline Typing

        f(x : Int) : Int = x^2 + 1

        -- Pure Typing

        f : Int -> Int // f is a Mapping-Type, mapping Int to Int

Types are Sets, Sets are Types.
```

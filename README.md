# Math-Based Language

## Sets, Infinite Sets, Types

One main aspect about this language is that sets are types and vice versa. So, this is totally valid and a correct usage:
```rs
Valids = { 1, 3, (5, 5), "hi" };
x : Valids
f : Valids -> T
```
However, there also exist infinite sets. For example, `Real`, `Int`, `Complex`, `Str`, etc. However, as sets are also values, how would the infinite sets work. How would something like `Set.contains(Real, 0)` work? I think the solution here is to actually change how this works. Recall that variables, functions, and values all belong to sets (like `x : Int`, `dir : { "up", "down" }`). However, instead of infinite sets being just regular sets, they are _generated_ from some type. For example, the infinite set of all 2D points in the real plane would be:
```rs
data Point = (Real, Real);
```
Then, the set of infinite points would be `Point`. However, the set never actually exists, nor does the user ever make the actual set. Instead, an element of `Point` would be any tuple of any two `Real`s.

This way there wouldn't be problems of creating infinite sets.
Now, onto functions like `Set.contains` and `Set.enumerate`. 

I think for `Set.contains` and other ones like that, the infinite set can contain a "rule" that is evaluated for membership. For example, for a type, it would check if it can be represented as `(Real, Real)`, and if it can, it would be true otherwise false.

For `Set.enumerate`, I think we get a problem. Since `Real` is uncountable, we can't enumerate over it. So perhaps, there is a `class` for `Countable` and `~Countable` (or `NotCountable`). You can enumerate over a countable set but not an uncountable one.

For example, this would work:
```rs
data Point = [Int ^ 2]; // Same as [Real, Real]

vals = Set.enumerate(Point)[0];
```
How? Well, `Int` can be enumerated by doing `0, 1, -1, 2, -2, ...`. However, for a set, the order of the enumeration is not guaranteed by the implementation, all that's guaranteed is that will enumerate it.

I think that all valid user-made infinite sets (and thus types) can be constructed from the builtin types (eg. `Int`, `Real`, `Complex`, `Str`, `D -> R`), set operations (eg. `|`, `&`, `\`), collections (eg. tuples, lists/matrices, records), and conditionals/things (eg. `x / 2 for x : Real`, `x is in set A if mod(x, 3) == 1` <-- not correct code, just pseudocode).

Also, the `Set` and `Type` types are probably the exact same under the hood. They will just be differently named to be easier to write and stuff.

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

Little Change (9/30/2024) -- Tuples vs Lists and Matricies:
```rs
// Tuple
origin : [Real^2]
origin = [0, 0]

mytup : [Real^3, Str, Bool^2]
mytup = [-4.05, 1.0036, 0.209, "Hello World", false, true]

// List
// A list is actually a convenience form of a tuple, and in fact is only used in types
// Essentially, it is a tuple with unknown size at write-time, but is still constant of course

real_n : [Real]
real_n = [1, 2, 3]

real_n2 : [Real]
real_n2 = [1, 2, 3, 4, 5, 6]

// Matrix
// A matrix can be created using semicolons to separate rows
I : Mat(Real, 2)
I = [1, 0; 0, 1]

// A List of List can be created, but it doesn't have to be rectangular, so isn't a matrix necessarily

A : [[Real]]
A = [[1, 2, 3], [1, 2, 3, 4, 5]]

// A list of tuples can be created as well
lt : [[Int^2, Str]]
lt = [[1, 2, "hi"], [0, 1, "bye"]]


// or a tuple of tuples, etc.
tt : [[Int^2], Str^2]
tt = [[1, 2], "hi", "bye"]

// The set power notation interacts with functions well
f : Real^2 -> Real
f(x, y) = x + y

// as Set^N = x1 : Set, x2 : Set, ... xn : Set (No bounding structure)
// Set^1 = Set
// Set^0 = Empty
// Set^negative is not defined
```

Little Change (10/3/2024) -- Arrays
```rs
// Now there will also be arrays in the language. However, an array is actually a list of values that is passed into an expression expecting a singular value, and executes the expression on each value of the array and outputs that.

f : Int -> Int
f(n) = n^2 - sin(pi/4 * n) + e^(1-e^-abs(x))

// On way
values_1 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
results_1 = Iter.map(values_1, f)

// Other way
values_2 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
results_2 : [_] = f(#array values_2)

// To create an array, use the #array macro

// This is also how the +/- and -/+ operators work
y = (-1 +/- sqrt(10)) / 2 -/+ 1/2 // y = #array [(-2 + sqrt10) / 2, (-sqrt10)/2]
```
    

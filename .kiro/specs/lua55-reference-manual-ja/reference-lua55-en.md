# Lua 5.5 Reference Manual (English Source)

> **Source**: https://www.lua.org/manual/5.5/manual.html
> **Purpose**: Source document for Japanese translation
> **Copyright**: Copyright © 2020–2025 Lua.org, PUC-Rio. Freely available under the terms of the Lua license.

---

## 1 – Introduction

Lua is a powerful, efficient, lightweight, embeddable scripting language. It supports procedural programming, object-oriented programming, functional programming, data-driven programming, and data description.

Lua combines simple procedural syntax with powerful data description constructs based on associative arrays and extensible semantics. Lua is dynamically typed, runs by interpreting bytecode with a register-based virtual machine, and has automatic memory management with a generational garbage collector, making it ideal for configuration, scripting, and rapid prototyping.

Lua is implemented as a library, written in clean C, the common subset of Standard C and C++. The Lua distribution includes a host program called `lua`, which uses the Lua library to offer a complete, standalone Lua interpreter, for interactive or batch use.

Lua is free software, and is provided as usual with no guarantees, as stated in its license.

---

## 2 – Basic Concepts

### 2.1 – Values and Types

Lua is a dynamically typed language. This means that variables do not have types; only values do. There are no type definitions in the language. All values carry their own type.

All values in Lua are first-class values. This means that all values can be stored in variables, passed as arguments to other functions, and returned as results.

There are eight basic types in Lua: **nil**, **boolean**, **number**, **string**, **function**, **userdata**, **thread**, and **table**.

- **nil**: A type with a single value, nil, whose main property is to be different from any other value. It usually represents the absence of a useful value.
- **boolean**: Has two values, false and true. Both nil and false make a condition false (falsy values).
- **number**: Has two subtypes, integer and float. Standard Lua uses 64-bit integers and double-precision floats.
- **string**: Immutable sequences of bytes. 8-bit clean.
- **function**: Represents both functions written in Lua and functions written in C.
- **userdata**: Allows arbitrary C data to be stored in Lua variables. Full userdata and light userdata.
- **thread**: Independent threads of execution. Used to implement coroutines.
- **table**: The only data-structuring mechanism in Lua. Associative arrays.

### 2.2 – Environments and the Global Environment

Lua keeps a distinguished environment called the global environment. This value is kept at a special index in the C registry. In Lua, the global variable `_G` is initialized with this same value.

### 2.3 – Scopes and Visibility

Any variable name that is not found in the current scope is converted to an access to a field in a special value called the environment. The environment is always the global environment for the main chunk and for Lua functions created with `load` or `loadfile`.

**Lua 5.5 New Feature: global keyword**

Lua 5.5 introduces the `global` keyword for explicit global variable declarations:

```lua
global x = 10  -- declares x as a global variable
```

The `global` declaration can include a type annotation (which is currently ignored):

```lua
global x: integer = 10
```

### 2.4 – Metatables and Metamethods

The operations that metatables control include:
- `__add`: addition (`+`)
- `__sub`: subtraction (`-`)
- `__mul`: multiplication (`*`)
- `__div`: division (`/`)
- `__mod`: modulo (`%`)
- `__pow`: exponentiation (`^`)
- `__unm`: unary negation (`-`)
- `__idiv`: floor division (`//`)
- `__band`: bitwise AND (`&`)
- `__bor`: bitwise OR (`|`)
- `__bxor`: bitwise XOR (`~`)
- `__bnot`: bitwise NOT (`~`)
- `__shl`: left shift (`<<`)
- `__shr`: right shift (`>>`)
- `__concat`: concatenation (`..`)
- `__len`: length (`#`)
- `__eq`: equality (`==`)
- `__lt`: less than (`<`)
- `__le`: less or equal (`<=`)
- `__index`: indexing access (`table[key]`)
- `__newindex`: indexing assignment
- `__call`: function call
- `__close`: to-be-closed marker
- `__gc`: garbage collection (finalizer)
- `__mode`: weak table mode
- `__name`: name for error messages
- `__pairs`: custom pairs iterator

### 2.5 – Garbage Collection

Lua performs automatic memory management using garbage collection.

#### 2.5.1 – Incremental Garbage Collection

The generational collector uses two generations: young and old.

#### 2.5.2 – Generational Garbage Collection

Alternates minor collections (only young objects) with major collections (all objects).

#### 2.5.3 – Garbage-Collection Metamethods

The `__gc` metamethod (finalizers).

#### 2.5.4 – Weak Tables

A weak table has a metatable with `__mode` field.

### 2.6 – Coroutines

Lua supports coroutines (collaborative multithreading).

---

## 3 – The Language

### 3.1 – Lexical Conventions

Lua is a free-form language.

**Reserved words:**
```
and       break     do        else      elseif    end
false     for       function  global    goto      if
in        local     nil       not       or        repeat
return    then      true      until     while
```

Note: `global` is new in Lua 5.5.

**Operators and punctuation:**
```
+     -     *     /     %     ^     #
&     ~     |     <<    >>    //
==    ~=    <=    >=    <     >     =
(     )     {     }     [     ]     ::
;     :     ,     .     ..    ...
```

### 3.2 – Variables

Three kinds of variables: global, local, and table fields.

### 3.3 – Statements

#### 3.3.1 – Blocks

#### 3.3.2 – Chunks

#### 3.3.3 – Assignment

```lua
var = exp
varlist = explist
```

#### 3.3.4 – Control Structures

```lua
-- while statement
while exp do block end

-- repeat statement
repeat block until exp

-- if statement
if exp then block {elseif exp then block} [else block] end

-- numeric for
for Name = exp, exp [, exp] do block end

-- generic for
for namelist in explist do block end
```

#### 3.3.5 – For Statement

#### 3.3.6 – Function Calls as Statements

#### 3.3.7 – Local Declarations

```lua
local namelist ['=' explist]
local function Name funcbody
```

#### 3.3.8 – Global Declarations (NEW in 5.5)

```lua
global namelist ['=' explist]
```

The `global` keyword explicitly declares global variables.

#### 3.3.9 – To-be-closed Variables

```lua
local <close> name = value
```

### 3.4 – Expressions

#### 3.4.1 – Arithmetic Operators

- `+`: addition
- `-`: subtraction
- `*`: multiplication
- `/`: float division
- `//`: floor division
- `%`: modulo
- `^`: exponentiation
- `-`: unary negation

#### 3.4.2 – Bitwise Operators

- `&`: AND
- `|`: OR
- `~`: XOR (binary), NOT (unary)
- `<<`: left shift
- `>>`: right shift

#### 3.4.3 – Coercions and Conversions

#### 3.4.4 – Relational Operators

- `<`, `>`, `<=`, `>=`, `==`, `~=`

#### 3.4.5 – Logical Operators

- `and`, `or`, `not`

#### 3.4.6 – Concatenation

- `..`: string concatenation

#### 3.4.7 – The Length Operator

- `#`: length operator

#### 3.4.8 – Precedence

#### 3.4.9 – Table Constructors

```lua
tableconstructor ::= '{' [fieldlist] '}'
fieldlist ::= field {fieldsep field} [fieldsep]
field ::= '[' exp ']' '=' exp | Name '=' exp | exp
fieldsep ::= ',' | ';'
```

#### 3.4.10 – Function Calls

#### 3.4.11 – Function Definitions

### 3.5 – Visibility Rules

---

## 4 – The Application Program Interface

### 4.1 – The Stack

Lua uses a virtual stack to pass values to and from C.

### 4.2 – Stack Size

### 4.3 – Valid and Acceptable Indices

### 4.4 – Pointers to strings

### 4.5 – C Closures

### 4.6 – Registry

The registry is a predefined table that can be used by any C code to store whatever Lua values it needs to store.

### 4.7 – Error Handling in C

### 4.8 – Handling Yields in C

### 4.9 – Functions and Types

#### Core Functions (Selection)

| Function | Description |
|----------|-------------|
| `lua_absindex` | Converts an acceptable index to an absolute index |
| `lua_call` | Calls a function |
| `lua_checkstack` | Ensures stack has at least n extra slots |
| `lua_close` | Close a Lua state |
| `lua_compare` | Compares two Lua values |
| `lua_createtable` | Creates a new empty table |
| `lua_error` | Raises a Lua error |
| `lua_gc` | Controls the garbage collector |
| `lua_getfield` | Pushes `t[k]` onto the stack |
| `lua_getglobal` | Pushes the global variable value |
| `lua_gettable` | Gets value from table |
| `lua_gettop` | Returns the index of the top element |
| `lua_isboolean` | Returns 1 if value is boolean |
| `lua_isfunction` | Returns 1 if value is function |
| `lua_isnil` | Returns 1 if value is nil |
| `lua_isnumber` | Returns 1 if value is number |
| `lua_isstring` | Returns 1 if value is string |
| `lua_istable` | Returns 1 if value is table |
| `lua_len` | Returns the length of value |
| `lua_newtable` | Creates a new empty table |
| `lua_newthread` | Creates a new thread |
| `lua_next` | Pops a key and pushes key-value pair |
| `lua_pcall` | Calls a function in protected mode |
| `lua_pop` | Pops n elements from the stack |
| `lua_pushboolean` | Pushes boolean |
| `lua_pushinteger` | Pushes integer |
| `lua_pushnil` | Pushes nil |
| `lua_pushnumber` | Pushes number |
| `lua_pushstring` | Pushes string |
| `lua_rawget` | Raw table access |
| `lua_rawset` | Raw table assignment |
| `lua_setfield` | Does `t[k] = v` |
| `lua_setglobal` | Sets global variable |
| `lua_settable` | Sets table element |
| `lua_settop` | Sets the stack top |
| `lua_toboolean` | Converts to boolean |
| `lua_tointeger` | Converts to integer |
| `lua_tolstring` | Converts to string |
| `lua_tonumber` | Converts to number |
| `lua_tostring` | Converts to C string |
| `lua_type` | Returns type of value |
| `lua_typename` | Returns type name |

#### New in Lua 5.5

| Function | Description |
|----------|-------------|
| `lua_pushexternalstring` | Pushes a string that is managed externally |
| `lua_numbertocstring` | Converts a Lua number to a C string |
| `lua_closethread` | Closes a thread (coroutine) |

### 4.10 – The Debug Interface

---

## 5 – The Auxiliary Library

Functions defined in header file `lauxlib.h` with prefix `luaL_`.

### Key Functions

| Function | Description |
|----------|-------------|
| `luaL_checkinteger` | Checks argument is integer |
| `luaL_checknumber` | Checks argument is number |
| `luaL_checkstring` | Checks argument is string |
| `luaL_checktype` | Checks argument type |
| `luaL_checkudata` | Checks argument is userdata |
| `luaL_dofile` | Loads and runs a file |
| `luaL_dostring` | Loads and runs a string |
| `luaL_error` | Raises an error |
| `luaL_loadfile` | Loads a file |
| `luaL_loadstring` | Loads a string |
| `luaL_newlib` | Creates a new library table |
| `luaL_newmetatable` | Creates a new metatable |
| `luaL_newstate` | Creates a new Lua state |
| `luaL_openlibs` | Opens all standard libraries |
| `luaL_ref` | Creates a reference |
| `luaL_unref` | Frees a reference |

---

## 6 – The Standard Libraries

### 6.1 – Basic Functions

| Function | Description |
|----------|-------------|
| `assert (v [, message])` | Issues error if v is false |
| `collectgarbage ([opt [, arg]])` | Garbage collector interface |
| `dofile ([filename])` | Executes a Lua file |
| `error (message [, level])` | Raises an error |
| `getmetatable (object)` | Returns metatable |
| `ipairs (t)` | Returns iterator for integer keys |
| `load (chunk [, chunkname [, mode [, env]]])` | Loads a chunk |
| `loadfile ([filename [, mode [, env]]])` | Loads a file |
| `next (table [, index])` | Allows traversal of table |
| `pairs (t)` | Returns iterator for table |
| `pcall (f [, arg1, ···])` | Protected call |
| `print (···)` | Prints arguments |
| `rawequal (v1, v2)` | Raw equality |
| `rawget (table, index)` | Raw table access |
| `rawlen (v)` | Raw length |
| `rawset (table, index, value)` | Raw table assignment |
| `select (index, ···)` | Returns arguments after index |
| `setmetatable (table, metatable)` | Sets metatable |
| `tonumber (e [, base])` | Converts to number |
| `tostring (v)` | Converts to string |
| `type (v)` | Returns type as string |
| `warn (msg1, ···)` | Emits a warning |
| `xpcall (f, msgh [, arg1, ···])` | Extended protected call |

### 6.2 – Coroutine Manipulation

| Function | Description |
|----------|-------------|
| `coroutine.close (co)` | Closes a coroutine |
| `coroutine.create (f)` | Creates a new coroutine |
| `coroutine.isyieldable ([co])` | Returns true if can yield |
| `coroutine.resume (co [, val1, ···])` | Resumes coroutine |
| `coroutine.running ()` | Returns running coroutine |
| `coroutine.status (co)` | Returns coroutine status |
| `coroutine.wrap (f)` | Creates a wrapper function |
| `coroutine.yield (···)` | Suspends coroutine |

### 6.3 – Modules

| Function/Variable | Description |
|-------------------|-------------|
| `require (modname)` | Loads a module |
| `package.config` | Configuration string |
| `package.cpath` | C loader search path |
| `package.loaded` | Loaded modules table |
| `package.loadlib (libname, funcname)` | Loads a C library |
| `package.path` | Lua loader search path |
| `package.preload` | Preload table |
| `package.searchers` | Searcher functions |
| `package.searchpath (name, path [, sep [, rep]])` | Searches for a file |

### 6.4 – String Manipulation

| Function | Description |
|----------|-------------|
| `string.byte (s [, i [, j]])` | Returns byte codes |
| `string.char (···)` | Returns string from byte codes |
| `string.dump (function [, strip])` | Dumps function to binary |
| `string.find (s, pattern [, init [, plain]])` | Finds pattern |
| `string.format (formatstring, ···)` | Formats string |
| `string.gmatch (s, pattern [, init])` | Global pattern iterator |
| `string.gsub (s, pattern, repl [, n])` | Global substitution |
| `string.len (s)` | Returns string length |
| `string.lower (s)` | Converts to lowercase |
| `string.match (s, pattern [, init])` | Pattern match |
| `string.pack (fmt, v1, v2, ···)` | Packs values to binary |
| `string.packsize (fmt)` | Returns pack size |
| `string.rep (s, n [, sep])` | Repeats string |
| `string.reverse (s)` | Reverses string |
| `string.sub (s, i [, j])` | Returns substring |
| `string.unpack (fmt, s [, pos])` | Unpacks binary |
| `string.upper (s)` | Converts to uppercase |

### 6.5 – UTF-8 Support

| Function/Constant | Description |
|-------------------|-------------|
| `utf8.char (···)` | Returns string from codepoints |
| `utf8.charpattern` | Pattern for UTF-8 character |
| `utf8.codepoint (s [, i [, j [, lax]]])` | Returns codepoints |
| `utf8.codes (s [, lax])` | Codepoint iterator |
| `utf8.len (s [, i [, j [, lax]]])` | Returns UTF-8 length |
| `utf8.offset (s, n [, i])` | Returns byte offset |

### 6.6 – Table Manipulation

| Function | Description |
|----------|-------------|
| `table.concat (list [, sep [, i [, j]]])` | Concatenates table |
| `table.insert (list, [pos,] value)` | Inserts element |
| `table.move (a1, f, e, t [,a2])` | Moves elements |
| `table.pack (···)` | Packs arguments to table |
| `table.remove (list [, pos])` | Removes element |
| `table.sort (list [, comp])` | Sorts table |
| `table.unpack (list [, i [, j]])` | Unpacks table |

### 6.7 – Mathematical Functions

| Function/Constant | Description |
|-------------------|-------------|
| `math.abs (x)` | Absolute value |
| `math.acos (x)` | Arc cosine |
| `math.asin (x)` | Arc sine |
| `math.atan (y [, x])` | Arc tangent |
| `math.ceil (x)` | Ceiling |
| `math.cos (x)` | Cosine |
| `math.deg (x)` | Radians to degrees |
| `math.exp (x)` | Exponential |
| `math.floor (x)` | Floor |
| `math.fmod (x, y)` | Float modulo |
| `math.huge` | Infinity |
| `math.log (x [, base])` | Logarithm |
| `math.max (x, ···)` | Maximum |
| `math.maxinteger` | Maximum integer |
| `math.min (x, ···)` | Minimum |
| `math.mininteger` | Minimum integer |
| `math.modf (x)` | Integer and fractional parts |
| `math.pi` | Pi |
| `math.rad (x)` | Degrees to radians |
| `math.random ([m [, n]])` | Random number |
| `math.randomseed ([x [, y]])` | Sets random seed |
| `math.sin (x)` | Sine |
| `math.sqrt (x)` | Square root |
| `math.tan (x)` | Tangent |
| `math.tointeger (x)` | Converts to integer |
| `math.type (x)` | Returns number type |
| `math.ult (m, n)` | Unsigned comparison |

### 6.8 – Input and Output Facilities

| Function | Description |
|----------|-------------|
| `io.close ([file])` | Closes file |
| `io.flush ()` | Flushes output |
| `io.input ([file])` | Sets input file |
| `io.lines ([filename, ···])` | Line iterator |
| `io.open (filename [, mode])` | Opens file |
| `io.output ([file])` | Sets output file |
| `io.popen (prog [, mode])` | Opens process |
| `io.read (···)` | Reads from input |
| `io.tmpfile ()` | Temporary file |
| `io.type (obj)` | Returns file type |
| `io.write (···)` | Writes to output |

#### File Methods

| Method | Description |
|--------|-------------|
| `file:close ()` | Closes file |
| `file:flush ()` | Flushes file |
| `file:lines (···)` | Line iterator |
| `file:read (···)` | Reads from file |
| `file:seek ([whence [, offset]])` | Sets/gets position |
| `file:setvbuf (mode [, size])` | Sets buffering |
| `file:write (···)` | Writes to file |

### 6.9 – Operating System Facilities

| Function | Description |
|----------|-------------|
| `os.clock ()` | CPU time used |
| `os.date ([format [, time]])` | Date/time string |
| `os.difftime (t2, t1)` | Time difference |
| `os.execute ([command])` | Executes command |
| `os.exit ([code [, close]])` | Exits program |
| `os.getenv (varname)` | Gets environment variable |
| `os.remove (filename)` | Removes file |
| `os.rename (oldname, newname)` | Renames file |
| `os.setlocale (locale [, category])` | Sets locale |
| `os.time ([table])` | Returns time |
| `os.tmpname ()` | Temporary filename |

### 6.10 – The Debug Library

| Function | Description |
|----------|-------------|
| `debug.debug ()` | Interactive debugger |
| `debug.gethook ([thread])` | Gets hook |
| `debug.getinfo (f [, what])` | Gets function info |
| `debug.getlocal ([thread,] f, local)` | Gets local variable |
| `debug.getmetatable (value)` | Gets metatable |
| `debug.getregistry ()` | Gets registry |
| `debug.getupvalue (f, up)` | Gets upvalue |
| `debug.getuservalue (u, n)` | Gets user value |
| `debug.sethook ([thread,] hook, mask [, count])` | Sets hook |
| `debug.setlocal ([thread,] level, local, value)` | Sets local |
| `debug.setmetatable (value, table)` | Sets metatable |
| `debug.setupvalue (f, up, value)` | Sets upvalue |
| `debug.setuservalue (udata, value, n)` | Sets user value |
| `debug.traceback ([thread,] [message [, level]])` | Traceback |
| `debug.upvalueid (f, n)` | Gets upvalue id |
| `debug.upvaluejoin (f1, n1, f2, n2)` | Joins upvalues |

---

## 7 – Lua Standalone

Usage: `lua [options] [script [args]]`

Options:
- `-e stat`: execute string 'stat'
- `-i`: enter interactive mode after running script
- `-l mod`: require library 'mod' into global 'mod'
- `-l g=mod`: require library 'mod' into global 'g'
- `-v`: show version information
- `-E`: ignore environment variables
- `-W`: turn warnings on
- `--`: stop handling options

---

## 8 – Incompatibilities with the Previous Version

### 8.1 – Incompatibilities in the Language

- The new keyword `global` is now reserved.
- Coercions between strings and numbers are more restrictive.

### 8.2 – Incompatibilities in the Libraries

(Details in official documentation)

### 8.3 – Incompatibilities in the API

- New functions: `lua_pushexternalstring`, `lua_numbertocstring`, `lua_closethread`
- Changed semantics for some functions

---

## 9 – The Complete Syntax of Lua

```ebnf
chunk ::= block

block ::= {stat} [retstat]

stat ::=  ';' | 
     varlist '=' explist | 
     functioncall | 
     label | 
     break | 
     goto Name | 
     do block end | 
     while exp do block end | 
     repeat block until exp | 
     if exp then block {elseif exp then block} [else block] end | 
     for Name '=' exp ',' exp [',' exp] do block end | 
     for namelist in explist do block end | 
     function funcname funcbody | 
     local function Name funcbody | 
     local attnamelist ['=' explist] |
     global namelist ['=' explist]

attnamelist ::=  Name attrib {',' Name attrib}

attrib ::= ['<' Name '>']

retstat ::= return [explist] [';']

label ::= '::' Name '::'

funcname ::= Name {'.' Name} [':' Name]

varlist ::= var {',' var}

var ::=  Name | prefixexp '[' exp ']' | prefixexp '.' Name 

namelist ::= Name {',' Name}

explist ::= exp {',' exp}

exp ::=  nil | false | true | Numeral | LiteralString | '...' | functiondef | 
     prefixexp | tableconstructor | exp binop exp | unop exp

prefixexp ::= var | functioncall | '(' exp ')'

functioncall ::=  prefixexp args | prefixexp ':' Name args 

args ::=  '(' [explist] ')' | tableconstructor | LiteralString 

functiondef ::= function funcbody

funcbody ::= '(' [parlist] ')' block end

parlist ::= namelist [',' '...'] | '...'

tableconstructor ::= '{' [fieldlist] '}'

fieldlist ::= field {fieldsep field} [fieldsep]

field ::= '[' exp ']' '=' exp | Name '=' exp | exp

fieldsep ::= ',' | ';'

binop ::=  '+' | '-' | '*' | '/' | '//' | '^' | '%' | 
     '&' | '~' | '|' | '>>' | '<<' | '..' | 
     '<' | '<=' | '>' | '>=' | '==' | '~=' | 
     and | or

unop ::= '-' | not | '#' | '~'
```

---

## Index of Key Changes in Lua 5.5

### New Features
- `global` keyword for explicit global variable declarations
- Type annotations (currently ignored, for documentation)
- `lua_pushexternalstring` - Push externally managed strings
- `lua_numbertocstring` - Convert Lua number to C string
- `lua_closethread` - Close a thread/coroutine

### Breaking Changes
- `global` is now a reserved word
- Stricter string-number coercions

---

Last updated: 2025

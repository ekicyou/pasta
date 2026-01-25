# Requirements Document

## Introduction
Pastaプロジェクトの `pasta_lua` クレートにおけるLuaスクリプトのコーディング規約を策定し、AIエージェントが参照可能な形式でステアリング配置する。バグ予防、コード品質向上、一貫性維持を目的とする。

## Project Description (Input)
Luaスクリプトのコーディングルールを決めておいて、バグがなるべく出ないようにする。AI参照用のコーディング規約を作ってステアリングに配置する。

**追加要件**: 作成したコーディング規約に従い、既存のコードのリファクタリングを行う。

## Requirements

### Requirement 1: コーディング規約ドキュメント作成
**Objective:** As a AI開発エージェント, I want Luaコーディング規約がステアリングに配置されている, so that 一貫したコードスタイルでLuaスクリプトを生成・修正できる

#### Acceptance Criteria
1. The steering system shall provide a `lua-coding.md` file in `.kiro/steering/` directory
2. When AI agent reads `.kiro/steering/lua-coding.md`, the steering system shall provide comprehensive Lua coding rules including naming conventions, module patterns, and error handling
3. The steering system shall document Pasta-specific Lua patterns (PASTA module API, EmmyLua annotations, メタテーブル設計)

### Requirement 2: 命名規約
**Objective:** As a 開発者, I want 明確な命名規約が定義されている, so that コードの可読性と一貫性が向上する

#### Acceptance Criteria
1. The coding standard shall define snake_case for local variables and functions
2. The coding standard shall define UPPER_CASE for module table names matching the module filename (e.g., `actor.lua` → `ACTOR`)
3. The coding standard shall define `_IMPL` suffix for class implementation metatables (e.g., `ACTOR_IMPL`)
4. When defining private module members, the coding standard shall require underscore prefix (e.g., `_internal_func`)
5. The coding standard shall permit Japanese identifiers for domain-specific terms (e.g., `アクター名`, `シーン`)

### Requirement 3: モジュール構造規約
**Objective:** As a 開発者, I want モジュールの構造パターンが標準化されている, so that 循環参照を防ぎ保守性が向上する

#### Acceptance Criteria
1. The coding standard shall require each module to define a single module table named after the filename in UPPER_CASE
2. The coding standard shall require all require statements at the top of the file
3. The coding standard shall require modules to return their main table at the end
4. If a module requires another module that could cause circular dependency, then the coding standard shall require using `pasta.store` pattern for shared state
5. The coding standard shall provide the canonical module structure template

### Requirement 4: クラス設計パターン
**Objective:** As a 開発者, I want Rust風の明示的なクラス設計パターンが定義されている, so that クラスとシングルトンの混同を防ぎ、インスタンス生成が明確になる

#### Acceptance Criteria
1. The coding standard shall require separating module table (`MODULE`) from class implementation metatable (`MODULE_IMPL`)
2. The coding standard shall require `MODULE.new(args)` as the constructor function that creates and returns new instances
3. When defining class methods, the coding standard shall require explicit `self` parameter with dot syntax (`function MODULE_IMPL.method(self, arg)`) instead of colon syntax
4. The coding standard shall require `setmetatable(obj, { __index = MODULE_IMPL })` pattern in constructor
5. The coding standard shall require module-level functions (`MODULE.func(args)`) to be separate from instance methods
6. When singleton state is needed, the coding standard shall require using the module table itself or module-local variables (leveraging Lua's `require` caching behavior)
7. The coding standard shall prohibit `MODULE.instance()` anti-pattern; use module-as-singleton or `pasta.store` pattern instead

#### Singleton Pattern (via require caching)
```lua
--- @module pasta.config
--- This module is a singleton (module table itself holds state)
local CONFIG = {
    debug = false,
    log_level = "info",
}

--- Module-local state (private singleton data)
local _cache = {}

--- @param key string
--- @param value any
function CONFIG.set(key, value)
    CONFIG[key] = value
end

--- @param key string
--- @return any
function CONFIG.get(key)
    return CONFIG[key]
end

return CONFIG
-- Usage: local CONFIG = require("pasta.config")
-- CONFIG is always the same instance due to require caching
```

#### Class Pattern Template
```lua
--- @module pasta.example
local EXAMPLE = {}

--- @class Example
--- @field name string
--- @field value number
local EXAMPLE_IMPL = {}
EXAMPLE_IMPL.__index = EXAMPLE_IMPL

--- Create new Example instance
--- @param name string
--- @param value number
--- @return Example
function EXAMPLE.new(name, value)
    local obj = {
        name = name,
        value = value,
    }
    setmetatable(obj, EXAMPLE_IMPL)
    return obj
end

--- Instance method (explicit self, dot syntax)
--- @param self Example
--- @param delta number
--- @return number
function EXAMPLE_IMPL.add(self, delta)
    self.value = self.value + delta
    return self.value
end

--- Module-level utility function (not instance method)
--- @param a Example
--- @param b Example
--- @return Example
function EXAMPLE.merge(a, b)
    return EXAMPLE.new(a.name .. b.name, a.value + b.value)
end

return EXAMPLE
```

### Requirement 5: EmmyLua型アノテーション規約
**Objective:** As a 開発者, I want 型アノテーション規約が定義されている, so that IDE補完とドキュメント生成が向上する

#### Acceptance Criteria
1. The coding standard shall require `@module` annotation at the top of each module file
2. The coding standard shall require `@class` annotation for class-like tables
3. The coding standard shall require `@param` and `@return` annotations for all public functions
4. The coding standard shall require `@field` annotations for class properties
5. When function may return nil, the coding standard shall require `|nil` in return type annotation
6. The coding standard shall require `@param ... type` syntax for variadic functions (not `@vararg`)

### Requirement 6: エラーハンドリング規約
**Objective:** As a 開発者, I want エラーハンドリングパターンが標準化されている, so that デバッグと障害対応が容易になる

#### Acceptance Criteria
1. The coding standard shall define nil-check patterns before table/function access
2. When accessing potentially nil tables, the coding standard shall require guard clause pattern
3. The coding standard shall define pcall usage pattern for external/risky operations
4. The coding standard shall prohibit silent nil returns without explicit documentation

### Requirement 7: Pastaランタイム固有規約
**Objective:** As a 開発者, I want Pasta固有のLuaパターンが文書化されている, so that ランタイムとの一貫した統合ができる

#### Acceptance Criteria
1. The coding standard shall document PASTA module API usage patterns (`PASTA.create_actor`, `PASTA.create_scene`, `PASTA.create_word`)
2. The coding standard shall document CTX (context) object patterns and lifecycle
3. The coding standard shall document ACT (action) object patterns for scene functions
4. The coding standard shall document PROXY patterns for actor-action interaction
5. The coding standard shall document STORE patterns for shared state management

### Requirement 8: 既存コードリファクタリング
**Objective:** As a 開発者, I want 既存のLuaコードが新しい規約に準拠している, so that コードベース全体の一貫性が保たれる

#### Acceptance Criteria
1. When refactoring `scripts/pasta/*.lua`, the refactoring system shall ensure EmmyLua annotations are complete and consistent
2. When refactoring, the refactoring system shall ensure all public functions have proper documentation
3. When refactoring, the refactoring system shall ensure naming conventions are followed (MODULE, MODULE_IMPL pattern)
4. When refactoring class-like modules, the refactoring system shall apply Requirement 4 class design pattern (separate MODULE from MODULE_IMPL, explicit self, dot syntax)
5. When refactoring, the refactoring system shall convert colon syntax (`:`) to dot syntax with explicit `self` parameter
6. The refactoring system shall preserve existing behavior (no functional changes)
7. After refactoring, the test system shall pass all existing tests without modification

#### Refactoring Target Files
The following files require class pattern refactoring:

| File | Current Issue | Required Change |
|------|---------------|-----------------|
| `act.lua` | ACT is class-like but used as singleton | Separate ACT/ACT_IMPL, use ACT.new() |
| `actor.lua` | ACTOR mixes module and class functions | Separate ACTOR/ACTOR_IMPL, explicit self |
| `ctx.lua` | CTX has .new() but uses colon syntax | Convert to dot syntax with explicit self |
| `scene.lua` | MOD naming, mixed patterns | Rename to SCENE, clarify structure |
| `word.lua` | MOD naming, WordBuilder pattern | Rename to WORD, apply _IMPL pattern |
| `store.lua` | Singleton pattern is correct | Minor naming alignment |
| `global.lua` | Simple table, no class | No major changes needed |
| `init.lua` | Entry point, no class | No major changes needed |

### Requirement 9: テストとLint規約
**Objective:** As a 開発者, I want Luaテストとlint規約が定義されている, so that コード品質を自動検証できる

#### Acceptance Criteria
1. The coding standard shall document lua_test framework usage patterns (`expect`, `describe`, `it`)
2. The coding standard shall define test file naming convention (`*_test.lua` or `*_spec.lua`)
3. The coding standard shall provide test structure template (describe/it pattern)
4. Where luacheck is available, the coding standard shall document recommended luacheck configuration

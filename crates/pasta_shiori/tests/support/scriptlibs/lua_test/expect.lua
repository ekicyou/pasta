local toDebugString = require("lua_test.toDebugString")

---@class Expectation
---@field value any
---@field negated boolean
---@field not_ Expectation
---@overload fun(value: any): Expectation
local expect = {}

---@param sub any
---@param sup any
---@return boolean
local function isSubtypeOfOrEqualsTo(sub, sup)
    if sub == sup then
        return true
    elseif type(sub) ~= type(sup) then
        return false
    end

    if type(sub) ~= "table" then
        return sub == sup
    end

    for key, value in pairs(sub) do
        if not isSubtypeOfOrEqualsTo(value, sup[key]) then
            return false
        end
    end

    return true
end

---@param a any
---@param b any
---@return boolean
local function equals(a, b)
    return isSubtypeOfOrEqualsTo(a, b) and isSubtypeOfOrEqualsTo(b, a)
end

---@param value any
---@return boolean
local function isExpectation(value)
    if type(value) ~= "table" then
        return false
    end

    local meta = getmetatable(value)
    while meta ~= nil do
        if meta.__index == expect then
            return true
        end

        meta = getmetatable(meta.__index)
    end

    return false
end

---@param value any
local function assertExpectation(value)
    assert(
        isExpectation(value),
        "Received value seems not to be Expectation.\nPerhaps you are using \".\" instead of \":\" to call the method?"
    )
end

---@private
---@param pass boolean
---@param message string
---@param negMessage string
function expect.assert(self, pass, message, negMessage)
    if not self.negated then
        assert(pass, message)
    else
        assert(not pass, negMessage)
    end
end

---Expects `==` equality.
---@param expected any
function expect.toBe(self, expected)
    assertExpectation(self)

    self:assert(
        self.value == expected,
        string.format(
            "expect(received):toBe(expected)\nExpected: %s\nReceived: %s",
            toDebugString(expected),
            toDebugString(self.value)
        ),
        string.format(
            "expect(received).not_:toBe(expected)\nExpected not: %s\nReceived: %s",
            toDebugString(expected),
            toDebugString(self.value)
        )
    )
end

---Expects approximate equality of floating numbers.
---@param number number
---@param numDigits integer?
function expect.toBeCloseTo(self, number, numDigits)
    assertExpectation(self)

    if numDigits == nil then
        numDigits = 2
    end

    local inf = 1 / 0

    if self.value == inf and number == inf then
        --pass
        return
    elseif self.value == -inf and number == -inf then
        --pass
        return
    end

    local received = math.abs(self.value - number)
    local expected = (10 ^ -(numDigits)) / 2

    self:assert(
        received < expected,
        string.format(
            "expect(received):toBeCloseTo\nExpected diff: <%f\nReceived:%f",
            expected,
            received
        ),
        string.format(
            "expect(received).not_toBeCloseTo\nExpected diff: >%f\nReceived:%f",
            expected,
            received
        )
    )
end

---Expects falsy values: false and nil.
function expect.toBeTruthy(self)
    assertExpectation(self)

    self:assert(
        not not self.value,
        string.format("expect(received):toBeTruthy()\nReceived: %s", toDebugString(self.value)),
        string.format("expect(received).not_:toBeTruthy()\nReceived: %s", toDebugString(self.value))
    )
end

---Expects falsy values: false and nil.
function expect.toBeFalsy(self)
    assertExpectation(self)

    self:assert(
        not self.value,
        string.format("expect(received):toBeFalsy()\nReceived: %s", toDebugString(self.value)),
        string.format("expect(received).not_:toBeFalsy()\nReceived: %s", toDebugString(self.value))
    )
end

---@param another any
function expect.toBeGraterThan(self, another)
    assertExpectation(self)

    self:assert(
        self.value > another,
        string.format(
            "expect(received):toBeGraterThan(number)\nExpected: >%s\nReceived: %s",
            toDebugString(another),
            toDebugString(self.value)
        ),
        string.format(
            "expect(received).not_:toBeGraterThan(number)\nExpected: <=%s\nReceived: %s",
            toDebugString(another),
            toDebugString(self.value)
        )
    )
end

---@param another any
function expect.toBeGraterThanOrEqual(self, another)
    assertExpectation(self)

    self:assert(
        self.value >= another,
        string.format(
            "expect(received):toBeGraterThanOrEqual(number)\nExpected: >=%s\nReceived: %s",
            toDebugString(another),
            toDebugString(self.value)
        ),
        string.format(
            "expect(received).not_:toBeGraterThanOrEqual(number)\nExpected: <%s\nReceived: %s",
            toDebugString(another),
            toDebugString(self.value)
        )
    )
end

---@param another any
function expect.toBeLessThan(self, another)
    assertExpectation(self)

    self:assert(
        self.value < another,
        string.format(
            "expect(received):toBeLessThan(number)\nExpected: <%s\nReceived: %s",
            toDebugString(another),
            toDebugString(self.value)
        ),
        string.format(
            "expect(received).not_:toBeLessThan(number)\nExpected: >=%s\nReceived: %s",
            toDebugString(another),
            toDebugString(self.value)
        )
    )
end

---@param another any
function expect.toBeLessThanOrEqual(self, another)
    assertExpectation(self)

    self:assert(
        self.value <= another,
        string.format(
            "expect(received):toBeLessThanOrEqual(number)\nExpected: <=%s\nReceived: %s",
            toDebugString(another),
            toDebugString(self.value)
        ),
        string.format(
            "expect(received).not_:toBeLessThanOrEqual(number)\nExpected: >%s\nReceived: %s",
            toDebugString(another),
            toDebugString(self.value)
        )
    )
end

function expect.toBeNil(self)
    assertExpectation(self)

    self:assert(
        self.value == nil,
        string.format(
            "expect(received):toBeNil()\nReceived: %s",
            toDebugString(self.value)
        ),
        string.format(
            "expect(received).not_:toBeNil()\nReceived: %s",
            toDebugString(self.value)
        )
    )
end

---Expects table (array) to contain a item as `==` equality.
---@param item any
function expect.toContain(self, item)
    assertExpectation(self)

    local pass = false
    if type(self.value) == "table" then
        for _, v in ipairs(self.value) do
            if v == item then
                pass = true
                break
            end
        end
    end

    self:assert(
        pass,
        string.format(
            "expect(received):toContain(item)\nReceived: %s",
            toDebugString(self.value)
        ),
        string.format(
            "expect(received).not_:toContain(item)\nReceived: %s",
            toDebugString(self.value)
        )
    )
end

---@param self Expectation
---@param another any
function expect.toEqual(self, another)
    assertExpectation(self)

    self:assert(
        equals(self.value, another),
        string.format(
            "expect(received):toEqual(item)\nExpected: %s\nReceived: %s",
            toDebugString(another),
            toDebugString(self.value)
        ),
        string.format(
            "expect(received).not_:toEqual(item)\nExpected: not %s\nReceived: %s",
            toDebugString(another),
            toDebugString(self.value)
        )
    )
end

---@param self Expectation
---@param length number
function expect.toHaveLength(self, length)
    assertExpectation(self)

    self:assert(
        #self.value == length,
        string.format(
            "expect(received):toHaveLength(length)\nExpected: %d\nReceived: %d, %s",
            length,
            #self.value,
            toDebugString(self.value)
        ),
        string.format(
            "expect(received).not_:toHaveLength(length)\nExpected: not %d\nReceived: %d, %s",
            length,
            #self.value,
            toDebugString(self.value)
        )
    )
end

---@param self Expectation
---@param pattern string
function expect.toMatch(self, pattern)
    assertExpectation(self)

    assert(
        type(self.value) == "string",
        string.format(
            "expect(received):toMatch(pattern)\nReceived value must be string.\nReceived: %s",
            self.value
        )
    )
    assert(
        type(pattern) == "string",
        string.format(
            "expect(received):toMatch(pattern)\nPattern must be string.",
            self.value
        )
    )

    self:assert(
        string.find(self.value, pattern) ~= nil,
        string.format(
            "expect(received):toMatch(pattern)\nPattern: %s\nReceived: %s",
            pattern,
            self.value
        ),
        string.format(
            "expect(received).not_:toMatch(pattern)\nPattern: %s\nReceived: %s",
            pattern,
            self.value
        )
    )
end

---@param self Expectation
---@param object any
function expect.toMatchObject(self, object)
    assertExpectation(self)

    self:assert(
        isSubtypeOfOrEqualsTo(object, self.value),
        string.format(
            "expect(received).toMatchObject(object)\nExpected: supertype of or equals to %s\nReceived: %s",
            toDebugString(object),
            toDebugString(self.value)
        ),
        string.format(
            "expect(received).toMatchObject(object)\nExpected: subtype of %s\nReceived: %s",
            toDebugString(object),
            toDebugString(self.value)
        )
    )
end

setmetatable(expect --[[@as table]], {
    __call = function(_, value)
        local obj = { value = value, negated = false }
        local notObj = { negated = true }
        obj.not_, notObj.not_ = notObj, obj

        setmetatable(obj, { __index = expect })
        setmetatable(notObj, { __index = obj })

        return obj
    end
})

return expect

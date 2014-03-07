
Vec2D = {}

function Vec2D.new(a, b)
    local tmp = {
        x = a,
        y = b
    }
    return setmetatable(tmp, { __index = Vec2D })
end

function Vec2D:add_assign(v)
    self.x = self.x + v.x
    self.y = self.y + v.y
    return self
end

function Vec2D:to_s()
    return "Vec2D.new(" .. self.x .. ", " .. self.y .. ")"
end

function foo()
	local v = Vec2D.new(0, 0)
	local u = Vec2D.new(1, 2)
	local i = 0
	while i<5000000 do
		v = v:add_assign(u)
		i = i + 1
	end
	
	print(v:to_s())
end

foo()

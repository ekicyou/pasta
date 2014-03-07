
class Vec2D
	def initialize(x, y)
		@x = x
		@y = y
	end
	
	attr_accessor :x
	attr_accessor :y
	
	def +(v)
		Vec2D.new(@x+v.x, @y+v.y)
	end
	
	def add_assign(v)
		@x += v.x
		@y += v.y
		return self
	end
	
	def to_s
		"Vec2D.new(x=#{@x}, y=#{@y})"
	end
end


def foo()
	v = Vec2D.new(0, 0)
	u = Vec2D.new(1, 2)
	i = 0
	while i<5000000
		#v = v + u
		v = v.add_assign(u)
		i += 1
	end
	
	puts v
end

foo

GC.start

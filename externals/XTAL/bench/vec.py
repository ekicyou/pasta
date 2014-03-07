
class Vec2D:
	def __init__(self, x=0, y=0):
		self.x = x
		self.y = y
	
	def __add__(self, v):
		return Vec2D(self.x+v.x, self.y+v.y)
		
	def __iadd__(self, v):
		self.x += v.x
		self.y += v.y
		return self

	def __str__(self):
		return "Vec2D(x=%d, y=%d)" % (self.x, self.y)


def foo():
	v = Vec2D(0, 0)
	u = Vec2D(1, 2)
	i = 0
	while i<5000000:
		v += u
		i += 1
	
	print v



foo()

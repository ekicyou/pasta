
class Foo:
	def bar(self):
		i = 0
		while i<10000000:
			i += 1
		
		return 0

print Foo().bar()
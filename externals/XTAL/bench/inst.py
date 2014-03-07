
class Foo:
	def bar(self):
		self.inst1 = 0
		self.inst2 = 0
		self.inst3 = 0
		self.inst4 = 0
		i = 0
		while i<10000000:
			self.inst1 += 1
			self.inst2 += 2
			self.inst3 += 3
			self.inst4 += 4
			i += 1
		
		return self.inst1

print Foo().bar()
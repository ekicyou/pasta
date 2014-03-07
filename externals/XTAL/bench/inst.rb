
class Foo

	def bar
		@inst1 = 0
		@inst2 = 0
		@inst3 = 0
		@inst4 = 0
		i = 0
		while i<10000000
			@inst1 += 1
			@inst2 += 2
			@inst3 += 3
			@inst4 += 4
			i += 1
		end
		
		return @inst1
	end

end

puts Foo.new.bar


class Foo

	def bar
		i = 0
		while i<10000000
			i += 1
		end
		
		return 0
	end

end

puts Foo.new.bar

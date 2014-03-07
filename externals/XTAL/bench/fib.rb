
def fib(i)
	if i<2
		return 1
	end
	return fib(i-1) + fib(i-2)
end

puts fib(33)

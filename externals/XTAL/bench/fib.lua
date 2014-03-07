
function fib(i)
	if i<2 then
		return 1
	end
	return fib(i-1) + fib(i-2)
end

print(fib(33))

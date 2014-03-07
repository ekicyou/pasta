
function nested_loops(n)
	local x = 0;
	local a = 0
	while a<n do
		local b = 0
		while b<n do
			local c = 0
			while c<n do
				local d = 0
				while d<n do
					local e = 0
					while e<n do
						local f = 0
						while f<n do
							x = x + 1
							f = f + 1
						end
						e = e + 1
					end
					d = d + 1
				end
				c = c + 1
			end
			b = b + 1
		end
		a = a + 1
	end
	return x
end

print(nested_loops(20))

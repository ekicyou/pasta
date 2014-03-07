def nested_loops(n)
	x = 0;
	a = 0
	while a<n
		b = 0
		while b<n
			c = 0
			while c<n
				d = 0
				while d<n
					e = 0
					while e<n
						f = 0
						while f<n
							x += 1;
							f += 1
						end
						e += 1
					end
					d += 1
				end
				c += 1
			end
			b += 1
		end
		a += 1
	end
	return x;
end

puts nested_loops(20)

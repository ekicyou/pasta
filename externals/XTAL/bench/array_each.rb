
ary = []
i = 0
while i<3000000
	ary.push(i)
	i += 1
end

ret = 0;
ary.each{ |it|
	ret = it
}

puts ret


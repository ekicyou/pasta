all: unicode/CombiningClassData.hx haxedoc.xml

UnicodeData.txt:
	wget http://www.unicode.org/Public/5.2.0/ucd/UnicodeData.txt

unicode/CombiningClassData.hx: UnicodeData.txt
	sh mkCombiningClassData.sh > unicode/CombiningClassData.hx

haxedoc.xml:
	haxe -xml haxedoc.xml unicode.UnicodeTools unicode.CombiningTools

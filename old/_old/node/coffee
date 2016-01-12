#!/bin/sh
basedir=`dirname "$0"`

case `uname` in
    *CYGWIN*) basedir=`cygpath -w "$basedir"`;;
esac

if [ -x "$basedir/node" ]; then
  "$basedir/node"  "$basedir/node_modules/coffee-script/bin/coffee" "$@"
  ret=$?
else 
  node  "$basedir/node_modules/coffee-script/bin/coffee" "$@"
  ret=$?
fi
exit $ret

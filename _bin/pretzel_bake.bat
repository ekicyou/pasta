setlocal
pushd ..
  rmdir /Q /S _site
  rmdir /Q /S _site
  rmdir /Q /S _site
  _bin\pretzel bake < NUL
  robocopy /MIR _site _out /XD .git
  popd

pushd ..\_out
  echo nojekyll > .nojekyll
  git add --all
  git commit -a -m "sync %COMPUTERNAME%"
  popd

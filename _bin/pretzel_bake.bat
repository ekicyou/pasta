setlocal
pushd ..
  rmdir /Q /S _site
  _bin\pretzel bake < NUL
  robocopy /MIR _site ..\ref\site-neith-pages /XD .git
  popd

pushd ..\..\ref\site-neith-pages
  echo nojekyll > .nojekyll
  git add --all
  git commit -a -m "sync %COMPUTERNAME%"
  popd

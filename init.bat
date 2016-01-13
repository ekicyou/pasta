setlocal
pushd %~dp0
  rmdir /Q /S site
  rmdir /Q /S site
  rmdir /Q /S site
      bin\pretzel create --source=site --withproject --template=razor  --wiki
  rem bin\pretzel create --source=site --withproject --template=liquid

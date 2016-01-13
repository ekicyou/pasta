setlocal
pushd %~dp0
  bin\pretzel taste --source=site --destination=..\bake --port 5000

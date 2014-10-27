setlocal
pushd %~dp0

  robocopy /MIR duktape\ %1\duktape\
  robocopy /MIR lib\     %1\lib\
  robocopy /MIR dic\     %1\dic\
  popd

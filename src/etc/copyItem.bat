setlocal
pushd %~dp0

  robocopy /MIR lib\     %1\lib\
  robocopy /MIR dic\     %1\dic\
  popd

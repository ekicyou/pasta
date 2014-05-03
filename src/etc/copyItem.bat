setlocal
pushd %~dp0

  robocopy /MIR js\      %1\js\
  robocopy /MIR jscache\ %1\jscache\
  robocopy /MIR dic\     %1\dic\
  popd

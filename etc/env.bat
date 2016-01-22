pushd %~dp0
  set PATH_ETC=%cd%

  pushd ..
    set PATH_ROOT=%cd%

    pushd node_modules
      set PATH_NODE_MODULES=%cd%
        pushd .bin
          set PATH_NODE_MODULES_BIN=%cd%
          popd
      popd

    pushd src
      set PATH_SRC=%cd%
      popd

    popd
  popd

set PATH=%PATH%;%PATH_NODE_MODULES_BIN%

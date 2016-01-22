@rem 参考サイト
@rem 今さらBrowserifyを使ってみる
@rem http://qiita.com/kazukitash/items/9cad31b7fa1d6dcca8b9

setlocal
pushd %~dp0
  set PATH=%PATH%;%~dp0node_modules\.bin
  pushd gulpfiles\tasks
    call gulp build

path    = require 'path'
current = process.cwd()
source  = current + '/src'
dist    = current + '/dist'

module.exports =
  # 入力元の設定
  es6:      source + '/js/**/[^_]*.js'
  jade:     source + '/**/*.jade'
  jadeBase: 'src'

  # 出力先の設定
  es5:  dist + '/js'
  html: dist

  # browserifyの設定
  browserify:
    extensions: ['.js']

  # browserSyncの設定
  browserSync:
    server:
      baseDir: dist
    port: 3000

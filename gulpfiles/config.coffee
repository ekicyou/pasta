path    = require 'path'
current = process.cwd()
source  = current + '/src'
dist    = current + '/dist'

module.exports =
  # “ü—ÍŒ³‚Ìİ’è
  es6:      source + '/js/**/[^_]*.js'
  jade:     source + '/**/*.jade'
  jadeBase: 'src'

  # o—Íæ‚Ìİ’è
  es5:  dist + '/js'
  html: dist

  # browserify‚Ìİ’è
  browserify:
    extensions: ['.js']

  # browserSync‚Ìİ’è
  browserSync:
    server:
      baseDir: dist
    port: 3000
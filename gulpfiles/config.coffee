path    = require 'path'
current = process.cwd()
source  = current + '/src'
dist    = current + '/dist'

module.exports =
  # ���͌��̐ݒ�
  es6:      source + '/js/**/[^_]*.js'
  jade:     source + '/**/*.jade'
  jadeBase: 'src'

  # �o�͐�̐ݒ�
  es5:  dist + '/js'
  html: dist

  # browserify�̐ݒ�
  browserify:
    extensions: ['.js']

  # browserSync�̐ݒ�
  browserSync:
    server:
      baseDir: dist
    port: 3000
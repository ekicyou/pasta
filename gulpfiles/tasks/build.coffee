gulp       = require 'gulp'
config     = require '../config'
browserify = require 'browserify'
babelify   = require 'babelify'
uglifyify  = require 'uglifyify'
stylify    = require 'stylify'
source     = require 'vinyl-source-stream'
plumber    = require 'gulp-plumber'
notify     = require 'gulp-notify'
glob       = require 'glob'
path       = require 'path'

gulp.task 'build', ['jade','md'], ->
  files = glob.sync config.es6
  files.forEach (file) ->
    browserify
      entries:    file
      extensions: config.browserify.extensions
      .transform babelify
      .transform stylify
      .transform {global: true}, uglifyify
      .bundle()
      .pipe source path.basename(file, path.extname(file)) + '.js'
      .pipe gulp.dest config.es5

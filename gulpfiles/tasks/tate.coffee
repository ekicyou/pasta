gulp    = require 'gulp'
config  = require '../config'
plumber = require 'gulp-plumber'
notify  = require 'gulp-notify'


gulp.task 'tate', ->
  gulp
    .src config.tate, base: config.tateBase
    .pipe plumber errorHandler: notify.onError('<%= error.message %>')
    .pipe jade()
    .pipe gulp.dest config.html
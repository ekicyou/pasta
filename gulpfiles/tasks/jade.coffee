gulp    = require 'gulp'
config  = require '../config'
jade    = require 'gulp-jade'
plumber = require 'gulp-plumber'
notify  = require 'gulp-notify'

gulp.task 'jade', ->
  gulp
    .src config.jade, base: config.jadeBase
    .pipe plumber errorHandler: notify.onError('<%= error.message %>')
    .pipe jade()
    .pipe gulp.dest config.html
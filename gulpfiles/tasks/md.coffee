gulp     = require 'gulp'
config   = require '../config'
markdown = require 'gulp-markdown'

gulp.task 'md', ->
  gulp
    .src config.md, base: config.mdBase
    .pipe markdown()
    .pipe gulp.dest config.html

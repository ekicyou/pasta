gulp        = require 'gulp'
path        = require 'path'
config      = require '../config'
browserify  = require 'browserify'
babelify    = require 'babelify'
uglifyify   = require 'uglifyify'
stylify     = require 'stylify'
watchify    = require 'watchify'
source      = require 'vinyl-source-stream'
glob        = require 'glob'
watch       = require 'gulp-watch'
gutil       = require 'gulp-util'
browserSync = require 'browser-sync'
notify      = require 'gulp-notify'

handleErrors = ->
  args = Array.prototype.slice.call(arguments)
  notify
    .onError {title: 'Browserify Error', message: '<%= error.message %>'}
    .apply @, args
  @emit 'end'

gulp.task 'watch', ->
  browserSync config.browserSync

  watch config.jade, -> gulp.start ['jade']
  watch config.browserSync.server.baseDir + '/**/*', -> browserSync.reload()

  files = glob.sync config.es6
  files.forEach (file) ->
    b = browserify
        entries: file
        extensions: config.browserify.extensions
        debug: true
        cache: {}
        packageCache: {}
        plugin: [watchify]
      .transform babelify
      .transform stylify

    bundle = (updatedFile) ->
      b.bundle()
        .on 'error', handleErrors
        .pipe source path.basename(file, path.extname(file)) + '.js'
        .pipe gulp.dest config.es5
      if updatedFile?
        updatedFile.map (filename) -> gutil.log 'File updated:', gutil.colors.yellow filename

    b.on 'update', bundle
    bundle()

var insertCss = require('insert-css');
insertCss(require('../css/index.styl'));

import Cat from './_cat.js';

window.onload = function() {
  var cat = new Cat('Mimi');
  var $content = document.querySelector('#content');
  $content.innerHTML = cat.meow();
}
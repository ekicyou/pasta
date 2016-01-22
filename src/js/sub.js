var insertCss = require('insert-css');
insertCss(require('../css/sub.styl'));

import Cat from './_cat.js';

window.onload = function() {
  var cat = new Cat('Mel');
  var $content = document.querySelector('#content');
  $content.innerHTML = cat.meow();
}
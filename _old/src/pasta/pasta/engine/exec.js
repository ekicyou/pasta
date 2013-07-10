
/* execute
*/


(function() {
  var m, uri;

  uri = document.baseURI;

  if (!(uri != null)) {
    uri = document.URL;
  }

  m = /[\/\\]((\w|-)+)\.html/i.exec(uri);

  require([m[1]]);

}).call(this);

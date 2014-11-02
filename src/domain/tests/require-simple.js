/*
 * https://github.com/gfx/require-simple.js
 *
 * AUTHOR: Fuji, Goro (gfx) <gfuji@cpan.org>
 * LICENSE: The MIT License
 *
 * Usage:
 *     <script src="require-simple.js"></script>
 *     <script>
 *         require.paths.unshift("assets/js");
 *         var Foo = require("foo");
 *     </script>
 */

if (typeof (window) === "undefined") {
    throw new Error("require-simple.js works only on browsers!");
}

(function () {
    "use strict";

    function absname(root, current, name) {
        var id;
        if (name.indexOf("./") === 0) id = current;
        else if (name.indexOf("../") === 0) id = current;
        else id = root;


        if (id.substr(id.length - 1) != '/') {
            id = id.split('/');
            id.length--;
            id = id.join('/');
        }
        else {
            id = id.substr(0, id.length - 1);
        }

        id = id.split('\\');

        while (0 == name.indexOf('../')) {
            id.length--;
            name = name.substr(3);
        }

        id = id.join('/');
        id = id + '/' + name;
        id = id.replace(/\.\//g, "");

        console.trace("[absname]=> [" + id + "] : name=" + name + " current=" + current + " root=" + root);

        return id;
    }

    function findFile(paths, name) {
        for (var i = 0; i < paths.length; ++i) {
            var url = paths[i] + name;
            console.trace("url: ", url);

            var xhr = new XMLHttpRequest();
            xhr.open("GET", url, false);
            xhr.send(null);
            if (require.debug) {
                console.log("require: " + xhr.status + " " + url);
            }

            if (xhr.status === 200 || xhr.status == 0) {
                return xhr.responseText;
            }

            if (xhr.status !== 404 || (i + 1) === paths.length) {
                throw new Error("Cannot load module \"" + name + "\": " + xhr.status);
            }
        }
    }

    function findModule(paths, id) {
        var name = (id.match(/\.js$/) ? id : id + ".js");
        return findFile(paths, name);
    }


    function require_simple(id, current) {
        var name = absname(require_simple.root, current, id);

        if (!require_simple.modules) {
            require_simple.modules = {};
        }
        if (require_simple.modules[name]) {
            if (require.debug) {
                console.log("require: \"" + name + "\" is already loaded");
            }
            return require_simple.modules[name].exports;
        }

        var module = require_simple.modules[name] = {
            id: name,
            exports: {}
        };
        var src = findModule(require_simple.paths, name);

        var m = "require_simple.modules['" + name.replace(/'/g, "\\'") + "']";

        var text = "(function (module, exports) { // " + "[id:" + name + "]\n";
        text += "var require = function(id){ return window.require_simple(id, \"" + name + "\");};\n\n";
        text += src + "\n\n";
        text += "}(" + m + ", " + m + ".exports));\n"

        var srcSection = document.createTextNode(text);

        var script = document.createElement("script");
        script.appendChild(srcSection);

        document.head.appendChild(script);
        return module.exports;
    }

    require_simple.findFile = findFile;
    require_simple.debug = false;
    require_simple.paths = ["."];
    require_simple.root = "/root";

    window.require_simple = require_simple;
    window.require = function (id) { return window.require_simple(id, "/root"); };
})();
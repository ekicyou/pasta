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

    function absname(base, path) {
        if (base.substr(base.length - 1) != '/') {
            base = base.split('/');
            base.length--;
            base = base.join('/');
        }
        else {
            base = base.substr(0, base.length - 1);
        }

        base = base.split('\\');

        while (0 == path.indexOf('../')) {
            base.length--;
            path = path.substr(3);
        }

        base = base.join('/');
        base = base + '/' + path;
        base = base.replace(/\.\//g, "");

        return base;
    }

    function findModule(paths, name) {
        for (var i = 0; i < paths.length; ++i) {
            var url = paths[i] + "/" + (name.match(/\.js$/) ? name : name + ".js");

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
                throw new Error("Cannot load module \"" + name + "\": " +
                                xhr.status);
            }
        }
    }

    function require(id) {
        var current = require.current[require.current.length - 1];
        var name = absname(current, id);

        if (!require.modules) {
            require.modules = {};
        }
        if (require.modules[name]) {
            if (require.debug) {
                console.log("require: \"" + name + "\" is already loaded");
            }
            return require.modules[name].exports;
        }

        var module = require.modules[name] = {
            id: name,
            exports: {}
        };
        var src = findModule(require.paths, name);

        var m = "require.modules['" + name.replace(/'/g, "\\'") + "']";
        var srcSection = document.createTextNode(
            "(function (module, exports) { // " + name + ".js\n" +
            src +
            "\n}(" + m + ", " + m + ".exports));\n"
        );

        var script = document.createElement("script");
        script.appendChild(srcSection);

        document.head.appendChild(script);

        return module.exports;
    }

    require.debug = false;
    require.paths = ["."];
    require.current = ["/base"];

    window.require = require;
})();
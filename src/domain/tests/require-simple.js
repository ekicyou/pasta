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

    function absname(currents, name) {
        var current = currents[require.current.length - 1];
        var root = currents[0];

        var id = current;

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
        var name = absname(require.current, id);

        if (!require.modules) {
            require.modules = {};
        }
        if (require.modules[name]) {
            if (require.debug) {
                console.log("require: \"" + name + "\" is already loaded");
            }
            return require.modules[name].exports;
        }

        require.current.push(name);

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

        require.current.pop();
        return module.exports;
    }

    require.debug = false;
    require.paths = ["."];
    require.current = ["/root"];

    window.require = require;
})();
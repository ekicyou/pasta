/*
 * claudia.js — pasta マニュアルの Claudia テーマのテーマメニュー（manual-claudia-theme タスク 2.5）。
 * mdBook の additional-js として全章と print.html に読み込まれ、book.js の後に実行される。
 *
 * 役割（要件 2.1、design「ThemeMenu」）:
 *   claudia.css が隠す rust・coal・ayu の項目（の li）を、テーマメニューの末尾へ移す。
 *   book.js の矢印キーは隣の li のボタンへ focus() するので、隠れた項目が間にあると
 *   Light から Navy へ進めない。末尾へ寄せれば Auto・Light・Navy の間を矢印キーで移動できる。
 *   book.js の End は最後の li のボタンへ移るが、末尾は隠れた項目なので、見えている最後の項目（Navy）へ移し直す。
 *
 * してはいけないこと（design「Security Considerations」）:
 *   外部への通信、保存してある設定値（テーマの保存値を含む）の読み書き、要素の追加や削除をしない。
 *   項目は消さずに移すだけなので、保存値が rust・coal・ayu でも book.js はそのまま適用する（要件 2.4）。
 *   テーマの保持と初回表示は book.js と book.toml のまま（要件 2.5・2.6）。
 *
 * 検査: book/tools/theme-menu-test.mjs（jsdom）。
 */
(function claudiaThemeMenu() {
    'use strict';

    var list = document.getElementById('mdbook-theme-list');
    if (!list) {
        return;
    }

    // claudia.css で隠すテーマ（mdBook 0.5.x のボタンの id は mdbook-theme-<名前>）
    var HIDDEN = ['rust', 'coal', 'ayu'];
    var isHidden = function (button) {
        return HIDDEN.indexOf(button.id.replace(/^mdbook-theme-/, '')) >= 0;
    };

    // 隠した項目の li をメニューの末尾へ移す（何度実行しても同じ並びになる）
    HIDDEN.forEach(function (name) {
        var button = document.getElementById('mdbook-theme-' + name);
        var li = button && button.parentElement;
        if (li && li.parentElement === list) {
            list.appendChild(li);
        }
    });

    // End: book.js の処理（隠れた最後の項目への focus は効かない）の後で、見えている最後の項目へ移す
    document.addEventListener('keydown', function (e) {
        if (e.key !== 'End' || e.altKey || e.ctrlKey || e.metaKey || e.shiftKey) {
            return;
        }
        if (!list.contains(e.target)) {
            return;
        }
        var buttons = list.querySelectorAll('li > button.theme');
        for (var i = buttons.length - 1; i >= 0; i--) {
            if (!isHidden(buttons[i])) {
                buttons[i].focus();
                return;
            }
        }
    });
})();

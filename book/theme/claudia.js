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
 *   外部への通信、保存してある設定値（テーマの保存値を含む）の書き込み、要素の追加や削除をしない。
 *   保存値は、目次の持ち越し（末尾の claudiaSidebarKeep）が目次の mdbook-sidebar を読むだけで、書かない。
 *   項目は消さずに移すだけなので、保存値が rust・coal・ayu でも book.js はそのまま適用する（要件 2.4）。
 *   テーマの保持と初回表示は book.js と book.toml のまま（要件 2.5・2.6）。
 *
 * 検査: book/tools/theme-menu-test.mjs・book/tools/theme-sidebar-test.mjs（jsdom）。
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

/*
 * 目次の持ち越し（getting-started-story-guide タスク 2.4、要件 13、design「SidebarKeeper」）。
 *
 * mdBook 0.5.x は、幅 1080px 未満では保存値（localStorage の mdbook-sidebar）を見ずに、毎回目次を閉じて始める。
 * 一方で book.js は、幅に関係なく開閉のたびに保存値を visible／hidden に書く。
 * そこで、幅 620px 以上 1080px 未満で保存値が visible のときだけ、閉じて始まった目次を開き直す。
 * 620px 未満（開いた目次が本文を画面の外へ押し出す幅）と 1080px 以上（mdBook が保存値どおりにする幅）には触れない。
 *
 * 開く手順は book.js に任せる（チェックボックスの change で、クラス・ARIA 属性・リンクの tabIndex・保存値がそろう）。
 * 目次の要素の判定は保存値を読むより前に行う（目次の無いページでは保存値に触れない）。
 */
(function claudiaSidebarKeep() {
    'use strict';

    var checkbox = document.getElementById('mdbook-sidebar-toggle-anchor');
    var sidebar = document.getElementById('mdbook-sidebar');
    if (!checkbox || !sidebar || checkbox.checked) {
        return;
    }
    var width = document.body.clientWidth;
    if (width < 620 || width >= 1080) {
        return;
    }
    var stored;
    try {
        stored = localStorage.getItem('mdbook-sidebar');
    } catch (e) {
        return;
    }
    if (stored !== 'visible') {
        return;
    }

    // 開くときのアニメーションを出さない（chrome.css は html:not(.sidebar-resizing) のときだけ目次を動かす）
    var html = document.documentElement;
    html.classList.add('sidebar-resizing');
    sidebar.style.display = '';
    checkbox.checked = true;
    checkbox.dispatchEvent(new Event('change'));
    // 開いた位置を確定させてから外す（確定の前に外すと、外した後の描画でアニメーションが出る）
    void sidebar.offsetHeight;
    html.classList.remove('sidebar-resizing');
})();

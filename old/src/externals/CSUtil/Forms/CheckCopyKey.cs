using System;
using System.Windows.Forms;

namespace CSUtil.Forms
{
    /// <summary>
    /// CTRL+Cボタンを監視し、TRUEになったときにイベントを実行します。
    /// </summary>
    public class CheckCopyKey
    {
        /// <summary>
        /// コピーキーが押されたときのイベント。
        /// </summary>
        public event EventHandler CopyKeyPress;

        /// <summary>
        /// コンストラクタ
        /// </summary>
        public CheckCopyKey()
        {
        }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="control"></param>
        /// <param name="copyKeyPressEvent"></param>
        public CheckCopyKey(Control control, EventHandler copyKeyPressEvent)
        {
            CopyKeyPress += copyKeyPressEvent;
            control.KeyDown += OnKeyDown;
            control.KeyUp += OnKeyUp;
            AddContextMenu(control);
        }

        #region キー監視

        private bool keyC = false;
        private bool keyCtrl = false;

        /// <summary>
        /// KeyDownイベントを監視します。
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        public void OnKeyDown(object sender, KeyEventArgs e)
        {
            SetKey(e.KeyCode, true);
            if (keyC && keyCtrl) {
                if (CopyKeyPress == null) return;
                CopyKeyPress(sender, new EventArgs());
                e.Handled = true;
            }
        }

        /// <summary>
        /// KeyUpイベントを監視します。
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        public void OnKeyUp(object sender, KeyEventArgs e)
        {
            SetKey(e.KeyCode, false);
        }

        private void SetKey(Keys keyCode, bool isDown)
        {
            switch (keyCode) {
                case Keys.C: keyC = isDown; break;
                case Keys.ControlKey: keyCtrl = isDown; break;
            }
        }

        #endregion
        #region 右クリックメニューでコピー
        private ContextMenu popUpMenu;

        private void AddContextMenu(Control control)
        {
            // コンテキストメニューの追加
            popUpMenu = new ContextMenu();
            popUpMenu.MenuItems.Add(new MenuItem("コピー", CopyMenuSelected));
            control.ContextMenu = popUpMenu;
        }

        private void CopyMenuSelected(object sender, EventArgs e)
        {
            if (CopyKeyPress == null) return;
            CopyKeyPress(sender, new EventArgs());
        }


        #endregion
    }
}
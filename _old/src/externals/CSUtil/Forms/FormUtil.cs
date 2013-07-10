using System;
using System.Runtime.InteropServices;
using System.Windows.Forms;

namespace CSUtil.Forms
{
    /// <summary>
    /// Formに関するユーティティです。
    /// </summary>
    public static class FormUtil
    {
        [DllImport("user32.dll")]
        private static extern IntPtr GetSystemMenu(IntPtr hWnd, bool bRevert);
        [DllImport("user32.dll")]
        private static extern int GetMenuItemCount(IntPtr hMenu);
        [DllImport("user32.dll")]
        private static extern bool DrawMenuBar(IntPtr hWnd);
        [DllImport("user32.dll")]
        private static extern bool RemoveMenu(IntPtr hMenu, uint uPosition, uint uFlags);

        /// <summary>
        /// フォームのクローズボタンを無効化します。
        /// OnLoadが実行される前に行なう必要があります。
        /// </summary>
        /// <param name="form"></param>
        public static void RemoveCloseButton(ILoggingForm form)
        {
            form.Load += OnLoadRemoveCloseButton;
        }

        private static void OnLoadRemoveCloseButton(object sender, EventArgs e)
        {
            Form form = sender as Form;
            if (form == null) return;

            const Int32 MF_BYPOSITION = 0x400;
            const Int32 MF_REMOVE = 0x1000;

            IntPtr menu = GetSystemMenu(form.Handle, false);
            int menuCount = GetMenuItemCount(menu);
            if (menuCount > 1) {
                //メニューの「閉じる」とセパレータを削除
                RemoveMenu(menu, (uint)(menuCount - 1), MF_BYPOSITION | MF_REMOVE);
                RemoveMenu(menu, (uint)(menuCount - 2), MF_BYPOSITION | MF_REMOVE);
                DrawMenuBar(form.Handle);
            }
        }

    }
}
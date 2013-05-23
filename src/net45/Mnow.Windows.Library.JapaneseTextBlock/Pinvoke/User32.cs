using System;
using System.Runtime.InteropServices;
namespace Mnow.Windows.Library.Pinvoke
{
    public static class User32
    {
        [DllImport("user32.dll")]
        public static extern IntPtr GetDC(IntPtr hWnd);
        [DllImport("user32.dll")]
        public static extern int ReleaseDC(IntPtr hWnd, IntPtr hDC);
    }
}

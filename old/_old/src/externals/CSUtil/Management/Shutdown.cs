using System;
using System.Management;

namespace CSUtil.Management
{
    /// <summary>
    /// シャットダウンを行なうための静的メソッドを実装します。
    /// </summary>
    public static class Shutdown
    {
        /// <summary>
        /// 電源切断を行ないます。
        /// このメソッドからの制御は返らない前提でプログラムを作成してください。
        /// </summary>
        public static void OnShutDown()
        {
            CallWmiShutdown(8);
        }

        /// <summary>
        /// リブートを行ないます。
        /// このメソッドからの制御は返らない前提でプログラムを作成してください。
        /// </summary>
        public static void OnReboot()
        {
            CallWmiShutdown(2);
        }


        private static void CallWmiShutdown(int flags)
        {
            // 実行前にガベージコレクトを行っておく
            GC.Collect();
            System.Threading.Thread.CurrentThread.Join(0);
            GC.WaitForPendingFinalizers();
            GC.Collect();

            //ユーザー特権を有効にするための設定を作成
            ConnectionOptions co = new ConnectionOptions();
            co.Impersonation = ImpersonationLevel.Impersonate;
            co.EnablePrivileges = true;
            //ManagementScopeを作成
            ManagementScope sc = new ManagementScope("\\ROOT\\CIMV2", co);

            //接続
            sc.Connect();
            ObjectQuery oq = new ObjectQuery("select * from Win32_OperatingSystem");
            using (ManagementObjectSearcher mos = new ManagementObjectSearcher(sc, oq)) {
                //Shutdownメソッドを呼び出す
                foreach (ManagementObject mo in mos.Get()) {
                    //パラメータを指定
                    ManagementBaseObject inParams = mo.GetMethodParameters("Win32Shutdown");
                    inParams["Flags"] = flags;
                    inParams["Reserved"] = 0;
                    //Win32Shutdownメソッドを呼び出す
                    ManagementBaseObject outParams = mo.InvokeMethod("Win32Shutdown", inParams, null);
                }
            }
        }
    }
}
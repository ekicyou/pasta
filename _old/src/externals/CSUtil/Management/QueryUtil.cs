using System.Diagnostics;
using System.Management;
using System.Text;

namespace CSUtil.Management
{
    /// <summary>
    /// システムへの問い合わせ処理サブルーチン集。
    /// </summary>
    public static class QueryUtil
    {
        /// <summary>
        /// 指定されたPathに属するHDDのフリースペースを取得します。
        /// </summary>
        /// <param name="driveName"></param>
        /// <returns></returns>
        public static ulong GetDickFreeSpace(string driveName)
        {
            using (ManagementObjectSearcher Searcher = new ManagementObjectSearcher("Select * from Win32_LogicalDisk where DriveType=3")) {
                using (ManagementObjectCollection moc = Searcher.Get()) {
                    StringBuilder MailBody = new StringBuilder();
                    foreach (ManagementObject mo in moc) {
#if DEBUG
                        Trace.Write("GetDickFreeSpace:");
                        foreach (PropertyData col in mo.Properties) {
                            Trace.Write(string.Format("[{0}={1}] ", col.Name, col.Value));
                        }
                        Trace.WriteLine("");
#endif
                        string DeviceID = (string)mo.Properties["DeviceID"].Value;
                        if (DeviceID != driveName) continue;
                        // long size = (long)mo.Properties["Size"].Value;
                        return (ulong)mo.Properties["FreeSpace"].Value;
                    }
                }
            }
            return 0;
        }
    }
}
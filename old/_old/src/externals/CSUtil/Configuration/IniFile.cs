using System;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;

namespace CSUtil.Configuration
{
    /// <summary>
    /// IniFileにアクセスします。
    /// </summary>
    public class IniFile
    {
        private const int BUFF_LEN = 256; // 256文字

        /// <summary>
        /// API宣言
        /// * iniﾌｧｲﾙ読込み関数宣言
        /// </summary>
        [DllImport("kernel32.dll", EntryPoint = "GetPrivateProfileString")]
        private static extern uint GetPrivateProfileString(
              string lpApplicationName
            , string lpKeyName
            , string lpDefault
            , System.Text.StringBuilder StringBuilder
            , uint nSize
            , string lpFileName);

        /// <summary>
        /// API宣言
        /// * iniﾌｧｲﾙ書込み関数宣言
        /// </summary>
        [DllImport("kernel32.dll", EntryPoint = "WritePrivateProfileString")]
        private static extern uint WritePrivateProfileString(
              string lpApplicationName
            , string lpEntryName
            , string lpEntryString
            , string lpFileName);


        private string iniFileName;

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="iniFileName">iniFile名</param>
        public IniFile(string iniFileName)
        {
            // メンバ変数にセット
            this.iniFileName = Path.GetFullPath(iniFileName);
        }

        /// <summary>
        /// IniFileメンバにアクセスするためのインデックスです。
        /// </summary>
        /// <param name="section">セクション名</param>
        /// <param name="key">Key名</param>
        /// <returns></returns>
        public string this[string section, string key]
        {
            get { return GetIniString(section, key); }
            set
            {
                if (!SetIniString(section, key, value)) {
                    throw new InvalidOperationException("値の書き込みに失敗しました");
                }
            }
        }

        /// <summary>
        /// 指定したsection,keyの情報を取得します。
        /// 情報が存在しない場合、defaultValue値を返します。
        /// </summary>
        /// <param name="section">セクション名</param>
        /// <param name="key">Key名</param>
        /// <param name="defaultValue">デフォルト値</param>
        /// <returns>IniFile値</returns>
        public string GetIniString(string section, string key, string defaultValue)
        {
            StringBuilder sb = new StringBuilder(BUFF_LEN);
            uint ret = GetPrivateProfileString(section, key, defaultValue, sb, Convert.ToUInt32(sb.Capacity), iniFileName);
            return sb.ToString();
        }

        /// <summary>
        /// 指定したsection,keyの情報を取得します。
        /// 情報が存在しない場合、""を返します。
        /// </summary>
        /// <param name="section">セクション名</param>
        /// <param name="key">Key名</param>
        /// <returns>IniFile値</returns>
        public string GetIniString(string section, string key)
        {
            return GetIniString(section, key, null);
        }

        /// <summary>
        /// 指定したsection,keyに値を書き込みます。
        /// </summary>
        /// <param name="section">セクション名</param>
        /// <param name="key">Key名</param>
        /// <param name="value"></param>
        /// <returns>書き込みが行えたらtrue</returns>
        public bool SetIniString(string section, string key, string value)
        {
            uint ret = WritePrivateProfileString(section, key, value, iniFileName);
            if (ret > 0) return true;
            else return false;
        }

        /// <summary>
        /// 文字列値をbool値に変換します。
        /// </summary>
        /// <param name="s">文字列</param>
        /// <returns>trueと判別されるべき文字列の場合true</returns>
        public static bool GetBool(string s)
        {
            if (string.IsNullOrEmpty(s)) return false;
            bool b;
            s = s.Trim();
            if (bool.TryParse(s, out b)) return b;
            int i;
            if (int.TryParse(s, out i)) return i != 0;
            return false;
        }


        /// <summary>
        /// 文字列値をint値に変換します。
        /// </summary>
        /// <param name="s">文字列</param>
        /// <param name="defaultValue">変換できなかったときのデフォルト値</param>
        /// <returns>trueと判別されるべき文字列の場合true</returns>
        public static int GetInt32(string s, int defaultValue)
        {
            if (string.IsNullOrEmpty(s)) return defaultValue;
            int rc = defaultValue;
            int.TryParse(s.Trim(), out rc);
            return rc;
        }


        /// <summary>
        /// 文字列値をdouble値に変換します。
        /// </summary>
        /// <param name="s">文字列</param>
        /// <param name="defaultValue">変換できなかったときのデフォルト値</param>
        /// <returns>trueと判別されるべき文字列の場合true</returns>
        public static double GetDouble(string s, int defaultValue)
        {
            if (string.IsNullOrEmpty(s)) return defaultValue;
            double rc = defaultValue;
            double.TryParse(s.Trim(), out rc);
            return rc;
        }

    }
}
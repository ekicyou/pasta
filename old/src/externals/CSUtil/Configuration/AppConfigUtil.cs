using System;
using System.Configuration;
using System.Diagnostics;
using System.IO;

namespace CSUtil.Configuration
{
    /// <summary>
    /// アプリケーション設定ファイルの読み込みに使うユーティリティ関数。
    /// </summary>
    public static class AppConfigUtil
    {
        /// <summary>
        /// アプリケーションコンフィグファイルの名前を取得します。
        /// </summary>
        private static string AppConfigName
        {
            get
            {
                System.Configuration.Configuration conf = ConfigurationManager.OpenExeConfiguration(ConfigurationUserLevel.None);
                return string.Format("アプリケーション構成ファイル\n[{0}]\n", conf.FilePath);
            }
        }

        /// <summary>
        /// App.configより、文字列を取得します。
        /// </summary>
        /// <param name="key"></param>
        /// <returns></returns>
        public static string GetValueString(string key)
        {
            string value;
            try {
                try {
                    value = ConfigurationManager.AppSettings.GetValues(key)[0];
                }
                catch (IndexOutOfRangeException e) {
                    string mes = string.Format("設定情報[{0}]が見つかりませんでした。\n{1}を確認してください。", key, AppConfigName);
                    throw new Exception(mes, e);
                }
                catch (Exception e) {
                    string mes = string.Format("設定情報[{0}]が見つかりませんでした。\n{1}を確認してください。", key, AppConfigName);
                    throw new Exception(mes, e);
                }
            }
            catch (Exception e) {
                Trace.WriteLine(e);
                throw e;
            }
            return value;
        }

        /// <summary>
        /// App.configより、数字を取得します。
        /// </summary>
        /// <param name="key"></param>
        /// <returns></returns>
        public static int GetValueInt(string key)
        {
            string text = GetValueString(key);
            try {
                try {
                    if (string.IsNullOrEmpty(text)) return 0;
                    if (text.Length > 2) {
                        if (text.Substring(0, 2) == "0x") goto HEX_PARSE;
                        if (text.Substring(0, 2) == "0X") goto HEX_PARSE;
                    }
                    return int.Parse(text);
                HEX_PARSE:
                    return int.Parse(text.Substring(2), System.Globalization.NumberStyles.AllowHexSpecifier);
                }
                catch (Exception e) {
                    string mes = string.Format("設定情報[{0}]の情報[{1}]は数字に変換できません。\n{2}を確認してください。", key, text, AppConfigName);
                    throw new Exception(mes, e);
                }

            }
            catch (Exception e) {
                Trace.WriteLine(e);
                throw e;
            }
        }

        /// <summary>
        /// App.configより、BOOL値を取得します。
        /// </summary>
        /// <param name="key"></param>
        /// <returns></returns>
        public static bool GetValueBool(string key)
        {
            string text = GetValueString(key);
            if (string.IsNullOrEmpty(text)) return false;
            if (text[0] == 't') return true;
            if (text[0] == 'T') return true;
            if (text[0] == '0') return false;
            if (text[0] == '1') return true;
            return false;
        }

        /// <summary>
        /// App.configより、有効なディレクトリ名を取得します。
        /// ディレクトリが存在しない場合は例外を返します。
        /// (デバッグモードの場合はディレクトリを作成します)
        /// </summary>
        /// <param name="key"></param>
        /// <returns></returns>
        public static string GetValueDirectory(string key)
        {
            string path = AppConfigUtil.GetValueString(key);
            path = Path.GetFullPath(path);
            if (!Directory.Exists(path)) {
#if DEBUG
                Directory.CreateDirectory(path);
#else
        throw new IOException(string.Format("キー[{0}]の[{1}]は有効なディレクトリではありません。", key, path));
#endif
            }
            return path;
        }

    }
}
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Threading.Tasks.Dataflow;

namespace Pasta.API
{
    /// <summary>
    /// ログ収集モジュールのファクトリ。
    /// </summary>
    public interface IPastaGleanerFactory
    {
        /// <summary>収集モジュールの名前</summary>
        string GleanerName { get; }

        /// <summary>
        /// デフォルト設定情報を取得します。
        /// </summary>
        /// <param name="data"></param>
        /// <returns></returns>
        dynamic CreateDefaultSetting();

        /// <summary>
        /// 設定情報バイナリを設定オブジェクトに変換します。
        /// </summary>
        /// <param name="data"></param>
        /// <returns></returns>
        dynamic AsSettingObject(byte[] data);

        /// <summary>
        /// 設定オブジェクトを設定情報バイナリに変換します。
        /// </summary>
        /// <param name="setting"></param>
        /// <returns></returns>
        byte[] AsSettingBytes(dynamic setting);

        /// <summary>
        /// ログ収集モジュールを作成します。
        /// </summary>
        /// <returns></returns>
        IPastaGleaner CreateGleaner();
    }
}

using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.IO;
using System.Threading.Tasks;
using System.Composition;
using Pasta.API;
using Pasta.Model;
using System.Threading;
using System.Threading.Tasks.Dataflow;
using ProtoBuf;

namespace Pasta.Gleaners
{
    public abstract class BasePastaGleanerFactory<TGleaner, TSetting> : IPastaGleanerFactory
        where TGleaner : BasePastaGleaner<TSetting>, new()
        where TSetting : class
    {
        /// <summary>リスナ名</summary>
        public abstract string GleanerName { get; }

        /// <summary>
        /// デフォルト設定情報を取得します。
        /// </summary>
        /// <param name="data"></param>
        /// <returns></returns>
        public abstract dynamic CreateDefaultSetting();

        /// <summary>
        /// 設定情報バイナリを設定オブジェクトに変換します。
        /// </summary>
        /// <param name="data"></param>
        /// <returns></returns>
        public dynamic AsSettingObject(byte[] data)
        {
            using (var st = new MemoryStream(data))
            {
                var rc = Serializer.Deserialize<TSetting>(st);
                return rc;
            }
        }

        /// <summary>
        /// 設定オブジェクトを設定データに変換します。
        /// </summary>
        /// <param name="setting"></param>
        /// <returns></returns>
        public byte[] AsSettingBytes(dynamic data)
        {
            var st = new MemoryStream();
            var rc = Serializer.Serialize<TSetting>(st, data);
            return st.GetBuffer();
        }

        /// <summary>
        /// 設定オブジェクトより、ログ収集モジュールを作成します。
        /// </summary>
        /// <param name="data">設定情報</param>
        /// <returns></returns>
        public IPastaGleaner CreateGleaner()
        {
            return new TGleaner();
        }
    }
}

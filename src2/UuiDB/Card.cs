using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

namespace UuiDB
{
    /// <summary>
    /// データベースに保存される情報。
    /// </summary>
    public sealed class Card
    {
        #region プロパティ：情報
        /// <summary>一意なID(GUID)</summary>
        public Guid ID { get; set; }

        /// <summary>ヘッダ情報</summary>
        public JObject Header { get; set; }

        /// <summary>本体テキスト</summary>
        public String Body { get; set; }

        /// <summary>Blob情報</summary>
        public byte[] Blob { get; set; }


        #endregion
        #region プロパティ：保存管理

        /// <summary>カード情報が保存されるパス</summary>
        public string CardPath { get; set; }

        /// <summary>バイナリ情報が保存されるパス</summary>
        public string BlobPath { get; set; }

        /// <summary>Blob拡張子</summary>
        public string BlobExt { get; set; }

        /// <summary>保存が必要ならtrue</summary>
        public bool HasSave { get;private set; }

        #endregion



    }
}

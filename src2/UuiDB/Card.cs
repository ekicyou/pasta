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
        /// <summary>一意なID(GUID)</summary>
        public Guid ID { get; set; }

        /// <summary>ヘッダ情報</summary>
        public JObject Header { get; set; }

        /// <summary>本体テキスト</summary>
        public String Body { get; set; }

        /// <summary>Blob情報</summary>
        public byte[] Blob { get; set; }



    }
}

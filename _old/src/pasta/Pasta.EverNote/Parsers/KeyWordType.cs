using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace Pasta.EverNote.Parsers
{
    public enum KeyWordType
    {
        /// <summary>一般キーワード</summary>
        Normal,

        /// <summary>ジャンプコマンド</summary>
        Jump,

        /// <summary>アンカーコマンド</summary>
        Anchor,
    }
}

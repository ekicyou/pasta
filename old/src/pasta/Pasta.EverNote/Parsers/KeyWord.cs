using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace Pasta.EverNote.Parsers
{
    public sealed class KeyWord : Word
    {
        public string Key { get; set; }

        public override string ToString()
        {
            return string.Format("[@{0}]", Key);
        }

        public char FirstChar { get { return Key.FirstOrDefault(); } }

        /// <summary>
        /// キーワードタイプ。
        /// </summary>
        public KeyWordType KeyWordType
        {
            get
            {
                switch(FirstChar) {
                    case '、':
                    case ',':
                    case '，':
                        return Parsers.KeyWordType.Jump;
                    case '－':
                    case 'ー':
                    case '-':
                        return Parsers.KeyWordType.Anchor;

                } return Parsers.KeyWordType.Normal;


            }
        }


    }
}
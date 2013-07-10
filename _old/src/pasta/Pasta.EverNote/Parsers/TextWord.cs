using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace Pasta.EverNote.Parsers
{
    public sealed class TextWord : Word
    {
        public string Text
        {
            get { return _Text; }
            set
            {
                if (value.EndsWith("\n")) value = value.TrimEnd();
                _Text = value;
            }
        }
        private string _Text;

        public override string ToString()
        {
            return string.Format("[{0}]", Text);
        }

    }
}

using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.IO;
using System.Xml.Linq;
using System.Reflection;

namespace Pasta.EverNote
{
    public class ResourceReader
    {
        public static Stream GetPastaStream()
        {
            var asm = Assembly.GetExecutingAssembly();
            var stm = asm.GetManifestResourceStream("Pasta.EverNote.pasta.enex");
            return stm;
        }

        public static XDocument GetPastaDocument()
        {
            var stm = GetPastaStream();
            var doc = XDocument.Load(stm);
            return doc;
        }
    }
}

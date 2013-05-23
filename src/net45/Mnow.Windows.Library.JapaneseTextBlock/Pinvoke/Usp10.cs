using System;
using System.Runtime.InteropServices;
namespace Mnow.Windows.Library.Pinvoke
{
    public static class Usp10
    {
        [DllImport("usp10.dll", CharSet = CharSet.Unicode)]
        public static extern int ScriptItemizeOpenType(string pwcInChars, int cInChars, int cMaxItems, SCRIPT_CONTROL psControl, ushort psState, [Out] SCRIPT_ITEM[] pItems, [Out] OPENTYPE_TAG[] pScriptTags, ref int pcItems);
        [DllImport("usp10.dll", CharSet = CharSet.Unicode)]
        public static extern int ScriptShapeOpenType(IntPtr hdc, ref SCRIPT_CACHE psc, ref SCRIPT_ANALYSIS psa, OPENTYPE_TAG tagScript, OPENTYPE_TAG tagLangSys, int[] rcRangeChars, ref IntPtr rpRangeProperties, int cRanges, string pwcChars, int cChars, int cMaxGlyphs, [In] [Out] ushort[] pwLogClust, [In] [Out] SCRIPT_CHARPROP[] pCharProps, [In] [Out] ushort[] pwOutGlyphs, [In] [Out] SCRIPT_GLYPHPROP[] pOutGlyphProps, ref int pcGlyphs);
        [DllImport("usp10.dll", CharSet = CharSet.Unicode)]
        public static extern int ScriptPlaceOpenType(IntPtr hdc, ref SCRIPT_CACHE psc, ref SCRIPT_ANALYSIS psa, OPENTYPE_TAG tagScript, OPENTYPE_TAG tagLangSys, int[] rcRangeChars, ref IntPtr rpRangeProperties, int cRanges, string pwcChars, [In] [Out] ushort[] pwLogClust, [In] [Out] SCRIPT_CHARPROP[] pCharProps, int cChars, [In] [Out] ushort[] pwOutGlyphs, [In] [Out] SCRIPT_GLYPHPROP[] pOutGlyphProps, int cGlyphs, [In] [Out] int[] piAdvance, [In] [Out] GOFFSET[] pGoffset, [In] [Out] ABC[] pABC);
    }
}

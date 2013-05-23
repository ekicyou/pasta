using Mnow.Windows.Library.Pinvoke;
using System;
using System.Runtime.InteropServices;
namespace Mnow.Windows.Library
{
    public class VerticalUniscribe
    {
        private SCRIPT_CACHE _psc = default(SCRIPT_CACHE);
        public int ScriptItemize(string inChars, out SCRIPT_ITEM[] items, out OPENTYPE_TAG[] scriptTags)
        {
            if (string.IsNullOrEmpty(inChars))
            {
                throw new ArgumentException("inChars");
            }
            int length = inChars.Length;
            items = new SCRIPT_ITEM[length + 1];
            for (int index = 0; index < length + 1; index++)
            {
                items[index] = default(SCRIPT_ITEM);
            }
            scriptTags = new OPENTYPE_TAG[length];
            for (int index = 0; index < length; index++)
            {
                scriptTags[index] = new OPENTYPE_TAG('\0', '\0', '\0', '\0');
            }
            int itemsCount = 0;
            int hresult = Usp10.ScriptItemizeOpenType(inChars, length, length + 1, null, 0, items, scriptTags, ref itemsCount);
            Marshal.ThrowExceptionForHR(hresult);
            return itemsCount;
        }
        public int ScriptShape(string inChars, int charPos, int length, SCRIPT_ITEM[] items, OPENTYPE_TAG[] scriptTags, int itemsCount, IntPtr hWnd, string faceName, int fontHeight, out ushort[] logClust, out SCRIPT_CHARPROP[] charProps, out ushort[] glyphs, out SCRIPT_GLYPHPROP[] glyphProps)
        {
            if (string.IsNullOrEmpty(inChars))
            {
                throw new ArgumentException("inChars");
            }
            if (items == null || items.Length <= 0)
            {
                throw new ArgumentException("items");
            }
            if (itemsCount >= items.Length)
            {
                throw new ArgumentException("itemsCount");
            }
            if (charPos < 0)
            {
                throw new ArgumentException("charPos");
            }
            if (charPos >= inChars.Length)
            {
                throw new ArgumentException("charPos");
            }
            if (length <= 0)
            {
                throw new ArgumentException("length");
            }
            if (charPos + length > inChars.Length)
            {
                throw new ArgumentOutOfRangeException("length");
            }
            if (string.IsNullOrEmpty(faceName))
            {
                throw new ArgumentException("faceName");
            }
            if (fontHeight <= 0)
            {
                throw new ArgumentException("fontHeight");
            }
            int itemIndex;
            for (itemIndex = 0; itemIndex < itemsCount - 1; itemIndex++)
            {
                if (items[itemIndex].iCharPos <= charPos)
                {
                    if (items[itemIndex + 1].iCharPos >= charPos + length)
                    {
                        break;
                    }
                }
            }
            if (itemIndex == itemsCount - 1)
            {
                if (items[itemIndex].iCharPos > charPos)
                {
                    throw new ArgumentOutOfRangeException("length");
                }
            }
            SCRIPT_ANALYSIS psa = items[itemIndex].a;
            string shapeChars = inChars.Substring(charPos, length);
            OPENTYPE_TAG tagScript = scriptTags[itemIndex];
            OPENTYPE_TAG tagLangSys = new OPENTYPE_TAG('\0', '\0', '\0', '\0');
            int[] rangeChars = new int[]
			{
				length
			};
            logClust = new ushort[length];
            charProps = new SCRIPT_CHARPROP[length];
            glyphs = new ushort[length];
            glyphProps = new SCRIPT_GLYPHPROP[length];
            for (int index = 0; index < length; index++)
            {
                glyphProps[index] = default(SCRIPT_GLYPHPROP);
                logClust[index] = 0;
                charProps[index] = default(SCRIPT_CHARPROP);
                glyphs[index] = 0;
            }
            int pcGlyphs = 0;
            OPENTYPE_FEATURE_RECORD[] arRecord = new OPENTYPE_FEATURE_RECORD[1];
            OPENTYPE_FEATURE_RECORD potfRecord = default(OPENTYPE_FEATURE_RECORD);
            arRecord[0] = potfRecord;
            potfRecord.tagFeature = new OPENTYPE_TAG('v', 'e', 'r', 't');
            potfRecord.lParameter = 1u;
            TEXTRANGE_PROPERTIES[] arRangeProperty = new TEXTRANGE_PROPERTIES[1];
            TEXTRANGE_PROPERTIES rpRangeProperty = default(TEXTRANGE_PROPERTIES);
            arRangeProperty[0] = rpRangeProperty;
            rpRangeProperty.cotfRecords = 1;
            int cRanges = 1;
            IntPtr arRecordPtr = Marshal.AllocHGlobal(Marshal.SizeOf(potfRecord));
            Marshal.StructureToPtr(potfRecord, arRecordPtr, false);
            IntPtr arRangePtr = Marshal.AllocHGlobal(Marshal.SizeOf(rpRangeProperty));
            rpRangeProperty.potfRecords = arRecordPtr;
            Marshal.StructureToPtr(rpRangeProperty, arRangePtr, false);
            IntPtr hDC = User32.GetDC(hWnd);
            IntPtr font = Gdi32.CreateFontIndirect(new LOGFONT
            {
                lfCharSet = FontCharSet.DEFAULT_CHARSET,
                lfHeight = fontHeight,
                lfFaceName = faceName
            });
            IntPtr oldFont = Gdi32.SelectObject(hDC, font);
            int hresult = Usp10.ScriptShapeOpenType(hDC, ref this._psc, ref psa, tagScript, tagLangSys, rangeChars, ref arRangePtr, cRanges, shapeChars, length, length, logClust, charProps, glyphs, glyphProps, ref pcGlyphs);
            Gdi32.SelectObject(hDC, oldFont);
            Gdi32.DeleteObject(font);
            User32.ReleaseDC(hWnd, hDC);
            Marshal.FreeHGlobal(arRecordPtr);
            Marshal.FreeHGlobal(arRangePtr);
            Marshal.ThrowExceptionForHR(hresult);
            return pcGlyphs;
        }
        public void ScriptPlace(string inChars, int charPos, int length, SCRIPT_ITEM[] items, OPENTYPE_TAG[] scriptTags, int itemsCount, IntPtr hWnd, string faceName, int fontHeight, ushort[] logClust, SCRIPT_CHARPROP[] charProps, ushort[] glyphs, SCRIPT_GLYPHPROP[] glyphProps, int glyphsCount, out int[] advance, out GOFFSET[] goffset, out ABC[] abc)
        {
            if (string.IsNullOrEmpty(inChars))
            {
                throw new ArgumentException("inChars");
            }
            if (items == null || items.Length <= 0)
            {
                throw new ArgumentException("items");
            }
            if (itemsCount >= items.Length)
            {
                throw new ArgumentException("itemsCount");
            }
            if (charPos < 0)
            {
                throw new ArgumentException("charPos");
            }
            if (charPos >= inChars.Length)
            {
                throw new ArgumentException("charPos");
            }
            if (length <= 0)
            {
                throw new ArgumentException("length");
            }
            if (charPos + length > inChars.Length)
            {
                throw new ArgumentOutOfRangeException("length");
            }
            if (string.IsNullOrEmpty(faceName))
            {
                throw new ArgumentException("faceName");
            }
            if (fontHeight <= 0)
            {
                throw new ArgumentException("fontHeight");
            }
            if (glyphsCount <= 0)
            {
                throw new ArgumentException("glyphsCount");
            }
            if (glyphsCount > logClust.Length)
            {
                throw new ArgumentException("logClust");
            }
            if (glyphsCount > charProps.Length)
            {
                throw new ArgumentException("charProps");
            }
            if (glyphsCount > glyphs.Length)
            {
                throw new ArgumentException("glyphs");
            }
            if (glyphsCount > glyphProps.Length)
            {
                throw new ArgumentException("glyphProps");
            }
            int itemIndex;
            for (itemIndex = 0; itemIndex < itemsCount - 1; itemIndex++)
            {
                if (items[itemIndex].iCharPos <= charPos)
                {
                    if (items[itemIndex + 1].iCharPos >= charPos + length)
                    {
                        break;
                    }
                }
            }
            if (itemIndex == itemsCount - 1)
            {
                if (items[itemIndex].iCharPos > charPos)
                {
                    throw new ArgumentOutOfRangeException("length");
                }
            }
            SCRIPT_ANALYSIS psa = items[itemIndex].a;
            string shapeChars = inChars.Substring(charPos, length);
            OPENTYPE_TAG tagScript = scriptTags[itemIndex];
            OPENTYPE_TAG tagLangSys = new OPENTYPE_TAG('\0', '\0', '\0', '\0');
            int[] rangeChars = new int[]
			{
				length
			};
            advance = new int[glyphsCount];
            goffset = new GOFFSET[glyphsCount];
            abc = new ABC[glyphsCount];
            for (int index = 0; index < glyphsCount; index++)
            {
                advance[index] = 0;
                goffset[index] = default(GOFFSET);
                abc[index] = default(ABC);
            }
            OPENTYPE_FEATURE_RECORD[] arRecord = new OPENTYPE_FEATURE_RECORD[1];
            OPENTYPE_FEATURE_RECORD potfRecord = default(OPENTYPE_FEATURE_RECORD);
            arRecord[0] = potfRecord;
            potfRecord.tagFeature = new OPENTYPE_TAG('v', 'e', 'r', 't');
            potfRecord.lParameter = 1u;
            TEXTRANGE_PROPERTIES[] arRangeProperty = new TEXTRANGE_PROPERTIES[1];
            TEXTRANGE_PROPERTIES rpRangeProperty = default(TEXTRANGE_PROPERTIES);
            arRangeProperty[0] = rpRangeProperty;
            rpRangeProperty.cotfRecords = 1;
            int cRanges = 1;
            IntPtr arRecordPtr = Marshal.AllocHGlobal(Marshal.SizeOf(potfRecord));
            Marshal.StructureToPtr(potfRecord, arRecordPtr, false);
            IntPtr arRangePtr = Marshal.AllocHGlobal(Marshal.SizeOf(rpRangeProperty));
            rpRangeProperty.potfRecords = arRecordPtr;
            Marshal.StructureToPtr(rpRangeProperty, arRangePtr, false);
            IntPtr hDC = User32.GetDC(hWnd);
            IntPtr font = Gdi32.CreateFontIndirect(new LOGFONT
            {
                lfCharSet = FontCharSet.DEFAULT_CHARSET,
                lfHeight = fontHeight,
                lfFaceName = faceName
            });
            IntPtr oldFont = Gdi32.SelectObject(hDC, font);
            int hresult = Usp10.ScriptPlaceOpenType(hDC, ref this._psc, ref psa, tagScript, tagLangSys, rangeChars, ref arRangePtr, cRanges, shapeChars, logClust, charProps, length, glyphs, glyphProps, glyphsCount, advance, goffset, abc);
            Gdi32.SelectObject(hDC, oldFont);
            Gdi32.DeleteObject(font);
            User32.ReleaseDC(hWnd, hDC);
            Marshal.FreeHGlobal(arRecordPtr);
            Marshal.FreeHGlobal(arRangePtr);
            Marshal.ThrowExceptionForHR(hresult);
        }
    }
}

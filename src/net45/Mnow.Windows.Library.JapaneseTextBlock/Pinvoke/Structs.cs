using System;
using System.Runtime.InteropServices;
using System.Text;

namespace Mnow.Windows.Library.Pinvoke
{
    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public struct SCRIPT_ANALYSIS
    {
        public ushort flags;
        public ushort scriptState;
    }

    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public struct SCRIPT_ITEM
    {
        public int iCharPos;
        public SCRIPT_ANALYSIS a;
    }

    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public struct OPENTYPE_TAG
    {
        public byte b0;
        public byte b1;
        public byte b2;
        public byte b3;
        public OPENTYPE_TAG(char b0, char b1, char b2, char b3)
        {
            this.b0 = (byte)b0;
            this.b1 = (byte)b1;
            this.b2 = (byte)b2;
            this.b3 = (byte)b3;
        }
        public override string ToString()
        {
            string result;
            if (this.b0 == 0)
            {
                result = "<null>";
            }
            else
            {
                StringBuilder sb = new StringBuilder();
                sb.Append((char)this.b0);
                sb.Append((char)this.b1);
                sb.Append((char)this.b2);
                sb.Append((char)this.b3);
                result = sb.ToString();
            }
            return result;
        }
    }

    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public struct SCRIPT_CACHE
    {
        public IntPtr cache;
    }

    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public class SCRIPT_CONTROL
    {
        public ushort uDefaultLanguage;
        public ushort flags;
    }

    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public struct SCRIPT_CHARPROP
    {
        public ushort flags;
    }

    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public struct SCRIPT_GLYPHPROP
    {
        public ushort sva;
        public ushort reserved;
    }

    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public struct OPENTYPE_FEATURE_RECORD
    {
        public OPENTYPE_TAG tagFeature;
        public uint lParameter;
    }

    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public struct GOFFSET
    {
        public uint du;
        public uint dv;
    }

    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public struct TEXTRANGE_PROPERTIES
    {
        public IntPtr potfRecords;
        public int cotfRecords;
    }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Auto)]
    public class LOGFONT
    {
        public const int LF_FACESIZE = 32;
        public int lfHeight;
        public int lfWidth;
        public int lfEscapement;
        public int lfOrientation;
        public FontWeight lfWeight;
        [MarshalAs(UnmanagedType.U1)]
        public bool lfItalic;
        [MarshalAs(UnmanagedType.U1)]
        public bool lfUnderline;
        [MarshalAs(UnmanagedType.U1)]
        public bool lfStrikeOut;
        public FontCharSet lfCharSet;
        public FontPrecision lfOutPrecision;
        public FontClipPrecision lfClipPrecision;
        public FontQuality lfQuality;
        public FontPitchAndFamily lfPitchAndFamily;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)]
        public string lfFaceName;
        public override string ToString()
        {
            StringBuilder sb = new StringBuilder();
            sb.Append("LOGFONT\n");
            sb.AppendFormat("   lfHeight: {0}\n", this.lfHeight);
            sb.AppendFormat("   lfWidth: {0}\n", this.lfWidth);
            sb.AppendFormat("   lfEscapement: {0}\n", this.lfEscapement);
            sb.AppendFormat("   lfOrientation: {0}\n", this.lfOrientation);
            sb.AppendFormat("   lfWeight: {0}\n", this.lfWeight);
            sb.AppendFormat("   lfItalic: {0}\n", this.lfItalic);
            sb.AppendFormat("   lfUnderline: {0}\n", this.lfUnderline);
            sb.AppendFormat("   lfStrikeOut: {0}\n", this.lfStrikeOut);
            sb.AppendFormat("   lfCharSet: {0}\n", this.lfCharSet);
            sb.AppendFormat("   lfOutPrecision: {0}\n", this.lfOutPrecision);
            sb.AppendFormat("   lfClipPrecision: {0}\n", this.lfClipPrecision);
            sb.AppendFormat("   lfQuality: {0}\n", this.lfQuality);
            sb.AppendFormat("   lfPitchAndFamily: {0}\n", this.lfPitchAndFamily);
            sb.AppendFormat("   lfFaceName: {0}\n", this.lfFaceName);
            return sb.ToString();
        }
    }

    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public struct ABC
    {
        public int abcA;
        public uint abcB;
        public int abcC;
    }

}

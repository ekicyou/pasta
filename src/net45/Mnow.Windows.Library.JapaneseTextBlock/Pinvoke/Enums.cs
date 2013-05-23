using System;
namespace Mnow.Windows.Library.Pinvoke
{
    public enum FontWeight
    {
        FW_DONTCARE,
        FW_THIN = 100,
        FW_EXTRALIGHT = 200,
        FW_LIGHT = 300,
        FW_NORMAL = 400,
        FW_MEDIUM = 500,
        FW_SEMIBOLD = 600,
        FW_BOLD = 700,
        FW_EXTRABOLD = 800,
        FW_HEAVY = 900
    }

    public enum FontCharSet : byte
    {
        ANSI_CHARSET,
        DEFAULT_CHARSET,
        SYMBOL_CHARSET,
        SHIFTJIS_CHARSET = 128,
        HANGEUL_CHARSET,
        HANGUL_CHARSET = 129,
        GB2312_CHARSET = 134,
        CHINESEBIG5_CHARSET = 136,
        OEM_CHARSET = 255,
        JOHAB_CHARSET = 130,
        HEBREW_CHARSET = 177,
        ARABIC_CHARSET,
        GREEK_CHARSET = 161,
        TURKISH_CHARSET,
        VIETNAMESE_CHARSET,
        THAI_CHARSET = 222,
        EASTEUROPE_CHARSET = 238,
        RUSSIAN_CHARSET = 204,
        MAC_CHARSET = 77,
        BALTIC_CHARSET = 186
    }

    public enum FontPrecision : byte
    {
        OUT_DEFAULT_PRECIS,
        OUT_STRING_PRECIS,
        OUT_CHARACTER_PRECIS,
        OUT_STROKE_PRECIS,
        OUT_TT_PRECIS,
        OUT_DEVICE_PRECIS,
        OUT_RASTER_PRECIS,
        OUT_TT_ONLY_PRECIS,
        OUT_OUTLINE_PRECIS,
        OUT_SCREEN_OUTLINE_PRECIS,
        OUT_PS_ONLY_PRECIS
    }

    public enum FontClipPrecision : byte
    {
        CLIP_DEFAULT_PRECIS,
        CLIP_CHARACTER_PRECIS,
        CLIP_STROKE_PRECIS,
        CLIP_MASK = 15,
        CLIP_LH_ANGLES,
        CLIP_TT_ALWAYS = 32,
        CLIP_DFA_DISABLE = 64,
        CLIP_EMBEDDED = 128
    }

    public enum FontQuality : byte
    {
        DEFAULT_QUALITY,
        DRAFT_QUALITY,
        PROOF_QUALITY,
        NONANTIALIASED_QUALITY,
        ANTIALIASED_QUALITY,
        CLEARTYPE_QUALITY,
        CLEARTYPE_NATURAL_QUALITY
    }

    [Flags]
    public enum FontPitchAndFamily : byte
    {
        DEFAULT_PITCH = 0,
        FIXED_PITCH = 1,
        VARIABLE_PITCH = 2,
        FF_DONTCARE = 0,
        FF_ROMAN = 16,
        FF_SWISS = 32,
        FF_MODERN = 48,
        FF_SCRIPT = 64,
        FF_DECORATIVE = 80
    }

}

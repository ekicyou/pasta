using Mnow.Windows.Library.Pinvoke;
using System;
using System.Globalization;
using System.Windows;
using System.Windows.Interop;
using System.Windows.Media.TextFormatting;
namespace Mnow.Windows.Library
{
    public class JapaneseTextSource : TextSource
    {
        private int _itemsCount = 0;
        private SCRIPT_ITEM[] _items;
        private OPENTYPE_TAG[] _scriptTags;
        private VerticalUniscribe _uniscribe = new VerticalUniscribe();
        private int _glyphCount = 0;
        private ushort[] _glyphs;
        private ushort[] _logClust;
        private SCRIPT_CHARPROP[] _charProps;
        private SCRIPT_GLYPHPROP[] _glyphProps;
        private int[] _advance;
        private GOFFSET[] _goffset;
        private ABC[] _abc;
        public string Text
        {
            get;
            set;
        }
        public JapaneseTextRunProperties JapaneseTextRunProperties
        {
            get;
            set;
        }
        public bool IsVarticalWriting
        {
            get;
            set;
        }
        public int ItemsCount
        {
            get
            {
                return this._itemsCount;
            }
        }
        public SCRIPT_ITEM[] Items
        {
            get
            {
                return this._items;
            }
        }
        public int GlyphCount
        {
            get
            {
                return this._glyphCount;
            }
        }
        public ushort[] Glyphs
        {
            get
            {
                return this._glyphs;
            }
        }
        public int[] Advance
        {
            get
            {
                return this._advance;
            }
        }
        public GOFFSET[] Goffset
        {
            get
            {
                return this._goffset;
            }
        }
        public ABC[] Abc
        {
            get
            {
                return this._abc;
            }
        }
        public override TextRun GetTextRun(int textSourceCharacterIndex)
        {
            if (this._itemsCount <= 0)
            {
                this.UniscribeScriptItemizeOpenType();
            }
            TextRun result;
            if (textSourceCharacterIndex < 0)
            {
                result = new TextEndOfParagraph(1);
            }
            else
            {
                if (textSourceCharacterIndex < this.Text.Length)
                {
                    for (int index = 0; index < this._itemsCount - 1; index++)
                    {
                        if (this._items[index].iCharPos <= textSourceCharacterIndex && this._items[index + 1].iCharPos > textSourceCharacterIndex)
                        {
                            result = this.MakeTextCharactors(index, textSourceCharacterIndex, this._items[index + 1].iCharPos - textSourceCharacterIndex);
                            return result;
                        }
                    }
                    result = this.MakeTextCharactors(this._itemsCount - 1, textSourceCharacterIndex, this.Text.Length - textSourceCharacterIndex);
                }
                else
                {
                    result = new TextEndOfParagraph(1);
                }
            }
            return result;
        }
        private TextRun MakeTextCharactors(int index, int textSourceCharacterIndex, int length)
        {
            TextCharacters textCharacters = new TextCharacters(this.Text, textSourceCharacterIndex, length, this.JapaneseTextRunProperties);
            this.UniscribeScriptShapeOpenType(this.Text, index, textSourceCharacterIndex, length);
            return textCharacters;
        }
        public override TextSpan<CultureSpecificCharacterBufferRange> GetPrecedingText(int textSourceCharacterIndexLimit)
        {
            CharacterBufferRange cbr = new CharacterBufferRange(this.Text, 0, textSourceCharacterIndexLimit);
            return new TextSpan<CultureSpecificCharacterBufferRange>(textSourceCharacterIndexLimit, new CultureSpecificCharacterBufferRange(CultureInfo.CurrentUICulture, cbr));
        }
        public override int GetTextEffectCharacterIndexFromTextSourceCharacterIndex(int textSourceCharacterIndex)
        {
            throw new NotImplementedException("GetTextEffectCharacterIndexFromTextSourceCharacterIndex");
        }
        private bool UniscribeScriptItemizeOpenType()
        {
            this._itemsCount = this._uniscribe.ScriptItemize(this.Text, out this._items, out this._scriptTags);
            return true;
        }
        public void UniscribeIndexedGlyphRun(IndexedGlyphRun indexedGlyphRun)
        {
            this._glyphCount = 0;
            this._logClust = null;
            this._charProps = null;
            this._glyphs = null;
            this._glyphProps = null;
            this._advance = null;
            this._goffset = null;
            this._abc = null;
            if (this._itemsCount > 0)
            {
                for (int index = 0; index < this._itemsCount - 1; index++)
                {
                    if (this._items[index].iCharPos <= indexedGlyphRun.TextSourceCharacterIndex && this._items[index + 1].iCharPos > indexedGlyphRun.TextSourceCharacterIndex)
                    {
                        this.UniscribeScriptShapeOpenType(this.Text, index, indexedGlyphRun.TextSourceCharacterIndex, indexedGlyphRun.TextSourceLength);
                        this.UniscribeScriptPlaceOpenType(this.Text, index, indexedGlyphRun.TextSourceCharacterIndex, indexedGlyphRun.TextSourceLength);
                    }
                }
                this.UniscribeScriptShapeOpenType(this.Text, this._itemsCount - 1, indexedGlyphRun.TextSourceCharacterIndex, indexedGlyphRun.TextSourceLength);
                this.UniscribeScriptPlaceOpenType(this.Text, this._itemsCount - 1, indexedGlyphRun.TextSourceCharacterIndex, indexedGlyphRun.TextSourceLength);
            }
        }
        private void UniscribeScriptShapeOpenType(string inChars, int itemIndex, int charIndex, int length)
        {
            this._glyphCount = 0;
            this._logClust = null;
            this._charProps = null;
            this._glyphs = null;
            this._glyphProps = null;
            if (this._itemsCount > 0 && this._itemsCount > itemIndex)
            {
                string shapeChars = this.Text.Substring(charIndex, length);
                char[] chars = shapeChars.ToCharArray();
                bool isAllHankaku = true;
                char[] array = chars;
                for (int i = 0; i < array.Length; i++)
                {
                    char data = array[i];
                    if (data >= 'Ā')
                    {
                        isAllHankaku = false;
                        break;
                    }
                }
                if (!isAllHankaku)
                {
                    IntPtr hWnd;
                    if (Application.Current == null || Application.Current.MainWindow == null)
                    {
                        hWnd = IntPtr.Zero;
                    }
                    else
                    {
                        WindowInteropHelper helper = new WindowInteropHelper(Application.Current.MainWindow);
                        hWnd = helper.Handle;
                    }
                    this._glyphCount = this._uniscribe.ScriptShape(inChars, charIndex, length, this._items, this._scriptTags, this._itemsCount, hWnd, this.JapaneseTextRunProperties.Typeface.FontFamily.Source, (int)this.JapaneseTextRunProperties.FontRenderingEmSize, out this._logClust, out this._charProps, out this._glyphs, out this._glyphProps);
                }
            }
        }
        private void UniscribeScriptPlaceOpenType(string inChars, int itemIndex, int charIndex, int length)
        {
            this._advance = null;
            this._goffset = null;
            this._abc = null;
            if (this._glyphCount > 0 && this._itemsCount > itemIndex)
            {
                string shapeChars = this.Text.Substring(charIndex, length);
                char[] chars = shapeChars.ToCharArray();
                bool isAllHankaku = true;
                char[] array = chars;
                for (int i = 0; i < array.Length; i++)
                {
                    char data = array[i];
                    if (data >= 'Ā')
                    {
                        isAllHankaku = false;
                        break;
                    }
                }
                if (!isAllHankaku)
                {
                    IntPtr hWnd;
                    if (Application.Current == null || Application.Current.MainWindow == null)
                    {
                        hWnd = IntPtr.Zero;
                    }
                    else
                    {
                        WindowInteropHelper helper = new WindowInteropHelper(Application.Current.MainWindow);
                        hWnd = helper.Handle;
                    }
                    this._uniscribe.ScriptPlace(inChars, charIndex, length, this._items, this._scriptTags, this._itemsCount, hWnd, this.JapaneseTextRunProperties.Typeface.FontFamily.Source, (int)this.JapaneseTextRunProperties.FontRenderingEmSize, this._logClust, this._charProps, this._glyphs, this._glyphProps, this._glyphCount, out this._advance, out this._goffset, out this._abc);
                }
            }
        }
    }
}

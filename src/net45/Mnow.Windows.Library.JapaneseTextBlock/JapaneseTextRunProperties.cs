using System;
using System.Globalization;
using System.Windows;
using System.Windows.Media;
using System.Windows.Media.TextFormatting;
namespace Mnow.Windows.Library
{
    public class JapaneseTextRunProperties : TextRunProperties
    {
        private Typeface _typeface;
        private double _emSize;
        private double _emHintingSize;
        private TextDecorationCollection _textDecorations;
        private Brush _foregroundBrush;
        private Brush _backgroundBrush;
        private BaselineAlignment _baselineAlignment;
        private CultureInfo _culture;
        public override Typeface Typeface
        {
            get
            {
                return this._typeface;
            }
        }
        public override double FontRenderingEmSize
        {
            get
            {
                return this._emSize;
            }
        }
        public override double FontHintingEmSize
        {
            get
            {
                return this._emHintingSize;
            }
        }
        public override TextDecorationCollection TextDecorations
        {
            get
            {
                return this._textDecorations;
            }
        }
        public override Brush ForegroundBrush
        {
            get
            {
                return this._foregroundBrush;
            }
        }
        public override Brush BackgroundBrush
        {
            get
            {
                return this._backgroundBrush;
            }
        }
        public override BaselineAlignment BaselineAlignment
        {
            get
            {
                return this._baselineAlignment;
            }
        }
        public override CultureInfo CultureInfo
        {
            get
            {
                return this._culture;
            }
        }
        public override TextRunTypographyProperties TypographyProperties
        {
            get
            {
                return null;
            }
        }
        public override TextEffectCollection TextEffects
        {
            get
            {
                return null;
            }
        }
        public override NumberSubstitution NumberSubstitution
        {
            get
            {
                return null;
            }
        }
        public JapaneseTextRunProperties(Typeface typeface, double size, double hintingSize, TextDecorationCollection textDecorations, Brush forgroundBrush, Brush backgroundBrush, BaselineAlignment baselineAlignment, CultureInfo culture)
        {
            this._typeface = typeface;
            this._emSize = size;
            this._emHintingSize = hintingSize;
            this._textDecorations = textDecorations;
            this._foregroundBrush = forgroundBrush;
            this._backgroundBrush = backgroundBrush;
            this._baselineAlignment = baselineAlignment;
            this._culture = culture;
        }
    }
}

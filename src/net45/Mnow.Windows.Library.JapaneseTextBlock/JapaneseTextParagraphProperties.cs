using System;
using System.Windows;
using System.Windows.Media.TextFormatting;
namespace Mnow.Windows.Library
{
    public class JapaneseTextParagraphProperties : TextParagraphProperties
    {
        private FlowDirection _flowDirection;
        private TextAlignment _textAlignment;
        private bool _firstLineInParagraph;
        private bool _alwaysCollapsible;
        private JapaneseTextRunProperties _defaultTextRunProperties;
        private TextWrapping _textWrap;
        private double _indent;
        private double _lineHeight;
        private bool _isVerticalWriting;
        public override FlowDirection FlowDirection
        {
            get
            {
                return this._flowDirection;
            }
        }
        public override TextAlignment TextAlignment
        {
            get
            {
                return this._textAlignment;
            }
        }
        public override bool FirstLineInParagraph
        {
            get
            {
                return this._firstLineInParagraph;
            }
        }
        public override bool AlwaysCollapsible
        {
            get
            {
                return this._alwaysCollapsible;
            }
        }
        public override TextRunProperties DefaultTextRunProperties
        {
            get
            {
                return this._defaultTextRunProperties;
            }
        }
        public virtual JapaneseTextRunProperties JapaneseTextRunProperties
        {
            get
            {
                return this._defaultTextRunProperties;
            }
        }
        public override TextWrapping TextWrapping
        {
            get
            {
                return this._textWrap;
            }
        }
        public override double LineHeight
        {
            get
            {
                return this._lineHeight;
            }
        }
        public override double Indent
        {
            get
            {
                return this._indent;
            }
        }
        public override TextMarkerProperties TextMarkerProperties
        {
            get
            {
                return null;
            }
        }
        public override double ParagraphIndent
        {
            get
            {
                return 0.0;
            }
        }
        public bool IsVerticalWriting
        {
            get
            {
                return this._isVerticalWriting;
            }
        }
        public JapaneseTextParagraphProperties(FlowDirection flowDirection, TextAlignment textAlignment, bool firstLineInParagraph, bool alwaysCollapsible, JapaneseTextRunProperties defaultTextRunProperties, TextWrapping textWrap, double lineHeight, double indent, bool isVerticalWriting)
        {
            this._flowDirection = flowDirection;
            this._textAlignment = textAlignment;
            this._firstLineInParagraph = firstLineInParagraph;
            this._alwaysCollapsible = alwaysCollapsible;
            this._defaultTextRunProperties = defaultTextRunProperties;
            this._textWrap = textWrap;
            this._lineHeight = lineHeight;
            this._indent = indent;
            this._isVerticalWriting = isVerticalWriting;
        }
    }
}

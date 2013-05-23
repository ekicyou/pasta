using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Globalization;
using System.Windows;
using System.Windows.Documents;
using System.Windows.Media;
using System.Windows.Media.TextFormatting;
namespace Mnow.Windows.Library
{
    public class JapaneseTextBlock : FrameworkElement
    {
        public static readonly DependencyProperty TextProperty = DependencyProperty.Register("Text", typeof(string), typeof(JapaneseTextBlock), new FrameworkPropertyMetadata(string.Empty, FrameworkPropertyMetadataOptions.AffectsMeasure | FrameworkPropertyMetadataOptions.AffectsRender));
        public static readonly DependencyProperty IsVerticalProperty = DependencyProperty.Register("IsVertical", typeof(bool), typeof(JapaneseTextBlock), new FrameworkPropertyMetadata(false, FrameworkPropertyMetadataOptions.AffectsMeasure | FrameworkPropertyMetadataOptions.AffectsRender));
        public static readonly DependencyProperty TextWrappingProperty = DependencyProperty.Register("TextWrapping", typeof(TextWrapping), typeof(JapaneseTextBlock), new FrameworkPropertyMetadata(TextWrapping.Wrap, FrameworkPropertyMetadataOptions.AffectsMeasure | FrameworkPropertyMetadataOptions.AffectsRender));
        public static readonly DependencyProperty FontFamilyProperty = TextElement.FontFamilyProperty.AddOwner(typeof(JapaneseTextBlock));
        public static readonly DependencyProperty FontStyleProperty = TextElement.FontStyleProperty.AddOwner(typeof(JapaneseTextBlock));
        public static readonly DependencyProperty FontWeightProperty = TextElement.FontWeightProperty.AddOwner(typeof(JapaneseTextBlock));
        public static readonly DependencyProperty FontStretchProperty = TextElement.FontStretchProperty.AddOwner(typeof(JapaneseTextBlock));
        public static readonly DependencyProperty FontSizeProperty = TextElement.FontSizeProperty.AddOwner(typeof(JapaneseTextBlock));
        public static readonly DependencyProperty ForegroundProperty = TextElement.ForegroundProperty.AddOwner(typeof(JapaneseTextBlock));
        public static readonly DependencyProperty BackgroundProperty = TextElement.BackgroundProperty.AddOwner(typeof(JapaneseTextBlock));
        public static readonly DependencyProperty PaddingProperty = Block.PaddingProperty.AddOwner(typeof(JapaneseTextBlock), new FrameworkPropertyMetadata(default(Thickness), FrameworkPropertyMetadataOptions.AffectsMeasure | FrameworkPropertyMetadataOptions.AffectsRender));
        [Category("Common")]
        public string Text
        {
            get
            {
                return (string)base.GetValue(JapaneseTextBlock.TextProperty);
            }
            set
            {
                base.SetValue(JapaneseTextBlock.TextProperty, value);
            }
        }
        [Category("Layout")]
        public bool IsVertical
        {
            get
            {
                return (bool)base.GetValue(JapaneseTextBlock.IsVerticalProperty);
            }
            set
            {
                base.SetValue(JapaneseTextBlock.IsVerticalProperty, value);
            }
        }
        [Category("Text")]
        public TextWrapping TextWrapping
        {
            get
            {
                return (TextWrapping)base.GetValue(JapaneseTextBlock.TextWrappingProperty);
            }
            set
            {
                base.SetValue(JapaneseTextBlock.TextWrappingProperty, value);
            }
        }
        [Category("Text"), Localizability(LocalizationCategory.Font)]
        public FontFamily FontFamily
        {
            get
            {
                return (FontFamily)base.GetValue(JapaneseTextBlock.FontFamilyProperty);
            }
            set
            {
                base.SetValue(JapaneseTextBlock.FontFamilyProperty, value);
            }
        }
        [Category("Text")]
        public FontStyle FontStyle
        {
            get
            {
                return (FontStyle)base.GetValue(JapaneseTextBlock.FontStyleProperty);
            }
            set
            {
                base.SetValue(JapaneseTextBlock.FontStyleProperty, value);
            }
        }
        [Category("Text")]
        public FontWeight FontWeight
        {
            get
            {
                return (FontWeight)base.GetValue(JapaneseTextBlock.FontWeightProperty);
            }
            set
            {
                base.SetValue(JapaneseTextBlock.FontWeightProperty, value);
            }
        }
        [Category("Text")]
        public FontStretch FontStretch
        {
            get
            {
                return (FontStretch)base.GetValue(JapaneseTextBlock.FontStretchProperty);
            }
            set
            {
                base.SetValue(JapaneseTextBlock.FontStretchProperty, value);
            }
        }
        [Category("Text"), TypeConverter(typeof(FontSizeConverter)), Localizability(LocalizationCategory.None)]
        public double FontSize
        {
            get
            {
                return (double)base.GetValue(JapaneseTextBlock.FontSizeProperty);
            }
            set
            {
                base.SetValue(JapaneseTextBlock.FontSizeProperty, value);
            }
        }
        [Category("Brushes")]
        public Brush Foreground
        {
            get
            {
                return (Brush)base.GetValue(JapaneseTextBlock.ForegroundProperty);
            }
            set
            {
                base.SetValue(JapaneseTextBlock.ForegroundProperty, value);
            }
        }
        [Category("Brushes")]
        public Brush Background
        {
            get
            {
                return (Brush)base.GetValue(JapaneseTextBlock.BackgroundProperty);
            }
            set
            {
                base.SetValue(JapaneseTextBlock.BackgroundProperty, value);
            }
        }
        [Category("Layout")]
        public Thickness Padding
        {
            get
            {
                return (Thickness)base.GetValue(JapaneseTextBlock.PaddingProperty);
            }
            set
            {
                base.SetValue(JapaneseTextBlock.PaddingProperty, value);
            }
        }
        public static void SetText(DependencyObject element, string value)
        {
            if (element == null)
            {
                throw new ArgumentNullException("element");
            }
            element.SetValue(JapaneseTextBlock.TextProperty, value);
        }
        public static string GetText(DependencyObject element)
        {
            if (element == null)
            {
                throw new ArgumentNullException("element");
            }
            return (string)element.GetValue(JapaneseTextBlock.TextProperty);
        }
        public static void SetIsVertical(DependencyObject element, bool value)
        {
            if (element == null)
            {
                throw new ArgumentNullException("element");
            }
            element.SetValue(JapaneseTextBlock.IsVerticalProperty, value);
        }
        public static bool GetIsVertical(DependencyObject element)
        {
            if (element == null)
            {
                throw new ArgumentNullException("element");
            }
            return (bool)element.GetValue(JapaneseTextBlock.IsVerticalProperty);
        }
        public static void SetTextWrapping(DependencyObject element, TextWrapping value)
        {
            if (element == null)
            {
                throw new ArgumentNullException("element");
            }
            element.SetValue(JapaneseTextBlock.TextWrappingProperty, value);
        }
        public static TextWrapping GetTextWrapping(DependencyObject element)
        {
            if (element == null)
            {
                throw new ArgumentNullException("element");
            }
            return (TextWrapping)element.GetValue(JapaneseTextBlock.TextWrappingProperty);
        }
        protected override void OnRender(DrawingContext drawingContext)
        {
            base.OnRender(drawingContext);
            Rect rect = new Rect(0.0, 0.0, base.ActualWidth, base.ActualHeight);
            JapaneseTextSource source = new JapaneseTextSource();
            source.Text = this.Text;
            JapaneseTextParagraphProperties textParagraphProperties = this.MakeTextProperties();
            source.JapaneseTextRunProperties = (JapaneseTextRunProperties)textParagraphProperties.DefaultTextRunProperties;
            source.IsVarticalWriting = textParagraphProperties.IsVerticalWriting;
            if (textParagraphProperties.DefaultTextRunProperties.BackgroundBrush != null)
            {
                drawingContext.DrawRectangle(textParagraphProperties.DefaultTextRunProperties.BackgroundBrush, null, rect);
            }
            Rect paddingRect = new Rect(this.Padding.Left, this.Padding.Top, Math.Max(0.0, base.ActualWidth - this.Padding.Left - this.Padding.Right), Math.Max(0.0, base.ActualHeight - this.Padding.Top - this.Padding.Bottom));
            Point center = new Point((paddingRect.Left + paddingRect.Right) / 2.0, (paddingRect.Top + paddingRect.Bottom) / 2.0);
            Point startPosition;
            double paragraphWidth;
            if (textParagraphProperties.IsVerticalWriting)
            {
                Point origin = paddingRect.TopRight;
                Transform transOrigin = new RotateTransform(-90.0, center.X, center.Y);
                startPosition = transOrigin.Transform(origin);
                paragraphWidth = Math.Abs(paddingRect.Height);
                Transform trans = new RotateTransform(90.0, center.X, center.Y);
                drawingContext.PushTransform(trans);
            }
            else
            {
                startPosition = paddingRect.TopLeft;
                paragraphWidth = Math.Abs(paddingRect.Width);
            }
            startPosition.Y += textParagraphProperties.JapaneseTextRunProperties.FontRenderingEmSize;
            int textStorePosition = 0;
            Point linePosition = startPosition;
            TextFormatter formatter = TextFormatter.Create();
            while (textStorePosition < source.Text.Length)
            {
                using (TextLine textLine = formatter.FormatLine(source, textStorePosition, paragraphWidth, textParagraphProperties, null))
                {
                    foreach (IndexedGlyphRun indexedrun in textLine.GetIndexedGlyphRuns())
                    {
                        if (textParagraphProperties.IsVerticalWriting)
                        {
                            source.UniscribeIndexedGlyphRun(indexedrun);
                            Rect runRect;
                            if (source.GlyphCount != 0 && source.Glyphs[0] != 0)
                            {
                                Point ansiLinePosition = linePosition;
                                ansiLinePosition.Y -= textParagraphProperties.JapaneseTextRunProperties.FontRenderingEmSize / 2.0;
                                runRect = this.DrawIndexedGlyphRun(drawingContext, indexedrun, ansiLinePosition, source, true);
                            }
                            else
                            {
                                Point ansiLinePosition = linePosition;
                                ansiLinePosition.Y -= textParagraphProperties.JapaneseTextRunProperties.FontRenderingEmSize / 10.0;
                                runRect = this.DrawIndexedGlyphRun(drawingContext, indexedrun, ansiLinePosition, source, false);
                            }
                            linePosition.X += runRect.Width;
                        }
                        else
                        {
                            Rect runRect = this.DrawIndexedGlyphRun(drawingContext, indexedrun, linePosition, source, false);
                            linePosition.X += runRect.Width;
                        }
                    }
                    textStorePosition += textLine.Length;
                    linePosition.X = startPosition.X;
                    linePosition.Y += textLine.Height;
                }
            }
            if (textParagraphProperties.IsVerticalWriting)
            {
                drawingContext.Pop();
            }
        }
        private Rect DrawIndexedGlyphRun(DrawingContext drawingContext, IndexedGlyphRun indexedrun, Point linePosition, JapaneseTextSource source, bool isVerticalWriting)
        {
            GlyphRun run;
            if (isVerticalWriting)
            {
                List<double> advanceWidths = new List<double>();
                int[] advance2 = source.Advance;
                for (int i = 0; i < advance2.Length; i++)
                {
                    int advance = advance2[i];
                    advanceWidths.Add((double)advance);
                }
                run = new GlyphRun(indexedrun.GlyphRun.GlyphTypeface, indexedrun.GlyphRun.BidiLevel, true, indexedrun.GlyphRun.FontRenderingEmSize, source.Glyphs, linePosition, indexedrun.GlyphRun.AdvanceWidths, indexedrun.GlyphRun.GlyphOffsets, indexedrun.GlyphRun.Characters, indexedrun.GlyphRun.DeviceFontName, indexedrun.GlyphRun.ClusterMap, indexedrun.GlyphRun.CaretStops, indexedrun.GlyphRun.Language);
            }
            else
            {
                run = new GlyphRun(indexedrun.GlyphRun.GlyphTypeface, indexedrun.GlyphRun.BidiLevel, false, indexedrun.GlyphRun.FontRenderingEmSize, indexedrun.GlyphRun.GlyphIndices, linePosition, indexedrun.GlyphRun.AdvanceWidths, indexedrun.GlyphRun.GlyphOffsets, indexedrun.GlyphRun.Characters, indexedrun.GlyphRun.DeviceFontName, indexedrun.GlyphRun.ClusterMap, indexedrun.GlyphRun.CaretStops, indexedrun.GlyphRun.Language);
            }
            drawingContext.DrawGlyphRun(source.JapaneseTextRunProperties.ForegroundBrush, run);
            return run.ComputeAlignmentBox();
        }
        private JapaneseTextParagraphProperties MakeTextProperties()
        {
            JapaneseTextRunProperties textRunProperties = new JapaneseTextRunProperties(new Typeface(this.FontFamily, this.FontStyle, this.FontWeight, this.FontStretch), this.FontSize, this.FontSize, null, this.Foreground, this.Background, BaselineAlignment.Baseline, CultureInfo.CurrentUICulture);
            return new JapaneseTextParagraphProperties(FlowDirection.LeftToRight, TextAlignment.Left, false, false, textRunProperties, this.TextWrapping, 0.0, 0.0, this.IsVertical);
        }
        public static void SetFontFamily(DependencyObject element, FontFamily value)
        {
            if (element == null)
            {
                throw new ArgumentNullException("element");
            }
            element.SetValue(JapaneseTextBlock.FontFamilyProperty, value);
        }
        public static FontFamily GetFontFamily(DependencyObject element)
        {
            if (element == null)
            {
                throw new ArgumentNullException("element");
            }
            return (FontFamily)element.GetValue(JapaneseTextBlock.FontFamilyProperty);
        }
        public static void SetFontStyle(DependencyObject element, FontStyle value)
        {
            if (element == null)
            {
                throw new ArgumentNullException("element");
            }
            element.SetValue(JapaneseTextBlock.FontStyleProperty, value);
        }
        public static FontStyle GetFontStyle(DependencyObject element)
        {
            if (element == null)
            {
                throw new ArgumentNullException("element");
            }
            return (FontStyle)element.GetValue(JapaneseTextBlock.FontStyleProperty);
        }
        public static void SetFontWeight(DependencyObject element, FontWeight value)
        {
            if (element == null)
            {
                throw new ArgumentNullException("element");
            }
            element.SetValue(JapaneseTextBlock.FontWeightProperty, value);
        }
        public static FontWeight GetFontWeight(DependencyObject element)
        {
            if (element == null)
            {
                throw new ArgumentNullException("element");
            }
            return (FontWeight)element.GetValue(JapaneseTextBlock.FontWeightProperty);
        }
        public static void SetFontStretch(DependencyObject element, FontStretch value)
        {
            if (element == null)
            {
                throw new ArgumentNullException("element");
            }
            element.SetValue(JapaneseTextBlock.FontStretchProperty, value);
        }
        public static FontStretch GetFontStretch(DependencyObject element)
        {
            if (element == null)
            {
                throw new ArgumentNullException("element");
            }
            return (FontStretch)element.GetValue(JapaneseTextBlock.FontStretchProperty);
        }
        public static void SetFontSize(DependencyObject element, double value)
        {
            if (element == null)
            {
                throw new ArgumentNullException("element");
            }
            element.SetValue(JapaneseTextBlock.FontSizeProperty, value);
        }
        public static double GetFontSize(DependencyObject element)
        {
            if (element == null)
            {
                throw new ArgumentNullException("element");
            }
            return (double)element.GetValue(JapaneseTextBlock.FontSizeProperty);
        }
        public static void SetForeground(DependencyObject element, Brush value)
        {
            if (element == null)
            {
                throw new ArgumentNullException("element");
            }
            element.SetValue(JapaneseTextBlock.ForegroundProperty, value);
        }
        public static Brush GetForeground(DependencyObject element)
        {
            if (element == null)
            {
                throw new ArgumentNullException("element");
            }
            return (Brush)element.GetValue(JapaneseTextBlock.ForegroundProperty);
        }
        public static void SetBackground(DependencyObject element, Brush value)
        {
            if (element == null)
            {
                throw new ArgumentNullException("element");
            }
            element.SetValue(JapaneseTextBlock.BackgroundProperty, value);
        }
        public static Brush GetBackground(DependencyObject element)
        {
            if (element == null)
            {
                throw new ArgumentNullException("element");
            }
            return (Brush)element.GetValue(JapaneseTextBlock.BackgroundProperty);
        }
        public static void SetPadding(DependencyObject element, Thickness value)
        {
            if (element == null)
            {
                throw new ArgumentNullException("element");
            }
            element.SetValue(JapaneseTextBlock.PaddingProperty, value);
        }
        public static Thickness GetPadding(DependencyObject element)
        {
            if (element == null)
            {
                throw new ArgumentNullException("element");
            }
            return (Thickness)element.GetValue(JapaneseTextBlock.PaddingProperty);
        }
    }
}

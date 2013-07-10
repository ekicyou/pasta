using System;
using System.Drawing;

namespace CSUtil.Forms
{
    /// <summary>
    /// アイテム情報構造体。
    /// </summary>
    public struct LogItem
    {
        /// <summary>
        /// タイムスタンプ。
        /// </summary>
        public readonly DateTime TimeStamp;

        /// <summary>
        /// 表示テキスト。
        /// </summary>
        public readonly string Text;

        /// <summary>
        /// 文字色。
        /// </summary>
        public readonly Color ForeColor;

        /// <summary>
        /// 背景色。
        /// </summary>
        public readonly Color BackColor;

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="timeStamp"></param>
        /// <param name="text"></param>
        /// <param name="foreColor"></param>
        /// <param name="backColor"></param>
        public LogItem(DateTime timeStamp, string text, Color foreColor, Color backColor)
        {
            TimeStamp = timeStamp;
            Text = text;
            ForeColor = foreColor;
            BackColor = backColor;
        }

        /// <summary>
        /// オブジェクトの文字列表現を返します。
        /// </summary>
        /// <returns>オブジェクトの文字列表現</returns>
        public override string ToString()
        {
            return Text;
        }
    }
}
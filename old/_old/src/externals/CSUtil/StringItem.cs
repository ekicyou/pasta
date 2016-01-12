
namespace CSUtil
{
    /// <summary>
    /// ToString値としてコンストラクタで与えられた文字列を返す構造体。
    /// </summary>
    /// <typeparam name="T"></typeparam>
    public class StringItem<T>
    {
        private T value;
        private string text;

        /// <summary>
        /// 値。
        /// </summary>
        public T Value { get { return value; } }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="value">値</param>
        /// <param name="text">文字列</param>
        public StringItem(T value, string text)
        {
            this.value = value;
            this.text = text;
        }

        /// <summary>
        /// コンストラクタで設定された文字列を返します。
        /// </summary>
        /// <returns></returns>
        public override string ToString()
        {
            return text;
        }
    }
}
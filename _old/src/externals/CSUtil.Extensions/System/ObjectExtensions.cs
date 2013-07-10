
namespace System
{
    /// <summary>
    /// オブジェクト拡張
    /// </summary>
    public static class ObjectExtensions
    {
        /// <summary>
        /// nullかどうかを判定します。
        /// </summary>
        /// <param name="obj"></param>
        /// <returns></returns>
        public static bool IsNull(this object obj) { return obj == null; }

        /// <summary>
        /// 両方がnullならtrueを返します。
        /// </summary>
        /// <param name="obj"></param>
        /// <param name="other"></param>
        /// <returns></returns>
        public static bool IsEqNull(object obj, object other)
        {
            return obj.IsNull() && other.IsNull();
        }

        /// <summary>
        /// いずれかがnullならtrueを返します。
        /// </summary>
        /// <param name="obj"></param>
        /// <param name="other"></param>
        /// <returns></returns>
        public static bool IsNotEqNull(object obj, object other)
        {
            return obj.IsNull() ^ other.IsNull();
        }

    }
}
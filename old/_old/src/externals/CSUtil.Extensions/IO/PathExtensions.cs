
namespace System.IO
{
    /// <summary>
    /// Pathクラス拡張。
    /// </summary>
    public static class PathExtensions
    {
        /// <summary>
        /// 指定したパス文字列のファイル名を拡張子を付けずに返します。
        /// </summary>
        /// <param name="path"></param>
        /// <returns></returns>
        public static string GetFileNameWithoutExtension(this string path)
        {
            return Path.GetFileNameWithoutExtension(path);
        }

        /// <summary>
        /// 指定したパス文字列のファイル名と拡張子を返します。
        /// </summary>
        /// <param name="path"></param>
        /// <returns></returns>
        public static string GetFileName(this string path)
        {
            return Path.GetFileName(path);
        }


        /// <summary>
        /// 指定したパス文字列の拡張子を返します。
        /// </summary>
        /// <param name="path">拡張子の取得元のパス文字列。</param>
        /// <returns>
        /// 指定したパスの拡張子 ("." を含む)、null、または System.String.Empty を含む System.String。path
        /// が null の場合、GetExtension は null を返します。path が拡張子情報を持たない場合、GetExtension は Empty
        /// を返します。
        /// </returns>
        public static string GetPathExtension(this string path)
        {
            return Path.GetExtension(path);
        }


        /// <summary>
        /// パス文字列の拡張子を変更します。
        /// </summary>
        /// <param name="path">変更するパス情報。パスに、System.IO.Path.GetInvalidPathChars() で定義された文字を含めることはできません。</param>
        /// <param name="extension">新しい拡張子 (先行ピリオド付き、またはなし)。null を指定して、path から既存の拡張子を削除します。</param>
        /// <returns>変更されたパス情報を含む文字列。 Windows ベースのデスクトップ プラットフォームでは、path が null または空の文字列 ("")の場合、パス情報は変更されずに返されます。extension が null の場合は、返される文字列に、削除した拡張子が付いた指定したパスが含まれます。pathに拡張子がなく、extension が null でない場合は、返されるパス文字列に path の末尾に追加される extension が含まれます。</returns>
        public static string ChangeExtension(this string path, string extension)
        {
            return Path.ChangeExtension(path, extension);
        }

        /// <summary>
        /// 指定したパス文字列のディレクトリ情報を返します。
        /// </summary>
        /// <param name="path"></param>
        /// <returns></returns>
        public static string GetDirectoryName(this string path)
        {
            return Path.GetDirectoryName(path);
        }

        /// <summary>
        /// 指定したパス文字列の絶対パスを返します。
        /// </summary>
        /// <param name="path"></param>
        /// <returns></returns>
        public static string GetFullPath(this string path)
        {
            return Path.GetFullPath(path);
        }

        /// <summary>
        /// 2 つの文字列を 1 つのパスに結合します。
        /// </summary>
        /// <param name="p1"></param>
        /// <param name="p2"></param>
        /// <returns></returns>
        public static string PathCombine(this string p1, string p2)
        {
            return Path.Combine(p1, p2);
        }

        /// <summary>
        /// 複数のの文字列を 1 つのパスに結合します。
        /// </summary>
        /// <param name="p1"></param>
        /// <param name="p2"></param>
        /// <returns></returns>
        public static string PathCombine(this string p1, params string[] paths)
        {
            foreach (var p in paths) p1 = p1.PathCombine(p);
            return p1;
        }

    }
}
using System;
using System.Collections.Generic;
using System.Diagnostics;

namespace CSUtil.Forms
{
    /// <summary>
    /// ILogItemインターフェースを持つ要素を管理するリストです。
    /// </summary>
    /// <typeparam name="T"></typeparam>
    public class LogItemCollection<T> : IList<LogItem> where T : ILogItem
    {
        /// <summary>
        /// 子要素配列の大きさを決定する定数
        /// </summary>
        private const int DIV_ARRAYS = 5;

        /// <summary>
        /// アイテム名称（ロギング用）
        /// </summary>
        public readonly string Name;

        /// <summary>
        /// 列削除を行なう閾値。
        /// </summary>
        private readonly int LimitCount;

        /// <summary>
        /// データ管理オブジェクト。
        /// </summary>
        private readonly List<T[]> Arrays;

        /// <summary>
        /// 子要素配列の最大値。
        /// </summary>
        private readonly int ChildMaxCount;

        /// <summary>
        /// 現在参照している親領域。
        /// </summary>
        private int ArraysIndex = 0;

        /// <summary>
        /// 現在参照している子領域。
        /// </summary>
        private int ChildNextIndex = 0;

        /// <summary>
        /// 全体の要素数。
        /// </summary>
        private int count = 0;

        /// <summary>
        /// 外部参照同期オブジェクト。
        /// </summary>
        public object SyncRoot { get { return Arrays; } }

        /// <summary>
        /// 追加処理専用の同期オブジェクト。
        /// </summary>
        private object lockAdd = new object();

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="name"></param>
        /// <param name="maxCount"></param>
        public LogItemCollection(string name, int maxCount)
        {
            if (maxCount < DIV_ARRAYS) throw new ArgumentOutOfRangeException("maxCount", "最低" + DIV_ARRAYS + "以上の値を指定してください");
            Name = name;
            LimitCount = maxCount;
            ChildMaxCount = LimitCount / DIV_ARRAYS;
            Arrays = new List<T[]>();
        }

        /// <summary>
        /// 要素を追加します。
        /// 要素数がリミット値を超えた場合、trueを返します。
        /// </summary>
        /// <param name="item"></param>
        /// <returns></returns>
        public bool Add(T item)
        {
            lock (lockAdd) {
                // 追加処理
                if (ChildNextIndex == 0) Arrays.Add(new T[ChildMaxCount]);
                Arrays[ArraysIndex][ChildNextIndex] = item;
                ChildNextIndex++;
                if (ChildNextIndex >= ChildMaxCount) {
                    ArraysIndex++;
                    ChildNextIndex = 0;
                }
                // 現在の要素数の更新
                count = ArraysIndex * ChildMaxCount + ChildNextIndex;

                // 要素数がリミット値を超えていたらtrue;
                return count > LimitCount;
            }
        }

        /// <summary>
        /// 要素数をリミット値以下に縮小します。
        /// 縮小数を返します。
        /// </summary>
        /// <returns></returns>
        public int RemoveLimit()
        {
            if (count <= LimitCount) return 0;
            lock (lockAdd) lock (SyncRoot) return RemoveLimitImpl();
        }
        private int RemoveLimitImpl()
        {
            // ロック後に再度縮退の可否判定
            if (count <= LimitCount) return 0;

            // 削除する親行列数を決定
            int RemoveItemCount = 0;  // 削除行数
            int RemoveArraysCount = 0;  // 削除配列数
            foreach (T[] childs in Arrays) {
                if (childs == null) break;
                RemoveArraysCount++;
                RemoveItemCount += childs.Length;
                if ((count - RemoveItemCount) < LimitCount) break;
            }
            if (RemoveArraysCount <= 0) return 0;

            // 削除ログ
            Debug.WriteLine(string.Format("{0}:ログバッファ縮退 {1}-->{2}(削除行数 {3})",
              Name, count, count - RemoveItemCount, RemoveItemCount));

            // 削除処理
            Arrays.RemoveRange(0, RemoveArraysCount);
            ArraysIndex -= RemoveArraysCount;
            count -= RemoveItemCount;
            return RemoveItemCount;
        }



        /// <summary>
        /// 要素を取得します。
        /// </summary>
        /// <param name="index"></param>
        /// <returns></returns>
        public LogItem GetLogItem(int index)
        {
            lock (SyncRoot) {
                // 範囲外ならブランクデータを返す（例外にはしない）
                if (index >= Count) return BlankItem;
                if (index < 0) return BlankItem;

                // 参照場所を計算する
                int iP = index / ChildMaxCount;
                int iC = index % ChildMaxCount;

                // 値を返す
                return Arrays[iP][iC].GetLogItem();
            }
        }
        private static LogItem BlankItem = new LogItem(DateTime.MinValue, "", System.Drawing.Color.Black, System.Drawing.Color.Transparent);

        /// <summary>
        /// 要素数を返します。
        /// </summary>
        public int Count
        {
            get { lock (SyncRoot) return count; }
        }

        /// <summary>
        /// 要素へアクセスします。
        /// </summary>
        /// <param name="index"></param>
        /// <returns></returns>
        public LogItem this[int index]
        {
            get { return GetLogItem(index); }
            set { throw new Exception("The method or operation is not implemented."); }
        }

        /// <summary>
        /// 指定日時範囲のデータを列挙します。
        /// </summary>
        /// <param name="from"></param>
        /// <param name="to"></param>
        /// <returns></returns>
        public IEnumerable<LogItem> EnLogItem(DateTime from, DateTime to)
        {
            lock (SyncRoot) {
                int end = Count;
                for (int i = 0; i < end; i++) {
                    LogItem item = this[i];
                    if (item.TimeStamp < from) continue;
                    if (item.TimeStamp > to) continue;
                    yield return item;
                }
            }
        }

        /// <summary>
        /// 指定位置の間に文字列が存在するかどうかを検索し、場所をindexで返します。
        /// 存在しない場合は-1を返します。
        /// </summary>
        /// <param name="text">検索文字列</param>
        /// <param name="from">開始Index</param>
        /// <param name="to">終了Index</param>
        /// <param name="isNext">後方検索の場合true</param>
        /// <returns></returns>
        public int FindText(string text, int from, int to, bool isNext)
        {
            lock (SyncRoot) {
                int cnt = Count;
                if (cnt <= 0) return -1;
                if (from < 0) from = 0;
                if (to < 0) to = 0;
                if (from > cnt - 1) from = cnt - 1;
                if (to > cnt - 1) to = cnt - 1;
                if (from > to) from = to;
                if (isNext) {
                    for (int i = from; i <= to; i++) {
                        string itemText = this[i].ToString();
                        if (itemText.Contains(text)) return i;
                    }
                }
                else {
                    for (int i = to; i >= from; i--) {
                        string itemText = this[i].ToString();
                        if (itemText.Contains(text)) return i;
                    }
                }
                return -1;
            }
        }

        /// <summary>
        /// 指定位置から最後までの間に文字列が存在するかどうかを検索し、場所をindexで返します。
        /// 存在しない場合は-1を返します。
        /// </summary>
        /// <param name="text">検索文字列</param>
        /// <param name="offset">開始index</param>
        /// <param name="isNext">後方検索の場合true</param>
        /// <returns></returns>
        public int FindText(string text, int offset, bool isNext)
        {
            lock (SyncRoot) {
                if (isNext) return FindText(text, offset, Count - 1, isNext);
                else return FindText(text, 0, offset, isNext);
            }
        }

        /// <summary>
        /// 先頭から最後までの間に文字列が存在するかどうかを検索し、場所をindexで返します。
        /// 存在しない場合は-1を返します。
        /// </summary>
        /// <param name="text"></param>
        /// <param name="isNext">後方検索の場合true</param>
        /// <returns></returns>
        public int FindText(string text, bool isNext)
        {
            lock (SyncRoot) {
                if (isNext) return FindText(text, 0, isNext);
                else return FindText(text, Count - 1, isNext);
            }
        }



        #region IList<LogItem> メンバ
        /// <summary>
        /// このメソッドは実装されません。
        /// </summary>
        /// <param name="item"></param>
        /// <returns></returns>
        public int IndexOf(LogItem item)
        {
            throw new Exception("The method or operation is not implemented.");
        }

        /// <summary>
        /// このメソッドは実装されません。
        /// </summary>
        /// <param name="index"></param>
        /// <param name="item"></param>
        public void Insert(int index, LogItem item)
        {
            throw new Exception("The method or operation is not implemented.");
        }

        /// <summary>
        /// このメソッドは実装されません。
        /// </summary>
        /// <param name="index"></param>
        public void RemoveAt(int index)
        {
            throw new Exception("The method or operation is not implemented.");
        }

        #endregion

        #region ICollection<LogItem> メンバ
        /// <summary>
        /// このメソッドは実装されません。
        /// </summary>
        /// <param name="item"></param>
        public void Add(LogItem item)
        {
            throw new Exception("The method or operation is not implemented.");
        }
        /// <summary>
        /// このメソッドは実装されません。
        /// </summary>
        public void Clear()
        {
            throw new Exception("The method or operation is not implemented.");
        }
        /// <summary>
        /// このメソッドは実装されません。
        /// </summary>
        /// <param name="item"></param>
        /// <returns></returns>
        public bool Contains(LogItem item)
        {
            throw new Exception("The method or operation is not implemented.");
        }
        /// <summary>
        /// このメソッドは実装されません。
        /// </summary>
        /// <param name="array"></param>
        /// <param name="arrayIndex"></param>
        public void CopyTo(LogItem[] array, int arrayIndex)
        {
            throw new Exception("The method or operation is not implemented.");
        }

        /// <summary>
        /// 読み込み専用ならtrue。常にtrueを返します。
        /// </summary>
        public bool IsReadOnly
        {
            get { return true; }
        }

        /// <summary>
        /// このメソッドは実装されません。
        /// </summary>
        /// <param name="item"></param>
        /// <returns></returns>
        public bool Remove(LogItem item)
        {
            throw new Exception("The method or operation is not implemented.");
        }

        #endregion

        #region IEnumerable<LogItem> メンバ

        /// <summary>
        /// 列挙子を返します。
        /// </summary>
        /// <returns></returns>
        public IEnumerator<LogItem> GetEnumerator()
        {
            return EnLogItem().GetEnumerator();
        }
        private IEnumerable<LogItem> EnLogItem()
        {
            lock (SyncRoot) {
                int end = Count;
                for (int i = 0; i < end; i++) yield return this[i];
            }
        }

        #endregion

        #region IEnumerable メンバ
        /// <summary>
        /// 列挙子を返します。
        /// </summary>
        /// <returns></returns>
        System.Collections.IEnumerator System.Collections.IEnumerable.GetEnumerator()
        {
            return EnObject().GetEnumerator();
        }
        IEnumerable<object> EnObject()
        {
            lock (SyncRoot) {
                int end = Count;
                for (int i = 0; i < end; i++) yield return this[i];
            }
        }


        #endregion
    }
}
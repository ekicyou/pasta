using System;
using System.Collections.Generic;

namespace CSUtil
{
    /// <summary>
    /// Byte配列をbool値の集合として扱います。
    /// </summary>
    public class BoolCollection : IList<bool>
    {
        private readonly byte[] data;
        private readonly int startIndex;
        private readonly int length;

        /// <summary>
        /// コンストラクタ。
        /// 与えられた配列をbool値の集合として取り扱います。
        /// </summary>
        /// <param name="data">byte配列。</param>
        /// <param name="startIndex">開始index</param>
        /// <param name="length">配列長</param>
        public BoolCollection(byte[] data, int startIndex, int length)
        {
            this.data = data;
            this.startIndex = startIndex;
            this.length = length;
        }

        /// <summary>
        /// コンストラクタ。
        /// 与えられた配列をbool値の集合として取り扱います。
        /// </summary>
        /// <param name="data">byte配列。</param>
        public BoolCollection(byte[] data)
            : this(data, 0, data.Length) { }


        /// <summary>
        /// 配列のサイズを取得します。
        /// </summary>
        public int Count { get { return length * 8; } }


        /// <summary>
        /// BitチェンジがあったIndexの情報を列挙します。
        /// </summary>
        /// <param name="old">旧BoolCollection</param>
        /// <param name="now">新BoolCollection</param>
        /// <returns>key/valueのペア。keyがIndex値、valueがbit値。</returns>
        public static IEnumerable<KeyValuePair<int, bool>> EnumBitChange(BoolCollection old, BoolCollection now)
        {
            if (old.length != now.length) {
                throw new InvalidOperationException("比較するBoolCollectionのサイズは同一である必要があります");
            }
            for (int i = 0; i < old.length; i++) {
                byte o = old.data[old.startIndex + i];
                byte n = now.data[now.startIndex + i];
                if (o == n) continue;
                int xor = o ^ n;
                for (int j = 0; j < 8; j++) {
                    if ((xor & (1 << j)) == 0) continue;
                    yield return new KeyValuePair<int, bool>(i * 8 + j, (n & (1 << j)) != 0);
                }
            }
        }

        /// <summary>
        /// BitチェンジがあったIndexの情報を列挙します。
        /// </summary>
        /// <param name="old">旧byte配列</param>
        /// <param name="now">新byte配列</param>
        /// <returns>key/valueのペア。keyがIndex値、valueがbit値。</returns>
        public static IEnumerable<KeyValuePair<int, bool>> EnumBitChange(byte[] old, byte[] now)
        {
            BoolCollection oldBits = new BoolCollection(old, 0, old.Length);
            BoolCollection nowBits = new BoolCollection(now, 0, now.Length);
            return EnumBitChange(oldBits, nowBits);
        }


        #region IList<bool> メンバ

        /// <summary>
        /// このメソッドはサポートしません。
        /// </summary>
        /// <param name="item"></param>
        /// <returns></returns>
        public int IndexOf(bool item)
        {
            throw new Exception("The method or operation is not implemented.");
        }

        /// <summary>
        /// このメソッドはサポートしません。
        /// </summary>
        /// <param name="index"></param>
        /// <param name="item"></param>
        public void Insert(int index, bool item)
        {
            throw new Exception("The method or operation is not implemented.");
        }

        /// <summary>
        /// このメソッドはサポートしません。
        /// </summary>
        /// <param name="index"></param>
        public void RemoveAt(int index)
        {
            throw new Exception("The method or operation is not implemented.");
        }

        /// <summary>
        /// 指定されたインデックスのbool値を取得または設定します。
        /// </summary>
        /// <param name="index"></param>
        /// <returns></returns>
        public bool this[int index]
        {
            get
            {
                if (index < 0) throw new IndexOutOfRangeException();
                if (index >= Count) throw new IndexOutOfRangeException();
                int byteIndex = index / 8 + startIndex;
                int bitOffset = index % 8;
                byte pattrn = (byte)(1 << bitOffset);
                return (data[byteIndex] & pattrn) != 0;
            }
            set
            {
                if (index < 0) throw new IndexOutOfRangeException();
                if (index >= Count) throw new IndexOutOfRangeException();
                int byteIndex = index / 8 + startIndex;
                int bitOffset = index % 8;
                int org = data[byteIndex];
                int mask = ~(1 << bitOffset);
                int bits = ((value ? 1 : 0) << bitOffset);
                int result = (org & mask) | bits;
                data[byteIndex] = (byte)result;
            }
        }

        #endregion

        #region ICollection<bool> メンバ
        /// <summary>
        /// このメソッドはサポートしません。
        /// </summary>
        /// <param name="item"></param>
        public void Add(bool item)
        {
            throw new Exception("The method or operation is not implemented.");
        }

        /// <summary>
        /// このメソッドはサポートしません。
        /// </summary>
        public void Clear()
        {
            throw new Exception("The method or operation is not implemented.");
        }

        /// <summary>
        /// このメソッドはサポートしません。
        /// </summary>
        /// <param name="item"></param>
        /// <returns></returns>
        public bool Contains(bool item)
        {
            throw new Exception("The method or operation is not implemented.");
        }

        /// <summary>
        /// 要素を Array にコピーします。Array の特定のインデックスからコピーが開始されます。 
        /// </summary>
        /// <param name="array"></param>
        /// <param name="arrayIndex"></param>
        public void CopyTo(bool[] array, int arrayIndex)
        {
            for (int i = 0; i < Count; i++) array[arrayIndex + i] = this[i];
        }

        /// <summary>
        /// 読み取り専用かどうかを示す値を取得します。常にtrueを返します。
        /// </summary>
        public bool IsReadOnly
        {
            get { return true; }
        }

        /// <summary>
        /// このメソッドはサポートしません。
        /// </summary>
        /// <param name="item"></param>
        /// <returns></returns>
        public bool Remove(bool item)
        {
            throw new Exception("The method or operation is not implemented.");
        }

        #endregion

        #region IEnumerable<bool> メンバ

        /// <summary>
        /// boolコレクションを反復処理する列挙子を返します。 
        /// </summary>
        /// <returns></returns>
        public IEnumerator<bool> GetEnumerator()
        {
            return EnumerableBool().GetEnumerator();
        }
        private IEnumerable<bool> EnumerableBool()
        {
            for (int i = 0; i < Count; i++) {
                yield return this[i];
            }
        }

        #endregion


        #region IEnumerable メンバ

        /// <summary>
        /// objectコレクションを反復処理する列挙子を返します。 
        /// </summary>
        /// <returns></returns>
        System.Collections.IEnumerator System.Collections.IEnumerable.GetEnumerator()
        {
            return EnumerableObject().GetEnumerator();
        }
        private IEnumerable<object> EnumerableObject()
        {
            for (int i = 0; i < Count; i++) {
                yield return this[i];
            }
        }

        #endregion
    }
}
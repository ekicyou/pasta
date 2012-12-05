using System;
using System.Collections.Generic;

namespace CSUtil
{
    /// <summary>
    /// bool値の２次元配列を管理します。
    /// 特定行を指定した時の操作が高速になるように設計されています。
    /// サイズは64の倍数で丸められます。
    /// </summary>
    public class Bool2DimCollection
    {
        private int rowCapacity;
        private int columnCapacity;

        private const int ROUND_SIZE = 64;
        private ulong[] data;
        private int dataRowOffset;

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="columnCapacity"></param>
        /// <param name="rowCapacity"></param>
        public Bool2DimCollection(int columnCapacity, int rowCapacity)
        {
            if (columnCapacity <= 0) throw new ArgumentOutOfRangeException("columnCount", "０以上の値を指定してください。");
            if (rowCapacity <= 0) throw new ArgumentOutOfRangeException("rowCount", "０以上の値を指定してください。");
            this.rowCapacity = RoundValue(rowCapacity);
            this.columnCapacity = RoundValue(columnCapacity);
            this.dataRowOffset = this.columnCapacity / ROUND_SIZE;
            Clear();
        }
        private static int RoundValue(int v)
        {
            int d = v / ROUND_SIZE;
            if ((v % ROUND_SIZE) > 0) d++;
            return d * ROUND_SIZE;
        }

        /// <summary>
        /// 正方配列を作成するコンストラクタ。
        /// </summary>
        /// <param name="capacity"></param>
        public Bool2DimCollection(int capacity) : this(capacity, capacity) { }

        /// <summary>行数</summary>
        public int RowCount { get { return rowCapacity; } }

        /// <summary>列数</summary>
        public int ColmunCount { get { return columnCapacity; } }

        /// <summary>データ配列へのアクセスオフセット</summary>
        private int DataRowOffset { get { return dataRowOffset; } }

        #region 基本列挙操作
        /// <summary>
        /// 指定した行のデータを列挙します。
        /// </summary>
        /// <param name="rowIndex"></param>
        /// <returns></returns>
        private IEnumerable<ulong> EnRowDataItems(int rowIndex)
        {
            int s = DataRowOffset * rowIndex;
            int e = DataRowOffset * (rowIndex + 1);
            for (int i = s; i < e; i++) yield return data[i];
        }

        /// <summary>
        /// 指定した行のうち、要素がtrueである列の場所を列挙します。
        /// </summary>
        /// <param name="rowIndex"></param>
        /// <returns></returns>
        public IEnumerable<int> EnRowTrueIndex(int rowIndex)
        {
            for (int colmunIndex = 0; colmunIndex < ColmunCount; ) {
                foreach (ulong d in EnRowDataItems(rowIndex)) {
                    ulong shift = d;
                    for (int i = 0; i < ROUND_SIZE; i++) {
                        if ((shift & 1) != 0) yield return colmunIndex;
                        shift >>= 1;
                        colmunIndex++;
                    }
                }
            }
        }

        /// <summary>
        /// 指令された行の指定列の要素にtrueを設定します。
        /// </summary>
        /// <param name="rowIndex"></param>
        /// <param name="colmunIndexSet"></param>
        public void SetColmunItemsInRow(int rowIndex, IEnumerable<int> colmunIndexSet)
        {
            foreach (ChangeData change in EnChangeColmunItemsInRow(rowIndex, colmunIndexSet)) {
                change.Set(ref data);
            }
        }
        /// <summary>
        /// 指令された行の指定列の要素にfalseを設定します。
        /// </summary>
        /// <param name="rowIndex"></param>
        /// <param name="colmunIndexSet"></param>
        public void ResetColmunItemsInRow(int rowIndex, IEnumerable<int> colmunIndexSet)
        {
            foreach (ChangeData change in EnChangeColmunItemsInRow(rowIndex, colmunIndexSet)) {
                change.Reset(ref data);
            }
        }

        /// <summary>
        /// 変更する必要がある場所とbitマスクを列挙します。
        /// </summary>
        /// <param name="rowIndex"></param>
        /// <param name="colmunIndexSet"></param>
        /// <returns></returns>
        private IEnumerable<ChangeData> EnChangeColmunItemsInRow(
            int rowIndex, IEnumerable<int> colmunIndexSet)
        {
            int dataOffset = DataRowOffset * rowIndex;
            int lastDataIndex = 0;
            int lastBit = 0;
            ulong value = 0;
            ulong shift = 1;
            foreach (int colIndex in colmunIndexSet) {
                int dataIndex = colIndex / ROUND_SIZE;
                int bit = colIndex % ROUND_SIZE;
                if (lastDataIndex != dataIndex) {
                    if (value != 0) yield return new ChangeData(
                        dataOffset + lastDataIndex, value);
                    lastDataIndex = dataIndex;
                    lastBit = 0;
                    value = 0;
                    shift = 1;
                }
                int shiftBit = bit - lastBit;
                shift <<= shiftBit;
                value |= shift;
                lastBit = bit;
            }
            if (value != 0) yield return new ChangeData(
                dataOffset + lastDataIndex, value);
        }

        private struct ChangeData
        {
            public readonly int Index;
            public readonly ulong Value;
            public ChangeData(int index, ulong value)
            {
                Index = index;
                Value = value;
            }
            public void Set(ref ulong[] data)
            {
                data[Index] |= Value;
            }
            public void Reset(ref ulong[] data)
            {
                data[Index] &= ~Value;
            }
        }

        #endregion
        #region クリア操作
        /// <summary>
        /// 要素を全てfalseでクリアします。
        /// </summary>
        public void Clear()
        {
            data = new ulong[DataRowOffset * RowCount];
        }

        /// <summary>
        /// 指定した行の要素をfalseでクリアします。
        /// </summary>
        /// <param name="rowIndex"></param>
        public void ClearRow(int rowIndex)
        {
            int s = DataRowOffset * rowIndex;
            int e = DataRowOffset * (rowIndex + 1);
            for (int i = s; i < e; i++) data[i] = 0;
        }

        /// <summary>
        /// 指定した列をクリアします。
        /// </summary>
        /// <param name="index"></param>
        public void ClearColmun(int index)
        {
            int[] colArray = { index };
            for (int i = 0; i < RowCount; i++) ResetColmunItemsInRow(i, colArray);
        }

        /// <summary>
        /// 指定した位置の行・列の値をクリアします。
        /// </summary>
        /// <param name="index"></param>
        public void ClearRowColmun(int index)
        {
            ClearRow(index);
            ClearColmun(index);
        }

        #endregion
    }
}
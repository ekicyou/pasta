
namespace CSUtil.Security.Cryptography
{
    /// <summary>
    /// CRC16計算用クラス。
    /// テーブル換算方法
    /// 
    /// 計算前のCRC [CH][CL]（上位8bit / 下位8bit）
    /// 入力値  B
    /// 
    /// 換算：[CRCの下位８bit][入力値のビット列を逆転させた値]
    /// 
    /// 換算値に、[CRC上位8bitから求めたXOR演算値]をXOR
    /// 
    /// →bit反転の配列と、XOR配列をそれぞれ作成して利用
    /// </summary>
    public sealed class CRC16
    {

        private const int BYTE_BITS = 8;
        private const int POLY_HIBIT = 1 << 16;
        private const int POLY_MASK = POLY_HIBIT - 1;
        private const int POLY_NOME = (1 << 5) + 1; // 0b 0000 0000 0010 0001

        private const int BYTE_TABLE_SIZE = byte.MaxValue + 1;
        private static readonly ushort[] xorTable = CreateXorTable();
        private static readonly byte[] reverseTable = CreateReverseTable();

        /// <summary>
        /// CRC上位8bitの値によるXOR演算の基準値テーブルを生成します。
        /// </summary>
        /// <returns></returns>
        private static ushort[] CreateXorTable()
        {
            ushort[] table = new ushort[BYTE_TABLE_SIZE];
            for (int i = 0; i < BYTE_TABLE_SIZE; i++) {
                unchecked {
                    ushort xor = 0;
                    int check = i << 8;
                    for (int j = 0; j < BYTE_BITS; j++) {
                        xor <<= 1;
                        check = (check & 0x0FFFF) << 1;
                        if (check > POLY_MASK) xor ^= POLY_NOME;
                    }
                    table[i] = xor;
                }
            }
            return table;
        }

        /// <summary>
        /// BYTE値のビット列反転テーブルを生成します。
        /// </summary>
        /// <returns></returns>
        private static byte[] CreateReverseTable()
        {
            byte[] table = new byte[BYTE_TABLE_SIZE];
            for (int i = 0; i < BYTE_TABLE_SIZE; i++) {
                byte rev = 0;
                int value = i;
                for (int j = 0; j < BYTE_BITS; j++) {
                    rev <<= 1;
                    rev |= (byte)(value & 1);
                    value >>= 1;
                }
                table[i] = rev;
            }
            return table;
        }

        /// <summary> CRC16の値を取得します.</summary>
        public ushort Value { get { return crc; } }
        private ushort crc = 0;

        /// <summary> CRC16オブジェクトを生成します。</summary>
        public CRC16()
        {
        }


        /// <summary> CRC16の値を１バイトの引数で更新します。</summary>
        /// <param name="b">CRC16を計算するデータ
        /// </param>
        public void Update(byte b)
        {
            crc = (ushort)(((crc << 8) | reverseTable[b]) ^ (int)xorTable[(crc & 0x0FF00) >> 8]);
        }

        /// <summary>
        /// CRC16の値をバイトの配列で更新します。
        /// </summary>
        /// <param name="buf">CRC16を計算するデータの配列</param>
        /// <param name="off">データの開始位置を示す配列上のインデックス</param>
        /// <param name="len">実際に計算するデータのバイト数</param>
        public void Update(byte[] buf, int off, int len)
        {
            int end = (off + len);
            for (int i = off; i < end; i++) {
                byte b = buf[i];
                crc = (ushort)(((crc << 8) | reverseTable[b]) ^ (int)xorTable[(crc & 0x0FF00) >> 8]);
            }
        }

        /// <summary> CRCの値をbyteの配列で更新します。
        /// 
        /// </summary>
        /// <param name="b">CRC16を計算するbyteの配列
        /// </param>
        public void Update(byte[] b)
        {
            Update(b, 0, b.Length);
        }

        /// <summary> CRC16の値を０にリセットします</summary>
        public void Reset()
        {
            crc = 0;
        }

    }
}
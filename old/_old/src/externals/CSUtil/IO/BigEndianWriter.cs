using System;
using System.IO;
using System.Text;

namespace CSUtil.IO
{

    /// <summary>
    /// ビックエンディアンのBinaryWriter。
    /// </summary>
    public class BigEndianWriter : BinaryWriter
    {

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        public BigEndianWriter() : base() { }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="output"></param>
        public BigEndianWriter(Stream output) : base(output) { }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="output"></param>
        /// <param name="encoding"></param>
        public BigEndianWriter(Stream output, Encoding encoding) : base(output, encoding) { }

        /// <summary>
        /// 8byteのlong値を書き込みます。
        /// </summary>
        /// <param name="value"></param>
        public override void Write(long value)
        {
            Write(BigEndianBitConverter.GetBytes(value));
        }

        /// <summary>
        /// 8byteのulong値を書き込みます。
        /// </summary>
        /// <param name="value"></param>
        public override void Write(ulong value)
        {
            Write(BigEndianBitConverter.GetBytes(value));
        }

        /// <summary>
        /// 4byteのint値を書き込みます。
        /// </summary>
        /// <param name="value"></param>
        public override void Write(int value)
        {
            Write(BigEndianBitConverter.GetBytes(value));
        }

        /// <summary>
        /// 4byteのuint値を書き込みます。
        /// </summary>
        /// <param name="value"></param>
        public override void Write(uint value)
        {
            Write(BigEndianBitConverter.GetBytes(value));
        }

        /// <summary>
        /// 2byteのshort値を書き込みます。
        /// </summary>
        /// <param name="value"></param>
        public override void Write(short value)
        {
            Write(BigEndianBitConverter.GetBytes(value));
        }

        /// <summary>
        /// 2byteのushort値を書き込みます。
        /// </summary>
        /// <param name="value"></param>
        public override void Write(ushort value)
        {
            Write(BigEndianBitConverter.GetBytes(value));
        }

        /// <summary>
        /// 4byteのfloat値を書き込みます。
        /// </summary>
        /// <param name="value"></param>
        public override void Write(float value)
        {
            Write(BigEndianBitConverter.GetBytes(value));
        }

        /// <summary>
        /// 8byteのdouble値を書き込みます。
        /// </summary>
        /// <param name="value"></param>
        public override void Write(double value)
        {
            Write(BigEndianBitConverter.GetBytes(value));
        }

        /// <summary>
        /// 数値をBCD（２進可１６進数）に変換して書き込みます。
        /// </summary>
        /// <param name="value">値</param>
        /// <param name="byteSize">byte数</param>
        public void WriteBCD(int value, int byteSize)
        {
            byte[] buf = new byte[byteSize];
            for (int i = 0; i < buf.Length; i++) {
                int l = value % 10;
                value /= 10;
                int h = (value % 10) << 4;
                value /= 10;
                byte b = (byte)(h | l);
                buf[i] = b;
            }
            Array.Reverse(buf);
            Write(buf);
        }

    }

}
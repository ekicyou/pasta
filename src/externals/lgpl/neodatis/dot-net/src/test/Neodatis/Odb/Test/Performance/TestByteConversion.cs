using NeoDatis.Tool.Wrappers;
using NUnit.Framework;
namespace NeoDatis.Odb.Test.Performance
{
	[TestFixture]
    public class TestByteConversion : NeoDatis.Odb.Test.ODBTest
	{
		internal static NeoDatis.Odb.Core.Layers.Layer3.Engine.IByteArrayConverter byteArrayConverter
			 = NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetByteArrayConverter();

		public const int Size = 1000;

		public const int Size0 = 1000;

		

		[Test]
        public virtual void TestLong()
		{
			long l1 = 785412;
			byte[] b = byteArrayConverter.LongToByteArray(l1);
			long l2 = byteArrayConverter.ByteArrayToLong(b, 0);
			AssertEquals(l1, l2);
			l1 = long.MaxValue;
			b = byteArrayConverter.LongToByteArray(l1);
			l2 = byteArrayConverter.ByteArrayToLong(b, 0);
			AssertEquals(l1, l2);
			l1 = long.MinValue;
			b = byteArrayConverter.LongToByteArray(l1);
			l2 = byteArrayConverter.ByteArrayToLong(b, 0);
			AssertEquals(l1, l2);
		}

		[Test]
        public virtual void TestInt()
		{
			int l1 = 785412;
			byte[] b = byteArrayConverter.IntToByteArray(l1);
			int l2 = byteArrayConverter.ByteArrayToInt(b, 0);
			AssertEquals(l1, l2);
		}

		[Test]
        public virtual void TestFloat()
		{
			float l1 = (float)785412.4875;
			byte[] b2 = byteArrayConverter.FloatToByteArray(l1);
			float l2 = byteArrayConverter.ByteArrayToFloat(b2);
			AssertEquals(l1, l2, 0);
		}

		[Test]
        public virtual void TestDouble()
		{
			double l1 = 785412.4875;
			byte[] b2 = byteArrayConverter.DoubleToByteArray(l1);
			double l2 = byteArrayConverter.ByteArrayToDouble(b2);
			AssertEquals(l1, l2, 0);
		}

		/// <exception cref="System.IO.IOException"></exception>
		[Test]
        public virtual void TestBoolean()
		{
			bool b1 = true;
			byte[] b2 = byteArrayConverter.BooleanToByteArray(b1);
			bool b3 = byteArrayConverter.ByteArrayToBoolean(b2, 0);
			AssertEquals(b1, b3);
			b1 = false;
			b2 = byteArrayConverter.BooleanToByteArray(b1);
			b3 = byteArrayConverter.ByteArrayToBoolean(b2, 0);
			AssertEquals(b1, b3);
		}

		/// <exception cref="System.IO.IOException"></exception>
		[Test]
        public virtual void TestChar()
		{
			char c = '\u00E1';
			byte[] b2 = byteArrayConverter.CharToByteArray(c);
			char c1 = byteArrayConverter.ByteArrayToChar(b2);
			AssertEquals(c, c1);
		}

		/// <exception cref="System.IO.IOException"></exception>
		[Test]
        public virtual void TestShort()
		{
			short s = 4598;
			byte[] b2 = byteArrayConverter.ShortToByteArray(s);
			short s2 = byteArrayConverter.ByteArrayToShort(b2);
			// assertEquals(s,s2);
			s = 10000;
			b2 = byteArrayConverter.ShortToByteArray(s);
			s2 = byteArrayConverter.ByteArrayToShort(b2);
			AssertEquals(s, s2);
			s = short.MaxValue;
			b2 = byteArrayConverter.ShortToByteArray(s);
			s2 = byteArrayConverter.ByteArrayToShort(b2);
			AssertEquals(s, s2);
			s = short.MinValue;
			b2 = byteArrayConverter.ShortToByteArray(s);
			s2 = byteArrayConverter.ByteArrayToShort(b2);
			AssertEquals(s, s2);
		}

		/// <exception cref="System.IO.IOException"></exception>
		[Test]
        public virtual void TestString()
		{
			string s = "test1";
			byte[] b2 = byteArrayConverter.StringToByteArray(s, true, -1, true);
			string s2 = byteArrayConverter.ByteArrayToString(b2, true, true);
			AssertEquals(s, s2);
		}

		/// <exception cref="System.IO.IOException"></exception>
		[Test]
        public virtual void TestBigDecimal1()
		{
			System.Decimal bd1 = new System.Decimal(10);
			byte[] b2 = byteArrayConverter.BigDecimalToByteArray(bd1, true);
			System.Decimal bd2 = byteArrayConverter.ByteArrayToBigDecimal(b2, true);
			AssertEquals(bd1, bd2);
		}

		/// <exception cref="System.IO.IOException"></exception>
		[Test]
        public virtual void TestBigDecimal2()
		{
			System.Decimal bd1 = new System.Decimal(10.123456789123456789);
			byte[] b2 = byteArrayConverter.BigDecimalToByteArray(bd1, true);
			System.Decimal bd2 = byteArrayConverter.ByteArrayToBigDecimal(b2, true);
			AssertEquals(bd1, bd2);
		}

		/// <exception cref="System.IO.IOException"></exception>
		[Test]
        public virtual void TestBigDecimal3()
		{
			System.Decimal bd1 = new System.Decimal(0);
			byte[] b2 = byteArrayConverter.BigDecimalToByteArray(bd1, true);
			System.Decimal bd2 = byteArrayConverter.ByteArrayToBigDecimal(b2, true);
			AssertEquals(bd1, bd2);
		}

		/// <exception cref="System.IO.IOException"></exception>
		[Test]
        public virtual void TestBigDecimal4()
		{
			System.Decimal bd1 = new System.Decimal(10);
			byte[] b2 = byteArrayConverter.BigDecimalToByteArray(bd1, true);
			System.Decimal bd2 = byteArrayConverter.ByteArrayToBigDecimal(b2, true);
			AssertEquals(bd1, bd2);
		}

		/// <exception cref="System.IO.IOException"></exception>
		[Test]
        public virtual void TestBigDecimal5()
		{
			System.Decimal bd1 = new System.Decimal(0.000);
			byte[] b2 = byteArrayConverter.BigDecimalToByteArray(bd1, true);
			System.Decimal bd2 = byteArrayConverter.ByteArrayToBigDecimal(b2, true);
			AssertEquals(bd1, bd2);
		}

		/// <exception cref="System.IO.IOException"></exception>
		[Test]
        public virtual void TestBigDecimal6()
		{
			System.Decimal bd1 = new System.Decimal(0.000000000000000123456789);
			byte[] b2 = byteArrayConverter.BigDecimalToByteArray(bd1, true);
			System.Decimal bd2 = byteArrayConverter.ByteArrayToBigDecimal(b2, true);
			AssertEquals(bd1, bd2);
		}

		/// <exception cref="System.IO.IOException"></exception>
		[Test]
        public virtual void TestBigDecimal7()
		{
			System.Decimal bd1 = new System.Decimal(-1);
			byte[] b2 = byteArrayConverter.BigDecimalToByteArray(bd1, true);
			System.Decimal bd2 = byteArrayConverter.ByteArrayToBigDecimal(b2, true);
			AssertEquals(bd1, bd2);
		}

		/// <exception cref="System.IO.IOException"></exception>
		[Test]
        public virtual void TestBigDecimal8()
		{
			System.Decimal bd1 = new System.Decimal(-123456789);
			byte[] b2 = byteArrayConverter.BigDecimalToByteArray(bd1, true);
			System.Decimal bd2 = byteArrayConverter.ByteArrayToBigDecimal(b2, true);
			AssertEquals(bd1, bd2);
		}

		/// <exception cref="System.IO.IOException"></exception>
		[Test]
        public virtual void TestBigDecimal9()
		{
			System.Decimal bd1 = new System.Decimal(-0.000000000000000000000000000000123456789
				);
			byte[] b2 = byteArrayConverter.BigDecimalToByteArray(bd1, true);
			System.Decimal bd2 = byteArrayConverter.ByteArrayToBigDecimal(b2, true);
			AssertEquals(bd1, bd2);
		}

		/// <exception cref="System.IO.IOException"></exception>
		[Test]
        public virtual void TestBigDecimal10()
		{
			System.Decimal bd1 = new System.Decimal(123456789123456789123456789.123456789123456789
				);
			byte[] b2 = byteArrayConverter.BigDecimalToByteArray(bd1, true);
			System.Decimal bd2 = byteArrayConverter.ByteArrayToBigDecimal(b2, true);
			AssertEquals(bd1, bd2);
		}

		/// <exception cref="System.IO.IOException"></exception>
		[Test]
        public virtual void TestBigDecimal11()
		{
			System.Decimal bd1 = new System.Decimal(-0.00000);
			byte[] b2 = byteArrayConverter.BigDecimalToByteArray(bd1, true);
			System.Decimal bd2 = byteArrayConverter.ByteArrayToBigDecimal(b2, true);
			AssertEquals(bd1, bd2);
		}
	}
}

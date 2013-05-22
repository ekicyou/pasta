namespace NeoDatis.Odb.Core.Layers.Layer3.Engine
{
	public interface IByteArrayConverter : NeoDatis.Odb.Core.ITwoPhaseInit
	{
		byte[] BooleanToByteArray(bool b);

		bool ByteArrayToBoolean(byte[] bytes, int offset);

		byte[] ShortToByteArray(short s);

		short ByteArrayToShort(byte[] bytes);

		byte[] CharToByteArray(char c);

		char ByteArrayToChar(byte[] bytes);

		int GetNumberOfBytesOfAString(string s, bool useEncoding);

		/// <param name="s"></param>
		/// <param name="withSize">if true, returns an array with an initial int with its size
		/// 	</param>
		/// <param name="totalSpace">The total space of the string (can be bigger that the real string size - to support later in place update)
		/// 	</param>
		/// <param name="withEncoding">To specify if SPECIFIC encoding must be used</param>
		/// <returns>The byte array that represent the string</returns>
		byte[] StringToByteArray(string s, bool withSize, int totalSpace, bool withEncoding
			);

		/// <param name="bytes"></param>
		/// <param name="hasSize">If hasSize is true, the first four bytes are the size of the string
		/// 	</param>
		/// <returns>The String represented by the byte array</returns>
		/// <exception cref="Java.IO.UnsupportedEncodingException">Java.IO.UnsupportedEncodingException
		/// 	</exception>
		string ByteArrayToString(byte[] bytes, bool hasSize, bool useEncoding);

		byte[] BigDecimalToByteArray(System.Decimal bigDecimal, bool withSize);

		System.Decimal ByteArrayToBigDecimal(byte[] bytes, bool hasSize);

		byte[] BigIntegerToByteArray(System.Decimal bigInteger, bool withSize);

		System.Decimal ByteArrayToBigInteger(byte[] bytes, bool hasSize);

		byte[] IntToByteArray(int l);

		/// <summary>This method writes the byte directly to the array parameter</summary>
		void IntToByteArray(int l, byte[] arrayWhereToWrite, int offset);

		int ByteArrayToInt(byte[] bytes, int offset);

		byte[] LongToByteArray(long l);

		/// <summary>This method writes the byte directly to the array parameter</summary>
		void LongToByteArray(long l, byte[] arrayWhereToWrite, int offset);

		long ByteArrayToLong(byte[] bytes, int offset);

		byte[] DateToByteArray(System.DateTime date);

		System.DateTime ByteArrayToDate(byte[] bytes);

		byte[] FloatToByteArray(float f);

		float ByteArrayToFloat(byte[] bytes);

		byte[] DoubleToByteArray(double d);

		double ByteArrayToDouble(byte[] bytes);

		void SetDatabaseCharacterEncoding(string databaseCharacterEncoding);

		/// <param name="b"></param>
		/// <param name="bytes"></param>
		/// <param name="i"></param>
		void BooleanToByteArray(bool b, byte[] arrayWhereToWrite, int offset);

		/// <exception cref="Java.IO.UnsupportedEncodingException"></exception>
		void TestEncoding(string encoding);
	}
}

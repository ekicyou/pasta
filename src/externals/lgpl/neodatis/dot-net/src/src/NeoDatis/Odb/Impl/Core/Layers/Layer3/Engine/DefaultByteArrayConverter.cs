using System;
using System.Text;
using NeoDatis.Odb.Core.Layers.Layer2.Meta;
using NeoDatis.Odb.Core;
using NeoDatis.Odb;
namespace NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine
{
   /// <summary>Converts array of bytes into native objects and native objects into array of bytes
	/// 	</summary>
	/// <author>osmadja</author>
   public class DefaultByteArrayConverter : NeoDatis.Odb.Core.Layers.Layer3.Engine.IByteArrayConverter
   {
      private const System.String ENCODING = "UTF-8";
      /// <summary>The encoding used for string to byte conversion</summary>
      private string encoding;
      
      private bool hasEncoding;
      
      private static readonly byte BYTE_FOR_TRUE = 1;
      private static readonly byte BYTE_FOR_FALSE = 0;
      
      private static readonly byte[] BYTES_FOR_TRUE = new byte[] { 1 };
      
      private static readonly byte[] BYTES_FOR_FALSE = new byte[] { 0 };
      
      private static int IntSize = 0;
      
      private static int IntSize_x_2 = 0;
      
      
      
      /// <summary>Two Phase Init method</summary>
      public virtual void Init2()
      {
         IntSize = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Integer.GetSize();
         IntSize_x_2 = IntSize * 2;
         SetDatabaseCharacterEncoding(OdbConfiguration.GetDatabaseCharacterEncoding());
      }
      
      
      public virtual void SetDatabaseCharacterEncoding(string databaseCharacterEncoding
                                                       )
      {
         encoding = databaseCharacterEncoding;
         if (encoding == null || encoding.Equals(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
                                                 .NoEncoding))
         {
            hasEncoding = false;
         }
         else
          {
            hasEncoding = true;
         }
      }
      
      
      
      public byte[] BooleanToByteArray(bool b)
      {
         if(b){
            return BYTES_FOR_TRUE;
         }
         return BYTES_FOR_FALSE;
      }
      
      public  void BooleanToByteArray(bool b, byte[] arrayWhereToWrite, int offset)  {
         if(b){
            arrayWhereToWrite[offset] = BYTE_FOR_TRUE;
         }else{
            arrayWhereToWrite[offset] = BYTE_FOR_FALSE;
         }
      }
      public bool ByteArrayToBoolean(byte[] bytes, int offset)
      {
         if (bytes[offset] == 0)
          {
            bytes = null;
            return false;
         }
         bytes = null;
         return true;
      }
      
      public byte[] ShortToByteArray(short s)
      {
         return BitConverter.GetBytes(s);
      }
      
      public short ByteArrayToShort(byte[] bytes)
      {
         return  BitConverter.ToInt16(bytes,0);
      }
      
      
      
      public byte[] CharToByteArray(char c)
      {
         return BitConverter.GetBytes(c);
      }
      public char ByteArrayToChar(byte[] bytes)
      {
         return BitConverter.ToChar(bytes, 0);
      }
      
      public int GetNumberOfBytesOfAString(System.String s, bool useEncoding)
      {
         if(useEncoding&&hasEncoding){
            try {
               return Encoding.GetEncoding(ENCODING).GetBytes(s).Length + ODBType.NativeInt.GetSize() * 2;
            } catch (Exception e) {
               throw new ODBRuntimeException(NeoDatisError.UnsupportedEncoding.AddParameter(encoding));
            }
         }
         return Encoding.ASCII.GetBytes(s).Length;
      }
      /// <summary> </summary>
      /// <param name="s">
      /// </param>
      /// <param name="withSize">if true, returns an array with an initial int with its size
      /// </param>
      /// <param name="totalSpace">The total space of the string (can be bigger that the real string size - to support later in place update)
      /// </param>
      /// <returns> The byte array that represent the string
      /// </returns>
      /// <throws>  UnsupportedEncodingException </throws>
      public byte[] StringToByteArray(System.String s, bool withSize, int totalSpace, bool withEncoding)
      {
         byte[] bytes = null; 
         if(withEncoding&&hasEncoding){
            try {
               bytes = Encoding.GetEncoding(ENCODING).GetBytes(s);
            } catch (Exception e) {
               throw new ODBRuntimeException(NeoDatisError.UnsupportedEncoding.AddParameter(encoding));
            }
         }else{
            bytes = Encoding.ASCII.GetBytes(s);
         }
         
         
         if (!withSize)
          {
            return bytes;
         }
         int totalSize = 0;
         
         if (totalSpace == - 1)
          {
            // we always store a string with X the size to enable in place update for bigger string later
            totalSize = OdbConfiguration.GetStringSpaceReserveFactor() * bytes.Length + 2 * ODBType.NativeInt.GetSize();
         }
         else
          {
            totalSize = totalSpace;
         }
         byte[] totalSizeBytes = IntToByteArray(totalSize);
         byte[] stringRealSize = IntToByteArray(bytes.Length);
         byte[] bytes2 = new byte[totalSize+IntSize_x_2];
         for (int i = 0; i < 4; i++)
         {
            bytes2[i] = totalSizeBytes[i];
         }
         for (int i = 4; i < 8; i++)
         {
            bytes2[i] = stringRealSize[i - 4];
         }
         for (int i = 0; i < bytes.Length; i++)
         {
            bytes2[i + 8] = bytes[i];
         }
         return bytes2;
      }
      
      /// <summary> </summary>
      /// <param name="bytes">
      /// </param>
      /// <param name="hasSize">If hasSize is true, the first four bytes are the size of the string
      /// </param>
      /// <returns> The String represented by the byte array
      /// </returns>
      /// <throws>  UnsupportedEncodingException </throws>
      public System.String ByteArrayToString(byte[] bytes, bool hasSize, bool useEncoding)
      {

          string s = null;
          if (hasSize)
          {
              int realSize = ByteArrayToInt(bytes, IntSize);

              if (useEncoding && hasEncoding)
              {
                s = Encoding.GetEncoding(ENCODING).GetString(bytes,IntSize_x_2, realSize);
              }
              else
              {
                 // check encoding
                 s = Encoding.GetEncoding(ENCODING).GetString(bytes, IntSize_x_2, realSize);
              }

              bytes = null;
              return s;
          }

          s =Encoding.GetEncoding(ENCODING).GetString(bytes);
          bytes = null;
          return s;
         /*
        if (hasSize)
        {
            byte[] realStringBytes = new byte[4];
            for (int i = 0; i < 4; i++)
            {
               realStringBytes[i] = bytes[i + 4];
            }
            int realSize = ByteArrayToInt(realStringBytes, IntSize);
            // skip four bytes - which is the size
            byte[] realBytes = new byte[realSize];
            
            for (int i = 0; i < realSize; i++)
            {
               realBytes[i] = bytes[i + 8];
            }
            bytes = null;
            //UPGRADE_TODO: The differences in the Format  of parameters for constructor 'java.lang.String.String'  may cause compilation errors.  "ms-help://MS.VSCC.v80/dv_commoner/local/redirect.htm?index='!DefaultContextWindowIndex'&keyword='jlca1092'"
            return System.Text.Encoding.GetEncoding(ENCODING).GetString(realBytes);
         }
         //UPGRADE_TODO: The differences in the Format  of parameters for constructor 'java.lang.String.String'  may cause compilation errors.  "ms-help://MS.VSCC.v80/dv_commoner/local/redirect.htm?index='!DefaultContextWindowIndex'&keyword='jlca1092'"
         System.String s = System.Text.Encoding.GetEncoding(ENCODING).GetString(bytes);
         bytes = null;
         return s;
          * */
      }
      
      public byte[] BigDecimalToByteArray(System.Decimal bigDecimal, bool withSize)
      {
         return StringToByteArray(bigDecimal.ToString(), withSize, - 1, false);
      }
      public System.Decimal ByteArrayToBigDecimal(byte[] bytes, bool hasSize)
      {
         return System.Decimal.Parse(ByteArrayToString(bytes, hasSize,false), System.Globalization.NumberStyles.Any);
      }
      
      public byte[] BigIntegerToByteArray(System.Decimal bigInteger, bool withSize)
      {
         return StringToByteArray(bigInteger.ToString(), withSize, - 1 , false);
      }
      
      public System.Decimal ByteArrayToBigInteger(byte[] bytes, bool hasSize)
      {
         return System.Decimal.Parse(ByteArrayToString(bytes, hasSize,false), System.Globalization.NumberStyles.Any);
      }
      public byte[] IntToByteArray(int l)
      {
         return BitConverter.GetBytes(l);
      }
      public  void IntToByteArray(int l, byte[] arrayWhereToWrite, int offset)  {
         int i, shift;
         byte[] bytes = BitConverter.GetBytes(l);
         for (i = 0; i < 4; i++) {
            arrayWhereToWrite[offset+i] = bytes[i];
         }
      }
      
      public int ByteArrayToInt(byte[] bytes, int offset)
      {
          try
          {
              return BitConverter.ToInt32(bytes, offset);
          }
          catch (Exception e)
          {
              throw e;
          }
          
      }
      
      
      
      public byte[] LongToByteArray(long l)
      {
         return BitConverter.GetBytes(l);
      }
      
      public  void LongToByteArray(long l, byte[] arrayWhereToWrite, int offset)  {
         int i, shift;
         byte[] bytes = BitConverter.GetBytes(l);
         for (i = 0; i < 8; i++) {
            arrayWhereToWrite[offset+i] = bytes[i];
         }
      }
      public long ByteArrayToLong(byte[] bytes, int offset)
      {
         return BitConverter.ToInt64(bytes, offset);
      }
      
      public byte[] DateToByteArray(System.DateTime date)
      {
         long ticks = date.Ticks;
         return LongToByteArray(ticks);
      }
      
      public System.DateTime ByteArrayToDate(byte[] bytes)
      {
          long ticks = ByteArrayToLong(bytes, 0);
          return new System.DateTime(ticks);
      }
      
      public byte[] FloatToByteArray(float f)
      {
         return BitConverter.GetBytes(f);
         
      }
      public float ByteArrayToFloat(byte[] bytes)
      {
         return BitConverter.ToSingle(bytes, 0);
      }
      
      
      public byte[] DoubleToByteArray(double d)
      {
         return BitConverter.GetBytes(d);
      }
      public double ByteArrayToDouble(byte[] bytes)
      {
         return BitConverter.ToDouble(bytes, 0);
      }
      
      public void TestEncoding(string encoding) {
         string s = "test encoding";        
         byte[] bytes = Encoding.GetEncoding(encoding).GetBytes(s);
      }
      
   }
}

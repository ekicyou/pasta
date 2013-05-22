namespace NeoDatis.Odb.Test.Buffer
{
	/// <author>olivier to test performance of java.nio.ByteBuffer against byte[]</author>
	public class ByteBufferTest : NeoDatis.Odb.Test.ODBTest
	{
		public virtual void Test1()
		{
			Java.Nio.ByteBuffer buffer = Java.Nio.ByteBuffer.Allocate(1000);
			byte b1 = 1;
			int i1 = 10;
			buffer.Put(b1);
			buffer.PutInt(i1);
			buffer.Rewind();
			AssertEquals(b1, buffer.Get());
			AssertEquals(i1, buffer.GetInt());
		}

		public virtual void Test2Perf()
		{
			int size = 1000000;
			NeoDatis.Odb.Core.Layers.Layer3.Engine.IByteArrayConverter byteArrayConverter = new 
				NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.DefaultByteArrayConverter();
			long startBuffer = Sharpen.Runtime.CurrentTimeMillis();
			Java.Nio.ByteBuffer buffer = Java.Nio.ByteBuffer.Allocate(size * 8);
			for (int i = 0; i < size; i++)
			{
				long l = i;
				buffer.PutLong(l);
			}
			buffer.Rewind();
			for (int i = 0; i < size; i++)
			{
				long l = i;
				AssertEquals(l, buffer.GetLong());
			}
			long endBuffer = Sharpen.Runtime.CurrentTimeMillis();
			long startArray = Sharpen.Runtime.CurrentTimeMillis();
			byte[] bytes = new byte[size * 8];
			for (int i = 0; i < size; i++)
			{
				long l = i;
				byte[] longBytes = byteArrayConverter.LongToByteArray(l);
				System.Array.Copy(longBytes, 0, bytes, i * 8, 8);
			}
			for (int i = 0; i < size; i++)
			{
				long l = i;
				long l2 = byteArrayConverter.ByteArrayToLong(bytes, i * 8);
				AssertEquals(l, l2);
			}
			long endArray = Sharpen.Runtime.CurrentTimeMillis();
			Println("time with ByteBuffer=" + (endBuffer - startBuffer));
			Println("time with byte array=" + (endArray - startArray));
		}
	}
}

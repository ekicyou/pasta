namespace NeoDatis.Odb.Tool
{
	public class ByteFileReader
	{
		/// <exception cref="System.IO.IOException"></exception>
		public static void Main3(string[] args)
		{
			System.IO.FileStream raf = new System.IO.FileStream("1141067269187.transaction", 
				"r");
			long length = raf.Length();
			System.Console.Out.WriteLine("File length = " + length);
			for (int i = 0; i < length; i++)
			{
				System.Console.Out.WriteLine(i + ":\t" + raf.Read());
			}
			raf.Close();
		}

		/// <exception cref="System.IO.IOException"></exception>
		public static void Main7(string[] args)
		{
			NeoDatis.Odb.Impl.Core.Layers.Layer3.Buffer.MultiBufferedFileIO braf = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Buffer.MultiBufferedFileIO
				(1, "data", "1141067269187.transaction", false, NeoDatis.Odb.OdbConfiguration.GetDefaultBufferSizeForData
				());
			long length = braf.GetLength();
			System.Console.Out.WriteLine("File length = " + length);
			for (int i = 0; i < length; i++)
			{
				System.Console.Out.WriteLine(i + "\t:" + braf.ReadByte());
			}
			braf.Close();
		}

		/// <exception cref="System.IO.IOException"></exception>
		public static void Main(string[] args)
		{
			NeoDatis.Odb.Impl.Core.Layers.Layer3.Buffer.MultiBufferedFileIO braf = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Buffer.MultiBufferedFileIO
				(1, "data", "C:/o/myProjects/NeoDatis/odb/java/test-xml-imp.odb", false, NeoDatis.Odb.OdbConfiguration
				.GetDefaultBufferSizeForData());
			byte[] bytes = braf.ReadBytes(8);
			long length = braf.GetLength();
			System.Console.Out.WriteLine("File length = " + length);
			for (int i = 0; i < length; i++)
			{
				System.Console.Out.WriteLine(i + "\t\t: b=" + Java.Nio.ByteBuffer.Wrap(bytes).Get
					() + "\ti=" + Java.Nio.ByteBuffer.Wrap(bytes).GetInt() + " \tl=" + Java.Nio.ByteBuffer
					.Wrap(bytes).GetLong());
				int b = braf.ReadByte();
				bytes = Shift(bytes);
				bytes[7] = (byte)b;
			}
			braf.Close();
		}

		public static byte[] Shift(byte[] array)
		{
			for (int i = 0; i < array.Length - 1; i++)
			{
				array[i] = array[i + 1];
			}
			return array;
		}
	}
}

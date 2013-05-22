namespace NeoDatis.Odb.Test.IO
{
	public class TestFileSystemInterface2 : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public virtual void TestReadWrite()
		{
			DeleteBase("testReadWrite.neodatis");
			NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface fsi = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.LocalFileSystemInterface
				("data", new NeoDatis.Odb.Core.Mock.MockSession("test"), NeoDatis.Odb.Test.ODBTest
				.Directory + "testReadWrite.neodatis", true, true, NeoDatis.Odb.OdbConfiguration
				.GetDefaultBufferSizeForData());
			fsi.SetWritePosition(fsi.GetLength(), false);
			for (int i = 0; i < 10000; i++)
			{
				fsi.WriteInt(i, false, "int");
				long currentPosition = fsi.GetPosition();
				if (i == 8000)
				{
					currentPosition = fsi.GetPosition();
					fsi.UseBuffer(false);
					// Using the for transaction method to avoid protected area
					// verification, check the setWritePosition method
					fsi.SetWritePositionNoVerification(4, false);
					AssertEquals(1, fsi.ReadInt());
					fsi.UseBuffer(true);
					fsi.SetWritePositionNoVerification(currentPosition, false);
				}
				if (i == 9000)
				{
					currentPosition = fsi.GetPosition();
					fsi.UseBuffer(false);
					fsi.SetWritePositionNoVerification(8, false);
					fsi.WriteInt(12, false, "int");
					fsi.UseBuffer(true);
					fsi.SetWritePositionNoVerification(currentPosition, false);
				}
			}
			fsi.SetReadPosition(0);
			for (int i = 0; i < 10000; i++)
			{
				int j = fsi.ReadInt();
				if (i == 2)
				{
					AssertEquals(12, j);
				}
				else
				{
					AssertEquals(i, j);
				}
			}
			fsi.Close();
			DeleteBase("testReadWrite.neodatis");
		}

		/// <exception cref="Java.IO.UnsupportedEncodingException"></exception>
		public virtual void TestStringGetBytesWithoutEncoding()
		{
			NeoDatis.Odb.Core.Layers.Layer3.Engine.IByteArrayConverter byteArrayConverter = NeoDatis.Odb.OdbConfiguration
				.GetCoreProvider().GetByteArrayConverter();
			string test = "How are you my friend?";
			int size = 1000000;
			long t0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			// Execute with encoding
			for (int i = 0; i < size; i++)
			{
				byteArrayConverter.StringToByteArray(test, true, -1, true);
			}
			long t1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			// Execute without encoding
			for (int i = 0; i < size; i++)
			{
			}
			// byteArrayConverter.stringToByteArray(test, false, -1, false);
			long t2 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			Println("With Encoding=" + (t1 - t0) + " / Without Encoding=" + (t2 - t1));
		}
	}
}

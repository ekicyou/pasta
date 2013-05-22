namespace NeoDatis.Odb.Test.IO
{
	public class TestFileSystemInterface1 : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public virtual void TestByte()
		{
			byte b = 127;
			NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface fsi = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.LocalFileSystemInterface
				("data", new NeoDatis.Odb.Core.Mock.MockSession("test"), NeoDatis.Odb.Test.ODBTest
				.Directory + "testByte.neodatis", true, true, NeoDatis.Odb.OdbConfiguration.GetDefaultBufferSizeForData
				());
			fsi.SetWritePosition(0, false);
			fsi.WriteByte(b, false);
			fsi.Close();
			fsi = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.LocalFileSystemInterface("data"
				, new NeoDatis.Odb.Core.Mock.MockSession("test"), NeoDatis.Odb.Test.ODBTest.Directory
				 + "testByte.neodatis", false, false, NeoDatis.Odb.OdbConfiguration.GetDefaultBufferSizeForData
				());
			fsi.SetReadPosition(0);
			byte b2 = fsi.ReadByte();
			AssertEquals(b, b2);
			fsi.Close();
			DeleteBase("testByte.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestInt()
		{
			int i = 259998;
			NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface fsi = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.LocalFileSystemInterface
				("data", new NeoDatis.Odb.Core.Mock.MockSession("test"), NeoDatis.Odb.Test.ODBTest
				.Directory + "testInt.neodatis", true, true, NeoDatis.Odb.OdbConfiguration.GetDefaultBufferSizeForData
				());
			fsi.SetWritePosition(0, false);
			fsi.WriteInt(i, false, "i");
			fsi.Close();
			fsi = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.LocalFileSystemInterface("data"
				, new NeoDatis.Odb.Core.Mock.MockSession("test"), NeoDatis.Odb.Test.ODBTest.Directory
				 + "testInt.neodatis", false, false, NeoDatis.Odb.OdbConfiguration.GetDefaultBufferSizeForData
				());
			fsi.SetReadPosition(0);
			int i2 = fsi.ReadInt();
			AssertEquals(i, i2);
			fsi.Close();
			DeleteBase("testInt.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestChar()
		{
			char c = '\u00E1';
			NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface fsi = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.LocalFileSystemInterface
				("data", new NeoDatis.Odb.Core.Mock.MockSession("test"), NeoDatis.Odb.Test.ODBTest
				.Directory + "testChar.neodatis", true, true, NeoDatis.Odb.OdbConfiguration.GetDefaultBufferSizeForData
				());
			fsi.SetWritePosition(0, false);
			fsi.WriteChar(c, false);
			fsi.Close();
			fsi = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.LocalFileSystemInterface("data"
				, new NeoDatis.Odb.Core.Mock.MockSession("test"), NeoDatis.Odb.Test.ODBTest.Directory
				 + "testChar.neodatis", false, false, NeoDatis.Odb.OdbConfiguration.GetDefaultBufferSizeForData
				());
			fsi.SetReadPosition(0);
			char c2 = fsi.ReadChar();
			AssertEquals(c, c2);
			fsi.Close();
			DeleteBase("testChar.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestShort()
		{
			short s = 4598;
			NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface fsi = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.LocalFileSystemInterface
				("data", new NeoDatis.Odb.Core.Mock.MockSession("test"), NeoDatis.Odb.Test.ODBTest
				.Directory + "testShort.neodatis", true, true, NeoDatis.Odb.OdbConfiguration.GetDefaultBufferSizeForData
				());
			fsi.SetWritePosition(0, false);
			fsi.WriteShort(s, false);
			fsi.Close();
			fsi = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.LocalFileSystemInterface("data"
				, new NeoDatis.Odb.Core.Mock.MockSession("test"), NeoDatis.Odb.Test.ODBTest.Directory
				 + "testShort.neodatis", false, false, NeoDatis.Odb.OdbConfiguration.GetDefaultBufferSizeForData
				());
			fsi.SetReadPosition(0);
			short s2 = fsi.ReadShort();
			AssertEquals(s, s2);
			fsi.Close();
			DeleteBase("testShort.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestBoolean()
		{
			bool b1 = true;
			bool b2 = false;
			NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface fsi = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.LocalFileSystemInterface
				("data", new NeoDatis.Odb.Core.Mock.MockSession("test"), NeoDatis.Odb.Test.ODBTest
				.Directory + "testBoolean.neodatis", true, true, NeoDatis.Odb.OdbConfiguration.GetDefaultBufferSizeForData
				());
			fsi.SetWritePosition(0, false);
			fsi.WriteBoolean(b1, false);
			fsi.WriteBoolean(b2, false);
			fsi.Close();
			fsi = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.LocalFileSystemInterface("data"
				, new NeoDatis.Odb.Core.Mock.MockSession("test"), NeoDatis.Odb.Test.ODBTest.Directory
				 + "testBoolean.neodatis", false, false, NeoDatis.Odb.OdbConfiguration.GetDefaultBufferSizeForData
				());
			fsi.SetReadPosition(0);
			bool b11 = fsi.ReadBoolean();
			bool b22 = fsi.ReadBoolean();
			AssertEquals(b1, b11);
			AssertEquals(b2, b22);
			fsi.Close();
			DeleteBase("testBoolean.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestLong()
		{
			long i = 259999865;
			NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface fsi = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.LocalFileSystemInterface
				("data", new NeoDatis.Odb.Core.Mock.MockSession("test"), NeoDatis.Odb.Test.ODBTest
				.Directory + "testLong.neodatis", true, true, NeoDatis.Odb.OdbConfiguration.GetDefaultBufferSizeForData
				());
			fsi.SetWritePosition(0, false);
			fsi.WriteLong(i, false, "i", NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				.PointerWriteAction);
			fsi.Close();
			fsi = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.LocalFileSystemInterface("data"
				, new NeoDatis.Odb.Core.Mock.MockSession("test"), NeoDatis.Odb.Test.ODBTest.Directory
				 + "testLong.neodatis", false, false, NeoDatis.Odb.OdbConfiguration.GetDefaultBufferSizeForData
				());
			fsi.SetReadPosition(0);
			long i2 = fsi.ReadLong();
			AssertEquals(i, i2);
			fsi.Close();
			DeleteBase("testLong.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestString()
		{
			string baseName = GetBaseName();
			string s = "ola chico, como voc√™ est√° ??? eu estou bem at√© amanh√£ de manh√£, √°√°√°√°'";
			NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface fsi = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.LocalFileSystemInterface
				("data", new NeoDatis.Odb.Core.Mock.MockSession("test"), baseName, true, true, NeoDatis.Odb.OdbConfiguration
				.GetDefaultBufferSizeForData());
			fsi.SetWritePosition(0, false);
			fsi.WriteString(s, false, true);
			fsi.Close();
			fsi = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.LocalFileSystemInterface("data"
				, new NeoDatis.Odb.Core.Mock.MockSession("test"), baseName, false, false, NeoDatis.Odb.OdbConfiguration
				.GetDefaultBufferSizeForData());
			fsi.SetReadPosition(0);
			string s2 = fsi.ReadString(true);
			fsi.Close();
			DeleteBase(baseName);
			AssertEquals(s, s2);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestBigDecimal()
		{
			System.Decimal bd = new System.Decimal("-128451.1234567899876543210");
			NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface fsi = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.LocalFileSystemInterface
				("data", new NeoDatis.Odb.Core.Mock.MockSession("test"), NeoDatis.Odb.Test.ODBTest
				.Directory + "testBigDecimal.neodatis", true, true, NeoDatis.Odb.OdbConfiguration
				.GetDefaultBufferSizeForData());
			fsi.SetWritePosition(0, false);
			fsi.WriteBigDecimal(bd, false);
			fsi.Close();
			fsi = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.LocalFileSystemInterface("data"
				, new NeoDatis.Odb.Core.Mock.MockSession("test"), NeoDatis.Odb.Test.ODBTest.Directory
				 + "testBigDecimal.neodatis", false, false, NeoDatis.Odb.OdbConfiguration.GetDefaultBufferSizeForData
				());
			fsi.SetReadPosition(0);
			System.Decimal bd2 = fsi.ReadBigDecimal();
			AssertEquals(bd, bd2);
			fsi.Close();
			DeleteBase("testBigDecimal.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestBigInteger()
		{
			System.Decimal bd = new System.Decimal("-128451");
			NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface fsi = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.LocalFileSystemInterface
				("data", new NeoDatis.Odb.Core.Mock.MockSession("test"), NeoDatis.Odb.Test.ODBTest
				.Directory + "testBigDecimal.neodatis", true, true, NeoDatis.Odb.OdbConfiguration
				.GetDefaultBufferSizeForData());
			fsi.SetWritePosition(0, false);
			fsi.WriteBigInteger(bd, false);
			fsi.Close();
			fsi = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.LocalFileSystemInterface("data"
				, new NeoDatis.Odb.Core.Mock.MockSession("test"), NeoDatis.Odb.Test.ODBTest.Directory
				 + "testBigDecimal.neodatis", false, false, NeoDatis.Odb.OdbConfiguration.GetDefaultBufferSizeForData
				());
			fsi.SetReadPosition(0);
			System.Decimal bd2 = fsi.ReadBigInteger();
			AssertEquals(bd, bd2);
			fsi.Close();
			DeleteBase("testBigDecimal.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestFloat()
		{
			float f = (float)12544548.12454;
			NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface fsi = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.LocalFileSystemInterface
				("data", new NeoDatis.Odb.Core.Mock.MockSession("test"), NeoDatis.Odb.Test.ODBTest
				.Directory + "testFloat.neodatis", true, true, NeoDatis.Odb.OdbConfiguration.GetDefaultBufferSizeForData
				());
			fsi.SetWritePosition(0, false);
			fsi.WriteFloat(f, false);
			fsi.Close();
			fsi = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.LocalFileSystemInterface("data"
				, new NeoDatis.Odb.Core.Mock.MockSession("test"), NeoDatis.Odb.Test.ODBTest.Directory
				 + "testFloat.neodatis", false, false, NeoDatis.Odb.OdbConfiguration.GetDefaultBufferSizeForData
				());
			fsi.SetReadPosition(0);
			float f2 = fsi.ReadFloat();
			AssertTrue(f == f2);
			fsi.Close();
			DeleteBase("testFloat.neodatis");
		}
	}
}

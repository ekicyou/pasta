using System.IO;
using NUnit.Framework;
namespace NeoDatis.Odb.Test.IO
{
	[TestFixture]
    public class TestFileLock : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.IO.IOException"></exception>
		[Test]
        public virtual void Test1()
		{
			System.IO.FileStream raf = new System.IO.FileStream(NeoDatis.Odb.Test.ODBTest.Directory
				 + "testLock1", FileMode.OpenOrCreate);
			raf.Seek(1024);
			raf.Write(10);
			//Java.Nio.Channels.FileLock fileLock = raf.GetChannel().Lock(0, 1024, false);
			//AssertEquals(true, fileLock != null);
			//System.IO.FileStream raf2 = new System.IO.FileStream(NeoDatis.Odb.Test.ODBTest.Directory		 + "testLock1", FileMode.OpenOrCreate);
			//Java.Nio.Channels.FileLock fileLock2 = raf2.GetChannel().Lock(0, 1024, false);
		}

		/// <exception cref="System.IO.IOException"></exception>
		[Test]
        public virtual void Test2NoWrite()
		{
			System.IO.FileStream raf = new System.IO.FileStream(NeoDatis.Odb.Test.ODBTest.Directory
				 + "testLock1",FileMode.OpenOrCreate);
			raf.Seek(1024);
			//Java.Nio.Channels.FileLock fileLock = raf.GetChannel().Lock(0, 1, false);
			//AssertEquals(true, fileLock != null);
			raf.Close();
		}

		/// <summary>Simple lock</summary>
		/// <exception cref="System.IO.IOException">System.IO.IOException</exception>
		[Test]
        public virtual void TestOdbFileIo()
		{
			NeoDatis.Tool.Wrappers.IO.OdbFileIO fileIO = new NeoDatis.Tool.Wrappers.IO.OdbFileIO
				(NeoDatis.Odb.Test.ODBTest.Directory + "testLock1", true, null);
			fileIO.Seek(1024);
			fileIO.Write((byte)10);
			fileIO.LockFile();
			AssertEquals(true, fileIO.IsLocked());
			fileIO.Close();
		}

		/// <summary>Simple lock</summary>
		/// <exception cref="System.IO.IOException">System.IO.IOException</exception>
		[Test]
        public virtual void TestOdbFileIo2()
		{
			NeoDatis.Tool.Wrappers.IO.OdbFileIO fileIO = new NeoDatis.Tool.Wrappers.IO.OdbFileIO
				(NeoDatis.Odb.Test.ODBTest.Directory + "testLock2", true, null);
			fileIO.Seek(1024);
			fileIO.Write((byte)10);
			bool isLocked = fileIO.LockFile();
			AssertTrue(isLocked);
			NeoDatis.Tool.Wrappers.IO.OdbFileIO fileIO2 = new NeoDatis.Tool.Wrappers.IO.OdbFileIO
				(NeoDatis.Odb.Test.ODBTest.Directory + "testLock2", true, null);
			fileIO2.LockFile();
			AssertEquals(true, fileIO2.IsLocked());
			fileIO.Close();
		}
	}
}

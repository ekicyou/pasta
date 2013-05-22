using NUnit.Framework;
namespace NeoDatis.Odb.Test.IO
{
	[TestFixture]
    public class TestReservingSpace : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestSize()
		{
			DeleteBase("writing.neodatis");
			DeleteBase("writing");
			DeleteBase("reserving.neodatis");
			DeleteBase("reserving");
			NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine1 = NeoDatis.Odb.OdbConfiguration
				.GetCoreProvider().GetClientStorageEngine(new NeoDatis.Odb.Core.Layers.Layer3.IOFileParameter
				(NeoDatis.Odb.Test.ODBTest.Directory + "writing.neodatis", true, null, null));
			NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine2 = NeoDatis.Odb.OdbConfiguration
				.GetCoreProvider().GetClientStorageEngine(new NeoDatis.Odb.Core.Layers.Layer3.IOFileParameter
				(NeoDatis.Odb.Test.ODBTest.Directory + "reserving.neodatis", true, null, null));
			NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface writingFsi = engine1.
				GetObjectWriter().GetFsi();
			NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface reservingFsi = engine2
				.GetObjectWriter().GetFsi();
			AssertEquals(writingFsi.GetLength(), reservingFsi.GetLength());
			Write(writingFsi, false);
			Write(reservingFsi, true);
			AssertEquals(writingFsi.GetLength(), reservingFsi.GetLength());
			engine1.Commit();
			engine1.Close();
			engine2.Commit();
			engine2.Close();
			DeleteBase("writing.neodatis");
			DeleteBase("reserving.neodatis");
		}

		/// <exception cref="System.IO.IOException"></exception>
		public virtual void Write(NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface
			 fsi, bool writeInTransaction)
		{
			fsi.WriteInt(1, writeInTransaction, "1");
		}
	}
}

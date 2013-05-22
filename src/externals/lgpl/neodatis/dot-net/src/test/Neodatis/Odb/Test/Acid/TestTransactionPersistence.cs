
using NUnit.Framework;
using NUnit.Framework;
namespace NeoDatis.Odb.Test.Acid
{
    [TestFixture]
	[TestFixture]
    public class TestTransactionPersistence : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.TypeLoadException"></exception>
		[Test]
        public virtual void Test4()
		{
			NeoDatis.Odb.Core.Layers.Layer3.Engine.IByteArrayConverter byteArrayConverter = NeoDatis.Odb.OdbConfiguration
				.GetCoreProvider().GetByteArrayConverter();
			NeoDatis.Odb.Core.Transaction.IWriteAction wa1 = new NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				(1, byteArrayConverter.IntToByteArray(1), "size");
			AssertEquals(wa1.GetBytes(0).Length, 4);
			NeoDatis.Odb.Core.Transaction.IWriteAction wa2 = new NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				(1, byteArrayConverter.StringToByteArray("ol√° chico", true, -1, true), "size");
			NeoDatis.Odb.Core.Transaction.IWriteAction wa3 = new NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				(1, byteArrayConverter.BigDecimalToByteArray(new System.Decimal(1.123456789), 
				true), "size");
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test2B()
		{
			if (!isLocal)
			{
				return;
			}
			NeoDatis.Odb.Core.Layers.Layer3.Engine.IByteArrayConverter byteArrayConverter = NeoDatis.Odb.OdbConfiguration
				.GetCoreProvider().GetByteArrayConverter();
			DeleteBase("test78.neodatis");
			// 155 : to avoid protected zone
			NeoDatis.Odb.Core.Transaction.IWriteAction wa1 = new NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				(300 + 1, byteArrayConverter.IntToByteArray(1), "size");
			NeoDatis.Odb.Core.Transaction.IWriteAction wa2 = new NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				(300 + 15, byteArrayConverter.StringToByteArray(" 1 - ol√° chico! - 1", true, -1
				, true), "name");
			NeoDatis.Odb.Core.Transaction.IWriteAction wa3 = new NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				(300 + 1, byteArrayConverter.IntToByteArray(2), "size");
			NeoDatis.Odb.Core.Transaction.IWriteAction wa4 = new NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
				(300 + 15, byteArrayConverter.StringToByteArray(" 2 - ol√° chico! - 2", true, -1
				, true), "name");
			NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine se = NeoDatis.Odb.OdbConfiguration
				.GetCoreProvider().GetClientStorageEngine(new NeoDatis.Odb.Core.Layers.Layer3.IOFileParameter
				(NeoDatis.Odb.Test.ODBTest.Directory + "test78.neodatis", true, null, null));
			// se.close();
			NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface fsi = se.GetObjectWriter
				().GetFsi();
			// new FileSystemInterface(null,se.getSession(),new
			// IOFileParameter("test.neodatis",true),false,Configuration.getDefaultBufferSizeForData());
			NeoDatis.Odb.Core.Transaction.ITransaction transaction = se.GetSession(true).GetTransaction
				();
			transaction.SetArchiveLog(true);
			transaction.ManageWriteAction(wa1.GetPosition(), wa1.GetBytes(0));
			transaction.ManageWriteAction(wa2.GetPosition(), wa2.GetBytes(0));
			transaction.ManageWriteAction(wa3.GetPosition(), wa3.GetBytes(0));
			transaction.ManageWriteAction(wa4.GetPosition(), wa4.GetBytes(0));
			// transaction.getFsi().flush();
			NeoDatis.Odb.Core.Transaction.IWriteAction wat1 = (NeoDatis.Odb.Core.Transaction.IWriteAction
				)((NeoDatis.Odb.Impl.Core.Transaction.DefaultTransaction)transaction).GetWriteActions
				()[2];
			byte[] bytes = wat1.GetBytes(0);
			transaction.Commit();
			NeoDatis.Odb.Impl.Core.Transaction.DefaultTransaction transaction2 = NeoDatis.Odb.Impl.Core.Transaction.DefaultTransaction
				.Read(NeoDatis.Odb.Test.ODBTest.Directory + transaction.GetName());
			NeoDatis.Odb.Core.Transaction.IWriteAction wat2 = (NeoDatis.Odb.Core.Transaction.IWriteAction
				)transaction2.GetWriteActions()[2];
			AssertEquals(NeoDatis.Tool.DisplayUtility.ByteArrayToString(bytes), NeoDatis.Tool.DisplayUtility
				.ByteArrayToString(wat2.GetBytes(0)));
			AssertEquals(wat1.GetPosition(), wat2.GetPosition());
			transaction2.Rollback();
			fsi.Close();
			DeleteBase("test78.neodatis");
			DeleteBase(transaction.GetName());
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test3()
		{
			NeoDatis.Odb.Core.Layers.Layer3.Engine.IByteArrayConverter byteArrayConverter = NeoDatis.Odb.OdbConfiguration
				.GetCoreProvider().GetByteArrayConverter();
			int size = 1000;
			NeoDatis.Odb.Core.Transaction.ISession session = new NeoDatis.Odb.Core.Mock.MockSession
				(NeoDatis.Odb.Test.ODBTest.Directory + "test2.neodatis");
			NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface fsi = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.LocalFileSystemInterface
				("test", session, new NeoDatis.Odb.Core.Layers.Layer3.IOFileParameter(NeoDatis.Odb.Test.ODBTest
				.Directory + "test2.neodatis", true, null, null), false, NeoDatis.Odb.OdbConfiguration
				.GetDefaultBufferSizeForData());
			NeoDatis.Odb.Impl.Core.Transaction.DefaultTransaction transaction = new NeoDatis.Odb.Impl.Core.Transaction.DefaultTransaction
				(session, fsi);
			transaction.SetArchiveLog(true);
			for (int i = 0; i < size; i++)
			{
				// 155 : to avoid protected zone
				transaction.ManageWriteAction(300 + i * 4 * 2, byteArrayConverter.IntToByteArray(
					i));
			}
			NeoDatis.Odb.Core.Transaction.IWriteAction wa1 = (NeoDatis.Odb.Core.Transaction.IWriteAction
				)transaction.GetWriteActions()[size - 2];
			byte[] bytes = wa1.GetBytes(0);
			transaction.Commit();
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			NeoDatis.Odb.Impl.Core.Transaction.DefaultTransaction transaction2 = NeoDatis.Odb.Impl.Core.Transaction.DefaultTransaction
				.Read(NeoDatis.Odb.Test.ODBTest.Directory + transaction.GetName());
			long t = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs() - start;
			NeoDatis.Odb.Core.Transaction.IWriteAction wa2 = (NeoDatis.Odb.Core.Transaction.IWriteAction
				)transaction2.GetWriteActions()[size - 2];
			AssertEquals(NeoDatis.Tool.DisplayUtility.ByteArrayToString(bytes), NeoDatis.Tool.DisplayUtility
				.ByteArrayToString(wa2.GetBytes(0)));
			AssertEquals(wa1.GetPosition(), wa2.GetPosition());
			transaction2.Rollback();
			fsi.Close();
			DeleteBase("test2.neodatis");
			DeleteBase(transaction.GetName());
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test5()
		{
			NeoDatis.Odb.Core.Layers.Layer3.Engine.IByteArrayConverter byteArrayConverter = NeoDatis.Odb.OdbConfiguration
				.GetCoreProvider().GetByteArrayConverter();
			int size = 1000;
			NeoDatis.Odb.Core.Transaction.ISession session = new NeoDatis.Odb.Core.Mock.MockSession
				(NeoDatis.Odb.Test.ODBTest.Directory + "test2.neodatis");
			NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface fsi = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.LocalFileSystemInterface
				("test", session, new NeoDatis.Odb.Core.Layers.Layer3.IOFileParameter(NeoDatis.Odb.Test.ODBTest
				.Directory + "test2.neodatis", true, null, null), false, NeoDatis.Odb.OdbConfiguration
				.GetDefaultBufferSizeForData());
			NeoDatis.Odb.Impl.Core.Transaction.DefaultTransaction transaction = new NeoDatis.Odb.Impl.Core.Transaction.DefaultTransaction
				(session, fsi);
			transaction.SetArchiveLog(true);
			for (int i = 0; i < size; i++)
			{
				// 155 : to avoid protected zone
				transaction.ManageWriteAction(300 + i * 4, byteArrayConverter.IntToByteArray(i));
			}
			// All write action were together so the transaction should have
			// appended all the bytes
			// in one WriteAction. As it as not been committed, the current
			// writeAction
			AssertEquals(0, transaction.GetWriteActions().Count);
			transaction.Commit();
			DeleteBase("test2.neodatis");
			DeleteBase(transaction.GetName());
		}
	}
}

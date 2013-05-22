namespace NeoDatis.Odb.Test.Ext
{
	public class TestExt : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public virtual void TestTransactionId()
		{
			DeleteBase("ext0");
			NeoDatis.Odb.ODB odb = Open("ext0");
			NeoDatis.Odb.TransactionId transactionId = odb.Ext().GetCurrentTransactionId();
			Println(transactionId);
			if (isLocal)
			{
				AssertTrue(transactionId.ToString().StartsWith("tid=01"));
			}
			else
			{
				// In Client Server, there is a first transaction created
				// automaticaly by the server, so the first user transaction is 2
				AssertTrue(transactionId.ToString().StartsWith("tid=02"));
			}
			odb.Close();
			odb = Open("ext0");
			transactionId = odb.Ext().GetCurrentTransactionId();
			Println(transactionId);
			if (isLocal)
			{
				AssertTrue(transactionId.ToString().StartsWith("tid=02"));
			}
			else
			{
				AssertTrue(transactionId.ToString().StartsWith("tid=03"));
			}
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestTransactionId2()
		{
			DeleteBase("exta");
			NeoDatis.Odb.ODB odb = null;
			NeoDatis.Odb.TransactionId transactionId = null;
			int size = isLocal ? 20000 : 1000;
			for (int i = 0; i < size; i++)
			{
				odb = Open("exta");
				transactionId = odb.Ext().GetCurrentTransactionId();
				// println(transactionId);
				if (isLocal)
				{
					AssertTrue(transactionId.ToString().StartsWith("tid=0" + (i + 1)));
				}
				else
				{
					AssertTrue(transactionId.ToString().StartsWith("tid=0" + (i + 2)));
				}
				odb.Close();
				if (i % 1000 == 0)
				{
					Println("Transaction " + i);
				}
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestGetObjectId()
		{
			DeleteBase("extb");
			NeoDatis.Odb.ODB odb = null;
			odb = Open("extb");
			NeoDatis.Odb.Test.VO.Login.Function f = new NeoDatis.Odb.Test.VO.Login.Function("Test Function"
				);
			NeoDatis.Odb.OID oid = odb.Store(f);
			NeoDatis.Odb.ExternalOID extOid = odb.Ext().GetObjectExternalOID(f);
			AssertEquals(oid.GetObjectId(), extOid.GetObjectId());
			AssertEquals(odb.Ext().GetDatabaseId(), extOid.GetDatabaseId());
			odb.Close();
			odb = Open("extb");
			// Getting object via external oid
			NeoDatis.Odb.Test.VO.Login.Function f2 = (NeoDatis.Odb.Test.VO.Login.Function)odb
				.GetObjectFromId(extOid);
			NeoDatis.Odb.OID lastOid = odb.GetObjectId(f2);
			AssertEquals(oid, lastOid);
			AssertEquals(f.GetName(), f2.GetName());
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestObjectVersion()
		{
			DeleteBase("extc");
			NeoDatis.Odb.ODB odb = Open("extc");
			int size = 1000;
			long updateDate = 0;
			long creationDate = 0;
			NeoDatis.Odb.OID oid = odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("f"));
			odb.Close();
			Java.Lang.Thread.Sleep(100);
			// LogUtil.allOn(true);
			for (int i = 0; i < size; i++)
			{
				odb = Open("extc");
				NeoDatis.Odb.Test.VO.Login.Function f = (NeoDatis.Odb.Test.VO.Login.Function)odb.
					GetObjectFromId(oid);
				int version = odb.Ext().GetObjectVersion(oid);
				// System.out.println("i="+i + " - v="+ version+ " - oid="+oid);
				updateDate = odb.Ext().GetObjectUpdateDate(oid);
				creationDate = odb.Ext().GetObjectCreationDate(oid);
				f.SetName(f.GetName() + "-" + i);
				// update the object, should increase the version number
				odb.Store(f);
				odb.Close();
				AssertEquals(i + 1, version);
				// System.out.println(creationDate + " - "+ updateDate+ "- "+
				// OdbTime.getCurrentTimeInMs());
				// in first iteration, creation & update date may be equal
				if (i > 0)
				{
					AssertTrue(creationDate < updateDate);
				}
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestObjectVersionWithoutClose()
		{
			DeleteBase("extd");
			NeoDatis.Odb.ODB odb = Open("extd");
			int size = 1000;
			long updateDate = 0;
			long creationDate = 0;
			NeoDatis.Odb.OID oid = odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("f"));
			odb.Close();
			odb = Open("extd");
			Java.Lang.Thread.Sleep(100);
			for (int i = 0; i < size; i++)
			{
				// odb = open("ext");
				NeoDatis.Odb.Test.VO.Login.Function f = (NeoDatis.Odb.Test.VO.Login.Function)odb.
					GetObjectFromId(oid);
				int version = odb.Ext().GetObjectVersion(oid);
				// System.out.println("i="+i + " - v="+ version+ " - oid="+oid);
				AssertEquals(i + 1, version);
				f.SetName("f" + i);
				// update the object, should increase the version number
				odb.Store(f);
				odb.Commit();
			}
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestObjectVersionWithoutClose2()
		{
			DeleteBase("ext2");
			NeoDatis.Odb.ODB odb = Open("ext2");
			int size = 1000;
			long updateDate = 0;
			long creationDate = 0;
			// LogUtil.allOn(true);
			NeoDatis.Odb.OID oid = odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("f"));
			odb.Close();
			odb = Open("ext2");
			Java.Lang.Thread.Sleep(100);
			// LogUtil.allOn(true);
			for (int i = 0; i < size; i++)
			{
				// odb = open("ext");
				NeoDatis.Odb.Test.VO.Login.Function f = (NeoDatis.Odb.Test.VO.Login.Function)odb.
					GetObjectFromId(oid);
				f.SetName("f" + i);
				odb.Store(f);
				odb.Commit();
			}
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestConcurrentObjectVersion()
		{
			// LogUtil.allOn(true);
			int port = Port + 8;
			NeoDatis.Tool.IOUtil.DeleteFile(Directory + "exta1");
			NeoDatis.Odb.ODBServer server = NeoDatis.Odb.ODBFactory.OpenServer(port);
			server.StartServer(true);
			NeoDatis.Odb.ODB odb = NeoDatis.Odb.ODBFactory.OpenClient("localhost", port, Directory
				 + "exta1");
			NeoDatis.Odb.OID oid = odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("f1"));
			int version = odb.Ext().GetObjectVersion(oid);
			Println(version);
			odb.Close();
			NeoDatis.Odb.ODB odb1 = NeoDatis.Odb.ODBFactory.OpenClient("localhost", port, Directory
				 + "exta1");
			NeoDatis.Odb.ODB odb2 = NeoDatis.Odb.ODBFactory.OpenClient("localhost", port, Directory
				 + "exta1");
			int v1 = odb1.Ext().GetObjectVersion(oid);
			int v2 = odb2.Ext().GetObjectVersion(oid);
			AssertEquals(1, v1);
			AssertEquals(1, v2);
			Println("v1=" + v1 + "- v2=" + v2);
			NeoDatis.Odb.Test.VO.Login.Function f1 = (NeoDatis.Odb.Test.VO.Login.Function)odb1
				.GetObjectFromId(oid);
			NeoDatis.Odb.Test.VO.Login.Function f2 = (NeoDatis.Odb.Test.VO.Login.Function)odb2
				.GetObjectFromId(oid);
			f1.SetName("function 1");
			odb1.Store(f1);
			v1 = odb1.Ext().GetObjectVersion(oid);
			Println("after update odb1 , v1=" + v1);
			odb1.Close();
			NeoDatis.Odb.ODB odb3 = NeoDatis.Odb.ODBFactory.OpenClient("localhost", port, Directory
				 + "exta1");
			// Check committed object value
			NeoDatis.Odb.Test.VO.Login.Function f3 = (NeoDatis.Odb.Test.VO.Login.Function)odb3
				.GetObjectFromId(oid);
			AssertEquals(f1.GetName(), f3.GetName());
			AssertEquals(2, odb3.Ext().GetObjectVersion(oid));
			odb3.Close();
			AssertEquals(2, v1);
			f2.SetName("function 2");
			odb2.Store(f2);
			v2 = odb2.Ext().GetObjectVersion(oid);
			Println("after update odb2 , v2=" + v2);
			odb2.Close();
			AssertEquals(3, v2);
			NeoDatis.Odb.ODB odb4 = NeoDatis.Odb.ODBFactory.OpenClient("localhost", port, Directory
				 + "exta1");
			// Check committed object value
			NeoDatis.Odb.Test.VO.Login.Function f4 = (NeoDatis.Odb.Test.VO.Login.Function)odb4
				.GetObjectFromId(oid);
			AssertEquals(f2.GetName(), f4.GetName());
			AssertEquals(3, odb4.Ext().GetObjectVersion(oid));
			odb4.Close();
			server.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestConcurrentObjectVersion2()
		{
			// LogUtil.allOn(true);
			int port = Port + 8;
			NeoDatis.Tool.IOUtil.DeleteFile(Directory + "exta1");
			NeoDatis.Odb.ODBServer server = NeoDatis.Odb.ODBFactory.OpenServer(port);
			server.StartServer(true);
			NeoDatis.Odb.ODB odb = NeoDatis.Odb.ODBFactory.OpenClient("localhost", port, Directory
				 + "exta1");
			NeoDatis.Odb.OID oid = odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("f1"));
			int version = odb.Ext().GetObjectVersion(oid);
			Println(version);
			odb.Close();
			int nbThreads = 100;
			NeoDatis.Odb.ODB[] odbs = new NeoDatis.Odb.ODB[nbThreads];
			int[] versions = new int[nbThreads];
			NeoDatis.Odb.Test.VO.Login.Function[] functions = new NeoDatis.Odb.Test.VO.Login.Function
				[nbThreads];
			// Open all Odbs and get the object
			for (int i = 0; i < nbThreads; i++)
			{
				odbs[i] = NeoDatis.Odb.ODBFactory.OpenClient("localhost", port, Directory + "exta1"
					);
				versions[i] = odbs[i].Ext().GetObjectVersion(oid);
				functions[i] = (NeoDatis.Odb.Test.VO.Login.Function)odbs[i].GetObjectFromId(oid);
				AssertEquals(1, versions[i]);
				AssertEquals("f1", functions[i].GetName());
			}
			// Open all Odbs and get the object
			for (int i = 0; i < nbThreads; i++)
			{
				functions[i].SetName("function " + i);
				odbs[i].Store(functions[i]);
				versions[i] = odbs[i].Ext().GetObjectVersion(oid);
				Println("Function with name " + functions[i].GetName() + " has version " + versions
					[i]);
				odbs[i].Close();
				AssertEquals(i + 2, versions[i]);
				// Just to check the version number after commit
				odb = NeoDatis.Odb.ODBFactory.OpenClient("localhost", port, Directory + "exta1");
				int committedVersionNumber = odb.Ext().GetObjectVersion(oid);
				Println("After commit = " + committedVersionNumber);
				AssertEquals(i + 2, committedVersionNumber);
				odb.Close();
			}
			NeoDatis.Odb.ODB odb4 = NeoDatis.Odb.ODBFactory.OpenClient("localhost", port, Directory
				 + "exta1");
			// Check committed object value
			NeoDatis.Odb.Test.VO.Login.Function f4 = (NeoDatis.Odb.Test.VO.Login.Function)odb4
				.GetObjectFromId(oid);
			AssertEquals("function " + (nbThreads - 1), f4.GetName());
			AssertEquals(nbThreads + 1, odb4.Ext().GetObjectVersion(oid));
			odb4.Close();
			server.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestConcurrentObjectVersion3()
		{
			if (isLocal || (useSameVmOptimization && !testNewFeature))
			{
				return;
			}
			// LogUtil.allOn(true);
			DeleteBase("exta1");
			NeoDatis.Odb.ODB odb = Open("exta1");
			NeoDatis.Odb.OID oid = odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("f1"));
			int version = odb.Ext().GetObjectVersion(oid);
			Println(version);
			odb.Close();
			int nbThreads = 100;
			NeoDatis.Odb.ODB[] odbs = new NeoDatis.Odb.ODB[nbThreads];
			int[] versions = new int[nbThreads];
			NeoDatis.Odb.Test.VO.Login.Function[] functions = new NeoDatis.Odb.Test.VO.Login.Function
				[nbThreads];
			// Open all Odbs and get the object
			for (int i = 0; i < nbThreads; i++)
			{
				odbs[i] = Open("exta1");
				versions[i] = odbs[i].Ext().GetObjectVersion(oid);
				functions[i] = (NeoDatis.Odb.Test.VO.Login.Function)odbs[i].GetObjectFromId(oid);
				AssertEquals(1, versions[i]);
				AssertEquals("f1", functions[i].GetName());
			}
			// Open all Odbs and get the object
			for (int i = 0; i < nbThreads; i++)
			{
				functions[i].SetName("function " + i);
				odbs[i].Store(functions[i]);
				versions[i] = odbs[i].Ext().GetObjectVersion(oid);
				Println("Function with name " + functions[i].GetName() + " has version " + versions
					[i]);
				odbs[i].Close();
				AssertEquals(i + 2, versions[i]);
				// Just to check the version number after commit
				odb = Open("exta1");
				int committedVersionNumber = odb.Ext().GetObjectVersion(oid);
				Println("After commit = " + committedVersionNumber);
				AssertEquals(i + 2, committedVersionNumber);
				odb.Close();
			}
			NeoDatis.Odb.ODB odb4 = Open("exta1");
			// Check committed object value
			NeoDatis.Odb.Test.VO.Login.Function f4 = (NeoDatis.Odb.Test.VO.Login.Function)odb4
				.GetObjectFromId(oid);
			AssertEquals("function " + (nbThreads - 1), f4.GetName());
			AssertEquals(nbThreads + 1, odb4.Ext().GetObjectVersion(oid));
			odb4.Close();
		}
	}
}

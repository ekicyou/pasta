namespace NeoDatis.Odb.Test.Btree.Odb
{
	public class TestODBLazyBTree : NeoDatis.Odb.Test.ODBTest
	{
		private const int Size = 10000;

		/// <exception cref="System.Exception"></exception>
		private NeoDatis.Btree.IBTreePersister GetPersister(string baseName)
		{
			NeoDatis.Odb.ODB odb = Open(baseName);
			return new NeoDatis.Odb.Impl.Core.Btree.LazyODBBTreePersister(odb);
		}

		// return new InMemoryBTreePersister();
		/// <exception cref="System.Exception"></exception>
		public virtual void Test01()
		{
			if (!isLocal)
			{
				return;
			}
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Btree.IBTreePersister persister = GetPersister(baseName);
			NeoDatis.Btree.IBTree tree = new NeoDatis.Odb.Impl.Core.Btree.ODBBTreeMultiple("test1"
				, 2, persister);
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < Size; i++)
			{
				tree.Insert(i + 1, "value " + (i + 1));
			}
			// println(i);
			// println(new BTreeDisplay().build(tree,true));
			long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			Println("time/object=" + (float)(end - start) / (float)Size);
			AssertTrue((end - start) < 0.34 * Size);
			// println("insert of "+SIZE+" elements in BTREE = " +
			// (end-start)+"ms");
			// persister.close();
			// persister = getPersister();
			AssertEquals(Size, tree.GetSize());
			System.Collections.IEnumerator iterator = tree.Iterator(NeoDatis.Odb.Core.OrderByConstants
				.OrderByAsc);
			int j = 0;
			while (iterator.MoveNext())
			{
				object o = iterator.Current;
				AssertEquals("value " + (j + 1), o);
				j++;
				if (j % 1000 == 0)
				{
					Println(j);
				}
			}
			persister.Close();
			DeleteBase(baseName);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			if (!isLocal)
			{
				return;
			}
			string baseName = GetBaseName();
			NeoDatis.Btree.IBTreePersister persister = GetPersister(baseName);
			NeoDatis.Btree.IBTree tree = new NeoDatis.Odb.Impl.Core.Btree.ODBBTreeMultiple("test1"
				, 2, persister);
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < Size; i++)
			{
				tree.Insert(i + 1, "value " + (i + 1));
			}
			long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			Println(end - start);
			if (testPerformance)
			{
				AssertTrue((end - start) < 0.34 * Size);
			}
			// println("insert of "+SIZE+" elements in BTREE = " +
			// (end-start)+"ms");
			// persister.close();
			// persister = getPersister();
			AssertEquals(Size, tree.GetSize());
			System.Collections.IEnumerator iterator = tree.Iterator(NeoDatis.Odb.Core.OrderByConstants
				.OrderByAsc);
			int j = 0;
			while (iterator.MoveNext())
			{
				object o = iterator.Current;
				AssertEquals("value " + (j + 1), o);
				j++;
				if (j % 1000 == 0)
				{
					Println(j);
				}
			}
			persister.Close();
			DeleteBase(baseName);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestLazyCache()
		{
			if (!isLocal)
			{
				return;
			}
			string baseName = GetBaseName();
			NeoDatis.Btree.IBTreePersister persister = GetPersister(baseName);
			NeoDatis.Btree.IBTree tree = new NeoDatis.Odb.Impl.Core.Btree.ODBBTreeMultiple("test1"
				, 2, persister);
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < Size; i++)
			{
				tree.Insert(i + 1, "value " + (i + 1));
			}
			long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			if (testPerformance)
			{
				AssertTrue((end - start) < 0.34 * Size);
			}
			// println("insert of "+SIZE+" elements in BTREE = " +
			// (end-start)+"ms");
			// persister.close();
			// persister = getPersister();
			// /assertEquals(SIZE,tree.size());
			System.Collections.IEnumerator iterator = tree.Iterator(NeoDatis.Odb.Core.OrderByConstants
				.OrderByAsc);
			int j = 0;
			while (iterator.MoveNext())
			{
				object o = iterator.Current;
				j++;
				if (j == Size)
				{
					AssertEquals("value " + Size, o);
				}
			}
			persister.Close();
			DeleteBase(baseName);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test1a()
		{
			if (!isLocal)
			{
				return;
			}
			string baseName = GetBaseName();
			// Configuration.setInPlaceUpdate(true);
			NeoDatis.Btree.IBTreePersister persister = GetPersister(baseName);
			NeoDatis.Btree.IBTree tree = new NeoDatis.Odb.Impl.Core.Btree.ODBBTreeMultiple("test1a"
				, 2, persister);
			for (int i = 0; i < Size; i++)
			{
				tree.Insert(i + 1, "value " + (i + 1));
			}
			// println(new BTreeDisplay().build(tree,true).toString());
			persister.Close();
			persister = GetPersister(baseName);
			tree = persister.LoadBTree(tree.GetId());
			// println(new BTreeDisplay().build(tree,true).toString());
			AssertEquals(Size, tree.GetSize());
			System.Collections.IEnumerator iterator = tree.Iterator(NeoDatis.Odb.Core.OrderByConstants
				.OrderByAsc);
			int j = 0;
			while (iterator.MoveNext())
			{
				object o = iterator.Current;
				AssertEquals("value " + (j + 1), o);
				j++;
				if (j == Size)
				{
					AssertEquals("value " + Size, o);
				}
			}
			persister.Close();
			DeleteBase(baseName);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test2a()
		{
			if (!isLocal)
			{
				return;
			}
			string baseName = GetBaseName();
			NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter.ResetNbUpdates();
			// LogUtil.allOn(true);
			DeleteBase(baseName);
			NeoDatis.Btree.IBTreePersister persister = GetPersister(baseName);
			NeoDatis.Btree.IBTreeMultipleValuesPerKey tree = new NeoDatis.Odb.Impl.Core.Btree.ODBBTreeMultiple
				("test2a", 20, persister);
			long start0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < Size; i++)
			{
				tree.Insert(i + 1, "value " + (i + 1));
			}
			// println("Commiting");
			persister.Close();
			long end0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			// println("insert of "+SIZE+" elements in BTREE = " +
			// (end0-start0)+"ms");
			// println("end Commiting");
			// println("updates : IP="+ObjectWriter.getNbInPlaceUpdates()+" , N="+ObjectWriter.getNbNormalUpdates());
			// ODB odb = open(baseName);
			// odb.getObjects(LazyNode.class);
			// odb.close();
			persister = GetPersister(baseName);
			// println("reloading btree");
			tree = (NeoDatis.Btree.IBTreeMultipleValuesPerKey)persister.LoadBTree(tree.GetId(
				));
			// println("end reloading btree , size="+tree.size());
			AssertEquals(Size, tree.GetSize());
			long totalSearchTime = 0;
			long oneSearchTime = 0;
			long minSearchTime = 10000;
			long maxSearchTime = -1;
			for (int i = 0; i < Size; i++)
			{
				long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				System.Collections.IList o = tree.Search(i + 1);
				long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				AssertEquals("value " + (i + 1), o[0]);
				oneSearchTime = (end - start);
				// println("Search time for "+o+" = "+oneSearchTime);
				if (oneSearchTime > maxSearchTime)
				{
					maxSearchTime = oneSearchTime;
				}
				if (oneSearchTime < minSearchTime)
				{
					minSearchTime = oneSearchTime;
				}
				totalSearchTime += oneSearchTime;
			}
			persister.Close();
			// println("total search time="+totalSearchTime +
			// " - mean st="+((double)totalSearchTime/SIZE));
			// println("min search time="+minSearchTime + " - max="+maxSearchTime);
			// Median search time must be smaller than 1ms
			DeleteBase(baseName);
			AssertTrue(totalSearchTime < 1 * Size);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test2()
		{
			if (!isLocal)
			{
				return;
			}
			string baseName = GetBaseName();
			NeoDatis.Btree.IBTreePersister persister = GetPersister(baseName);
			NeoDatis.Btree.IBTree tree = new NeoDatis.Odb.Impl.Core.Btree.ODBBTreeMultiple("test2"
				, 2, persister);
			for (int i = 0; i < Size; i++)
			{
				tree.Insert(i + 1, "value " + (i + 1));
			}
			AssertEquals(Size, tree.GetSize());
			System.Collections.IEnumerator iterator = tree.Iterator(NeoDatis.Odb.Core.OrderByConstants
				.OrderByDesc);
			int j = 0;
			while (iterator.MoveNext())
			{
				object o = iterator.Current;
				// println(o);
				j++;
				if (j == Size)
				{
					AssertEquals("value " + 1, o);
				}
			}
			persister.Close();
			DeleteBase(baseName);
		}

		/// <exception cref="System.Exception"></exception>
		public static void Main2(string[] args)
		{
			NeoDatis.Odb.Test.Btree.Odb.TestODBLazyBTree t = new NeoDatis.Odb.Test.Btree.Odb.TestODBLazyBTree
				();
			for (int i = 0; i < 1000; i++)
			{
				try
				{
					t.Test1a();
				}
				catch (System.Exception e)
				{
					System.Console.Out.WriteLine("ERROR On loop " + i);
					throw;
				}
			}
		}
	}
}

namespace NeoDatis.Odb.Test.Index
{
	public class TestIndex : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public virtual void TestSaveIndex()
		{
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB @base = Open(baseName);
			NeoDatis.Odb.ClassRepresentation clazz = @base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject
				));
			string[] indexFields = new string[] { "name", "duration" };
			clazz.AddUniqueIndexOn("index1", indexFields, true);
			string[] indexFields2 = new string[] { "name", "creation" };
			clazz.AddUniqueIndexOn("index2", indexFields2, true);
			string[] indexFields3 = new string[] { "duration", "creation" };
			clazz.AddUniqueIndexOn("index3", indexFields3, true);
			@base.Close();
			@base = Open(baseName);
			NeoDatis.Odb.Core.Transaction.ISession session = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy
				.GetEngine(@base).GetSession(true);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel = session.GetStorageEngine
				().GetSession(true).GetMetaModel();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = metaModel.GetClassInfo(typeof(
				NeoDatis.Odb.Test.Index.IndexedObject).FullName, true);
			AssertEquals(3, ci.GetNumberOfIndexes());
			AssertEquals(ci.GetIndex(0).GetName(), "index1");
			AssertEquals(1, ci.GetIndex(0).GetAttributeIds()[0]);
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex.Enabled, ci.GetIndex
				(0).GetStatus());
			AssertEquals(ci.GetIndex(1).GetName(), "index2");
			AssertEquals(1, ci.GetIndex(1).GetAttributeIds()[0]);
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex.Enabled, ci.GetIndex
				(1).GetStatus());
			AssertEquals(ci.GetIndex(2).GetName(), "index3");
			AssertEquals(2, ci.GetIndex(2).GetAttributeIds()[0]);
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex.Enabled, ci.GetIndex
				(0).GetStatus());
			@base.Close();
			DeleteBase(baseName);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestInsertWithIndex()
		{
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB @base = Open(baseName);
			NeoDatis.Odb.ClassRepresentation clazz = @base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject
				));
			string[] indexFields = new string[] { "name", "duration" };
			clazz.AddUniqueIndexOn("index1", indexFields, true);
			@base.Close();
			@base = Open(baseName);
			NeoDatis.Odb.Test.Index.IndexedObject io1 = new NeoDatis.Odb.Test.Index.IndexedObject
				("olivier", 15, new System.DateTime());
			@base.Store(io1);
			@base.Close();
			@base = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
				.IsNotNull("name"));
			NeoDatis.Odb.Objects objects = @base.GetObjects(q, true);
			@base.Close();
			AssertEquals(1, objects.Count);
			NeoDatis.Odb.Test.Index.IndexedObject io2 = (NeoDatis.Odb.Test.Index.IndexedObject
				)objects.GetFirst();
			AssertEquals("olivier", io2.GetName());
			AssertEquals(15, io2.GetDuration());
			AssertFalse(q.GetExecutionPlan().GetDetails().IndexOf("index1") != -1);
		}

		// deleteBase(baseName);
		/// <exception cref="System.Exception"></exception>
		public virtual void TestIndexWithOneFieldAndQueryWithTwoFields()
		{
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB @base = Open(baseName);
			NeoDatis.Odb.ClassRepresentation clazz = @base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject
				));
			string[] indexFields = new string[] { "name" };
			clazz.AddUniqueIndexOn("index1", indexFields, true);
			@base.Close();
			@base = Open(baseName);
			NeoDatis.Odb.Test.Index.IndexedObject io1 = new NeoDatis.Odb.Test.Index.IndexedObject
				("olivier", 15, new System.DateTime());
			@base.Store(io1);
			@base.Close();
			@base = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
				.And().Add(NeoDatis.Odb.Core.Query.Criteria.Where.Equal("name", "olivier")).Add(
				NeoDatis.Odb.Core.Query.Criteria.Where.Equal("duration", 15)));
			NeoDatis.Odb.Objects objects = @base.GetObjects(q, true);
			@base.Close();
			Println(q.GetExecutionPlan().ToString());
			AssertEquals(false, q.GetExecutionPlan().UseIndex());
			AssertEquals(1, objects.Count);
			DeleteBase(baseName);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestInsertWithIndex1()
		{
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB @base = Open(baseName);
			// base.store(new IndexedObject());
			NeoDatis.Odb.ClassRepresentation clazz = @base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject
				));
			string[] indexFields = new string[] { "name" };
			clazz.AddUniqueIndexOn("index1", indexFields, true);
			@base.Close();
			@base = Open(baseName);
			int size = 1000;
			long start0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Test.Index.IndexedObject io1 = new NeoDatis.Odb.Test.Index.IndexedObject
					("olivier" + (i + 1), 15 + i, new System.DateTime());
				@base.Store(io1);
				if (i % 1000 == 0)
				{
					Println(i);
				}
			}
			long tt0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			@base.Close();
			long tt1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			long end0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			Println("IPU=" + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter
				.GetNbInPlaceUpdates() + " - NU=" + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter
				.GetNbNormalUpdates());
			Println("inserting time with index=" + (end0 - start0));
			Println("commit time=" + (tt1 - tt0));
			Println(NeoDatis.Odb.Impl.Core.Btree.LazyODBBTreePersister.Counters());
			@base = Open(baseName);
			long totalTime = 0;
			long maxTime = 0;
			long minTime = 100000;
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
					.Equal("name", "olivier" + (i + 1)));
				long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				NeoDatis.Odb.Objects objects = @base.GetObjects(q, true);
				long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				AssertEquals(1, objects.Count);
				NeoDatis.Odb.Test.Index.IndexedObject io2 = (NeoDatis.Odb.Test.Index.IndexedObject
					)objects.GetFirst();
				AssertEquals("olivier" + (i + 1), io2.GetName());
				AssertEquals(15 + i, io2.GetDuration());
				long d = end - start;
				totalTime += d;
				if (d > maxTime)
				{
					maxTime = d;
				}
				if (d < minTime)
				{
					minTime = d;
				}
			}
			@base.Close();
			DeleteBase(baseName);
			Println("total duration=" + totalTime + " / " + (double)totalTime / size);
			Println("duration max=" + maxTime + " / min=" + minTime);
			if (testPerformance && totalTime / size > 2)
			{
				Fail("Total/size is > than 2 : " + totalTime);
			}
			if (testPerformance)
			{
				// TODO Try to get maxTime < 10!
				AssertTrue(maxTime < 100);
				AssertTrue(minTime < 1);
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestInsertWithIndex2()
		{
			string baseName = GetBaseName();
			if (!runAll)
			{
				return;
			}
			DeleteBase(baseName);
			NeoDatis.Odb.ODB @base = Open(baseName);
			// base.store(new IndexedObject());
			NeoDatis.Odb.ClassRepresentation clazz = @base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject
				));
			string[] indexFields = new string[] { "name" };
			clazz.AddUniqueIndexOn("index1", indexFields, true);
			int size = 10000;
			long start0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Test.Index.IndexedObject io1 = new NeoDatis.Odb.Test.Index.IndexedObject
					("olivier" + (i + 1), 15 + i, new System.DateTime());
				@base.Store(io1);
				if (i % 1000 == 0)
				{
					Println(i);
				}
			}
			long tt0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			@base.Close();
			long tt1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			long end0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			Println("IPU=" + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter
				.GetNbInPlaceUpdates() + " - NU=" + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter
				.GetNbNormalUpdates());
			Println("inserting time with index=" + (end0 - start0));
			Println("commit time=" + (tt1 - tt0));
			Println(NeoDatis.Odb.Impl.Core.Btree.LazyODBBTreePersister.Counters());
			@base = Open(baseName);
			long totalTime = 0;
			long maxTime = 0;
			long minTime = 100000;
			long t0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			long t1 = 0;
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
					.Equal("name", "olivier" + (i + 1)));
				long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				NeoDatis.Odb.Objects objects = @base.GetObjects(q, true);
				long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				AssertEquals(1, objects.Count);
				NeoDatis.Odb.Test.Index.IndexedObject io2 = (NeoDatis.Odb.Test.Index.IndexedObject
					)objects.GetFirst();
				AssertEquals("olivier" + (i + 1), io2.GetName());
				AssertEquals(15 + i, io2.GetDuration());
				long d = end - start;
				totalTime += d;
				if (d > maxTime)
				{
					maxTime = d;
				}
				if (d < minTime)
				{
					minTime = d;
				}
				if (i % 1000 == 0)
				{
					t1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
					Println("i=" + i + " - time=" + (t1 - t0));
					t0 = t1;
				}
			}
			// /println(LazyODBBTreePersister.counters());
			@base.Close();
			DeleteBase(baseName);
			// println("total duration=" + totalTime + " / " + (double) totalTime /
			// size);
			// println("duration max=" + maxTime + " / min=" + minTime);
			if (totalTime / size > 1)
			{
				Fail("Total/size is > than 1 : " + (float)((float)totalTime / (float)size));
			}
			Println("Max time=" + maxTime);
			Println("Min time=" + minTime);
			// TODO Try to get maxTime < 10!
			AssertTrue(maxTime < 250);
			AssertTrue(minTime < 1);
		}

		/// <summary>Test with on e key index</summary>
		/// <exception cref="System.Exception"></exception>
		public virtual void TestInsertWithIndex3()
		{
			string baseName = GetBaseName();
			// LogUtil.logOn(LazyODBBTreePersister.LOG_ID, true);
			DeleteBase(baseName);
			NeoDatis.Odb.ODB @base = Open(baseName);
			NeoDatis.Odb.OdbConfiguration.SetUseLazyCache(false);
			// base.store(new IndexedObject());
			NeoDatis.Odb.ClassRepresentation clazz = @base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject
				));
			string[] indexFields = new string[] { "name" };
			clazz.AddUniqueIndexOn("index1", indexFields, true);
			@base.Close();
			@base = Open(baseName);
			int size = isLocal ? 1300 : 300;
			int commitInterval = 10;
			long start0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy
				.GetEngine(@base);
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Test.Index.IndexedObject io1 = new NeoDatis.Odb.Test.Index.IndexedObject
					("olivier" + (i + 1), 15 + size, new System.DateTime());
				@base.Store(io1);
				if (i % commitInterval == 0)
				{
					@base.Commit();
					@base.Close();
					@base = Open(baseName);
					engine = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy.GetEngine(@base);
				}
				if (io1.GetName().Equals("olivier" + size))
				{
					Println("Ola chico");
				}
			}
			engine = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy.GetEngine(@base);
			// println(new
			// BTreeDisplay().build(engine.getSession(true).getMetaModel().getClassInfo(IndexedObject.class.getName(),
			// true).getIndex(0).getBTree(), true));
			@base.Close();
			long end0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			// println("IPU=" + ObjectWriter.getNbInPlaceUpdates() + " - NU=" +
			// ObjectWriter.getNbNormalUpdates());
			// println("inserting time with index=" + (end0 - start0));
			@base = Open(baseName);
			engine = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy.GetEngine(@base);
			// println("After load = unconnected : "+
			// engine.getSession(true).getMetaModel().getClassInfo(IndexedObject.class.getName(),
			// true).getUncommittedZoneInfo());
			// println("After Load = connected : "+
			// engine.getSession(true).getMetaModel().getClassInfo(IndexedObject.class.getName(),
			// true).getCommitedZoneInfo());
			// println(new
			// BTreeDisplay().build(engine.getSession(true).getMetaModel().getClassInfo(IndexedObject.class.getName(),
			// true).getIndex(0).getBTree(), true));
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("name", "olivier" + size));
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			NeoDatis.Odb.Objects objects = @base.GetObjects(q, false);
			long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			try
			{
				AssertEquals(1, objects.Count);
				NeoDatis.Odb.Test.Index.IndexedObject io2 = (NeoDatis.Odb.Test.Index.IndexedObject
					)objects.GetFirst();
				AssertEquals("olivier" + size, io2.GetName());
				AssertEquals(15 + size, io2.GetDuration());
				long duration = end - start;
				Println("duration=" + duration);
				NeoDatis.Odb.OdbConfiguration.SetUseLazyCache(false);
				if (testPerformance)
				{
					if (isLocal)
					{
						if (duration > 2)
						{
							Fail("Time of search in index is greater than 2ms : " + duration);
						}
					}
					else
					{
						if (duration > 32)
						{
							Fail("Time of search in index is greater than 2ms : " + duration);
						}
					}
				}
			}
			finally
			{
				@base.Close();
				DeleteBase(baseName);
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestInsertWithIndex3Part1()
		{
			string baseName = "index.neodatis";
			// LogUtil.logOn(LazyODBBTreePersister.LOG_ID, true);
			DeleteBase(baseName);
			NeoDatis.Odb.ODB @base = Open(baseName);
			NeoDatis.Odb.OdbConfiguration.SetUseLazyCache(false);
			// base.store(new IndexedObject());
			NeoDatis.Odb.ClassRepresentation clazz = @base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject
				));
			string[] indexFields = new string[] { "name" };
			clazz.AddUniqueIndexOn("index1", indexFields, true);
			@base.Close();
			@base = Open(baseName);
			int size = 1300;
			int commitInterval = 10;
			long start0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy
				.GetEngine(@base);
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Test.Index.IndexedObject io1 = new NeoDatis.Odb.Test.Index.IndexedObject
					("olivier" + (i + 1), 15 + size, new System.DateTime());
				@base.Store(io1);
				if (i % commitInterval == 0)
				{
					@base.Commit();
					@base.Close();
					@base = Open(baseName);
					engine = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy.GetEngine(@base);
				}
				if (io1.GetName().Equals("olivier" + size))
				{
					Println("Ola chico");
				}
			}
			engine = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy.GetEngine(@base);
			// println(new
			// BTreeDisplay().build(engine.getSession(true).getMetaModel().getClassInfo(IndexedObject.class.getName(),
			// true).getIndex(0).getBTree(), true));
			@base.Close();
			long end0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestInsertWithIndex3Part2()
		{
			string baseName = "index.neodatis";
			int size = 1300;
			NeoDatis.Odb.ODB @base = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("name", "olivier" + size));
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			NeoDatis.Odb.Objects objects = @base.GetObjects(q, false);
			long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			try
			{
				AssertEquals(1, objects.Count);
				NeoDatis.Odb.Test.Index.IndexedObject io2 = (NeoDatis.Odb.Test.Index.IndexedObject
					)objects.GetFirst();
				AssertEquals("olivier" + size, io2.GetName());
				AssertEquals(15 + size, io2.GetDuration());
				long duration = end - start;
				Println("duration=" + duration);
				NeoDatis.Odb.OdbConfiguration.SetUseLazyCache(false);
				if (testPerformance)
				{
					if (isLocal)
					{
						if (duration > 2)
						{
							Fail("Time of search in index is greater than 2ms : " + duration);
						}
					}
					else
					{
						if (duration > 32)
						{
							Fail("Time of search in index is greater than 2ms : " + duration);
						}
					}
				}
			}
			finally
			{
				@base.Close();
			}
		}

		// deleteBase(baseName);
		/// <summary>Test with one key index</summary>
		/// <exception cref="System.Exception"></exception>
		public virtual void TestInsertWithIntIndex3CheckAll()
		{
			string baseName = GetBaseName();
			// LogUtil.logOn(LazyODBBTreePersister.LOG_ID, true);
			DeleteBase(baseName);
			NeoDatis.Odb.ODB @base = Open(baseName);
			NeoDatis.Odb.OdbConfiguration.SetUseLazyCache(false);
			// base.store(new IndexedObject());
			NeoDatis.Odb.ClassRepresentation clazz = @base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject
				));
			string[] indexFields = new string[] { "duration" };
			clazz.AddUniqueIndexOn("index1", indexFields, true);
			@base.Close();
			@base = Open(baseName);
			int size = 1300;
			int commitInterval = 10;
			long start0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Test.Index.IndexedObject io1 = new NeoDatis.Odb.Test.Index.IndexedObject
					("olivier" + (i + 1), i, new System.DateTime());
				@base.Store(io1);
				if (i % commitInterval == 0)
				{
					@base.Commit();
				}
			}
			// println(i+" : commit / " + size);
			@base.Close();
			long end0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			// println("IPU=" + ObjectWriter.getNbInPlaceUpdates() + " - NU=" +
			// ObjectWriter.getNbNormalUpdates());
			// println("inserting time with index=" + (end0 - start0));
			@base = Open(baseName);
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
					.Equal("duration", i));
				NeoDatis.Odb.Objects objects = @base.GetObjects(q, false);
				// println("olivier" + (i+1));
				AssertEquals(1, objects.Count);
			}
			long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			try
			{
				float duration = (float)(end - start) / (float)size;
				if (testPerformance && duration > 2)
				{
					Fail("Time of search in index is greater than 2ms : " + duration);
				}
			}
			finally
			{
				@base.Close();
				DeleteBase(baseName);
			}
		}

		/// <summary>Test with one key index</summary>
		/// <exception cref="System.Exception"></exception>
		public virtual void TestInsertWithDateIndex3CheckAll()
		{
			string baseName = GetBaseName();
			// LogUtil.logOn(LazyODBBTreePersister.LOG_ID, true);
			DeleteBase(baseName);
			NeoDatis.Odb.ODB @base = Open(baseName);
			NeoDatis.Odb.OdbConfiguration.SetUseLazyCache(false);
			// base.store(new IndexedObject());
			NeoDatis.Odb.ClassRepresentation clazz = @base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject
				));
			string[] indexFields = new string[] { "creation" };
			clazz.AddUniqueIndexOn("index1", indexFields, true);
			@base.Close();
			@base = Open(baseName);
			int size = 1300;
			int commitInterval = 1000;
			long start0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Test.Index.IndexedObject io1 = new NeoDatis.Odb.Test.Index.IndexedObject
					("olivier" + (i + 1), i, new System.DateTime(start0 + i));
				@base.Store(io1);
				if (i % commitInterval == 0)
				{
					@base.Commit();
				}
			}
			// println(i+" : commit / " + size);
			@base.Close();
			long end0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			// println("IPU=" + ObjectWriter.getNbInPlaceUpdates() + " - NU=" +
			// ObjectWriter.getNbNormalUpdates());
			// println("inserting time with index=" + (end0 - start0));
			@base = Open(baseName);
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
					.Equal("creation", new System.DateTime(start0 + i)));
				NeoDatis.Odb.Objects objects = @base.GetObjects(q, false);
				// println("olivier" + (i+1));
				AssertEquals(1, objects.Count);
			}
			long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			try
			{
				float duration = (float)(end - start) / (float)size;
				Println(duration);
				double d = 0.144;
				if (!isLocal)
				{
					d = 1.16;
				}
				if (testPerformance && duration > d)
				{
					Fail("Time of search in index is greater than " + d + " ms : " + duration);
				}
			}
			finally
			{
				@base.Close();
				DeleteBase(baseName);
			}
		}

		/// <summary>Test with 3 indexes</summary>
		/// <exception cref="System.Exception"></exception>
		public virtual void TestInsertWith3IndexesCheckAll()
		{
			string baseName = GetBaseName();
			// LogUtil.logOn(LazyODBBTreePersister.LOG_ID, true);
			DeleteBase(baseName);
			NeoDatis.Odb.ODB @base = Open(baseName);
			NeoDatis.Odb.OdbConfiguration.SetUseLazyCache(false);
			// base.store(new IndexedObject());
			NeoDatis.Odb.ClassRepresentation clazz = @base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject
				));
			string[] indexFields = new string[] { "duration" };
			clazz.AddIndexOn("index1", indexFields, true);
			string[] indexFields2 = new string[] { "creation" };
			clazz.AddIndexOn("index2", indexFields2, true);
			string[] indexFields3 = new string[] { "name" };
			clazz.AddIndexOn("index3", indexFields3, true);
			@base.Close();
			@base = Open(baseName);
			int size = 1300;
			int commitInterval = 10;
			long start0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Test.Index.IndexedObject io1 = new NeoDatis.Odb.Test.Index.IndexedObject
					("olivier" + (i + 1), i, new System.DateTime());
				@base.Store(io1);
				if (i % commitInterval == 0)
				{
					@base.Commit();
					Println(i + " : commit / " + size);
				}
			}
			@base.Close();
			long end0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			// println("IPU=" + ObjectWriter.getNbInPlaceUpdates() + " - NU=" +
			// ObjectWriter.getNbNormalUpdates());
			// println("inserting time with index=" + (end0 - start0));
			@base = Open(baseName);
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
					.Equal("duration", i));
				NeoDatis.Odb.Objects objects = @base.GetObjects(q, false);
				// println("olivier" + (i+1));
				AssertEquals(1, objects.Count);
			}
			long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			try
			{
				float duration = (float)(end - start) / (float)size;
				Println(duration);
				double d = 0.144;
				if (!isLocal)
				{
					d = 1.16;
				}
				if (testPerformance && duration > d)
				{
					Fail("Time of search in index is greater than " + d + " ms : " + duration);
				}
			}
			finally
			{
				@base.Close();
				DeleteBase(baseName);
			}
		}

		/// <summary>Test with on e key index</summary>
		/// <exception cref="System.Exception"></exception>
		public virtual void TestInsertWithoutIndex3()
		{
			string baseName = GetBaseName();
			if (!runAll)
			{
				return;
			}
			DeleteBase(baseName);
			NeoDatis.Odb.ODB @base = Open(baseName);
			int size = 30000;
			int commitInterval = 1000;
			long start0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Test.Index.IndexedObject io1 = new NeoDatis.Odb.Test.Index.IndexedObject
					("olivier" + (i + 1), 15 + size, new System.DateTime());
				@base.Store(io1);
				if (i % commitInterval == 0)
				{
					@base.Commit();
				}
			}
			// println(i+" : commit");
			@base.Close();
			long end0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			Println("IPU=" + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter
				.GetNbInPlaceUpdates() + " - NU=" + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter
				.GetNbNormalUpdates());
			Println("inserting time with index=" + (end0 - start0));
			@base = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("name", "olivier" + size));
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			NeoDatis.Odb.Objects objects = @base.GetObjects(q, false);
			long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			AssertEquals(1, objects.Count);
			NeoDatis.Odb.Test.Index.IndexedObject io2 = (NeoDatis.Odb.Test.Index.IndexedObject
				)objects.GetFirst();
			AssertEquals("olivier" + size, io2.GetName());
			AssertEquals(15 + size, io2.GetDuration());
			long duration = end - start;
			Println("duration=" + duration);
			@base.Close();
			DeleteBase(baseName);
			NeoDatis.Odb.OdbConfiguration.SetUseLazyCache(false);
			Println(duration);
			double d = 408;
			if (!isLocal)
			{
				d = 3500;
			}
			if (duration > d)
			{
				Fail("Time of search in index is greater than " + d + " ms : " + duration);
			}
		}

		/// <summary>Test with two key index</summary>
		/// <exception cref="System.Exception"></exception>
		public virtual void TestInsertWith3Indexes()
		{
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB @base = Open(baseName);
			// Configuration.setUseLazyCache(true);
			// base.store(new IndexedObject());
			NeoDatis.Odb.ClassRepresentation clazz = @base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject
				));
			string[] indexFields3 = new string[] { "name" };
			clazz.AddUniqueIndexOn("index3", indexFields3, true);
			string[] indexFields2 = new string[] { "name", "creation" };
			clazz.AddUniqueIndexOn("index2", indexFields2, true);
			string[] indexField4 = new string[] { "duration", "creation" };
			clazz.AddUniqueIndexOn("inde3", indexField4, true);
			@base.Close();
			@base = Open(baseName);
			int size = isLocal ? 10000 : 1000;
			long start0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			System.DateTime[] dates = new System.DateTime[size];
			for (int i = 0; i < size; i++)
			{
				// println(i);
				dates[i] = new System.DateTime();
				NeoDatis.Odb.Test.Index.IndexedObject io1 = new NeoDatis.Odb.Test.Index.IndexedObject
					("olivier" + (i + 1), i, dates[i]);
				@base.Store(io1);
				if (i % 100 == 0)
				{
					Println(i);
				}
			}
			@base.Close();
			long end0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			Println("IPU=" + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter
				.GetNbInPlaceUpdates() + " - NU=" + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter
				.GetNbNormalUpdates());
			Println("inserting time with index=" + (end0 - start0));
			@base = Open(baseName);
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
					.And().Add(NeoDatis.Odb.Core.Query.Criteria.Where.Equal("duration", i)).Add(NeoDatis.Odb.Core.Query.Criteria.Where
					.Equal("creation", dates[i])));
				NeoDatis.Odb.Objects objects = @base.GetObjects(q, true);
				AssertEquals(1, objects.Count);
				AssertTrue(q.GetExecutionPlan().UseIndex());
			}
			long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			double duration = (end - start);
			duration = duration / size;
			Println("duration=" + duration);
			@base.Close();
			DeleteBase(baseName);
			NeoDatis.Odb.OdbConfiguration.SetUseLazyCache(false);
			Println(duration);
			double d = 0.11;
			if (!isLocal)
			{
				d = 10;
			}
			if (duration > d)
			{
				Fail("Time of search in index is greater than " + d + " ms : " + duration);
			}
		}

		/// <summary>Test with two key index</summary>
		/// <exception cref="System.Exception"></exception>
		public virtual void TestInsertWith4IndexesAndCommits()
		{
			string baseName = GetBaseName();
			if (!runAll)
			{
				return;
			}
			DeleteBase(baseName);
			NeoDatis.Odb.ODB @base = Open(baseName);
			// Configuration.setUseLazyCache(true);
			// base.store(new IndexedObject());
			NeoDatis.Odb.ClassRepresentation clazz = @base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject
				));
			string[] indexField1 = new string[] { "duration" };
			clazz.AddUniqueIndexOn("inde1", indexField1, true);
			string[] indexFields3 = new string[] { "name" };
			clazz.AddUniqueIndexOn("index3", indexFields3, true);
			string[] indexFields2 = new string[] { "name", "creation" };
			clazz.AddUniqueIndexOn("index2", indexFields2, true);
			string[] indexField4 = new string[] { "duration", "creation" };
			clazz.AddUniqueIndexOn("inde4", indexField4, true);
			@base.Close();
			@base = Open(baseName);
			int size = 10000;
			int commitInterval = 10;
			long start0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				// println(i);
				NeoDatis.Odb.Test.Index.IndexedObject io1 = new NeoDatis.Odb.Test.Index.IndexedObject
					("olivier" + (i + 1), i, new System.DateTime());
				@base.Store(io1);
				if (i % 1000 == 0)
				{
					Println(i);
				}
				if (i % commitInterval == 0)
				{
					@base.Commit();
				}
			}
			@base.Close();
			long end0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			Println("IPU=" + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter
				.GetNbInPlaceUpdates() + " - NU=" + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter
				.GetNbNormalUpdates());
			Println("inserting time with index=" + (end0 - start0));
			@base = Open(baseName);
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
					.Equal("duration", i));
				NeoDatis.Odb.Objects objects = @base.GetObjects(q, false);
				// println("olivier" + (i+1));
				AssertEquals(1, objects.Count);
			}
			long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			long duration = end - start;
			Println("duration=" + duration);
			@base.Close();
			DeleteBase(baseName);
			NeoDatis.Odb.OdbConfiguration.SetUseLazyCache(false);
			if (testPerformance && duration > 111)
			{
				Fail("Time of search in index : " + duration + ", should be less than 111");
			}
		}

		/// <summary>Test with two key index</summary>
		/// <exception cref="System.Exception"></exception>
		public virtual void TestInsertWithIndex4()
		{
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB @base = Open(baseName);
			// base.store(new IndexedObject());
			NeoDatis.Odb.ClassRepresentation clazz = @base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject
				));
			string[] indexFields3 = new string[] { "name" };
			clazz.AddUniqueIndexOn("index3", indexFields3, true);
			string[] indexFields2 = new string[] { "name", "creation" };
			clazz.AddUniqueIndexOn("index2", indexFields2, true);
			string[] indexField4 = new string[] { "duration", "creation" };
			clazz.AddUniqueIndexOn("inde3", indexField4, true);
			@base.Close();
			@base = Open(baseName);
			int size = isLocal ? 50000 : 1000;
			int commitInterval = 1000;
			long start0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				// println(i);
				NeoDatis.Odb.Test.Index.IndexedObject ioio = new NeoDatis.Odb.Test.Index.IndexedObject
					("olivier" + (i + 1), i + 15 + size, new System.DateTime());
				@base.Store(ioio);
				if (i % commitInterval == 0)
				{
					long t0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
					@base.Commit();
					long t1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
					Println(i + " : commit - ctime " + (t1 - t0) + " -ttime=");
					// println(LazyODBBTreePersister.counters());
					NeoDatis.Odb.Impl.Core.Btree.LazyODBBTreePersister.ResetCounters();
				}
			}
			System.DateTime theDate = new System.DateTime();
			string theName = "name indexed";
			NeoDatis.Odb.Test.Index.IndexedObject io1 = new NeoDatis.Odb.Test.Index.IndexedObject
				(theName, 45, theDate);
			@base.Store(io1);
			@base.Close();
			long end0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			Println("IPU=" + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter
				.GetNbInPlaceUpdates() + " - NU=" + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter
				.GetNbNormalUpdates());
			Println("inserting time with index=" + (end0 - start0));
			@base = Open(baseName);
			// IQuery q = new
			// CriteriaQuery(IndexedObject.class,Restrictions.and().add(Restrictions.equal("name",theName)).add(Restrictions.equal("creation",
			// theDate)));
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("name", theName));
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			NeoDatis.Odb.Objects objects = @base.GetObjects(q, true);
			long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			if (isLocal)
			{
				AssertEquals("index3", q.GetExecutionPlan().GetIndex().GetName());
			}
			AssertEquals(1, objects.Count);
			NeoDatis.Odb.Test.Index.IndexedObject io2 = (NeoDatis.Odb.Test.Index.IndexedObject
				)objects.GetFirst();
			AssertEquals(theName, io2.GetName());
			AssertEquals(45, io2.GetDuration());
			AssertEquals(theDate, io2.GetCreation());
			long duration = end - start;
			Println("duration=" + duration);
			@base.Close();
			DeleteBase(baseName);
			NeoDatis.Odb.OdbConfiguration.SetUseLazyCache(false);
			if (testPerformance && duration > 1)
			{
				Fail("Time of search in index > 1 : " + duration);
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestInsertAndDeleteWithIndex()
		{
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB @base = Open(baseName);
			// base.store(new IndexedObject());
			NeoDatis.Odb.ClassRepresentation clazz = @base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject
				));
			string[] indexFields = new string[] { "name" };
			clazz.AddUniqueIndexOn("index1", indexFields, true);
			@base.Close();
			@base = Open(baseName);
			int size = 1000;
			long start0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Test.Index.IndexedObject io1 = new NeoDatis.Odb.Test.Index.IndexedObject
					("olivier" + (i + 1), 15 + i, new System.DateTime());
				@base.Store(io1);
				if (i % 1000 == 0)
				{
					Println(i);
				}
			}
			long tt0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			@base.Close();
			long tt1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			long end0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			Println("IPU=" + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter
				.GetNbInPlaceUpdates() + " - NU=" + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter
				.GetNbNormalUpdates());
			Println("inserting time with index=" + (end0 - start0));
			Println("commit time=" + (tt1 - tt0));
			Println(NeoDatis.Odb.Impl.Core.Btree.LazyODBBTreePersister.Counters());
			@base = Open(baseName);
			long totalTime = 0;
			long maxTime = 0;
			long minTime = 100000;
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Core.Query.IQuery query = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
					.Equal("name", "olivier" + (i + 1)));
				long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				NeoDatis.Odb.Objects objects = @base.GetObjects(query, true);
				long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				AssertEquals(1, objects.Count);
				NeoDatis.Odb.Test.Index.IndexedObject io2 = (NeoDatis.Odb.Test.Index.IndexedObject
					)objects.GetFirst();
				AssertEquals("olivier" + (i + 1), io2.GetName());
				AssertEquals(15 + i, io2.GetDuration());
				long d = end - start;
				totalTime += d;
				if (d > maxTime)
				{
					maxTime = d;
				}
				if (d < minTime)
				{
					minTime = d;
				}
				@base.Delete(io2);
			}
			@base.Close();
			@base = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.Index.IndexedObject));
			NeoDatis.Odb.Objects oos = @base.GetObjects(q, true);
			for (int i = 0; i < size; i++)
			{
				q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(typeof(NeoDatis.Odb.Test.Index.IndexedObject
					), NeoDatis.Odb.Core.Query.Criteria.Where.Equal("name", "olivier" + (i + 1)));
				oos = @base.GetObjects(q, true);
				AssertEquals(0, oos.Count);
			}
			@base.Close();
			DeleteBase(baseName);
			Println("total duration=" + totalTime + " / " + (double)totalTime / size);
			Println("duration max=" + maxTime + " / min=" + minTime);
			if (testPerformance)
			{
				AssertTrue(totalTime / size < 0.9);
				// TODO Try to get maxTime < 10!
				AssertTrue(maxTime < 20);
				AssertTrue(minTime == 0);
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestInsertAndDeleteWithIndex1()
		{
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB @base = Open(baseName);
			// base.store(new IndexedObject());
			NeoDatis.Odb.ClassRepresentation clazz = @base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject
				));
			string[] indexFields = new string[] { "name" };
			clazz.AddUniqueIndexOn("index1", indexFields, true);
			@base.Close();
			@base = Open(baseName);
			int size = 1400;
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Test.Index.IndexedObject io1 = new NeoDatis.Odb.Test.Index.IndexedObject
					("olivier" + (i + 1), 15 + i, new System.DateTime());
				@base.Store(io1);
			}
			@base.Close();
			System.Console.Out.WriteLine("----ola");
			@base = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.Index.IndexedObject));
			NeoDatis.Odb.Objects<NeoDatis.Odb.Test.Index.IndexedObject> objects = @base.GetObjects
				(q);
			while (objects.HasNext())
			{
				NeoDatis.Odb.Test.Index.IndexedObject io = objects.Next();
				Println(io);
				@base.Delete(io);
			}
			@base.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestInsertAndDeleteWithIndexWith10000()
		{
			string baseName = GetBaseName();
			if (!runAll)
			{
				return;
			}
			DeleteBase(baseName);
			NeoDatis.Odb.ODB @base = Open(baseName);
			// base.store(new IndexedObject());
			NeoDatis.Odb.ClassRepresentation clazz = @base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject
				));
			string[] indexFields = new string[] { "name" };
			clazz.AddUniqueIndexOn("index1", indexFields, true);
			@base.Close();
			@base = Open(baseName);
			int size = 10000;
			long start0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Test.Index.IndexedObject io1 = new NeoDatis.Odb.Test.Index.IndexedObject
					("olivier" + (i + 1), 15 + i, new System.DateTime());
				@base.Store(io1);
				if (i % 1000 == 0)
				{
					Println(i);
				}
			}
			long tt0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			@base.Close();
			long tt1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			long end0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			Println("IPU=" + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter
				.GetNbInPlaceUpdates() + " - NU=" + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter
				.GetNbNormalUpdates());
			Println("inserting time with index=" + (end0 - start0));
			Println("commit time=" + (tt1 - tt0));
			Println(NeoDatis.Odb.Impl.Core.Btree.LazyODBBTreePersister.Counters());
			@base = Open(baseName);
			long totalSelectTime = 0;
			long maxTime = 0;
			long minTime = 100000;
			long t0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			long t1 = 0;
			long ta1 = 0;
			long ta2 = 0;
			long totalTimeDelete = 0;
			long totalTimeSelect = 0;
			for (int j = 0; j < size; j++)
			{
				NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
					.Equal("name", "olivier" + (j + 1)));
				long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				NeoDatis.Odb.Objects objects = @base.GetObjects(q, true);
				long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				AssertEquals(1, objects.Count);
				NeoDatis.Odb.Test.Index.IndexedObject io2 = (NeoDatis.Odb.Test.Index.IndexedObject
					)objects.GetFirst();
				AssertEquals("olivier" + (j + 1), io2.GetName());
				AssertEquals(15 + j, io2.GetDuration());
				long d = end - start;
				totalSelectTime += d;
				if (d > maxTime)
				{
					maxTime = d;
				}
				if (d < minTime)
				{
					minTime = d;
				}
				ta1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				@base.Delete(io2);
				ta2 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				totalTimeDelete += (ta2 - ta1);
				totalTimeSelect += (end - start);
				if (j % 100 == 0 && j > 0)
				{
					t1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
					Println(j + " - t= " + (t1 - t0) + " - delete=" + (totalTimeDelete / j) + " / select="
						 + (totalTimeSelect / j));
					Println(NeoDatis.Odb.Impl.Core.Btree.LazyODBBTreePersister.Counters());
					NeoDatis.Odb.Impl.Core.Btree.LazyODBBTreePersister.ResetCounters();
					t0 = t1;
				}
			}
			@base.Close();
			Println("total select=" + totalSelectTime + " / " + (double)totalSelectTime / size
				);
			Println("total delete=" + totalTimeDelete + " / " + (double)totalTimeDelete / size
				);
			Println("duration max=" + maxTime + " / min=" + minTime);
			@base = Open(baseName);
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
					.Equal("name", "olivier" + (i + 1)));
				long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				NeoDatis.Odb.Objects objects = @base.GetObjects(q, true);
				long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				AssertEquals(0, objects.Count);
				if (i % 100 == 0)
				{
					Println(i);
				}
			}
			@base.Close();
			DeleteBase(baseName);
			float timePerObject = (float)totalSelectTime / (float)size;
			Println("Time per object = " + timePerObject);
			if (timePerObject > 1)
			{
				Println("Time per object = " + timePerObject);
			}
			AssertTrue(timePerObject < 0.16);
			// TODO Try to get maxTime < 10!
			AssertTrue(maxTime < 250);
			AssertTrue(minTime < 1);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestInsertAndDeleteWithIndexWith4Elements()
		{
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB @base = Open(baseName);
			// base.store(new IndexedObject());
			NeoDatis.Odb.ClassRepresentation clazz = @base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject
				));
			string[] indexFields = new string[] { "name" };
			clazz.AddUniqueIndexOn("index1", indexFields, true);
			@base.Close();
			@base = Open(baseName);
			int size = 4;
			long start0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Test.Index.IndexedObject io1 = new NeoDatis.Odb.Test.Index.IndexedObject
					("olivier" + (i + 1), 15 + i, new System.DateTime());
				@base.Store(io1);
				if (i % 1000 == 0)
				{
					Println(i);
				}
			}
			long tt0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			@base.Close();
			long tt1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			long end0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			Println("IPU=" + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter
				.GetNbInPlaceUpdates() + " - NU=" + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter
				.GetNbNormalUpdates());
			Println("inserting time with index=" + (end0 - start0));
			Println("commit time=" + (tt1 - tt0));
			Println(NeoDatis.Odb.Impl.Core.Btree.LazyODBBTreePersister.Counters());
			@base = Open(baseName);
			long totalTime = 0;
			long maxTime = 0;
			long minTime = 100000;
			long t0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			long t1 = 0;
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
					.Equal("name", "olivier" + (i + 1)));
				long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				NeoDatis.Odb.Objects objects = @base.GetObjects(q, true);
				long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				AssertEquals(1, objects.Count);
				NeoDatis.Odb.Test.Index.IndexedObject io2 = (NeoDatis.Odb.Test.Index.IndexedObject
					)objects.GetFirst();
				AssertEquals("olivier" + (i + 1), io2.GetName());
				AssertEquals(15 + i, io2.GetDuration());
				long d = end - start;
				totalTime += d;
				if (d > maxTime)
				{
					maxTime = d;
				}
				if (d < minTime)
				{
					minTime = d;
				}
				@base.Delete(io2);
				if (i % 100 == 0)
				{
					t1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
					Println(i + " - t= " + (t1 - t0));
					t0 = t1;
				}
			}
			@base.Close();
			@base = Open(baseName);
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
					.Equal("name", "olivier" + (i + 1)));
				long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				NeoDatis.Odb.Objects objects = @base.GetObjects(q, true);
				long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				AssertEquals(0, objects.Count);
				if (i % 100 == 0)
				{
					Println(i);
				}
			}
			@base.Close();
			DeleteBase(baseName);
			double unitTime = (double)totalTime / size;
			Println("total duration=" + totalTime + " / " + (double)totalTime / size);
			Println("duration max=" + maxTime + " / min=" + minTime);
			if (isLocal)
			{
				AssertTrue(unitTime < 1);
			}
			else
			{
				AssertTrue(unitTime < 6);
			}
			// TODO Try to get maxTime < 10!
			if (testPerformance)
			{
				AssertTrue(maxTime < 250);
				AssertTrue(minTime <= 1);
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestInsertAndDeleteWithIndexWith40Elements()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.OdbConfiguration.SetDefaultIndexBTreeDegree(3);
			DeleteBase(baseName);
			NeoDatis.Odb.ODB @base = Open(baseName);
			// base.store(new IndexedObject());
			NeoDatis.Odb.ClassRepresentation clazz = @base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject
				));
			string[] indexFields = new string[] { "name" };
			clazz.AddUniqueIndexOn("index1", indexFields, true);
			@base.Close();
			@base = Open(baseName);
			int size = 6;
			long start0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Test.Index.IndexedObject io1 = new NeoDatis.Odb.Test.Index.IndexedObject
					("olivier" + (i + 1), 15 + i, new System.DateTime());
				@base.Store(io1);
				if (i % 1000 == 0)
				{
					Println(i);
				}
			}
			long tt0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			@base.Close();
			long tt1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			long end0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			Println("IPU=" + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter
				.GetNbInPlaceUpdates() + " - NU=" + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter
				.GetNbNormalUpdates());
			Println("inserting time with index=" + (end0 - start0));
			Println("commit time=" + (tt1 - tt0));
			Println(NeoDatis.Odb.Impl.Core.Btree.LazyODBBTreePersister.Counters());
			@base = Open(baseName);
			long totalTime = 0;
			long maxTime = 0;
			long minTime = 100000;
			long t0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			long t1 = 0;
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
					.Equal("name", "olivier" + (i + 1)));
				long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				NeoDatis.Odb.Objects objects = @base.GetObjects(q, true);
				long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				AssertEquals(1, objects.Count);
				NeoDatis.Odb.Test.Index.IndexedObject io2 = (NeoDatis.Odb.Test.Index.IndexedObject
					)objects.GetFirst();
				AssertEquals("olivier" + (i + 1), io2.GetName());
				AssertEquals(15 + i, io2.GetDuration());
				long d = end - start;
				totalTime += d;
				if (d > maxTime)
				{
					maxTime = d;
				}
				if (d < minTime)
				{
					minTime = d;
				}
				@base.Delete(io2);
				if (i % 100 == 0)
				{
					t1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
					Println(i + " - t= " + (t1 - t0));
					t0 = t1;
				}
			}
			// println(new BTreeDisplay().build(cii.getBTree(), true));
			@base.Close();
			@base = Open(baseName);
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
					.Equal("name", "olivier" + (i + 1)));
				long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				NeoDatis.Odb.Objects objects = @base.GetObjects(q, true);
				long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				AssertEquals(0, objects.Count);
				if (i % 100 == 0)
				{
					Println(i);
				}
			}
			double unitTime = (double)totalTime / size;
			Println("total duration=" + totalTime + " / " + unitTime);
			Println("duration max=" + maxTime + " / min=" + minTime);
			@base.Close();
			DeleteBase(baseName);
			if (isLocal)
			{
				AssertTrue(unitTime < 1);
			}
			else
			{
				AssertTrue(unitTime < 6);
			}
			// TODO Try to get maxTime < 10!
			if (testPerformance)
			{
				AssertTrue(maxTime < 250);
				AssertTrue(minTime <= 1);
			}
			NeoDatis.Odb.OdbConfiguration.SetDefaultIndexBTreeDegree(20);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestSizeBTree()
		{
			if (!isLocal)
			{
				return;
			}
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB @base = Open(baseName);
			// base.store(new IndexedObject());
			NeoDatis.Odb.ClassRepresentation clazz = @base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject
				));
			string[] indexFields = new string[] { "name" };
			clazz.AddUniqueIndexOn("index1", indexFields, true);
			@base.Close();
			@base = Open(baseName);
			int size = 4;
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Test.Index.IndexedObject io1 = new NeoDatis.Odb.Test.Index.IndexedObject
					("olivier" + (i + 1), 15 + i, new System.DateTime());
				@base.Store(io1);
				if (i % 1000 == 0)
				{
					Println(i);
				}
			}
			@base.Close();
			@base = Open(baseName);
			NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine e = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy
				.GetEngine(@base);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex cii = e.GetSession(true).GetMetaModel
				().GetClassInfo(typeof(NeoDatis.Odb.Test.Index.IndexedObject).FullName, true).GetIndex
				(0);
			@base.Close();
			DeleteBase(baseName);
			AssertEquals(size, cii.GetBTree().GetSize());
		}

		/// <summary>Test index with 3 keys .</summary>
		/// <remarks>
		/// Test index with 3 keys .
		/// Select using only one field to verify that query does not use index, then
		/// execute a query with the 3 fields and checks than index is used
		/// </remarks>
		/// <exception cref="System.Exception"></exception>
		public virtual void TestInsertWith3Keys()
		{
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB @base = Open(baseName);
			// base.store(new IndexedObject());
			NeoDatis.Odb.ClassRepresentation clazz = @base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject
				));
			string[] indexFields = new string[] { "name", "duration", "creation" };
			clazz.AddUniqueIndexOn("index", indexFields, true);
			@base.Close();
			@base = Open(baseName);
			int size = isLocal ? 50000 : 500;
			int commitInterval = isLocal ? 10000 : 100;
			long start0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Test.Index.IndexedObject io2 = new NeoDatis.Odb.Test.Index.IndexedObject
					("olivier" + (i + 1), i + 15 + size, new System.DateTime());
				@base.Store(io2);
				if (i % commitInterval == 0)
				{
					long t0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
					@base.Commit();
					long t1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
					Println(i + " : commit - ctime " + (t1 - t0) + " -ttime=");
					// println(LazyODBBTreePersister.counters());
					NeoDatis.Odb.Impl.Core.Btree.LazyODBBTreePersister.ResetCounters();
				}
			}
			System.DateTime theDate = new System.DateTime();
			string theName = "name indexed";
			int theDuration = 45;
			NeoDatis.Odb.Test.Index.IndexedObject io1 = new NeoDatis.Odb.Test.Index.IndexedObject
				(theName, theDuration, theDate);
			@base.Store(io1);
			@base.Close();
			@base = Open(baseName);
			// first search without index
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("name", theName));
			NeoDatis.Odb.Objects objects = @base.GetObjects(q, true);
			AssertFalse(q.GetExecutionPlan().UseIndex());
			Println(q.GetExecutionPlan().GetDetails());
			AssertEquals(1, objects.Count);
			NeoDatis.Odb.Test.Index.IndexedObject io3 = (NeoDatis.Odb.Test.Index.IndexedObject
				)objects.GetFirst();
			AssertEquals(theName, io3.GetName());
			AssertEquals(theDuration, io3.GetDuration());
			AssertEquals(theDate, io3.GetCreation());
			@base.Close();
			@base = Open(baseName);
			// Then search usin index
			q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(typeof(NeoDatis.Odb.Test.Index.IndexedObject
				), NeoDatis.Odb.Core.Query.Criteria.Where.And().Add(NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("name", theName)).Add(NeoDatis.Odb.Core.Query.Criteria.Where.Equal("creation"
				, theDate)).Add(NeoDatis.Odb.Core.Query.Criteria.Where.Equal("duration", theDuration
				)));
			objects = @base.GetObjects(q, true);
			AssertTrue(q.GetExecutionPlan().UseIndex());
			if (isLocal)
			{
				AssertEquals("index", q.GetExecutionPlan().GetIndex().GetName());
			}
			Println(q.GetExecutionPlan().GetDetails());
			AssertEquals(1, objects.Count);
			io3 = (NeoDatis.Odb.Test.Index.IndexedObject)objects.GetFirst();
			AssertEquals(theName, io3.GetName());
			AssertEquals(theDuration, io3.GetDuration());
			AssertEquals(theDate, io3.GetCreation());
			@base.Close();
		}

		/// <summary>Test index.</summary>
		/// <remarks>
		/// Test index. Creates 1000 objects. Take 10 objects to update 10000 times.
		/// Then check if all objects are ok
		/// </remarks>
		/// <exception cref="System.Exception"></exception>
		public virtual void TestXUpdatesWithIndex()
		{
			string baseName = GetBaseName();
			try
			{
				DeleteBase(baseName);
				NeoDatis.Odb.ODB @base = Open(baseName);
				// base.store(new IndexedObject());
				NeoDatis.Odb.ClassRepresentation clazz = @base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject
					));
				string[] indexFields = new string[] { "name" };
				clazz.AddUniqueIndexOn("index", indexFields, true);
				@base.Close();
				@base = Open(baseName);
				long start = Sharpen.Runtime.CurrentTimeMillis();
				int size = 1000;
				int nbObjects = 10;
				int nbUpdates = isLocal ? 100 : 50;
				for (int i = 0; i < size; i++)
				{
					NeoDatis.Odb.Test.Index.IndexedObject io1 = new NeoDatis.Odb.Test.Index.IndexedObject
						("IO-" + i + "-0", i + 15 + size, new System.DateTime());
					@base.Store(io1);
				}
				@base.Close();
				Println("Time of insert " + size + " objects = " + size);
				string[] indexes = new string[] { "IO-0-0", "IO-100-0", "IO-200-0", "IO-300-0", "IO-400-0"
					, "IO-500-0", "IO-600-0", "IO-700-0", "IO-800-0", "IO-900-0" };
				long t1 = 0;
				long t2 = 0;
				long t3 = 0;
				long t4 = 0;
				long t5 = 0;
				long t6 = 0;
				for (int i = 0; i < nbUpdates; i++)
				{
					start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
					for (int j = 0; j < nbObjects; j++)
					{
						t1 = Sharpen.Runtime.CurrentTimeMillis();
						@base = Open(baseName);
						t2 = Sharpen.Runtime.CurrentTimeMillis();
						NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
							(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
							.Equal("name", indexes[j]));
						NeoDatis.Odb.Objects os = @base.GetObjects(q);
						t3 = Sharpen.Runtime.CurrentTimeMillis();
						AssertTrue(q.GetExecutionPlan().UseIndex());
						AssertEquals(1, os.Count);
						// check if index has been used
						AssertTrue(q.GetExecutionPlan().UseIndex());
						NeoDatis.Odb.Test.Index.IndexedObject io = (NeoDatis.Odb.Test.Index.IndexedObject
							)os.GetFirst();
						if (i > 0)
						{
							AssertTrue(io.GetName().EndsWith(("-" + (i - 1))));
						}
						io.SetName(io.GetName() + "-updated-" + i);
						@base.Store(io);
						t4 = Sharpen.Runtime.CurrentTimeMillis();
						if (isLocal && j == 0)
						{
							NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy
								.GetEngine(@base);
							NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = engine.GetSession(true).GetMetaModel
								().GetClassInfo(typeof(NeoDatis.Odb.Test.Index.IndexedObject).FullName, true);
							NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex cii = ci.GetIndex(0);
							AssertEquals(size, cii.GetBTree().GetSize());
						}
						indexes[j] = io.GetName();
						AssertEquals(new System.Decimal(string.Empty + size), @base.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
							(typeof(NeoDatis.Odb.Test.Index.IndexedObject))));
						t5 = Sharpen.Runtime.CurrentTimeMillis();
						@base.Commit();
						@base.Close();
						t6 = Sharpen.Runtime.CurrentTimeMillis();
					}
					long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
					System.Console.Out.WriteLine("Nb Updates of " + nbObjects + " =" + i + " - " + (end
						 - start) + "ms  -- open=" + (t2 - t1) + " - getObjects=" + (t3 - t2) + " - update="
						 + (t4 - t3) + " - count=" + (t5 - t4) + " - close=" + (t6 - t5));
				}
			}
			finally
			{
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void SimpleUniqueIndex()
		{
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB odb = Open(baseName);
			NeoDatis.Odb.ClassRepresentation clazz = odb.GetClassRepresentation(typeof(NeoDatis.Odb.Test.VO.Login.Function
				));
			string[] indexFields = new string[] { "name" };
			clazz.AddUniqueIndexOn("index", indexFields, true);
			odb.Close();
			odb = Open(baseName);
			// inserting 3 objects with 3 different index keys
			odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("function1"));
			odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("function2"));
			odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("function3"));
			odb.Close();
			odb = Open(baseName);
			try
			{
				// Tries to store another function with name function1 => send an
				// exception because of duplicated keys
				odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("function1"));
				Fail("Should have thrown Exception");
			}
			catch (System.Exception e)
			{
				Sharpen.Runtime.PrintStackTrace(e);
			}
		}

		public virtual void TestIndexExist1()
		{
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB odb = Open(baseName);
			NeoDatis.Odb.ClassRepresentation clazz = odb.GetClassRepresentation(typeof(NeoDatis.Odb.Test.VO.Login.Function
				));
			string[] indexFields = new string[] { "name" };
			clazz.AddUniqueIndexOn("my-index", indexFields, true);
			odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("test"));
			odb.Close();
			odb = Open(baseName);
			AssertTrue(odb.GetClassRepresentation(typeof(NeoDatis.Odb.Test.VO.Login.Function)
				).ExistIndex("my-index"));
			AssertFalse(odb.GetClassRepresentation(typeof(NeoDatis.Odb.Test.VO.Login.Function
				)).ExistIndex("my-indexdhfdjkfhdjkhj"));
			odb.Close();
		}

		public virtual void TestIndexExist2()
		{
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB odb = Open(baseName);
			NeoDatis.Odb.ClassRepresentation clazz = odb.GetClassRepresentation(typeof(NeoDatis.Odb.Test.VO.Login.Function
				));
			string[] indexFields = new string[] { "name" };
			clazz.AddUniqueIndexOn("my-index", indexFields, true);
			odb.Close();
			odb = Open(baseName);
			AssertTrue(odb.GetClassRepresentation(typeof(NeoDatis.Odb.Test.VO.Login.Function)
				).ExistIndex("my-index"));
			AssertFalse(odb.GetClassRepresentation(typeof(NeoDatis.Odb.Test.VO.Login.Function
				)).ExistIndex("my-indexdhfdjkfhdjkhj"));
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public static void Main2(string[] args)
		{
			NeoDatis.Odb.Test.Index.TestIndex ti = new NeoDatis.Odb.Test.Index.TestIndex();
			ti.TestInsertWithIndex3Part2();
		}
	}
}

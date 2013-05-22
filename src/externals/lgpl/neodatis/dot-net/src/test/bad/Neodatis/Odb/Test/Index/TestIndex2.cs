namespace NeoDatis.Odb.Test.Index
{
	public class TestIndex2 : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public virtual void TestIndexFail()
		{
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB @base = Open(baseName);
			string indexName = "index1";
			NeoDatis.Odb.ClassRepresentation clazz = @base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject3
				));
			string[] indexFields1 = new string[] { "i1", "i2", "i3" };
			clazz.AddUniqueIndexOn(indexName, indexFields1, true);
			@base.Close();
			@base = Open(baseName);
			NeoDatis.Odb.Test.Index.IndexedObject3 io = new NeoDatis.Odb.Test.Index.IndexedObject3
				(1, 2, 3, "1", "2", "3", new System.DateTime(), new System.DateTime(), new System.DateTime
				());
			@base.Store(io);
			try
			{
				NeoDatis.Odb.Test.Index.IndexedObject3 io2 = new NeoDatis.Odb.Test.Index.IndexedObject3
					(1, 2, 3, "1", "2", "3", new System.DateTime(), new System.DateTime(), new System.DateTime
					());
				@base.Store(io2);
			}
			catch (System.Exception e)
			{
				AssertTrue(e.Message.IndexOf(indexName) != -1);
			}
			// println(e.getMessage());
			@base.Close();
			@base = Open(baseName);
			NeoDatis.Odb.Objects<NeoDatis.Odb.Test.Index.IndexedObject3> oo3 = @base.GetObjects
				(typeof(NeoDatis.Odb.Test.Index.IndexedObject3));
			@base.Close();
			AssertEquals(0, oo3.Count);
			DeleteBase(baseName);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestSaveIndex()
		{
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB @base = Open(baseName);
			NeoDatis.Odb.ClassRepresentation clazz = @base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject3
				));
			string[] indexFields1 = new string[] { "i1", "i2", "i3" };
			clazz.AddUniqueIndexOn("index1", indexFields1, true);
			string[] indexFields2 = new string[] { "s1", "s2", "s3" };
			clazz.AddUniqueIndexOn("index2", indexFields2, true);
			string[] indexFields3 = new string[] { "dt1", "dt2", "dt3" };
			clazz.AddUniqueIndexOn("index3", indexFields3, true);
			string[] indexFields4 = new string[] { "i1", "i2", "i3", "s1", "s2", "s3", "dt1", 
				"dt2", "dt3" };
			clazz.AddUniqueIndexOn("index4", indexFields4, true);
			@base.Close();
			@base = Open(baseName);
			NeoDatis.Odb.Core.Transaction.ISession session = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy
				.GetEngine(@base).GetSession(true);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel = session.GetStorageEngine
				().GetSession(true).GetMetaModel();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = metaModel.GetClassInfo(typeof(
				NeoDatis.Odb.Test.Index.IndexedObject3).FullName, true);
			AssertEquals(4, ci.GetNumberOfIndexes());
			AssertEquals(ci.GetIndex(0).GetName(), "index1");
			AssertEquals(3, ci.GetIndex(0).GetAttributeIds().Length);
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex.Enabled, ci.GetIndex
				(0).GetStatus());
			AssertEquals(ci.GetIndex(1).GetName(), "index2");
			AssertEquals(3, ci.GetIndex(1).GetAttributeIds().Length);
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex.Enabled, ci.GetIndex
				(1).GetStatus());
			AssertEquals(ci.GetIndex(2).GetName(), "index3");
			AssertEquals(3, ci.GetIndex(2).GetAttributeIds().Length);
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex.Enabled, ci.GetIndex
				(2).GetStatus());
			AssertEquals(ci.GetIndex(3).GetName(), "index4");
			AssertEquals(9, ci.GetIndex(3).GetAttributeIds().Length);
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex.Enabled, ci.GetIndex
				(3).GetStatus());
			@base.Close();
			@base = Open(baseName);
			for (int i = 0; i < 10; i++)
			{
				NeoDatis.Odb.Test.Index.IndexedObject3 io = new NeoDatis.Odb.Test.Index.IndexedObject3
					(1 + i, 2, 3, "1" + i, "2", "3", new System.DateTime(2009, i, 1), new System.DateTime
					(), new System.DateTime());
				@base.Store(io);
			}
			@base.Close();
			DeleteBase(baseName);
		}

		/// <summary>Test index creation without commit</summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void TestCreateIndexWithoutCommit()
		{
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB @base = Open(baseName);
			NeoDatis.Odb.ClassRepresentation clazz = @base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject3
				));
			string[] indexFields1 = new string[] { "i1" };
			clazz.AddUniqueIndexOn("index1", indexFields1, true);
			for (int i = 0; i < 10; i++)
			{
				NeoDatis.Odb.Test.Index.IndexedObject3 io = new NeoDatis.Odb.Test.Index.IndexedObject3
					(1 + i, 2, 3, "1" + i, "2", "3", new System.DateTime(2009, i, 1), new System.DateTime
					(), new System.DateTime());
				@base.Store(io);
			}
			@base.Close();
			@base = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.Index.IndexedObject3), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("i1", 1));
			NeoDatis.Odb.Objects<NeoDatis.Odb.Test.Index.IndexedObject3> iis = @base.GetObjects
				(q);
			@base.Close();
			AssertEquals(1, iis.Count);
			AssertTrue(q.GetExecutionPlan().UseIndex());
			DeleteBase(baseName);
		}

		/// <summary>
		/// Opens a connection C1, then create the index in another connection C2 and
		/// then stores the object in connection C1
		/// </summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void TestCreateIndexInOtherConnection()
		{
			if (isLocal || !testNewFeature)
			{
				return;
			}
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB odb1 = Open(baseName);
			NeoDatis.Odb.ODB odb2 = Open(baseName);
			NeoDatis.Odb.ClassRepresentation clazz = odb2.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject3
				));
			string[] indexFields1 = new string[] { "i1" };
			clazz.AddUniqueIndexOn("index1", indexFields1, true);
			odb2.Close();
			for (int i = 0; i < 10; i++)
			{
				NeoDatis.Odb.Test.Index.IndexedObject3 io = new NeoDatis.Odb.Test.Index.IndexedObject3
					(1 + i, 2, 3, "1" + i, "2", "3", new System.DateTime(2009, i, 1), new System.DateTime
					(), new System.DateTime());
				odb1.Store(io);
			}
			odb1.Close();
			NeoDatis.Odb.ODB odb = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.Index.IndexedObject3), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("i1", 1));
			NeoDatis.Odb.Objects<NeoDatis.Odb.Test.Index.IndexedObject3> iis = odb.GetObjects
				(q);
			odb.Close();
			AssertEquals(1, iis.Count);
			AssertTrue(q.GetExecutionPlan().UseIndex());
			DeleteBase(baseName);
		}

		/// <summary>
		/// Opens a connection C1, then create the index in another connection C2 and
		/// then stores the object in connection C1
		/// </summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void TestCreateIndexInOtherConnectionNoClose()
		{
			if (isLocal || !testNewFeature)
			{
				return;
			}
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB odb1 = Open(baseName);
			NeoDatis.Odb.ODB odb2 = Open(baseName);
			NeoDatis.Odb.ClassRepresentation clazz = odb2.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject3
				));
			string[] indexFields1 = new string[] { "i1" };
			clazz.AddUniqueIndexOn("index1", indexFields1, true);
			odb2.Commit();
			for (int i = 0; i < 10; i++)
			{
				NeoDatis.Odb.Test.Index.IndexedObject3 io = new NeoDatis.Odb.Test.Index.IndexedObject3
					(1 + i, 2, 3, "1" + i, "2", "3", new System.DateTime(2009, i, 1), new System.DateTime
					(), new System.DateTime());
				odb1.Store(io);
			}
			odb1.Close();
			NeoDatis.Odb.ODB odb = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.Index.IndexedObject3), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("i1", 1));
			NeoDatis.Odb.Objects<NeoDatis.Odb.Test.Index.IndexedObject3> iis = odb.GetObjects
				(q);
			odb.Close();
			odb2.Close();
			AssertEquals(1, iis.Count);
			AssertTrue(q.GetExecutionPlan().UseIndex());
			DeleteBase(baseName);
		}

		/// <summary>
		/// Opens a connection C1, then create the index in another connection C2 and
		/// then stores the object in connection C1
		/// </summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void TestCreateIndexInOtherConnectionNoCommit1()
		{
			if (isLocal || !testNewFeature)
			{
				return;
			}
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB odb1 = Open(baseName);
			NeoDatis.Odb.ODB odb2 = Open(baseName);
			NeoDatis.Odb.ClassRepresentation clazz = odb2.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject3
				));
			string[] indexFields1 = new string[] { "i1" };
			clazz.AddUniqueIndexOn("index1", indexFields1, true);
			for (int i = 0; i < 10; i++)
			{
				NeoDatis.Odb.Test.Index.IndexedObject3 io = new NeoDatis.Odb.Test.Index.IndexedObject3
					(1 + i, 2, 3, "1" + i, "2", "3", new System.DateTime(2009, i, 1), new System.DateTime
					(), new System.DateTime());
				odb1.Store(io);
			}
			odb2.Close();
			odb1.Close();
			NeoDatis.Odb.ODB odb = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.Index.IndexedObject3), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("i1", 1));
			NeoDatis.Odb.Objects<NeoDatis.Odb.Test.Index.IndexedObject3> iis = odb.GetObjects
				(q);
			odb.Close();
			AssertEquals(1, iis.Count);
			AssertTrue(q.GetExecutionPlan().UseIndex());
			DeleteBase(baseName);
		}

		/// <summary>
		/// Opens a connection C1, then create the index in another connection C2 and
		/// then stores the object in connection C1
		/// </summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void TestCreateIndexInOtherConnectionNoCommit2()
		{
			if (isLocal || !testNewFeature)
			{
				return;
			}
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB odb1 = Open(baseName);
			NeoDatis.Odb.ODB odb2 = Open(baseName);
			NeoDatis.Odb.ClassRepresentation clazz = odb2.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject3
				));
			string[] indexFields1 = new string[] { "i1" };
			clazz.AddUniqueIndexOn("index1", indexFields1, true);
			for (int i = 0; i < 10; i++)
			{
				NeoDatis.Odb.Test.Index.IndexedObject3 io = new NeoDatis.Odb.Test.Index.IndexedObject3
					(1 + i, 2, 3, "1" + i, "2", "3", new System.DateTime(2009, i, 1), new System.DateTime
					(), new System.DateTime());
				odb1.Store(io);
			}
			odb1.Close();
			odb2.Close();
			NeoDatis.Odb.ODB odb = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.Index.IndexedObject3), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("i1", 1));
			NeoDatis.Odb.Objects<NeoDatis.Odb.Test.Index.IndexedObject3> iis = odb.GetObjects
				(q);
			odb.Close();
			AssertEquals(1, iis.Count);
			AssertTrue(q.GetExecutionPlan().UseIndex());
			DeleteBase(baseName);
		}

		/// <summary>
		/// Create objects, then create index, then execute a select with index, then
		/// rebuild index e execute
		/// </summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void TestRebuildIndex()
		{
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB @base = Open(baseName);
			for (int i = 0; i < 2500; i++)
			{
				NeoDatis.Odb.Test.Index.IndexedObject3 io = new NeoDatis.Odb.Test.Index.IndexedObject3
					(1 + i, 2, 3, "1" + i, "2", "3", new System.DateTime(2009, i, 1), new System.DateTime
					(), new System.DateTime());
				@base.Store(io);
			}
			@base.Close();
			@base = Open(baseName);
			NeoDatis.Odb.ClassRepresentation clazz = @base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject3
				));
			string[] indexFields1 = new string[] { "i1", "i2", "i3" };
			clazz.AddUniqueIndexOn("index1", indexFields1, true);
			@base.Close();
			@base = Open(baseName);
			NeoDatis.Odb.Core.Transaction.ISession session = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy
				.GetEngine(@base).GetSession(true);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel = session.GetStorageEngine
				().GetSession(true).GetMetaModel();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = metaModel.GetClassInfo(typeof(
				NeoDatis.Odb.Test.Index.IndexedObject3).FullName, true);
			AssertEquals(1, ci.GetNumberOfIndexes());
			AssertEquals(ci.GetIndex(0).GetName(), "index1");
			AssertEquals(3, ci.GetIndex(0).GetAttributeIds().Length);
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex.Enabled, ci.GetIndex
				(0).GetStatus());
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.Index.IndexedObject3), NeoDatis.Odb.Core.Query.Criteria.Where
				.And().Add(NeoDatis.Odb.Core.Query.Criteria.Where.Equal("i1", 10)).Add(NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("i2", 2)).Add(NeoDatis.Odb.Core.Query.Criteria.Where.Equal("i3", 3)));
			NeoDatis.Odb.Objects<NeoDatis.Odb.Test.Index.IndexedObject3> objects = @base.GetObjects
				(q);
			AssertEquals(true, q.GetExecutionPlan().UseIndex());
			@base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject3)).RebuildIndex
				("index1", true);
			@base.Close();
			@base = Open(baseName);
			objects = @base.GetObjects(q);
			AssertEquals(true, q.GetExecutionPlan().UseIndex());
			@base.Close();
			DeleteBase(baseName);
		}

		/// <summary>
		/// Create objects, then create index, then execute a select with index, then
		/// rebuild index e execute
		/// </summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void TestDeleteIndex()
		{
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB @base = Open(baseName);
			for (int i = 0; i < 2500; i++)
			{
				NeoDatis.Odb.Test.Index.IndexedObject3 io = new NeoDatis.Odb.Test.Index.IndexedObject3
					(1 + i, 2, 3, "1" + i, "2", "3", new System.DateTime(2009, i, 1), new System.DateTime
					(), new System.DateTime());
				@base.Store(io);
			}
			@base.Close();
			@base = Open(baseName);
			NeoDatis.Odb.ClassRepresentation clazz = @base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject3
				));
			string[] indexFields1 = new string[] { "i1", "i2", "i3" };
			clazz.AddUniqueIndexOn("index1", indexFields1, true);
			@base.Close();
			@base = Open(baseName);
			NeoDatis.Odb.Core.Transaction.ISession session = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy
				.GetEngine(@base).GetSession(true);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel = session.GetStorageEngine
				().GetSession(true).GetMetaModel();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = metaModel.GetClassInfo(typeof(
				NeoDatis.Odb.Test.Index.IndexedObject3).FullName, true);
			AssertEquals(1, ci.GetNumberOfIndexes());
			AssertEquals(ci.GetIndex(0).GetName(), "index1");
			AssertEquals(3, ci.GetIndex(0).GetAttributeIds().Length);
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex.Enabled, ci.GetIndex
				(0).GetStatus());
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.Index.IndexedObject3), NeoDatis.Odb.Core.Query.Criteria.Where
				.And().Add(NeoDatis.Odb.Core.Query.Criteria.Where.Equal("i1", 10)).Add(NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("i2", 2)).Add(NeoDatis.Odb.Core.Query.Criteria.Where.Equal("i3", 3)));
			NeoDatis.Odb.Objects<NeoDatis.Odb.Test.Index.IndexedObject3> objects = @base.GetObjects
				(q);
			AssertEquals(true, q.GetExecutionPlan().UseIndex());
			@base.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject3)).DeleteIndex
				("index1", true);
			@base.Close();
			@base = Open(baseName);
			objects = @base.GetObjects(q);
			AssertEquals(false, q.GetExecutionPlan().UseIndex());
			@base.Close();
			DeleteBase(baseName);
		}
	}
}

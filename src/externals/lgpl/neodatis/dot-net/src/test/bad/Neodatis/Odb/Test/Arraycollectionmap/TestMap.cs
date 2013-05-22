namespace NeoDatis.Odb.Test.Arraycollectionmap
{
	public class TestMap : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public override void SetUp()
		{
			DeleteBase("map.neodatis");
			NeoDatis.Odb.ODB odb = Open("map.neodatis");
			NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary dictionnary1 = new NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary
				("test1");
			dictionnary1.AddEntry("olivier", "Smadja");
			dictionnary1.AddEntry("kiko", "vidal");
			dictionnary1.AddEntry("karine", "galvao");
			NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary dictionnary2 = new NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary
				("test2");
			dictionnary2.AddEntry("f1", new NeoDatis.Odb.Test.VO.Login.Function("function1"));
			dictionnary2.AddEntry("f2", new NeoDatis.Odb.Test.VO.Login.Function("function2"));
			dictionnary2.AddEntry("f3", new NeoDatis.Odb.Test.VO.Login.Function("function3"));
			dictionnary2.AddEntry(dictionnary1, new NeoDatis.Odb.Test.VO.Login.Function("function4"
				));
			dictionnary2.AddEntry(null, new NeoDatis.Odb.Test.VO.Login.Function("function3"));
			dictionnary2.AddEntry(null, null);
			dictionnary2.AddEntry("f4", null);
			odb.Store(dictionnary1);
			odb.Store(dictionnary2);
			odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("login"));
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			NeoDatis.Odb.ODB odb = Open("map.neodatis");
			NeoDatis.Odb.Objects l = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary
				), true);
			// assertEquals(2,l.size());
			NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary dictionnary = (NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary
				)l.GetFirst();
			AssertEquals("Smadja", dictionnary.Get("olivier"));
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test2()
		{
			NeoDatis.Odb.ODB odb = Open("map.neodatis");
			NeoDatis.Odb.Objects l = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary
				));
			NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery aq = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("name", "test2"));
			l = odb.GetObjects(aq);
			NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary dictionnary = (NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary
				)l.GetFirst();
			AssertEquals(new NeoDatis.Odb.Test.VO.Login.Function("function2").GetName(), ((NeoDatis.Odb.Test.VO.Login.Function
				)dictionnary.Get("f2")).GetName());
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test3()
		{
			NeoDatis.Odb.ODB odb = Open("map.neodatis");
			long size = odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(typeof(
				NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary)));
			NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary dictionnary1 = new NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary
				("test1");
			dictionnary1.SetMap(null);
			odb.Store(dictionnary1);
			odb.Close();
			odb = Open("map.neodatis");
			AssertEquals(size + 1, odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary
				)).Count);
			AssertEquals(size + 1, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary))));
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test4()
		{
			NeoDatis.Odb.ODB odb = Open("map.neodatis");
			long n = odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(typeof(
				NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary)));
			NeoDatis.Odb.Core.Query.IQuery query = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("name", "test2"));
			NeoDatis.Odb.Objects l = odb.GetObjects(query);
			NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary dictionnary = (NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary
				)l.GetFirst();
			dictionnary.SetMap(null);
			odb.Store(dictionnary);
			odb.Close();
			odb = Open("map.neodatis");
			AssertEquals(n, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary))));
			NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary dic = (NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary
				)odb.GetObjects(query).GetFirst();
			AssertEquals(null, dic.GetMap());
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test5updateIncreasingSize()
		{
			NeoDatis.Odb.ODB odb = Open("map.neodatis");
			long n = odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(typeof(
				NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary)));
			NeoDatis.Odb.Core.Query.IQuery query = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("name", "test2"));
			NeoDatis.Odb.Objects l = odb.GetObjects(query);
			NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary dictionnary = (NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary
				)l.GetFirst();
			dictionnary.SetMap(null);
			odb.Store(dictionnary);
			odb.Close();
			odb = Open("map.neodatis");
			AssertEquals(n, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary))));
			NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary dic = (NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary
				)odb.GetObjects(query).GetFirst();
			AssertNull(dic.GetMap());
			odb.Close();
			odb = Open("map.neodatis");
			dic = (NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary)odb.GetObjects(query).
				GetFirst();
			dic.AddEntry("olivier", "Smadja");
			odb.Store(dic);
			odb.Close();
			odb = Open("map.neodatis");
			dic = (NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary)odb.GetObjects(query).
				GetFirst();
			AssertNotNull(dic.GetMap());
			AssertEquals("Smadja", dic.GetMap()["olivier"]);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test6updateDecreasingSize()
		{
			NeoDatis.Odb.ODB odb = Open("map.neodatis");
			long n = odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(typeof(
				NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary)));
			NeoDatis.Odb.Core.Query.IQuery query = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("name", "test2"));
			NeoDatis.Odb.Objects l = odb.GetObjects(query);
			NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary dictionnary = (NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary
				)l.GetFirst();
			int mapSize = dictionnary.GetMap().Count;
			dictionnary.GetMap().Remove("f1");
			odb.Store(dictionnary);
			odb.Close();
			odb = Open("map.neodatis");
			AssertEquals(n, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary))));
			NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary dic = (NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary
				)odb.GetObjects(query).GetFirst();
			AssertEquals(mapSize - 1, dic.GetMap().Count);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test6updateChangingKeyValue()
		{
			// to monitor in place updates
			NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter.ResetNbUpdates();
			NeoDatis.Odb.ODB odb = Open("map.neodatis");
			long n = odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(typeof(
				NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary)));
			NeoDatis.Odb.Core.Query.IQuery query = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("name", "test2"));
			NeoDatis.Odb.Objects l = odb.GetObjects(query);
			NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary dictionnary = (NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary
				)l.GetFirst();
			dictionnary.GetMap().Add("f1", "changed function");
			odb.Store(dictionnary);
			odb.Close();
			odb = Open("map.neodatis");
			AssertEquals(n, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary))));
			NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary dic = (NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary
				)odb.GetObjects(query).GetFirst();
			AssertEquals("changed function", dic.GetMap()["f1"]);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public override void TearDown()
		{
			DeleteBase("map.neodatis");
		}
	}
}

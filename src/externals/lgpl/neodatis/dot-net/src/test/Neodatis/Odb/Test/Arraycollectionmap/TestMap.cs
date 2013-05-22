using NeoDatis.Odb.Test.VO.Arraycollectionmap;
using NeoDatis.Odb.Impl.Core.Query.Criteria;
using NUnit.Framework;
using NeoDatis.Odb.Core.Query.Criteria;
using NeoDatis.Odb.Test.VO.Login;
using Test.Neodatis.Odb.Test.Arraycollectionmap;
using NeoDatis.Odb.Core.Query;
namespace NeoDatis.Odb.Test.Arraycollectionmap
{
	[TestFixture]
    public class TestMap : NeoDatis.Odb.Test.ODBTest
	{
		[SetUp]
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
			
			dictionnary2.AddEntry("f4", null);
			odb.Store(dictionnary1);
			odb.Store(dictionnary2);
			odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("login"));
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test1()
		{
			NeoDatis.Odb.ODB odb = Open("map.neodatis");
			NeoDatis.Odb.Objects<Dictionnary> l = odb.GetObjects<Dictionnary>(true);
			// assertEquals(2,l.size());
			Dictionnary dictionnary = l.GetFirst();
			AssertEquals("Smadja", dictionnary.Get("olivier"));
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test2()
		{
			NeoDatis.Odb.ODB odb = Open("map.neodatis");
			NeoDatis.Odb.Objects<Dictionnary> l = odb.GetObjects<Dictionnary>();
			CriteriaQuery aq = new CriteriaQuery(typeof(Dictionnary), Where.Equal("name", "test2"));
			l = odb.GetObjects<Dictionnary>(aq);
			Dictionnary dictionnary = l.GetFirst();
			AssertEquals("function2", ((Function)dictionnary.Get("f2")).GetName());
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test3()
		{
			NeoDatis.Odb.ODB odb = Open("map.neodatis");
			long size = odb.Count(new CriteriaQuery(typeof(Dictionnary)));
			Dictionnary dictionnary1 = new Dictionnary("test1");
			dictionnary1.SetMap(null);
			odb.Store(dictionnary1);
			odb.Close();
			odb = Open("map.neodatis");
			AssertEquals(size + 1, odb.GetObjects<Dictionnary>().Count);
			AssertEquals(size + 1, odb.Count(new CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary))));
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test4()
		{
			NeoDatis.Odb.ODB odb = Open("map.neodatis");
			long n = odb.Count(new CriteriaQuery(typeof(
				NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary)));
			NeoDatis.Odb.Core.Query.IQuery query = new CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary), Where
				.Equal("name", "test2"));
			NeoDatis.Odb.Objects<Dictionnary> l = odb.GetObjects<Dictionnary>(query);
			NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary dictionnary = (NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary
				)l.GetFirst();
			dictionnary.SetMap(null);
			odb.Store(dictionnary);
			odb.Close();
			odb = Open("map.neodatis");
			AssertEquals(n, odb.Count(new CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary))));
			NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary dic = (NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary
				)odb.GetObjects<Dictionnary>(query).GetFirst();
			AssertEquals(null, dic.GetMap());
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test5updateIncreasingSize()
		{
			NeoDatis.Odb.ODB odb = Open("map.neodatis");
			long n = odb.Count(new CriteriaQuery(typeof(
				NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary)));
			NeoDatis.Odb.Core.Query.IQuery query = new CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary), Where
				.Equal("name", "test2"));
			NeoDatis.Odb.Objects<Dictionnary> l = odb.GetObjects<Dictionnary>(query);
			NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary dictionnary = (NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary
				)l.GetFirst();
			dictionnary.SetMap(null);
			odb.Store(dictionnary);
			odb.Close();
			odb = Open("map.neodatis");
			AssertEquals(n, odb.Count(new CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary))));
			NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary dic = (NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary
				)odb.GetObjects<Dictionnary>(query).GetFirst();
			AssertNull(dic.GetMap());
			odb.Close();
			odb = Open("map.neodatis");
			dic = (NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary)odb.GetObjects<Dictionnary>(query).GetFirst();
			dic.AddEntry("olivier", "Smadja");
			odb.Store(dic);
			odb.Close();
			odb = Open("map.neodatis");
			dic = (NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary)odb.GetObjects<Dictionnary>(query).
				GetFirst();
			AssertNotNull(dic.GetMap());
			AssertEquals("Smadja", dic.GetMap()["olivier"]);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test6updateDecreasingSize()
		{
			NeoDatis.Odb.ODB odb = Open("map.neodatis");
			long n = odb.Count(new CriteriaQuery(typeof(
				NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary)));
			NeoDatis.Odb.Core.Query.IQuery query = new CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary), Where
				.Equal("name", "test2"));
			NeoDatis.Odb.Objects<Dictionnary> l = odb.GetObjects<Dictionnary>(query);
			NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary dictionnary = (NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary
				)l.GetFirst();
			int mapSize = dictionnary.GetMap().Count;
			dictionnary.GetMap().Remove("f1");
			odb.Store(dictionnary);
			odb.Close();
			odb = Open("map.neodatis");
			AssertEquals(n, odb.Count(new CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary))));
			NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary dic = (NeoDatis.Odb.Test.VO.Arraycollectionmap.Dictionnary
				)odb.GetObjects<Dictionnary>(query).GetFirst();
			AssertEquals(mapSize - 1, dic.GetMap().Count);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test6updateChangingKeyValue()
		{
			// to monitor in place updates
			NeoDatis.Odb.ODB odb = Open("map.neodatis");
			long n = odb.Count(new CriteriaQuery(typeof(Dictionnary)));
			IQuery query = new CriteriaQuery(typeof(Dictionnary), Where.Equal("name", "test2"));
			NeoDatis.Odb.Objects<Dictionnary> l = odb.GetObjects<Dictionnary>(query);
			Dictionnary dictionnary = (Dictionnary)l.GetFirst();
			dictionnary.GetMap()["f1"] = "changed function";
			odb.Store(dictionnary);
			odb.Close();
			odb = Open("map.neodatis");
			AssertEquals(n, odb.Count(new CriteriaQuery(typeof(Dictionnary))));
			Dictionnary dic = (Dictionnary)odb.GetObjects<Dictionnary>(query).GetFirst();
			AssertEquals("changed function", dic.GetMap()["f1"]);
			odb.Close();
		}

        [Test]
        public virtual void TestNonGenericMap()
        {
            NeoDatis.Odb.ODB odb = Open("map.neodatis");
            ClassWithNonGenericMap cm = new ClassWithNonGenericMap("test1");
            cm.Add("key1", "value1");
            cm.Add("key2", "value2");
            odb.Store(cm);
            odb.Close();
            odb = Open("map.neodatis");
            AssertEquals(1, odb.Count(new CriteriaQuery (typeof(ClassWithNonGenericMap))));
            ClassWithNonGenericMap cm2 = odb.GetObjects<ClassWithNonGenericMap>(new CriteriaQuery(typeof(ClassWithNonGenericMap),Where.Equal("name","test1"))).GetFirst();
            AssertEquals("test1", cm2.GetName());
            AssertEquals(2, cm2.Size());
            AssertEquals("value1", cm2.Get("key1"));
            AssertEquals("value2", cm2.Get("key2"));
            odb.Close();
        }

		/// <exception cref="System.Exception"></exception>
		public override void TearDown()
		{
			DeleteBase("map.neodatis");
		}
	}
}

using NeoDatis.Odb.Test.VO.Login;
using NUnit.Framework;
namespace NeoDatis.Odb.Test.Arraycollectionmap
{
	[TestFixture]
    public class TestMapContainingCollection : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test1()
		{
			DeleteBase("map-with-collections");
			NeoDatis.Odb.ODB odb = null;
			odb = Open("map-with-collections");
			NeoDatis.Odb.Test.Arraycollectionmap.MyMapObject o = new NeoDatis.Odb.Test.Arraycollectionmap.MyMapObject
				("test");
			System.Collections.Generic.IList<string> c = new System.Collections.Generic.List<string>();
			c.Add("ola");
			o.GetMap().Add("c", c);
			odb.Store(o);
			odb.Close();
			odb = Open("map-with-collections");
			NeoDatis.Odb.Objects<MyMapObject> os = odb.GetObjects<MyMapObject>();
			MyMapObject mmo = os.GetFirst();
			odb.Close();
			DeleteBase("map-with-collections");
			AssertEquals(o.GetName(), mmo.GetName());
			AssertEquals(o.GetMap().Count, mmo.GetMap().Count);
			AssertEquals(o.GetMap()["c"], mmo.GetMap()["c"]);
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test2()
		{
			DeleteBase("map-with-collections");
			NeoDatis.Odb.ODB odb = null;
			odb = Open("map-with-collections");
			NeoDatis.Odb.Test.Arraycollectionmap.MyMapObject o = new NeoDatis.Odb.Test.Arraycollectionmap.MyMapObject
				("test");
			System.Collections.Generic.IList<MyMapObject> c = new System.Collections.Generic.List<MyMapObject>();
			c.Add(o);
			o.GetMap().Add("c", c);
			odb.Store(o);
			odb.Close();
			odb = Open("map-with-collections");
			NeoDatis.Odb.Objects<MyMapObject> os = odb.GetObjects<MyMapObject>();
			MyMapObject mmo = os.GetFirst();
			odb.Close();
			DeleteBase("map-with-collections");
			AssertEquals(o.GetName(), mmo.GetName());
			AssertEquals(o.GetMap().Count, mmo.GetMap().Count);
			System.Collections.ICollection c1 = (System.Collections.ICollection)o.GetMap()["c"
				];
			System.Collections.ICollection c2 = (System.Collections.ICollection)mmo.GetMap()[
				"c"];
			AssertEquals(c1.Count, c2.Count);
			AssertEquals(mmo, c2.GetEnumerator().Current);
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test3()
		{
			// LogUtil.objectReaderOn(true);
			DeleteBase("map-with-collections");
			NeoDatis.Odb.ODB odb = null;
			odb = Open("map-with-collections");
			MyMapObject o = new MyMapObject("test");
			System.Collections.Generic.IList<MyMapObject> c = new System.Collections.Generic.List<MyMapObject>();
			c.Add(o);
			Function f1 = new Function("function1");
			o.GetMap().Add("a", c);
			int size = 1;
			for (int i = 0; i < size; i++)
			{
				o.GetMap().Add("A" + i, f1);
			}
			o.GetMap().Add("c", f1);
			Println("RealMap" + o.GetMap());
			odb.Store(o);
			odb.Close();
			odb = Open("map-with-collections");
            NeoDatis.Odb.Objects<MyMapObject> os = odb.GetObjects<MyMapObject>();
			NeoDatis.Odb.Test.Arraycollectionmap.MyMapObject mmo = (NeoDatis.Odb.Test.Arraycollectionmap.MyMapObject
				)os.GetFirst();
			odb.Close();
			DeleteBase("map-with-collections");
			AssertEquals(o.GetName(), mmo.GetName());
			AssertEquals(size + 2, mmo.GetMap().Count);
			AssertEquals(mmo, ((System.Collections.ICollection)mmo.GetMap()["a"]).GetEnumerator
				().Current);
			AssertEquals("function1", mmo.GetMap()["c"].ToString());
		}
	}

	internal class MyMapObject
	{
		private string name;

		private NeoDatis.Tool.Wrappers.Map.OdbHashMap<object, object> map;

		public MyMapObject(string name)
		{
			this.name = name;
			this.map = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<object, object>();
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public virtual NeoDatis.Tool.Wrappers.Map.OdbHashMap<object, object> GetMap()
		{
			return map;
		}

		public virtual void SetMap(NeoDatis.Tool.Wrappers.Map.OdbHashMap<object, object> 
			map)
		{
			this.map = map;
		}
	}
}

namespace NeoDatis.Odb.Test.Nullobject
{
	public class TestNullObject : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			DeleteBase("null.neodatis");
			NeoDatis.Odb.ODB odb = Open("null.neodatis");
			NeoDatis.Odb.Test.VO.Login.User user1 = new NeoDatis.Odb.Test.VO.Login.User("oli"
				, "oli@sdsadf", null);
			NeoDatis.Odb.Test.VO.Login.User user2 = new NeoDatis.Odb.Test.VO.Login.User("karine"
				, "karine@sdsadf", null);
			NeoDatis.Odb.Test.VO.Login.User user3 = new NeoDatis.Odb.Test.VO.Login.User(null, 
				null, null);
			odb.Store(user1);
			odb.Store(user2);
			odb.Store(user3);
			odb.Close();
			odb = Open("null.neodatis");
			NeoDatis.Odb.Objects l = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.User), 
				true);
			AssertEquals(3, l.Count);
			user1 = (NeoDatis.Odb.Test.VO.Login.User)l.Next();
			AssertEquals("oli", user1.GetName());
			AssertEquals("oli@sdsadf", user1.GetEmail());
			AssertEquals(null, user1.GetProfile());
			user2 = (NeoDatis.Odb.Test.VO.Login.User)l.Next();
			AssertEquals("karine", user2.GetName());
			AssertEquals("karine@sdsadf", user2.GetEmail());
			AssertEquals(null, user2.GetProfile());
			user3 = (NeoDatis.Odb.Test.VO.Login.User)l.Next();
			AssertEquals(null, user3.GetName());
			AssertEquals(null, user3.GetEmail());
			AssertEquals(null, user3.GetProfile());
			odb.Close();
			DeleteBase("null.neodatis");
		}

		/// <summary>Test generic attribute of type Object receving a native type</summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test2()
		{
			DeleteBase("nullo");
			NeoDatis.Odb.Test.Nullobject.GenericClass gc = new NeoDatis.Odb.Test.Nullobject.GenericClass
				(null);
			NeoDatis.Odb.ODB odb = Open("nullo");
			odb.Store(gc);
			odb.Close();
			odb = Open("nullo");
			NeoDatis.Odb.Objects objects = odb.GetObjects(typeof(NeoDatis.Odb.Test.Nullobject.GenericClass
				));
			NeoDatis.Odb.Test.Nullobject.GenericClass gc2 = (NeoDatis.Odb.Test.Nullobject.GenericClass
				)objects.GetFirst();
			gc2.SetObject("Ola");
			odb.Store(gc2);
			odb.Close();
			odb = Open("nullo");
			objects = odb.GetObjects(typeof(NeoDatis.Odb.Test.Nullobject.GenericClass));
			AssertEquals(1, objects.Count);
			NeoDatis.Odb.Test.Nullobject.GenericClass gc3 = (NeoDatis.Odb.Test.Nullobject.GenericClass
				)objects.GetFirst();
			AssertEquals("Ola", gc3.GetObject());
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test21()
		{
			DeleteBase("nullo");
			NeoDatis.Odb.Test.Nullobject.GenericClass gc = new NeoDatis.Odb.Test.Nullobject.GenericClass
				(null);
			NeoDatis.Odb.ODB odb = Open("nullo");
			odb.Store(gc);
			odb.Close();
			odb = Open("nullo");
			NeoDatis.Odb.Objects objects = odb.GetObjects(typeof(NeoDatis.Odb.Test.Nullobject.GenericClass
				));
			NeoDatis.Odb.Test.Nullobject.GenericClass gc2 = (NeoDatis.Odb.Test.Nullobject.GenericClass
				)objects.GetFirst();
			long[] longs = new long[] { System.Convert.ToInt64(1), System.Convert.ToInt64(2) };
			gc2.SetObjects(longs);
			odb.Store(gc2);
			odb.Close();
			odb = Open("nullo");
			objects = odb.GetObjects(typeof(NeoDatis.Odb.Test.Nullobject.GenericClass));
			AssertEquals(1, objects.Count);
			NeoDatis.Odb.Test.Nullobject.GenericClass gc3 = (NeoDatis.Odb.Test.Nullobject.GenericClass
				)objects.GetFirst();
			long[] longs2 = (long[])gc3.GetObjects();
			AssertEquals(2, longs2.Length);
			AssertEquals(System.Convert.ToInt64(1), longs2[0]);
			AssertEquals(System.Convert.ToInt64(2), longs2[1]);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test22()
		{
			DeleteBase("nullo");
			NeoDatis.Odb.Test.Nullobject.GenericClass gc = new NeoDatis.Odb.Test.Nullobject.GenericClass
				(null);
			NeoDatis.Odb.ODB odb = Open("nullo");
			odb.Store(gc);
			odb.Close();
			odb = Open("nullo");
			NeoDatis.Odb.Objects objects = odb.GetObjects(typeof(NeoDatis.Odb.Test.Nullobject.GenericClass
				));
			NeoDatis.Odb.Test.Nullobject.GenericClass gc2 = (NeoDatis.Odb.Test.Nullobject.GenericClass
				)objects.GetFirst();
			gc2.GetObjects()[0] = System.Convert.ToInt64(1);
			gc2.GetObjects()[1] = System.Convert.ToInt64(2);
			odb.Store(gc2);
			odb.Close();
			odb = Open("nullo");
			objects = odb.GetObjects(typeof(NeoDatis.Odb.Test.Nullobject.GenericClass));
			AssertEquals(1, objects.Count);
			NeoDatis.Odb.Test.Nullobject.GenericClass gc3 = (NeoDatis.Odb.Test.Nullobject.GenericClass
				)objects.GetFirst();
			object[] longs2 = (object[])gc3.GetObjects();
			AssertEquals(10, longs2.Length);
			AssertEquals(System.Convert.ToInt64(1), longs2[0]);
			AssertEquals(System.Convert.ToInt64(2), longs2[1]);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test23()
		{
			DeleteBase("nullo");
			NeoDatis.Odb.Test.Nullobject.GenericClass gc = new NeoDatis.Odb.Test.Nullobject.GenericClass
				(null);
			gc.GetObjects()[0] = new NeoDatis.Odb.Test.VO.Login.Function("f1");
			NeoDatis.Odb.ODB odb = Open("nullo");
			odb.Store(gc);
			odb.Close();
			odb = Open("nullo");
			NeoDatis.Odb.Objects objects = odb.GetObjects(typeof(NeoDatis.Odb.Test.Nullobject.GenericClass
				));
			NeoDatis.Odb.Test.Nullobject.GenericClass gc2 = (NeoDatis.Odb.Test.Nullobject.GenericClass
				)objects.GetFirst();
			gc2.GetObjects()[0] = System.Convert.ToInt64(1);
			gc2.GetObjects()[1] = System.Convert.ToInt64(2);
			odb.Store(gc2);
			odb.Close();
			odb = Open("nullo");
			objects = odb.GetObjects(typeof(NeoDatis.Odb.Test.Nullobject.GenericClass));
			AssertEquals(1, objects.Count);
			NeoDatis.Odb.Test.Nullobject.GenericClass gc3 = (NeoDatis.Odb.Test.Nullobject.GenericClass
				)objects.GetFirst();
			object[] longs2 = (object[])gc3.GetObjects();
			AssertEquals(10, longs2.Length);
			AssertEquals(System.Convert.ToInt64(1), longs2[0]);
			AssertEquals(System.Convert.ToInt64(2), longs2[1]);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test3()
		{
			DeleteBase("nullo");
			NeoDatis.Odb.Test.Nullobject.GenericClass gc = new NeoDatis.Odb.Test.Nullobject.GenericClass
				(null);
			string[] strings = new string[] { "OBJ1", "obj2" };
			gc.SetObjects(strings);
			NeoDatis.Odb.ODB odb = Open("nullo");
			odb.Store(gc);
			odb.Close();
			odb = Open("nullo");
			NeoDatis.Odb.Objects objects = odb.GetObjects(typeof(NeoDatis.Odb.Test.Nullobject.GenericClass
				));
			NeoDatis.Odb.Test.Nullobject.GenericClass gc2 = (NeoDatis.Odb.Test.Nullobject.GenericClass
				)objects.GetFirst();
			gc2.SetObject("Ola");
			odb.Store(gc2);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test4()
		{
			DeleteBase("nullo");
			NeoDatis.Odb.Test.Nullobject.GenericClass gc = new NeoDatis.Odb.Test.Nullobject.GenericClass
				(null);
			string[] strings = new string[] { "OBJ1", "obj2" };
			gc.SetObject(strings);
			NeoDatis.Odb.ODB odb = Open("nullo");
			odb.Store(gc);
			odb.Close();
			odb = Open("nullo");
			NeoDatis.Odb.Objects objects = odb.GetObjects(typeof(NeoDatis.Odb.Test.Nullobject.GenericClass
				));
			NeoDatis.Odb.Test.Nullobject.GenericClass gc2 = (NeoDatis.Odb.Test.Nullobject.GenericClass
				)objects.GetFirst();
			gc2.SetObject("Ola");
			odb.Store(gc2);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test5()
		{
			DeleteBase("nullo");
			NeoDatis.Odb.Test.VO.Login.Function f = new NeoDatis.Odb.Test.VO.Login.Function("a simple value"
				);
			NeoDatis.Odb.ODB odb = Open("nullo");
			odb.Store(f);
			odb.Close();
			odb = Open("nullo");
			NeoDatis.Odb.Objects objects = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
				));
			NeoDatis.Odb.Test.VO.Login.Function f2 = (NeoDatis.Odb.Test.VO.Login.Function)objects
				.GetFirst();
			f2.SetName(null);
			odb.Store(f2);
			odb.Close();
			odb = Open("nullo");
			objects = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function));
			f2 = (NeoDatis.Odb.Test.VO.Login.Function)objects.GetFirst();
			odb.Close();
			AssertEquals(null, f2.GetName());
		}
	}
}

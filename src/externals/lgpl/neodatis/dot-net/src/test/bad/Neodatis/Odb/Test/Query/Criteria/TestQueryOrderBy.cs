using NeoDatis.Odb.Impl.Core.Query.Criteria;
using NeoDatis.Odb.Test.VO.Login;
using NeoDatis.Odb.Core.Query.Criteria;
namespace NeoDatis.Odb.Test.Query.Criteria
{
	/// <author>olivier</author>
	public class TestQueryOrderBy : NeoDatis.Odb.Test.ODBTest
	{
		public virtual void Test1()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			odb.Store(new Class1("c1"));
			odb.Store(new Class1("c1"));
			odb.Store(new Class1("c2"));
			odb.Store(new Class1("c2"));
			odb.Store(new Class1("c3"));
			odb.Store(new Class1("c4"));
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new CriteriaQuery();
			q.OrderByAsc("name");
			NeoDatis.Odb.Objects<Class1> objects = odb.GetObjects<Class1>(q);
			AssertEquals(6, objects.Count);
			while (objects.HasNext())
			{
				System.Console.Out.WriteLine(objects.Next());
			}
			// println(objects);
			odb.Close();
		}

		public virtual void Test2()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			odb.Store(new Class1("c1"));
			odb.Store(new Class1("c1"));
			odb.Store(new Class1("c2"));
			odb.Store(new Class1("c2"));
			odb.Store(new Class1("c3"));
			odb.Store(new Class1("c4"));
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new CriteriaQuery();
			// q.orderByAsc("name");
			NeoDatis.Odb.Objects<Class1> objects = odb.GetObjects<Class1>(q);
			AssertEquals(6, objects.Count);
			Println(objects);
			odb.Close();
		}

		public virtual void Test3()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			int size = 500;
			for (int i = 0; i < size; i++)
			{
				odb.Store(new Class1("c1"));
			}
			for (int i = 0; i < size; i++)
			{
				odb.Store(new Class1("c2"));
			}
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new CriteriaQuery();
			// q.orderByAsc("name");
			NeoDatis.Odb.Objects<Class1> objects = odb.GetObjects<Class1>(q);
			AssertEquals(size * 2, objects.Count);
			for (int i = 0; i < size; i++)
			{
				Class1 c1 = (Class1
					)objects.Next();
				AssertEquals("c1", c1.GetName());
			}
			for (int i = 0; i < size; i++)
			{
				Class1 c1 = (Class1
					)objects.Next();
				AssertEquals("c2", c1.GetName());
			}
			odb.Close();
		}

		public virtual void Test4()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			int size = 5;
			for (int i = 0; i < size; i++)
			{
				odb.Store(new Function("f" + (i + 1)));
			}
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new CriteriaQuery();
			// q.orderByAsc("name");
			NeoDatis.Odb.Objects<Function> objects = odb.GetObjects<Function>(q, true, 0, 2);
			System.Collections.IList l = new System.Collections.Generic.List<Function
				>(objects);
			AssertEquals(2, l.Count);
			odb.Close();
			odb = Open(baseName);
			q = new CriteriaQuery();
			q.OrderByAsc("name");
			objects = odb.GetObjects<Function>(q, true, 0, 2);
			l = new System.Collections.Generic.List<Function>(objects
				);
			AssertEquals(2, l.Count);
			odb.Close();
			odb = Open(baseName);
			q = new CriteriaQuery();
			q.OrderByDesc("name");
			objects = odb.GetObjects<Function>(q, true, 0, 2);
			l = new System.Collections.Generic.List<Function>(objects
				);
			AssertEquals(2, l.Count);
			odb.Close();
		}

		public virtual void Test51()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			odb.Store(new Function("Not Null"));
			odb.Store(new Function(null));
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new CriteriaQuery(Where.IsNotNull("name"));
			// q.orderByAsc("name");
			NeoDatis.Odb.Objects<Function> objects = odb.GetObjects<Function>(q, true, 0, 10);
			System.Collections.IList l = new System.Collections.Generic.List<Function
				>(objects);
			odb.Close();
			AssertEquals(1, l.Count);
		}

		public virtual void Test5()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			int size = 5;
			for (int i = 0; i < size; i++)
			{
				odb.Store(new Function("f1"));
			}
			odb.Store(new Function(null));
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new CriteriaQuery(Where.IsNotNull("name"));
			// q.orderByAsc("name");
			NeoDatis.Odb.Objects<Function> objects = odb.GetObjects<Function>(q, true, 0, 10);
			System.Collections.IList l = new System.Collections.Generic.List<Function
				>(objects);
			AssertEquals(size, l.Count);
			odb.Close();
			odb = Open(baseName);
			q = new CriteriaQuery(Where.IsNotNull("name"));
			q.OrderByAsc("name");
			objects = odb.GetObjects<Function>(q, true, 0, 10);
			l = new System.Collections.Generic.List<Function>(objects
				);
			AssertEquals(5, l.Count);
			odb.Close();
			odb = Open(baseName);
			q = new CriteriaQuery(Where.IsNotNull("name"));
			q.OrderByDesc("name");
			objects = odb.GetObjects<Function>(q, true, 0, 10);
			l = new System.Collections.Generic.List<Function>(objects
				);
			AssertEquals(5, l.Count);
			odb.Close();
		}

		public virtual void Test6()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			int size = 5;
			for (int i = 0; i < size; i++)
			{
				odb.Store(new Function("f1"));
			}
			odb.Store(new Function(null));
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new CriteriaQuery(Where.IsNull("name"));
			// q.orderByAsc("name");
			NeoDatis.Odb.Objects<Function> objects = odb.GetObjects<Function>(q, true, 0, 10);
			System.Collections.IList l = new System.Collections.Generic.List<Function
				>(objects);
			AssertEquals(1, l.Count);
			odb.Close();
			odb = Open(baseName);
			q = new CriteriaQuery(Where.IsNull("name"));
			q.OrderByAsc("name");
			objects = odb.GetObjects<Function>(q, true, 0, 10);
			l = new System.Collections.Generic.List<Function>(objects
				);
			AssertEquals(1, l.Count);
			odb.Close();
			odb = Open(baseName);
			q = new CriteriaQuery(Where.IsNull("name"));
			q.OrderByDesc("name");
			objects = odb.GetObjects<Function>(q, true, 0, 10);
			l = new System.Collections.Generic.List<Function>(objects
				);
			AssertEquals(1, l.Count);
			odb.Close();
		}
	}
}

using NeoDatis.Odb.Test.VO.Login;
using NeoDatis.Odb.Impl.Core.Query.Criteria;
using NeoDatis.Tool.Wrappers;
using NeoDatis.Odb.Core.Query.Criteria;
namespace NeoDatis.Odb.Test.Query.Criteria
{
	public class TestIndexFromTo : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public virtual void TestGetLimitedResult1()
		{
			string baseName = GetBaseName();
			int size = 1000;
			NeoDatis.Odb.ODB odb = Open(baseName);
			for (int i = 0; i < size; i++)
			{
				odb.Store(new Function("function " + i));
			}
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new CriteriaQuery();
			NeoDatis.Odb.Objects<Function> os = odb.GetObjects<Function>(q, true, 0, 1);
			AssertEquals(1, os.Count);
			for (int i = 0; i < os.Count; i++)
			{
				Function f = (Function)os.Next
					();
				AssertEquals("function " + i, f.GetName());
			}
			odb.Close();
			DeleteBase(baseName);
		}

		public virtual void Test()
		{
			string s = "olivier";
			string ss = OdbString.Substring(s, 0, 1);
			AssertEquals(1, ss.Length);
			ss = OdbString.Substring(s, 0, 2);
			AssertEquals(2, ss.Length);
			System.Collections.Generic.IList<string> l = new System.Collections.Generic.List<
				string>();
			l.Add("s1");
			l.Add("s2");
			l.Add("s3");
			l.Add("s4");
			l.Add("s5");
			AssertEquals(1, l.SubList(0, 1).Count);
			AssertEquals(2, l.SubList(0, 2).Count);
			AssertEquals(3, l.SubList(0, 3).Count);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestGetLimitedResult()
		{
			string baseName = GetBaseName();
			int size = 1000;
			NeoDatis.Odb.ODB odb = Open(baseName);
			for (int i = 0; i < size; i++)
			{
				odb.Store(new Function("function " + i));
			}
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new CriteriaQuery();
			NeoDatis.Odb.Objects<Function> os = odb.GetObjects<Function>(q, true, 0, 10);
			AssertEquals(10, os.Count);
			for (int i = 0; i < 10; i++)
			{
				Function f = (Function)os.Next
					();
				AssertEquals("function " + i, f.GetName());
			}
			odb.Close();
			DeleteBase(baseName);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestGetLimitedResult2()
		{
			string baseName = GetBaseName();
			int size = 1000;
			NeoDatis.Odb.ODB odb = Open(baseName);
			for (int i = 0; i < size; i++)
			{
				odb.Store(new Function("function " + i));
			}
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new CriteriaQuery();
			NeoDatis.Odb.Objects<Function> os = odb.GetObjects<Function>(q, true, 10, 20);
			AssertEquals(10, os.Count);
			for (int i = 10; i < 20; i++)
			{
				Function f = (Function)os.Next
					();
				AssertEquals("function " + i, f.GetName());
			}
			odb.Close();
			DeleteBase(baseName);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestGetLimitedResult3()
		{
			string baseName = GetBaseName();
			int size = 1000;
			NeoDatis.Odb.ODB odb = Open(baseName);
			for (int i = 0; i < size; i++)
			{
				if (i < size / 2)
				{
					odb.Store(new Function("function " + i));
				}
				else
				{
					odb.Store(new Function("FUNCTION " + i));
				}
			}
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new CriteriaQuery(Where.Like("name", "FUNCTION%"));
			NeoDatis.Odb.Objects<Function> os = odb.GetObjects<Function>(q, true, 0, 10);
			AssertEquals(10, os.Count);
			for (int i = size / 2; i < size / 2 + 10; i++)
			{
				Function f = (Function)os.Next
					();
				AssertEquals("FUNCTION " + i, f.GetName());
			}
			odb.Close();
			DeleteBase(baseName);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestGetLimitedResult4()
		{
			string baseName = GetBaseName();
			int size = 1000;
			NeoDatis.Odb.ODB odb = Open(baseName);
			for (int i = 0; i < size; i++)
			{
				if (i < size / 2)
				{
					odb.Store(new Function("function " + i));
				}
				else
				{
					odb.Store(new Function("FUNCTION " + i));
				}
			}
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new CriteriaQuery(Where.Like("name", "FUNCTION%"));
			NeoDatis.Odb.Objects<Function> os = odb.GetObjects<Function>(q, true, 10, 20);
			AssertEquals(10, os.Count);
			for (int i = size / 2 + 10; i < size / 2 + 20; i++)
			{
				Function f = (Function)os.Next
					();
				AssertEquals("FUNCTION " + i, f.GetName());
			}
			odb.Close();
			DeleteBase(baseName);
		}
	}
}

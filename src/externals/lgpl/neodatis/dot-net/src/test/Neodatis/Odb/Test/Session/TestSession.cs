using NeoDatis.Odb.Test.VO.Arraycollectionmap;
using NeoDatis.Odb.Test.VO.Login;
using NUnit.Framework;
namespace NeoDatis.Odb.Test.Session
{
	[TestFixture]
    public class TestSession : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test1()
		{
			DeleteBase("session.neodatis");
			NeoDatis.Odb.ODB odb = Open("session.neodatis");
			odb.Close();
			NeoDatis.Odb.ODB odb2 = Open("session.neodatis");
			NeoDatis.Odb.Objects<PlayerWithList> l = odb2.GetObjects<PlayerWithList>(true);
			AssertEquals(0, l.Count);
			odb2.Close();
			DeleteBase("session.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test2()
		{
			DeleteBase("session.neodatis");
			NeoDatis.Odb.ODB odb = Open("session.neodatis");
			Function f = new Function("f1"
				);
			odb.Store(f);
			odb.Commit();
			f.SetName("f1 -1");
			odb.Store(f);
			odb.Close();
			odb = Open("session.neodatis");
			NeoDatis.Odb.Objects<Function> os = odb.GetObjects<Function>();
			AssertEquals(1, os.Count);
			Function f2 = (Function)os.
				GetFirst();
			odb.Close();
			DeleteBase("session.neodatis");
			AssertEquals("f1 -1", f2.GetName());
		}
	}
}

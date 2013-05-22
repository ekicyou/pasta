namespace NeoDatis.Odb.Test.Other
{
	/// <summary>Reported bug by Moises &gt; on 1.5.6</summary>
	/// <author>osmadja</author>
	public class TestObjectWithOid : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			DeleteBase("test-object-with-oid");
			NeoDatis.Odb.ODB odb = Open("test-object-with-oid");
			NeoDatis.Odb.Test.Other.ObjectWithOid o = new NeoDatis.Odb.Test.Other.ObjectWithOid
				("15", "test");
			NeoDatis.Odb.OID oid = odb.Store(o);
			odb.Close();
			odb = Open("test-object-with-oid");
			NeoDatis.Odb.Test.Other.ObjectWithOid o2 = (NeoDatis.Odb.Test.Other.ObjectWithOid
				)odb.GetObjectFromId(oid);
			odb.Close();
			AssertEquals(o.GetOid(), o2.GetOid());
			AssertEquals(o.GetName(), o2.GetName());
		}
	}
}

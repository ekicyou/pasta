namespace NeoDatis.Odb.Test.Oid
{
	/// <author>olivier</author>
	public class TestStoreWithOID : NeoDatis.Odb.Test.ODBTest
	{
		public virtual void Test1()
		{
			NeoDatis.Odb.ODB odb = Open("withoid");
			NeoDatis.Odb.OID oid = odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("f1"));
			odb.Close();
			odb = Open("withoid");
			NeoDatis.Odb.Test.VO.Login.Function f2 = new NeoDatis.Odb.Test.VO.Login.Function(
				"f2");
			NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy
				.GetEngine(odb);
			engine.Store(oid, f2);
			odb.Close();
			odb = Open("withoid");
			NeoDatis.Odb.Test.VO.Login.Function f = (NeoDatis.Odb.Test.VO.Login.Function)odb.
				GetObjectFromId(oid);
			odb.Close();
			AssertEquals("f2", f.GetName());
		}
	}
}

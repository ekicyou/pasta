namespace NeoDatis.Odb.Test.Other
{
	/// <author>olivier</author>
	public class TestGetDatabaseName : NeoDatis.Odb.Test.ODBTest
	{
		public virtual void Test1()
		{
			string baseName = "name.neodatis";
			NeoDatis.Odb.ODB odb = Open(baseName);
			NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy
				.GetEngine(odb);
			string s = engine.GetBaseIdentification().GetIdentification();
			if (isLocal)
			{
				AssertEquals(baseName, s);
			}
			else
			{
				AssertEquals("unit-test-data/name.neodatis@127.0.0.1:13000", s);
			}
		}

		public virtual void Test2()
		{
			string baseName = "name.neodatis";
			NeoDatis.Odb.ODB odb = Open(baseName);
			string s = odb.GetName();
			if (isLocal)
			{
				AssertEquals(baseName, s);
			}
			else
			{
				AssertEquals("unit-test-data/name.neodatis@127.0.0.1:13000", s);
			}
		}
	}
}

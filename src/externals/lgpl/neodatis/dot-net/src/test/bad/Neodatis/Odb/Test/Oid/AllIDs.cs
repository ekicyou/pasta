namespace NeoDatis.Odb.Test.Oid
{
	public class AllIDs : NeoDatis.Odb.Test.ODBTest
	{
		public static string FileName = NeoDatis.Odb.Test.ODBTest.Directory + "ids.neodatis";

		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			if (!isLocal)
			{
				return;
			}
			DeleteBase(FileName);
			NeoDatis.Odb.Core.Layers.Layer3.IBaseIdentification parameter = new NeoDatis.Odb.Core.Layers.Layer3.IOFileParameter
				(NeoDatis.Odb.Test.ODBTest.Directory + FileName, true, null, null);
			NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = NeoDatis.Odb.OdbConfiguration
				.GetCoreProvider().GetClientStorageEngine(parameter);
			NeoDatis.Odb.Test.VO.Login.Function function1 = new NeoDatis.Odb.Test.VO.Login.Function
				("login");
			engine.Store(function1);
			NeoDatis.Odb.Test.VO.Login.Function function2 = new NeoDatis.Odb.Test.VO.Login.Function
				("login2");
			engine.Store(function2);
			engine.Commit();
			engine.Close();
			engine = NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetClientStorageEngine(parameter
				);
			System.Collections.IList l = engine.GetAllObjectIds();
			AssertEquals(2, l.Count);
			engine.Close();
			DeleteBase(FileName);
		}
	}
}

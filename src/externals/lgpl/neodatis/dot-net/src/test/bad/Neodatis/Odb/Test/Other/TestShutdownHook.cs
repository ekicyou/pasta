namespace NeoDatis.Odb.Test.Other
{
	public class TestShutdownHook : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			DeleteBase("hook.neodatis");
			NeoDatis.Odb.ODB obase = Open("hook.neodatis");
			obase.GetObjects<TestClass>();
			obase.Store(new NeoDatis.Odb.Test.VO.Attribute.TestClass());
			obase.Close();
			DeleteBase("hook.neodatis");
		}
	}
}

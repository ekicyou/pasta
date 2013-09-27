namespace NeoDatis.Odb.Test.Other
{
	public class Test2Closes : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			DeleteBase("hook.neodatis");
			NeoDatis.Odb.ODB obase = Open("hook.neodatis");
			obase.GetObjects(typeof(NeoDatis.Odb.Test.VO.Attribute.TestClass));
			obase.Store(new NeoDatis.Odb.Test.VO.Attribute.TestClass());
			obase.Close();
			bool exception = false;
			try
			{
				obase.Close();
			}
			catch (System.Exception e)
			{
				exception = true;
				AssertTrue(e.Message.IndexOf("ODB session has already been closed") != -1);
			}
			AssertTrue(exception);
		}
	}
}
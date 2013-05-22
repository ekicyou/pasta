namespace NeoDatis.Odb.Test.Other
{
	public class TestAgainstNativeObjects : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			DeleteBase("native.neodatis");
			NeoDatis.Odb.ODB @base = Open("native.neodatis");
			try
			{
				@base.Store("olivier");
			}
			catch (NeoDatis.Odb.ODBRuntimeException)
			{
				@base.Close();
				DeleteBase("native.neodatis");
				return;
			}
			@base.Close();
			Fail("Allow native object direct persistence");
			DeleteBase("native.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test2()
		{
			DeleteBase("native.neodatis");
			NeoDatis.Odb.ODB @base = Open("native.neodatis");
			try
			{
				string[] array = new string[] { "olivier", "joao", "peter" };
				@base.Store(array);
			}
			catch (NeoDatis.Odb.ODBRuntimeException)
			{
				@base.Close();
				DeleteBase("native.neodatis");
				return;
			}
			@base.Close();
			Fail("Allow native object direct persistence");
			DeleteBase("native.neodatis");
		}
	}
}

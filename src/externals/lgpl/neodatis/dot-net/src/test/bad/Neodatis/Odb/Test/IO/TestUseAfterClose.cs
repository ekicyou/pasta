namespace NeoDatis.Odb.Test.IO
{
	public class TestUseAfterClose : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public virtual void Test()
		{
			NeoDatis.Odb.ODB odb = Open("t111.neodatis");
			odb.Close();
			try
			{
				odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("login"));
			}
			catch (System.Exception e)
			{
				string s = NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(e, false);
				AssertTrue(e.Message.IndexOf("has already been closed") != -1);
			}
			DeleteBase("t111.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestTwoCloses()
		{
			NeoDatis.Odb.ODB odb = Open("t111.neodatis");
			odb.Close();
			try
			{
				odb.Close();
			}
			catch (System.Exception e)
			{
				string s = NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(e, false);
				AssertTrue(e.Message.IndexOf("has already been closed") != -1);
			}
			DeleteBase("t111.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestReOpenWithoutClose()
		{
			NeoDatis.Odb.ODB odb = Open("t111.neodatis");
			try
			{
				odb = Open("t111.neodatis");
			}
			catch (System.Exception e)
			{
				string s = NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(e, false);
				AssertTrue(e.Message.IndexOf("file is locked by the current Virtual machine - check if the database has not been opened in the current VM!"
					) != -1);
			}
			odb.Close();
			DeleteBase("t111.neodatis");
		}
	}
}

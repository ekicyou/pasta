namespace NeoDatis.Odb.Test.Newbie
{
	/// <summary>It is just a simple test to help the newbies</summary>
	/// <author>mayworm at <xmpp://mayworm@gmail.com></author>
	public class CreateDataBaseTest : NeoDatis.Odb.Test.ODBTest
	{
		private static readonly string NewbieOdb = "newbie.neodatis";

		/// <summary>Test if a new database could be created</summary>
		public virtual void TestCreateDataBase()
		{
			try
			{
				DeleteBase(NewbieOdb);
				NeoDatis.Odb.ODB odb = Open(NewbieOdb);
				odb.Close();
				bool existFile = NeoDatis.Tool.IOUtil.ExistFile(Directory + NewbieOdb);
				AssertTrue("ODB data file couldn't created", existFile);
				DeleteBase(NewbieOdb);
			}
			catch (System.Exception e)
			{
				Sharpen.Runtime.PrintStackTrace(e);
			}
		}
	}
}

namespace NeoDatis.Odb.Test.Other
{
	public class TestJavaClassPersistence : NeoDatis.Odb.Test.ODBTest
	{
		public static readonly string DbName = "class.neodatis";

		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			DeleteBase(DbName);
			NeoDatis.Odb.ODB odb = Open(DbName);
			odb.Store(new System.Exception("test"));
			odb.Close();
			odb = Open(DbName);
			NeoDatis.Odb.Objects l = odb.GetObjects(typeof(System.Exception));
			odb.Close();
			DeleteBase(DbName);
		}
	}
}

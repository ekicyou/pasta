using NeoDatis.Odb.Test.VO.Interfaces;
namespace NeoDatis.Odb.Test.Interfaces
{
	public class TestObjectWithInterfaces : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public virtual void TestInsert()
		{
			DeleteBase("tinterfaces.neodatis");
			NeoDatis.Odb.ODB odb = Open("tinterfaces.neodatis");
			ObjectWithInterfaces owi = new ObjectWithInterfaces("Ol√° chico");
			odb.Store(owi);
			odb.Commit();
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestInsertAndSelect()
		{
			DeleteBase("tinterfaces.neodatis");
			NeoDatis.Odb.ODB odb = Open("tinterfaces.neodatis");
			ObjectWithInterfaces owi = new ObjectWithInterfaces("Ol√° chico");
			odb.Store(owi);
			odb.Close();
			odb = Open("tinterfaces.neodatis");
			NeoDatis.Odb.Objects<ObjectWithInterfaces> os = odb.GetObjects<ObjectWithInterfaces>();
			AssertEquals(1, os.Count);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestStoreObjectByInterfaces()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			object o = new MyObject("f");
			odb.Store(o);
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Objects<MyObject> os = odb.GetObjects<MyObject>();
			AssertEquals(1, os.Count);
			odb.Close();
			// deleteBase(baseName);
			Println(baseName);
		}
	}
}

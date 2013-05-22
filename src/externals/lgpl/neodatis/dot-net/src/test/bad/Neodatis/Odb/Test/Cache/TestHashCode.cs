namespace NeoDatis.Odb.Test.Cache
{
	public class TestHashCode : NeoDatis.Odb.Test.ODBTest
	{
		/// <summary>a problem reported by glsender - 1875544</summary>
		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			NeoDatis.Odb.Test.Cache.MyObjectWithMyHashCode my = null;
			// creates 1000 objects
			for (int i = 0; i < 1000; i++)
			{
				my = new NeoDatis.Odb.Test.Cache.MyObjectWithMyHashCode(System.Convert.ToInt64(1000
					));
				odb.Store(my);
			}
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Objects objects = odb.GetObjects(typeof(NeoDatis.Odb.Test.Cache.MyObjectWithMyHashCode
				));
			AssertEquals(1000, objects.Count);
			while (objects.HasNext())
			{
				my = (NeoDatis.Odb.Test.Cache.MyObjectWithMyHashCode)objects.Next();
				odb.Delete(my);
			}
			odb.Close();
			odb = Open(baseName);
			objects = odb.GetObjects(typeof(NeoDatis.Odb.Test.Cache.MyObjectWithMyHashCode));
			odb.Close();
			NeoDatis.Tool.IOUtil.DeleteFile(baseName);
			AssertEquals(0, objects.Count);
		}

		/// <summary>a problem reported by glsender</summary>
		/// <exception cref="System.Exception"></exception>
		public virtual void Test2()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			NeoDatis.Odb.Test.Cache.MyObjectWithMyHashCode2 my = null;
			// creates 1000 objects
			for (int i = 0; i < 1000; i++)
			{
				my = new NeoDatis.Odb.Test.Cache.MyObjectWithMyHashCode2(System.Convert.ToInt64(1000
					));
				odb.Store(my);
			}
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Objects objects = odb.GetObjects(typeof(NeoDatis.Odb.Test.Cache.MyObjectWithMyHashCode2
				));
			AssertEquals(1000, objects.Count);
			while (objects.HasNext())
			{
				my = (NeoDatis.Odb.Test.Cache.MyObjectWithMyHashCode2)objects.Next();
				odb.Delete(my);
			}
			odb.Close();
			odb = Open(baseName);
			objects = odb.GetObjects(typeof(NeoDatis.Odb.Test.Cache.MyObjectWithMyHashCode2));
			odb.Close();
			NeoDatis.Tool.IOUtil.DeleteFile(baseName);
			AssertEquals(0, objects.Count);
		}
	}
}

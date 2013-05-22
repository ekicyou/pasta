namespace NeoDatis.Odb.Test.Jdk15.Generics
{
	/// <author>olivier</author>
	public class TestGenerics : NeoDatis.Odb.Test.ODBTest
	{
		public virtual void TestGetObects()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("Test"));
			NeoDatis.Odb.Objects<NeoDatis.Odb.Test.VO.Login.Function> functions = odb.GetObjects
				(typeof(NeoDatis.Odb.Test.VO.Login.Function));
			NeoDatis.Odb.Test.VO.Login.Function f = functions.GetFirst();
			odb.Close();
			AssertEquals(1, functions.Count);
		}

		public virtual void TestGetObects2()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("Test"));
			NeoDatis.Odb.Objects<NeoDatis.Odb.Test.VO.Login.Function> functions = odb.GetObjects
				(typeof(NeoDatis.Odb.Test.VO.Login.Function));
			NeoDatis.Odb.Test.VO.Login.Function f = functions.Next();
			odb.Close();
			AssertEquals(1, functions.Count);
		}

		public virtual void TestGetObects3()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("Test"));
			NeoDatis.Odb.Objects<NeoDatis.Odb.Test.VO.Login.Function> functions = odb.GetObjects
				(typeof(NeoDatis.Odb.Test.VO.Login.Function));
			System.Collections.Generic.IEnumerator<NeoDatis.Odb.Test.VO.Login.Function> iterator
				 = functions.GetEnumerator();
			NeoDatis.Odb.Test.VO.Login.Function f = iterator.Current;
			odb.Close();
			AssertEquals(1, functions.Count);
		}
	}
}

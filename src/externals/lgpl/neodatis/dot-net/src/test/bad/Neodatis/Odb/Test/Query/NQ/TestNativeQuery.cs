namespace NeoDatis.Odb.Test.Query.NQ
{
	public class TestNativeQuery : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			if (!isLocal || !useSameVmOptimization)
			{
				// native must be serializable to be executed in cs mode
				return;
			}
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			bool[] bbs1 = new bool[2];
			bbs1[0] = true;
			bbs1[1] = false;
			bool[] bbs2 = new bool[2];
			bbs2[0] = true;
			bbs2[1] = false;
			NeoDatis.Odb.Test.VO.ClassWithArrayOfBoolean o = new NeoDatis.Odb.Test.VO.ClassWithArrayOfBoolean
				("test", bbs1, bbs2);
			odb.Store(o);
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery query = new _SimpleNativeQuery_55();
			NeoDatis.Odb.Objects objects = odb.GetObjects(query);
			AssertEquals(1, objects.Count);
			NeoDatis.Odb.Test.VO.ClassWithArrayOfBoolean o2 = (NeoDatis.Odb.Test.VO.ClassWithArrayOfBoolean
				)objects.GetFirst();
			AssertEquals("test", o2.GetName());
			AssertEquals(true, o2.GetBools1()[0]);
			AssertEquals(false, o2.GetBools1()[1]);
			AssertEquals(true, o2.GetBools2()[0]);
			AssertEquals(false, o2.GetBools2()[1]);
		}

		private sealed class _SimpleNativeQuery_55 : NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery
		{
			public _SimpleNativeQuery_55()
			{
			}

			public bool Match(NeoDatis.Odb.Test.VO.ClassWithArrayOfBoolean o)
			{
				return true;
			}
		}
	}
}

using NUnit.Framework;
using NeoDatis.Odb.Core.Query.NQ;
using NeoDatis.Odb.Test.VO;
using NeoDatis.Odb.Core.Query;
using NeoDatis.Odb.Test.VO.Sport;
namespace NeoDatis.Odb.Test.Query.NQ
{
	[TestFixture]
    public class TestNativeQuery : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test1()
		{
			
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			bool[] bbs1 = new bool[2];
			bbs1[0] = true;
			bbs1[1] = false;
			bool[] bbs2 = new bool[2];
			bbs2[0] = true;
			bbs2[1] = false;
			ClassWithArrayOfBoolean o = new ClassWithArrayOfBoolean("test", bbs1, bbs2);
			odb.Store(o);
			odb.Close();
			odb = Open(baseName);
            IQuery query = new Query1();
			Objects<ClassWithArrayOfBoolean> objects = odb.GetObjects<ClassWithArrayOfBoolean>(new Query1());
			AssertEquals(1, objects.Count);
			ClassWithArrayOfBoolean o2 = objects.GetFirst();
			AssertEquals("test", o2.GetName());
			AssertEquals(true, o2.GetBools1()[0]);
			AssertEquals(false, o2.GetBools1()[1]);
			AssertEquals(true, o2.GetBools2()[0]);
			AssertEquals(false, o2.GetBools2()[1]);
		}

		public class Query1 : SimpleNativeQuery
		{
			public bool Match(ClassWithArrayOfBoolean o)
			{
				return true;
			}
		}
        public class Query2 : SimpleNativeQuery
        {
            public bool Match(Player player)
            {
                return player.GetFavoriteSport().GetName().ToLower().StartsWith("volley");
            }
        }
	}
}

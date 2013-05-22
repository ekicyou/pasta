using NeoDatis.Odb.Test.VO.Login;
namespace NeoDatis.Odb.Test.Performance
{
	public class TestGetWithStartIndex : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public override void SetUp()
		{
			base.SetUp();
			DeleteBase("start-index.neodatis");
			NeoDatis.Odb.ODB odb = Open("start-index.neodatis");
			for (int i = 0; i < 10; i++)
			{
				odb.Store(new Function("function " + i));
			}
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			NeoDatis.Odb.ODB odb = Open("start-index.neodatis");
			string s = null;
			NeoDatis.Odb.Objects<Function> l = odb.GetObjects<Function>( false, 4, 7);
			AssertEquals(3, l.Count);
			AssertEquals("function 4", l.GetFirst().ToString());
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public override void TearDown()
		{
			DeleteBase("start-index.neodatis");
		}
	}
}

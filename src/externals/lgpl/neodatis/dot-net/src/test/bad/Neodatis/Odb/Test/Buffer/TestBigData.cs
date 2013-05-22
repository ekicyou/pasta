namespace NeoDatis.Odb.Test.Buffer
{
	public class TestBigData : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			NeoDatis.Odb.ODB odb = Open("big-data.neodatis");
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			for (int i = 0; i < 30000; i++)
			{
				buffer.Append('a');
			}
			NeoDatis.Odb.Test.VO.Login.Function function = new NeoDatis.Odb.Test.VO.Login.Function
				(buffer.ToString());
			odb.Store(function);
			odb.Close();
			odb = Open("big-data.neodatis");
			NeoDatis.Odb.Test.VO.Login.Function f2 = (NeoDatis.Odb.Test.VO.Login.Function)odb
				.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function)).GetFirst();
			AssertEquals(30000, f2.GetName().Length);
			odb.Close();
			odb = Open("big-data.neodatis");
			f2 = (NeoDatis.Odb.Test.VO.Login.Function)odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
				)).GetFirst();
			f2.SetName(f2.GetName() + "ola chico");
			int newSize = f2.GetName().Length;
			odb.Store(f2);
			odb.Close();
			odb = Open("big-data.neodatis");
			f2 = (NeoDatis.Odb.Test.VO.Login.Function)odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
				)).GetFirst();
			AssertEquals(newSize, f2.GetName().Length);
			AssertEquals(buffer.ToString() + "ola chico", f2.GetName());
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public override void SetUp()
		{
			DeleteBase("big-data.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		public override void TearDown()
		{
			DeleteBase("big-data.neodatis");
		}
	}
}

namespace NeoDatis.Odb.Test.Error
{
	public class TestError : NeoDatis.Odb.Test.ODBTest
	{
		/// <summary>Submitted by Tom Davies (tgdavies) Source forge Feature request 1900092</summary>
		public virtual void TestDollarInParam()
		{
			NeoDatis.Odb.Core.NeoDatisError e = new NeoDatis.Odb.Core.NeoDatisError(0, "x @1 y"
				);
			e.AddParameter("foo$bar");
			AssertEquals("0:x foo$bar y", e.ToString());
		}

		public virtual void Test2()
		{
			NeoDatis.Odb.Core.NeoDatisError e = new NeoDatis.Odb.Core.NeoDatisError(0, "x @1 @2 @3 @5 y"
				);
			e.AddParameter("param1");
			e.AddParameter("param2");
			e.AddParameter("param3");
			e.AddParameter("param4");
			AssertEquals("0:x param1 param2 param3 @5 y", e.ToString());
		}

		public virtual void Test3()
		{
			NeoDatis.Odb.Core.NeoDatisError e = new NeoDatis.Odb.Core.NeoDatisError(0, "x y");
			e.AddParameter("param1");
			e.AddParameter("param2");
			e.AddParameter("param3");
			e.AddParameter("param4");
			AssertEquals("0:x y", e.ToString());
		}

		public virtual void Test4()
		{
			NeoDatis.Odb.Core.NeoDatisError e = new NeoDatis.Odb.Core.NeoDatisError(12, "x @1 @2 @3 @5 y"
				);
			AssertEquals("12:x @1 @2 @3 @5 y", e.ToString());
		}
	}
}

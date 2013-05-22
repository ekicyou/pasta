namespace NeoDatis.Odb.Test.Acid
{
	public class TestConsistency : NeoDatis.Odb.Test.ODBTest
	{
		public static string OdbFileName = "consistency.neodatis";

		/// <exception cref="System.Exception"></exception>
		public virtual void CreateInconsistentFile()
		{
			NeoDatis.Odb.ODB odb = Open(OdbFileName);
			for (int i = 0; i < 10; i++)
			{
				object o = GetUserInstance();
				odb.Store(o);
			}
			odb.Close();
			odb = Open(OdbFileName);
			for (int i = 0; i < 10; i++)
			{
				object o = GetUserInstance();
				odb.Store(o);
			}
		}

		private NeoDatis.Odb.Test.VO.Attribute.TestClass GetTestClassInstance()
		{
			NeoDatis.Odb.Test.VO.Attribute.TestClass tc = new NeoDatis.Odb.Test.VO.Attribute.TestClass
				();
			tc.SetBigDecimal1(new System.Decimal(1.123456789));
			tc.SetBoolean1(true);
			tc.SetChar1('d');
			tc.SetDouble1(154.78998989);
			tc.SetInt1(78964);
			tc.SetString1("Ola chico como vc est√° ???");
			tc.SetDate1(new System.DateTime());
			return tc;
		}

		private object GetUserInstance()
		{
			NeoDatis.Odb.Test.VO.Login.Function login = new NeoDatis.Odb.Test.VO.Login.Function
				("login");
			NeoDatis.Odb.Test.VO.Login.Function logout = new NeoDatis.Odb.Test.VO.Login.Function
				("logout");
			System.Collections.Generic.IList<NeoDatis.Odb.Test.VO.Login.Function> list = new 
				System.Collections.Generic.List<NeoDatis.Odb.Test.VO.Login.Function>();
			list.Add(login);
			list.Add(logout);
			NeoDatis.Odb.Test.VO.Login.Profile profile = new NeoDatis.Odb.Test.VO.Login.Profile
				("operator", list);
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", profile);
			return user;
		}

		public virtual void Test1()
		{
			AssertTrue(true);
		}

		/// <exception cref="System.Exception"></exception>
		public static void Main2(string[] args)
		{
			new NeoDatis.Odb.Test.Acid.TestConsistency().CreateInconsistentFile();
		}
		// new TestConsistency().openFile();
	}
}

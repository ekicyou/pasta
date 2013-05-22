namespace NeoDatis.Odb.Test.VO.Login
{
	public class User2 : NeoDatis.Odb.Test.VO.Login.User
	{
		private int nbLogins;

		public User2(int nbLogins) : base()
		{
			this.nbLogins = nbLogins;
		}

		public User2() : base()
		{
		}

		public User2(string name, string email, NeoDatis.Odb.Test.VO.Login.Profile profile
			, int nbLogins) : base(name, email, profile)
		{
			this.nbLogins = nbLogins;
		}
	}
}

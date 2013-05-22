namespace NeoDatis.Odb.Test.VO.Login
{
	public class User
	{
		private string name;

		private NeoDatis.Odb.Test.VO.Login.Profile profile;

		private string email;

		public User()
		{
		}

		public User(string name, string email, NeoDatis.Odb.Test.VO.Login.Profile profile
			) : base()
		{
			this.name = name;
			this.email = email;
			this.profile = profile;
		}

		public virtual string GetEmail()
		{
			return email;
		}

		public virtual void SetEmail(string email)
		{
			this.email = email;
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public virtual NeoDatis.Odb.Test.VO.Login.Profile GetProfile()
		{
			return profile;
		}

		public virtual void SetProfile(NeoDatis.Odb.Test.VO.Login.Profile profile)
		{
			this.profile = profile;
		}

		public override string ToString()
		{
			return name + " - " + email + " - " + profile;
		}
	}
}

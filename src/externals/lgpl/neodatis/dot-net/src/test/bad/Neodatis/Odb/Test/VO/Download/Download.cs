namespace NeoDatis.Odb.Test.VO.Download
{
	public class Download
	{
		private string type;

		private string fileName;

		private System.DateTime when;

		private NeoDatis.Odb.Test.VO.Download.User user;

		public Download() : base()
		{
		}

		public virtual string GetFileName()
		{
			return fileName;
		}

		public virtual void SetFileName(string fileName)
		{
			this.fileName = fileName;
		}

		public virtual string GetType()
		{
			return type;
		}

		public virtual void SetType(string type)
		{
			this.type = type;
		}

		public virtual NeoDatis.Odb.Test.VO.Download.User GetUser()
		{
			return user;
		}

		public virtual void SetUser(NeoDatis.Odb.Test.VO.Download.User user)
		{
			this.user = user;
		}

		public virtual System.DateTime GetWhen()
		{
			return when;
		}

		public virtual void SetWhen(System.DateTime when)
		{
			this.when = when;
		}
	}
}

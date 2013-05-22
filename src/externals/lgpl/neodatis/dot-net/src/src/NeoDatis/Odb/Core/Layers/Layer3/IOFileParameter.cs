namespace NeoDatis.Odb.Core.Layers.Layer3
{
	/// <summary>Database Parameters for local database access</summary>
	/// <author>osmadja</author>
	public class IOFileParameter : NeoDatis.Odb.Core.Layers.Layer3.IBaseIdentification
	{
		private string fileName;

		private bool canWrite;

		private string user;

		private string password;

		public IOFileParameter(string name, bool write, string user, string password) : base
			()
		{
			fileName = name;
			canWrite = write;
			this.user = user;
			this.password = password;
		}

		public virtual bool CanWrite()
		{
			return canWrite;
		}

		public virtual void SetCanWrite(bool canWrite)
		{
			this.canWrite = canWrite;
		}

		public virtual string GetFileName()
		{
			return fileName;
		}

		public virtual void SetFileName(string fileName)
		{
			this.fileName = fileName;
		}

		public override string ToString()
		{
			return fileName;
		}

		public virtual string GetDirectory()
		{
			return new NeoDatis.Tool.Wrappers.IO.OdbFile(fileName).GetDirectory();
		}

		public virtual string GetCleanFileName()
		{
			return new NeoDatis.Tool.Wrappers.IO.OdbFile(fileName).GetCleanFileName();
		}

		public virtual string GetIdentification()
		{
			return GetCleanFileName();
		}

		public virtual bool IsNew()
		{
			return !NeoDatis.Tool.IOUtil.ExistFile(fileName);
		}

		public virtual bool IsLocal()
		{
			return true;
		}

		public virtual string GetUserName()
		{
			return user;
		}

		public virtual void SetUserName(string user)
		{
			this.user = user;
		}

		public virtual string GetPassword()
		{
			return password;
		}

		public virtual void SetPassword(string password)
		{
			this.password = password;
		}
	}
}

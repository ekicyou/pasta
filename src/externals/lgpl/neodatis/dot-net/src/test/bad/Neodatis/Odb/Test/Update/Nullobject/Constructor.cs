namespace NeoDatis.Odb.Test.Update.Nullobject
{
	/// <summary>Fornecedor</summary>
	/// <author>Jeremias</author>
	public class Constructor
	{
		private string name;

		private string description;

		private bool deleted;

		private System.DateTime creationDate;

		private System.DateTime updateDate;

		private NeoDatis.Odb.Test.Update.Nullobject.User user;

		// S ou N
		public virtual System.DateTime GetCreationDate()
		{
			return creationDate;
		}

		public virtual bool GetDeleted()
		{
			return deleted;
		}

		public virtual string GetDescription()
		{
			return description;
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual System.DateTime GetUpdateDate()
		{
			return updateDate;
		}

		public virtual NeoDatis.Odb.Test.Update.Nullobject.User GetUser()
		{
			return user;
		}

		public virtual void SetCreationDate(System.DateTime creationDate)
		{
			this.creationDate = creationDate;
		}

		public virtual void SetDeleted(bool deleted)
		{
			this.deleted = deleted;
		}

		public virtual void SetDescription(string description)
		{
			this.description = description;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public virtual void SetUpdateDate(System.DateTime updateDate)
		{
			this.updateDate = updateDate;
		}

		public virtual void SetUser(NeoDatis.Odb.Test.Update.Nullobject.User user)
		{
			this.user = user;
		}
	}
}

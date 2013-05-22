using NUnit.Framework;
namespace NeoDatis.Odb.Test.Update.Nullobject
{
	/// <summary>AT</summary>
	public class AT : NeoDatis.Odb.Test.Update.Nullobject.Device
	{
		private string ipAddress;

		private int port;

		private string physicalAddress;

		private string name;

		private string type;

		private bool deleted;

		private bool status;

		private NeoDatis.Odb.Test.Update.Nullobject.Constructor constructor;

		private System.DateTime creationDate;

		private System.DateTime updateDate;

		private NeoDatis.Odb.Test.Update.Nullobject.User user;

		public override string ToString()
		{
			return "[" + ipAddress + "][" + port + "][" + name + "][" + type + "]";
		}

		public virtual string GetIpAddress()
		{
			return ipAddress;
		}

		public virtual string GetType()
		{
			return type;
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual int GetPort()
		{
			return port;
		}

		public virtual NeoDatis.Odb.Test.Update.Nullobject.Constructor GetConstructor()
		{
			return constructor;
		}

		public virtual System.DateTime GetCreationDate()
		{
			return creationDate;
		}

		public virtual bool GetDeleted()
		{
			return deleted;
		}

		public virtual bool GetStatus()
		{
			return status;
		}

		public virtual System.DateTime GetUpdateDate()
		{
			return updateDate;
		}

		public virtual NeoDatis.Odb.Test.Update.Nullobject.User GetUser()
		{
			return user;
		}

		public virtual void SetConstructor(NeoDatis.Odb.Test.Update.Nullobject.Constructor
			 constructor)
		{
			this.constructor = constructor;
		}

		public virtual void SetCreationDate(System.DateTime creationDate)
		{
			this.creationDate = creationDate;
		}

		public virtual void SetDeleted(bool deleted)
		{
			this.deleted = deleted;
		}

		public virtual void SetStatus(bool status)
		{
			this.status = status;
		}

		public virtual void SetUpdateDate(System.DateTime updateDate)
		{
			this.updateDate = updateDate;
		}

		public virtual void SetUser(NeoDatis.Odb.Test.Update.Nullobject.User user)
		{
			this.user = user;
		}

		public virtual void SetIpAddress(string ipAddress)
		{
			this.ipAddress = ipAddress;
		}

		public virtual void SetType(string type)
		{
			this.type = type;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public virtual void SetPort(int port)
		{
			this.port = port;
		}

		public virtual string GetPhysicalAddress()
		{
			return physicalAddress;
		}

		public virtual void SetPhysicalAddress(string physicalAddress)
		{
			this.physicalAddress = physicalAddress;
		}
	}
}

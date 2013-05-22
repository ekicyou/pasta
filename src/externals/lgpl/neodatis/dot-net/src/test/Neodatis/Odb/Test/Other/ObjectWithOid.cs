using NUnit.Framework;
namespace NeoDatis.Odb.Test.Other
{
	public class ObjectWithOid
	{
		private string oid;

		private string name;

		public ObjectWithOid(string oid, string name) : base()
		{
			this.oid = oid;
			this.name = name;
		}

		public virtual string GetOid()
		{
			return oid;
		}

		public virtual void SetOid(string oid)
		{
			this.oid = oid;
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}
	}
}

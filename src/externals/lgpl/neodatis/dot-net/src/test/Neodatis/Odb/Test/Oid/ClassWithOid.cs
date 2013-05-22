using NUnit.Framework;
namespace NeoDatis.Odb.Test.Oid
{
	public class ClassWithOid
	{
		private string name;

		private NeoDatis.Odb.OID oid;

		public ClassWithOid(string name, NeoDatis.Odb.OID oid) : base()
		{
			this.name = name;
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

		public virtual NeoDatis.Odb.OID GetOid()
		{
			return oid;
		}

		public virtual void SetOid(NeoDatis.Odb.OID oid)
		{
			this.oid = oid;
		}
	}
}

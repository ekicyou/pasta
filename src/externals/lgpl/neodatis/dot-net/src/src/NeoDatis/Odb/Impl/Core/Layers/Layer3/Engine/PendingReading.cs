namespace NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine
{
	public class PendingReading
	{
		private int id;

		private NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci;

		private NeoDatis.Odb.OID attributeOID;

		public PendingReading(int id, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci, 
			NeoDatis.Odb.OID attributeOID) : base()
		{
			this.id = id;
			this.ci = ci;
			this.attributeOID = attributeOID;
		}

		public virtual int GetId()
		{
			return id;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo GetCi()
		{
			return ci;
		}

		public virtual NeoDatis.Odb.OID GetAttributeOID()
		{
			return attributeOID;
		}
	}
}

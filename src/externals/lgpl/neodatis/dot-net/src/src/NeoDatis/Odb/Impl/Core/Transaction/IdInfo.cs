namespace NeoDatis.Odb.Impl.Core.Transaction
{
	public class IdInfo
	{
		public NeoDatis.Odb.OID oid;

		public long position;

		public byte status;

		public IdInfo(NeoDatis.Odb.OID oid, long position, byte status) : base()
		{
			this.oid = oid;
			this.position = position;
			this.status = status;
		}
	}
}

namespace NeoDatis.Odb.Impl.Core.Transaction
{
	public class ObjectInsertingInfo
	{
		public NeoDatis.Odb.OID oid;

		public int level;

		public ObjectInsertingInfo(NeoDatis.Odb.OID oid, int level)
		{
			this.oid = oid;
			this.level = level;
		}
	}
}

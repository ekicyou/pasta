namespace NeoDatis.Odb
{
	public interface ExternalOID : NeoDatis.Odb.OID
	{
		NeoDatis.Odb.DatabaseId GetDatabaseId();
	}
}

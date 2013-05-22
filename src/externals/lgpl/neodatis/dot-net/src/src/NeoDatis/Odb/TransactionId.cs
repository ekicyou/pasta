namespace NeoDatis.Odb
{
	public interface TransactionId
	{
		long GetId1();

		long GetId2();

		NeoDatis.Odb.DatabaseId GetDatabaseId();

		NeoDatis.Odb.TransactionId Next();

		NeoDatis.Odb.TransactionId Prev();
	}
}

namespace NeoDatis.Odb.Core.Transaction
{
	public interface IWriteAction
	{
		byte[] GetBytes(int index);

		void ApplyTo(NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface fsi, int
			 index);

		void AddBytes(byte[] bytes);

		void Persist(NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface fsi, int
			 index);

		bool IsEmpty();

		long GetPosition();
	}
}

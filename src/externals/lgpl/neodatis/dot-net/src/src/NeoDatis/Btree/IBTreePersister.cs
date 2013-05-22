namespace NeoDatis.Btree
{
	/// <summary>
	/// Interface used to persist and load btree and btree node from a persistent
	/// layer
	/// </summary>
	/// <author>osmadja</author>
	public interface IBTreePersister
	{
		NeoDatis.Btree.IBTreeNode LoadNodeById(object id);

		object SaveNode(NeoDatis.Btree.IBTreeNode node);

		NeoDatis.Odb.OID SaveBTree(NeoDatis.Btree.IBTree tree);

		NeoDatis.Btree.IBTree LoadBTree(object id);

		/// <exception cref="System.Exception"></exception>
		void Close();

		object DeleteNode(NeoDatis.Btree.IBTreeNode parent);

		void SetBTree(NeoDatis.Btree.IBTree tree);

		void Clear();

		void Flush();
	}
}

namespace NeoDatis.Btree.Impl
{
	/// <summary>TODO check if this class must exist</summary>
	/// <author>osmadja</author>
	public class InMemoryPersister : NeoDatis.Btree.IBTreePersister
	{
		public virtual NeoDatis.Btree.IBTreeNode LoadNodeById(object id)
		{
			// TODO Auto-generated method stub
			return null;
		}

		public virtual object SaveNode(NeoDatis.Btree.IBTreeNode node)
		{
			// TODO Auto-generated method stub
			return null;
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Close()
		{
		}

		// TODO Auto-generated method stub
		public virtual object DeleteNode(NeoDatis.Btree.IBTreeNode parent)
		{
			// TODO Auto-generated method stub
			return null;
		}

		public virtual NeoDatis.Btree.IBTree LoadBTree(object id)
		{
			// TODO Auto-generated method stub
			return null;
		}

		public virtual NeoDatis.Odb.OID SaveBTree(NeoDatis.Btree.IBTree tree)
		{
			// TODO Auto-generated method stub
			return null;
		}

		public virtual void SetBTree(NeoDatis.Btree.IBTree tree)
		{
		}

		// TODO Auto-generated method stub
		public virtual void Clear()
		{
		}

		// TODO Auto-generated method stub
		public virtual void Flush()
		{
		}
		// TODO Auto-generated method stub
	}
}

namespace NeoDatis.Btree
{
	public interface IBTree
	{
		void Insert(System.IComparable key, object value);

		void Split(NeoDatis.Btree.IBTreeNode parent, NeoDatis.Btree.IBTreeNode node2Split
			, int childIndex);

		object Delete(System.IComparable key, object value);

		int GetDegree();

		long GetSize();

		int GetHeight();

		NeoDatis.Btree.IBTreeNode GetRoot();

		NeoDatis.Btree.IBTreePersister GetPersister();

		void SetPersister(NeoDatis.Btree.IBTreePersister persister);

		NeoDatis.Btree.IBTreeNode BuildNode();

		object GetId();

		void SetId(object id);

		void Clear();

		NeoDatis.Btree.IKeyAndValue GetBiggest(NeoDatis.Btree.IBTreeNode node, bool delete
			);

		NeoDatis.Btree.IKeyAndValue GetSmallest(NeoDatis.Btree.IBTreeNode node, bool delete
			);

		System.Collections.IEnumerator Iterator<T>(NeoDatis.Odb.Core.OrderByConstants orderBy
			);
	}
}

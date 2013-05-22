namespace NeoDatis.Odb.Impl.Core.Query.List.Objects
{
	/// <summary>An implementation of an ordered Collection based on a BTree implementation that holds all objects in memory
	/// 	</summary>
	/// <author>osmadja</author>
	[System.Serializable]
	public class InMemoryBTreeCollection<T> : NeoDatis.Odb.Impl.Core.Query.List.Objects.AbstractBTreeCollection
		<T>
	{
		public InMemoryBTreeCollection(int size) : base(size, NeoDatis.Odb.Core.OrderByConstants
			.OrderByAsc)
		{
		}

		public InMemoryBTreeCollection(int size, NeoDatis.Odb.Core.OrderByConstants orderByType
			) : base(size, orderByType)
		{
		}

		public InMemoryBTreeCollection()
		{
		}

		public override NeoDatis.Btree.IBTree BuildTree(int degree)
		{
			return new NeoDatis.Btree.Impl.Multiplevalue.InMemoryBTreeMultipleValuesPerKey("default"
				, degree);
		}
	}
}

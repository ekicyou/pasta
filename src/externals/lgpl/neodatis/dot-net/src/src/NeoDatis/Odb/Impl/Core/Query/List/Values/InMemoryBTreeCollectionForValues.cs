using System;
namespace NeoDatis.Odb.Impl.Core.Query.List.Values
{
	/// <summary>An ordered Collection to hold values (not objects) based on a BTree implementation.
	/// 	</summary>
	/// <remarks>An ordered Collection to hold values (not objects) based on a BTree implementation. It holds all values in memory.
	/// 	</remarks>
	/// <author>osmadja</author>
	[System.Serializable]
	public class InMemoryBTreeCollectionForValues : NeoDatis.Odb.Impl.Core.Query.List.Objects.AbstractBTreeCollection
		<NeoDatis.Odb.ObjectValues>, NeoDatis.Odb.Values
	{
		public InMemoryBTreeCollectionForValues(int size) : base(size, NeoDatis.Odb.Core.OrderByConstants
			.OrderByAsc)
		{
		}

		public InMemoryBTreeCollectionForValues(int size, NeoDatis.Odb.Core.OrderByConstants
			 orderByType) : base(size, orderByType)
		{
		}

		public InMemoryBTreeCollectionForValues()
		{
		}

		public override NeoDatis.Btree.IBTree BuildTree(int degree)
		{
			return new NeoDatis.Btree.Impl.Multiplevalue.InMemoryBTreeMultipleValuesPerKey("default"
				, degree);
		}

		public virtual NeoDatis.Odb.ObjectValues NextValues()
		{
			return Next();
		}
        public void AddOid(OID oid)
        {
            throw new Exception("Add Oid not implemented ");
        }
	}
}

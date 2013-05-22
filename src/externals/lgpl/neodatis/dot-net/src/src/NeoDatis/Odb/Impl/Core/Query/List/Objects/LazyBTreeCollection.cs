using NeoDatis.Odb.Impl.Core.Btree;
namespace NeoDatis.Odb.Impl.Core.Query.List.Objects
{
	/// <summary>A collection using a BTtree as a back-end component.</summary>
	/// <remarks>
	/// A collection using a BTtree as a back-end component. Lazy because it only keeps the oids of the objects. When asked for an object, loads
	/// it on demand and returns  it
	/// </remarks>
	/// <author>osmadja</author>
	[System.Serializable]
	public class LazyBTreeCollection<T> : NeoDatis.Odb.Impl.Core.Query.List.Objects.AbstractBTreeCollection
		<T>
	{
		private NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine;

		private bool returnObjects;

		public LazyBTreeCollection(int size, NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 engine, bool returnObjects) : base(size, NeoDatis.Odb.Core.OrderByConstants.OrderByAsc
			)
		{
			this.storageEngine = engine;
			this.returnObjects = returnObjects;
		}

		public LazyBTreeCollection(int size, NeoDatis.Odb.Core.OrderByConstants orderByType
			) : base(size, orderByType)
		{
		}

		public override NeoDatis.Btree.IBTree BuildTree(int degree)
		{
			return new NeoDatis.Btree.Impl.Multiplevalue.InMemoryBTreeMultipleValuesPerKey("default"
				, degree);
		}

		public override System.Collections.Generic.IEnumerator<T> Iterator(NeoDatis.Odb.Core.OrderByConstants
			 orderByType)
		{
			return (System.Collections.Generic.IEnumerator<T>)new LazyODBBTreeIteratorMultiple<T>(GetTree(), orderByType, storageEngine, returnObjects);
		}
	}
}

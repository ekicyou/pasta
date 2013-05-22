namespace NeoDatis.Odb.Impl.Core.Btree
{
	/// <summary>
	/// A Lazy BTree Iterator : It iterate on the object OIDs and lazy load objects from them (OIDs)
	/// Used by the LazyBTreeCollection
	/// </summary>
	/// <author>osmadja</author>
	public class LazyODBBTreeIteratorSIngle<T> : NeoDatis.Btree.BTreeIteratorSingleValuePerKey<T>
	{
		private NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine;

		private bool returnObjects;

		/// <param name="tree"></param>
		/// <param name="orderByType"></param>
		/// <param name="storageEngine"></param>
		/// <param name="returnObjects"></param>
		public LazyODBBTreeIteratorSIngle(NeoDatis.Btree.IBTree tree, NeoDatis.Odb.Core.OrderByConstants
			 orderByType, NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine, bool
			 returnObjects) : base(tree, orderByType)
		{
			this.storageEngine = storageEngine;
			this.returnObjects = returnObjects;
		}

		public override T Current
		{
			get
			{
				NeoDatis.Odb.OID oid = (NeoDatis.Odb.OID)base.Current;
				try
				{
					return (T)LoadObject(oid);
				}
				catch (System.Exception e)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Btree.BTreeError.LazyLoadingNode
						.AddParameter(oid), e);
				}
			}
		}

		/// <exception cref="System.Exception"></exception>
		private object LoadObject(NeoDatis.Odb.OID oid)
		{
			// true = to use cache
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)storageEngine.GetObjectReader().ReadNonNativeObjectInfoFromOid(null, oid, true, 
				returnObjects);
			if (returnObjects)
			{
				object o = nnoi.GetObject();
				if (o != null)
				{
					return o;
				}
				return storageEngine.GetObjectReader().BuildOneInstance(nnoi);
			}
			return nnoi;
		}
	}
}

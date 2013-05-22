using NeoDatis.Odb.Impl.Core.Query.List.Objects;
using NeoDatis.Tool.Wrappers;
using NeoDatis.Odb.Core.Layers.Layer2.Meta;
namespace NeoDatis.Odb.Impl.Core.Query.Criteria
{
	/// <summary>Class that manage normal query.</summary>
	/// <remarks>
	/// Class that manage normal query. Query that return a list of objects. For each object
	/// That matches the query criteria, the objectMatch method is called and it keeps the objects in the 'objects' instance.
	/// </remarks>
	/// <author>olivier</author>
	public class CollectionQueryResultAction<T> : NeoDatis.Odb.Core.Query.Execution.IMatchingObjectAction
	{
		private NeoDatis.Odb.Core.Query.IQuery query;

		private bool inMemory;

		private long nbObjects;

		private NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine;

		private bool returnObjects;

		private NeoDatis.Odb.Objects<T> result;

		private bool queryHasOrderBy;

		/// <summary>An object to build instances</summary>
		protected NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder instanceBuilder;

		public CollectionQueryResultAction(NeoDatis.Odb.Core.Query.IQuery query, bool inMemory
			, NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine, bool returnObjects
			, NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder instanceBuilder) : base
			()
		{
			// TODO check if Object is ok here
			this.query = query;
			this.inMemory = inMemory;
			this.storageEngine = storageEngine;
			this.returnObjects = returnObjects;
			this.queryHasOrderBy = query.HasOrderBy();
			this.instanceBuilder = instanceBuilder;
		}

		public virtual void ObjectMatch(NeoDatis.Odb.OID oid, NeoDatis.Tool.Wrappers.OdbComparable
			 orderByKey)
		{
			if (queryHasOrderBy)
			{
				result.AddWithKey(orderByKey, (T)oid);
			}
			else
			{
				result.Add((T)oid);
			}
		}

		public virtual void ObjectMatch(OID oid, object o, OdbComparable orderByKey)
		{
			NonNativeObjectInfo nnoi = (NonNativeObjectInfo)o;
			if (inMemory)
			{
				if (returnObjects)
				{
					if (queryHasOrderBy)
					{
						result.AddWithKey(orderByKey,(T) GetCurrentInstance(nnoi));
					}
					else
					{
						result.Add((T)GetCurrentInstance(nnoi));
					}
				}
				else
				{
					if (queryHasOrderBy)
					{
						//result.AddWithKey(orderByKey, (T)nnoi);
					}
					else
					{
						//result.Add((T)nnoi);
					}
				}
			}
			else
			{
				if (queryHasOrderBy)
				{
					result.AddWithKey(orderByKey,(T) oid);
				}
				else
				{
					result.AddOid(oid);
				}
			}
		}

		public virtual void Start()
		{
			if (inMemory)
			{
				if (query != null && query.HasOrderBy())
				{
					result = new InMemoryBTreeCollection<T>((int)nbObjects, query.GetOrderByType());
				}
				else
				{
					result = new SimpleList<T>((int)nbObjects);
				}
			}
			else
			{
				// result = new InMemoryBTreeCollection((int) nbObjects);
				if (query != null && query.HasOrderBy())
				{
					result = new LazyBTreeCollection<T>((int)nbObjects, storageEngine, returnObjects);
				}
				else
				{
					result = new LazySimpleListFromOid<T>((int)nbObjects, storageEngine, returnObjects);
				}
			}
		}

		public virtual void End()
		{
		}

		public virtual object GetCurrentInstance(NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi)
		{
			//FIXME no need
			if (nnoi.GetObject() != null)
			{
				return nnoi.GetObject();
			}
			return instanceBuilder.BuildOneInstance(nnoi);
		}

		public virtual NeoDatis.Odb.Objects<T> GetObjects<T>()
		{
			return (NeoDatis.Odb.Objects<T>) result;
		}
	}
}

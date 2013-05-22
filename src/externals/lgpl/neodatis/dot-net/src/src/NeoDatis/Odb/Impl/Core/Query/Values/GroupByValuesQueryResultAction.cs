namespace NeoDatis.Odb.Impl.Core.Query.Values
{
	public class GroupByValuesQueryResultAction : NeoDatis.Odb.Core.Query.Execution.IMatchingObjectAction
	{
		private NeoDatis.Odb.Core.Query.IValuesQuery query;

		private long nbObjects;

		/// <summary>
		/// When executing a group by result, results are temporary stored in a hash map and at the end transfered to a Values objects
		/// In this case, the key of the map is the group by composed key, the value is a ValuesQueryResultAction
		/// </summary>
		private System.Collections.Generic.IDictionary<NeoDatis.Tool.Wrappers.OdbComparable
			, NeoDatis.Odb.Impl.Core.Query.Values.ValuesQueryResultAction> groupByResult;

		private NeoDatis.Odb.Values result;

		private bool queryHasOrderBy;

		/// <summary>An object to build instances</summary>
		protected NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder instanceBuilder;

		protected NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo;

		private int returnArraySize;

		private string[] groupByFieldList;

		public GroupByValuesQueryResultAction(NeoDatis.Odb.Core.Query.IValuesQuery query, 
			NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine, NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder
			 instanceBuilder) : base()
		{
			this.query = query;
			this.queryHasOrderBy = query.HasOrderBy();
			this.instanceBuilder = instanceBuilder;
			this.returnArraySize = query.GetObjectActions().Count;
			this.groupByFieldList = query.GetGroupByFieldList();
			this.groupByResult = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<NeoDatis.Tool.Wrappers.OdbComparable
				, NeoDatis.Odb.Impl.Core.Query.Values.ValuesQueryResultAction>();
		}

		public virtual void ObjectMatch(NeoDatis.Odb.OID oid, NeoDatis.Tool.Wrappers.OdbComparable
			 orderByKey)
		{
		}

		// This method os not used in Values Query API
		public virtual void ObjectMatch(NeoDatis.Odb.OID oid, object @object, NeoDatis.Tool.Wrappers.OdbComparable
			 orderByKey)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap values = (NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap
				)@object;
			NeoDatis.Tool.Wrappers.OdbComparable groupByKey = NeoDatis.Odb.Core.Query.Execution.IndexTool
				.BuildIndexKey("GroupBy", values, groupByFieldList);
			NeoDatis.Odb.Impl.Core.Query.Values.ValuesQueryResultAction result = groupByResult
				[groupByKey];
			if (result == null)
			{
				result = new NeoDatis.Odb.Impl.Core.Query.Values.ValuesQueryResultAction(query, null
					, instanceBuilder);
				result.Start();
				groupByResult.Add(groupByKey, result);
			}
			result.ObjectMatch(oid, @object, orderByKey);
		}

		public virtual void Start()
		{
		}

		// Nothing to do
		public virtual void End()
		{
			if (query != null && query.HasOrderBy())
			{
				result = new NeoDatis.Odb.Impl.Core.Query.List.Values.InMemoryBTreeCollectionForValues
					((int)nbObjects, query.GetOrderByType());
			}
			else
			{
				result = new NeoDatis.Odb.Impl.Core.Query.List.Values.SimpleListForValues((int)nbObjects
					);
			}
			System.Collections.IEnumerator iterator = groupByResult.Keys.GetEnumerator();
			NeoDatis.Odb.Impl.Core.Query.Values.ValuesQueryResultAction vqra = null;
			NeoDatis.Tool.Wrappers.OdbComparable key = null;
			while (iterator.MoveNext())
			{
				key = (NeoDatis.Tool.Wrappers.OdbComparable)iterator.Current;
				vqra = (NeoDatis.Odb.Impl.Core.Query.Values.ValuesQueryResultAction)groupByResult
					[key];
				vqra.End();
				Merge(key, vqra.GetValues());
			}
		}

		private void Merge(NeoDatis.Tool.Wrappers.OdbComparable key, NeoDatis.Odb.Values 
			values)
		{
			while (values.HasNext())
			{
				if (queryHasOrderBy)
				{
					result.AddWithKey(key, values.NextValues());
				}
				else
				{
					result.Add(values.NextValues());
				}
			}
		}

		public virtual NeoDatis.Odb.Objects<T> GetObjects<T>()
		{
			return (NeoDatis.Odb.Objects<T>)result;
		}
	}
}

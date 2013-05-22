namespace NeoDatis.Odb.Impl.Core.Query.NQ
{
	public class NativeQueryExecutor : NeoDatis.Odb.Core.Query.Execution.GenericQueryExecutor
	{
		private object currentObject;

		private NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder instanceBuilder;

		public NativeQueryExecutor(NeoDatis.Odb.Core.Query.IQuery query, NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 engine, NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder instanceBuilder
			) : base(query, engine)
		{
			this.instanceBuilder = instanceBuilder;
		}

		public override NeoDatis.Odb.Core.Query.Execution.IQueryExecutionPlan GetExecutionPlan
			()
		{
			NeoDatis.Odb.Core.Query.Execution.IQueryExecutionPlan plan = new NeoDatis.Odb.Impl.Core.Query.NQ.NativeQueryExecutionPlan
				(classInfo, query);
			return plan;
		}

		public override void PrepareQuery()
		{
		}

		/// <summary>
		/// Check if the object at position currentPosition matches the query, returns true
		/// This method must compute the next object position and the orderBy key if it exists!
		/// </summary>
		public override bool MatchObjectWithOid(NeoDatis.Odb.OID oid, bool loadObjectInfo
			, bool inMemory)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo aoitemp = objectReader.ReadNonNativeObjectInfoFromOid
				(classInfo, oid, true, true);
			bool objectMatches = false;
			if (!aoitemp.IsDeletedObject())
			{
				currentNnoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo)aoitemp;
				currentObject = instanceBuilder.BuildOneInstance(currentNnoi);
				objectMatches = query == null || NeoDatis.Odb.Core.Query.QueryManager.Match(query
					, currentObject);
				nextOID = currentNnoi.GetNextObjectOID();
			}
			return objectMatches;
		}

		public override System.IComparable ComputeIndexKey(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 ci, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex index)
		{
			return null;
		}

		public virtual System.IComparable BuildOrderByKey()
		{
			return NeoDatis.Odb.Core.Query.Execution.IndexTool.BuildIndexKey("OrderBy", currentNnoi
				, NeoDatis.Odb.Core.Query.QueryManager.GetOrderByAttributeIds(classInfo, query));
		}

		/// <exception cref="System.Exception"></exception>
		public virtual object GetCurrentInstance()
		{
			return currentObject;
		}

		public override object GetCurrentObjectMetaRepresentation()
		{
			return currentNnoi;
		}
	}
}

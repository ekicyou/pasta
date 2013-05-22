namespace NeoDatis.Odb.Impl.Core.Query.Values
{
	public class ValuesCriteriaQueryExecutor : NeoDatis.Odb.Core.Query.Execution.GenericQueryExecutor
	{
		private NeoDatis.Tool.Wrappers.List.IOdbList<string> involvedFields;

		private NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery criteriaQuery;

		private NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap values;

		public ValuesCriteriaQueryExecutor(NeoDatis.Odb.Core.Query.IQuery query, NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 engine) : base(query, engine)
		{
			criteriaQuery = (NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery)query;
		}

		public override NeoDatis.Odb.Core.Query.Execution.IQueryExecutionPlan GetExecutionPlan
			()
		{
			NeoDatis.Odb.Core.Query.Execution.IQueryExecutionPlan plan = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQueryExecutionPlan
				(classInfo, (NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery)query);
			return plan;
		}

		public override void PrepareQuery()
		{
			criteriaQuery = (NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery)query;
			criteriaQuery.SetStorageEngine(storageEngine);
			involvedFields = criteriaQuery.GetAllInvolvedFields();
		}

		public override bool MatchObjectWithOid(NeoDatis.Odb.OID oid, bool returnObject, 
			bool inMemory)
		{
			currentOid = oid;
			// Gets a map with the values with the fields involved in the query
			values = objectReader.ReadObjectInfoValuesFromOID(classInfo, currentOid, true, involvedFields
				, involvedFields, 0, criteriaQuery.GetOrderByFieldNames());
			bool objectMatches = true;
			if (!criteriaQuery.IsForSingleOid())
			{
				// Then apply the query on the field values
				objectMatches = NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQueryManager.Match(
					criteriaQuery, values);
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih = values.GetObjectInfoHeader
				();
			// Stores the next position
			nextOID = oih.GetNextObjectOID();
			return objectMatches;
		}

		public override System.IComparable ComputeIndexKey(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 ci, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex index)
		{
			return NeoDatis.Odb.Core.Query.Execution.IndexTool.ComputeKey(classInfo, index, (
				NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery)query);
		}

		public override object GetCurrentObjectMetaRepresentation()
		{
			return values;
		}
	}
}

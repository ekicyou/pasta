using NeoDatis.Odb.Core.Layers.Layer2.Meta;
using NeoDatis.Odb.Core.Transaction;
namespace NeoDatis.Odb.Impl.Core.Query.Criteria
{
	public class CriteriaQueryExecutor : NeoDatis.Odb.Core.Query.Execution.GenericQueryExecutor
	{
		private NeoDatis.Tool.Wrappers.List.IOdbList<string> involvedFields;

		private NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery criteriaQuery;

		public CriteriaQueryExecutor(NeoDatis.Odb.Core.Query.IQuery query, NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
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
			ITmpCache tmpCache = session.GetTmpCache();
			ObjectInfoHeader oih = null;
			try
			{
				if (!criteriaQuery.HasCriteria())
				{
					// true, false = use cache, false = do not return object
					// TODO Warning setting true to useCache will put all objects in
					// the cache
					// This is not a good idea for big queries!, But use cache=true
					// resolves when object have not been committed yet!
					// for big queries, user should use a LazyCache!
					if (inMemory)
					{
						currentNnoi = objectReader.ReadNonNativeObjectInfoFromOid(classInfo, currentOid, 
							true, returnObject);
						if (currentNnoi.IsDeletedObject())
						{
							return false;
						}
						currentOid = currentNnoi.GetOid();
						nextOID = currentNnoi.GetNextObjectOID();
					}
					else
					{
						oih = objectReader.ReadObjectInfoHeaderFromOid(currentOid, false);
						nextOID = oih.GetNextObjectOID();
					}
					return true;
				}
				// Gets a map with the values with the fields involved in the query
				AttributeValuesMap attributeValues = objectReader.ReadObjectInfoValuesFromOID(classInfo, currentOid, true, involvedFields, involvedFields
					, 0, criteriaQuery.GetOrderByFieldNames());
				// Then apply the query on the field values
				bool objectMatches =CriteriaQueryManager.Match(criteriaQuery, attributeValues);
				if (objectMatches)
				{
					// Then load the entire object
					// true, false = use cache
					currentNnoi = objectReader.ReadNonNativeObjectInfoFromOid(classInfo, currentOid, 
						true, returnObject);
					currentOid = currentNnoi.GetOid();
				}
				oih = attributeValues.GetObjectInfoHeader();
				// Stores the next position
				nextOID = oih.GetNextObjectOID();
				return objectMatches;
			}
			finally
			{
				tmpCache.ClearObjectInfos();
			}
		}

		public override System.IComparable ComputeIndexKey(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 ci, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex index)
		{
			NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery q = (NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				)query;
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap values = q.GetCriteria().
				GetValues();
			// if values.hasOid() is true, this means that we are working of the full object,
			// the index key is then the oid and not the object itself
			if (values.HasOid())
			{
				return new NeoDatis.Odb.Core.Query.SimpleCompareKey(values.GetOid());
			}
			return NeoDatis.Odb.Core.Query.Execution.IndexTool.ComputeKey(classInfo, index, (
				NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery)query);
		}

		public override object GetCurrentObjectMetaRepresentation()
		{
			return currentNnoi;
		}
	}
}

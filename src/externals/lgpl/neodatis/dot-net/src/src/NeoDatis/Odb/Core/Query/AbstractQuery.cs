using System;
namespace NeoDatis.Odb.Core.Query
{
	[System.Serializable]
	public abstract class AbstractQuery : NeoDatis.Odb.Core.Query.IQuery
	{
		protected string[] orderByFields;

		protected NeoDatis.Odb.Core.OrderByConstants orderByType;

		[System.NonSerialized]
		protected NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine;

		protected NeoDatis.Odb.Core.Query.Execution.IQueryExecutionPlan executionPlan;

		protected bool polymorphic;

		/// <summary>The OID attribute is used when the query must be restricted the object with this OID
		/// 	</summary>
		protected NeoDatis.Odb.OID oidOfObjectToQuery;

		public AbstractQuery()
		{
			orderByType = NeoDatis.Odb.Core.OrderByConstants.OrderByNone;
			polymorphic = false;
		}

		public virtual NeoDatis.Odb.Core.Query.IQuery OrderByDesc(string fields)
		{
			orderByType = NeoDatis.Odb.Core.OrderByConstants.OrderByDesc;
			orderByFields = NeoDatis.Tool.Wrappers.OdbString.Split(fields, ",");
			return this;
		}

		public virtual NeoDatis.Odb.Core.Query.IQuery OrderByAsc(string fields)
		{
			orderByType = NeoDatis.Odb.Core.OrderByConstants.OrderByAsc;
			orderByFields = NeoDatis.Tool.Wrappers.OdbString.Split(fields, ",");
			return this;
		}

		public virtual string[] GetOrderByFieldNames()
		{
			return orderByFields;
		}

		public virtual void SetOrderByFields(string[] orderByFields)
		{
			this.orderByFields = orderByFields;
		}

		public virtual NeoDatis.Odb.Core.OrderByConstants GetOrderByType()
		{
			return orderByType;
		}

		public virtual void SetOrderByType(NeoDatis.Odb.Core.OrderByConstants orderByType
			)
		{
			this.orderByType = orderByType;
		}

		public virtual bool HasOrderBy()
		{
			return !orderByType.IsOrderByNone();
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine GetStorageEngine()
		{
			return storageEngine;
		}

		public virtual void SetStorageEngine(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 storageEngine)
		{
			this.storageEngine = storageEngine;
		}

		public virtual NeoDatis.Odb.Core.Query.Execution.IQueryExecutionPlan GetExecutionPlan
			()
		{
			if (executionPlan == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ExecutionPlanIsNullQueryHasNotBeenExecuted
					);
			}
			return executionPlan;
		}

		public virtual void SetExecutionPlan(NeoDatis.Odb.Core.Query.Execution.IQueryExecutionPlan
			 plan)
		{
			executionPlan = plan;
		}

		public virtual bool IsPolymorphic()
		{
			return polymorphic;
		}

		public virtual NeoDatis.Odb.Core.Query.IQuery SetPolymorphic(bool yes)
		{
			polymorphic = yes;
			return this;
		}

		public virtual NeoDatis.Odb.OID GetOidOfObjectToQuery()
		{
			return oidOfObjectToQuery;
		}

		public virtual void SetOidOfObjectToQuery(NeoDatis.Odb.OID oidOfObjectToQuery)
		{
			this.oidOfObjectToQuery = oidOfObjectToQuery;
		}

		/// <summary>Returns true is query must apply on a single object OID</summary>
		public virtual bool IsForSingleOid()
		{
			return oidOfObjectToQuery != null;
		}
        public abstract void SetFullClassName(Type type);
    }
}

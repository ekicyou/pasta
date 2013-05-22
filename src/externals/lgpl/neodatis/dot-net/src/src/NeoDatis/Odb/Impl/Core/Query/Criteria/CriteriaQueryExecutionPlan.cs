namespace NeoDatis.Odb.Impl.Core.Query.Criteria
{
	/// <summary>
	/// A simple Criteria execution plan Check if the query can use index and tries
	/// to find the best index to be used
	/// </summary>
	/// <author>osmadja</author>
	[System.Serializable]
	public class CriteriaQueryExecutionPlan : NeoDatis.Odb.Core.Query.Execution.IQueryExecutionPlan
	{
		[System.NonSerialized]
		protected NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo;

		[System.NonSerialized]
		protected NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery query;

		protected bool useIndex;

		[System.NonSerialized]
		protected NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex classInfoIndex;

		/// <summary>to keep track of the start date time of the plan</summary>
		protected long start;

		/// <summary>to keep track of the end date time of the plan</summary>
		protected long end;

		/// <summary>To keep the execution detail</summary>
		protected string details;

		public CriteriaQueryExecutionPlan()
		{
		}

		public CriteriaQueryExecutionPlan(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo 
			classInfo, NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery query)
		{
			this.classInfo = classInfo;
			this.query = query;
			this.query.SetExecutionPlan(this);
			Init();
		}

		protected virtual void Init()
		{
			start = 0;
			end = 0;
			// for instance, only manage index for one field query using 'equal'
			if (classInfo.HasIndex() && query.HasCriteria() && CanUseIndex(query.GetCriteria(
				)))
			{
				NeoDatis.Tool.Wrappers.List.IOdbList<string> fields = query.GetAllInvolvedFields(
					);
				if (fields.IsEmpty())
				{
					useIndex = false;
				}
				else
				{
					int[] fieldIds = GetAllInvolvedFieldIds(fields);
					classInfoIndex = classInfo.GetIndexForAttributeIds(fieldIds);
					if (classInfoIndex != null)
					{
						useIndex = true;
					}
				}
			}
			// Keep the detail
			details = GetDetails();
		}

		/// <summary>Transform a list of field names into a list of field ids</summary>
		/// <param name="fields"></param>
		/// <returns>The array of field ids</returns>
		protected virtual int[] GetAllInvolvedFieldIds(NeoDatis.Tool.Wrappers.List.IOdbList
			<string> fields)
		{
			int nbFields = fields.Count;
			int[] fieldIds = new int[nbFields];
			for (int i = 0; i < nbFields; i++)
			{
				fieldIds[i] = classInfo.GetAttributeId(fields[i].ToString());
			}
			return fieldIds;
		}

		private bool CanUseIndex(NeoDatis.Odb.Core.Query.Criteria.ICriterion criteria)
		{
			return criteria.CanUseIndex();
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex GetIndex()
		{
			return classInfoIndex;
		}

		public virtual bool UseIndex()
		{
			return useIndex;
		}

		public virtual string GetDetails()
		{
			if (details != null)
			{
				return details;
			}
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			if (classInfoIndex == null)
			{
				buffer.Append("No index used, Execution time=").Append(GetDuration()).Append("ms"
					);
				return buffer.ToString();
			}
			return buffer.Append("Following indexes have been used : ").Append(classInfoIndex
				.GetName()).Append(", Execution time=").Append(GetDuration()).Append("ms").ToString
				();
		}

		public virtual void End()
		{
			end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
		}

		public virtual long GetDuration()
		{
			return (end - start);
		}

		public virtual void Start()
		{
			start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
		}
	}
}

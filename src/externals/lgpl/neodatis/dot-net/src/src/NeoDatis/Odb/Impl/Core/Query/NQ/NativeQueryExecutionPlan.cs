namespace NeoDatis.Odb.Impl.Core.Query.NQ
{
	/// <summary>
	/// A simple Criteria execution plan
	/// Check if the query can use index and tries to find the best index to be used
	/// </summary>
	/// <author>osmadja</author>
	[System.Serializable]
	public class NativeQueryExecutionPlan : NeoDatis.Odb.Core.Query.Execution.IQueryExecutionPlan
	{
		protected NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo;

		protected bool useIndex;

		protected NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex classInfoIndex;

		protected NeoDatis.Odb.Core.Query.IQuery query;

		/// <summary>to keep track of the start date time of the plan</summary>
		protected long start;

		/// <summary>to keep track of the end date time of the plan</summary>
		protected long end;

		public NativeQueryExecutionPlan(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo
			, NeoDatis.Odb.Core.Query.IQuery query)
		{
			this.classInfo = classInfo;
			this.query = query;
			query.SetExecutionPlan(this);
			Init();
		}

		protected virtual void Init()
		{
			useIndex = false;
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

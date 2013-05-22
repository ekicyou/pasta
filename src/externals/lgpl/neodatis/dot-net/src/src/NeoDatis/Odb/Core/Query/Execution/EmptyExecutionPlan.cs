namespace NeoDatis.Odb.Core.Query.Execution
{
	[System.Serializable]
	public class EmptyExecutionPlan : NeoDatis.Odb.Core.Query.Execution.IQueryExecutionPlan
	{
		public virtual void End()
		{
		}

		public virtual string GetDetails()
		{
			return "empty plan";
		}

		public virtual long GetDuration()
		{
			return 0;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex GetIndex()
		{
			return null;
		}

		public virtual void Start()
		{
		}

		public virtual bool UseIndex()
		{
			return false;
		}
	}
}

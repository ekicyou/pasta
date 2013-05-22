namespace NeoDatis.Odb.Core.Query.Execution
{
	public interface IQueryExecutionPlan
	{
		bool UseIndex();

		NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex GetIndex();

		string GetDetails();

		long GetDuration();

		void Start();

		void End();
	}
}

namespace NeoDatis.Odb.Core.Server.Message
{
	/// <summary>A response to a GetObjectValuesMessage command</summary>
	/// <author>olivier s</author>
	[System.Serializable]
	public class GetObjectValuesMessageResponse : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		/// <summary>List of values</summary>
		private NeoDatis.Odb.Values values;

		private NeoDatis.Odb.Core.Query.Execution.IQueryExecutionPlan plan;

		public GetObjectValuesMessageResponse(string baseId, string connectionId, string 
			error) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.GetObjectValues
			, baseId, connectionId)
		{
			SetError(error);
		}

		public GetObjectValuesMessageResponse(string baseId, string connectionId, NeoDatis.Odb.Values
			 values, NeoDatis.Odb.Core.Query.Execution.IQueryExecutionPlan queryExecutionPlan
			) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.GetObjectValues, 
			baseId, connectionId)
		{
			this.values = values;
			this.plan = queryExecutionPlan;
		}

		public virtual NeoDatis.Odb.Values GetValues()
		{
			return values;
		}

		/// <returns></returns>
		public virtual NeoDatis.Odb.Core.Query.Execution.IQueryExecutionPlan GetPlan()
		{
			return plan;
		}
	}
}

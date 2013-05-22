namespace NeoDatis.Odb.Core.Server.Message
{
	/// <summary>A response to a GetMessage comamnd</summary>
	/// <author>olivier s</author>
	[System.Serializable]
	public class GetMessageResponse : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		/// <summary>List of meta representation of the objects</summary>
		private NeoDatis.Odb.Objects<NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			> metaObjects;

		private NeoDatis.Odb.Core.Query.Execution.IQueryExecutionPlan plan;

		public GetMessageResponse(string baseId, string connectionId, string error) : base
			(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.Get, baseId, connectionId
			)
		{
			SetError(error);
		}

		public GetMessageResponse(string baseId, string connectionId, NeoDatis.Odb.Objects
			<NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo> metaObjects, NeoDatis.Odb.Core.Query.Execution.IQueryExecutionPlan
			 plan) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.Get, baseId, 
			connectionId)
		{
			this.metaObjects = metaObjects;
			this.plan = plan;
		}

		public virtual NeoDatis.Odb.Objects<NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			> GetMetaObjects()
		{
			return metaObjects;
		}

		public virtual NeoDatis.Odb.Core.Query.Execution.IQueryExecutionPlan GetPlan()
		{
			return plan;
		}
	}
}

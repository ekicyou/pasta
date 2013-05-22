namespace NeoDatis.Odb.Core.Server.Message
{
	/// <summary>A response to a GetMessage comamnd</summary>
	/// <author>olivier s</author>
	[System.Serializable]
	public class GetObjectFromIdMessageResponse : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		/// <summary>meta representation of the objects</summary>
		private NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi;

		public GetObjectFromIdMessageResponse(string baseId, string connectionId, string 
			error) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.GetObjectFromId
			, baseId, connectionId)
		{
			SetError(error);
		}

		public GetObjectFromIdMessageResponse(string baseId, string connectionId, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 metaRepresentation) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command
			.GetObjectFromId, baseId, connectionId)
		{
			this.nnoi = metaRepresentation;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo GetMetaRepresentation
			()
		{
			return nnoi;
		}
	}
}

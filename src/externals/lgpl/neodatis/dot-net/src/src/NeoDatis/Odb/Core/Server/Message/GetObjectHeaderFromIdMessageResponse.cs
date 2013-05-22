namespace NeoDatis.Odb.Core.Server.Message
{
	/// <summary>A response to a GetObjectHeaderFromIdMessage comamnd</summary>
	/// <author>olivier s</author>
	[System.Serializable]
	public class GetObjectHeaderFromIdMessageResponse : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		/// <summary>header of meta representation of the object</summary>
		private NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih;

		public GetObjectHeaderFromIdMessageResponse(string baseId, string connectionId, string
			 error) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.GetObjectHeaderFromId
			, baseId, connectionId)
		{
			SetError(error);
		}

		public GetObjectHeaderFromIdMessageResponse(string baseId, string connectionId, NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader
			 oih) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.GetObjectHeaderFromId
			, baseId, connectionId)
		{
			this.oih = oih;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader GetObjectInfoHeader
			()
		{
			return oih;
		}
	}
}

namespace NeoDatis.Odb.Core.Server.Message
{
	/// <summary>
	/// A NewClassInfoMessageResponse is used by the Client/Server mode to answer a NewClassInfoMessage,
	/// it returns all the class infos of the new server model
	/// </summary>
	/// <author>olivier s</author>
	[System.Serializable]
	public class NewClassInfoListMessageResponse : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		private NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			> classInfos;

		public NewClassInfoListMessageResponse(string baseId, string connectionId, string
			 error) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.AddClassInfoList
			, baseId, connectionId)
		{
			SetError(error);
		}

		public NewClassInfoListMessageResponse(string baseId, string connectionId, NeoDatis.Tool.Wrappers.List.IOdbList
			<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo> classInfos) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command
			.AddClassInfoList, baseId, connectionId)
		{
			this.classInfos = classInfos;
		}

		public virtual NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			> GetClassInfos()
		{
			return classInfos;
		}
	}
}

namespace NeoDatis.Odb.Core.Server.Message
{
	/// <summary>A NewClassInfosMessage is used by the Client/Server mode to add a new class infos (List) to the meta model on the server
	/// 	</summary>
	/// <author>osmadja</author>
	[System.Serializable]
	public class NewClassInfoListMessage : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		private NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList classInfoList;

		public NewClassInfoListMessage(string baseId, string connectionId, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList
			 ciList) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.AddClassInfoList
			, baseId, connectionId)
		{
			this.classInfoList = ciList;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList GetClassInfoList
			()
		{
			return classInfoList;
		}

		public override string ToString()
		{
			return "AddClassInfos";
		}
	}
}

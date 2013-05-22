namespace NeoDatis.Odb.Core.Server.Message
{
	/// <summary>A response to a CountMessage command</summary>
	/// <author>olivier s</author>
	[System.Serializable]
	public class CountMessageResponse : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		private long nbObjects;

		public CountMessageResponse(string baseId, string connectionId, string error) : base
			(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.Count, baseId, connectionId
			)
		{
			SetError(error);
		}

		public CountMessageResponse(string baseId, string connectionId, long nbObjects) : 
			base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.Count, baseId, connectionId
			)
		{
			this.nbObjects = nbObjects;
		}

		public virtual long GetNbObjects()
		{
			return nbObjects;
		}
	}
}

namespace NeoDatis.Odb.Core.Server.Message
{
	[System.Serializable]
	public class GetSessionsMessageResponse : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		internal System.Collections.Generic.IList<string> sessions;

		public GetSessionsMessageResponse(System.Collections.Generic.IList<string> sessions
			) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.GetSessions, null
			, null)
		{
			this.sessions = sessions;
		}

		public GetSessionsMessageResponse(string error) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command
			.GetSessions, null, null)
		{
		}

		public virtual System.Collections.Generic.IList<string> GetSessions()
		{
			return sessions;
		}
	}
}

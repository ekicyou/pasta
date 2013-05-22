namespace NeoDatis.Odb.Impl.Main
{
	/// <summary>The client implementation of ODB.</summary>
	/// <remarks>The client implementation of ODB.</remarks>
	/// <author>osmadja</author>
	public class RemoteODBClient : NeoDatis.Odb.Impl.Main.ODBAdapter
	{
		public RemoteODBClient(string hostName, int port, string baseId) : this(hostName, 
			port, baseId, null, null)
		{
		}

		public RemoteODBClient(string hostName, int port, string baseId, string user, string
			 password) : base(NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetClientStorageEngine
			(new NeoDatis.Odb.Core.Layers.Layer3.IOSocketParameter(hostName, port, baseId, NeoDatis.Odb.Core.Layers.Layer3.IOSocketParameter
			.TypeDatabase, user, password)))
		{
		}

		public override void Close()
		{
			storageEngine.Close();
		}
	}
}

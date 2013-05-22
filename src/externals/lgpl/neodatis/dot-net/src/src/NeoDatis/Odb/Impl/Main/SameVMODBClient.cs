namespace NeoDatis.Odb.Impl.Main
{
	/// <summary>The client implementation of ODB.</summary>
	/// <remarks>The client implementation of ODB.</remarks>
	/// <author>osmadja</author>
	public class SameVMODBClient : NeoDatis.Odb.Impl.Main.ODBAdapter
	{
		/// <summary>TODO set the constructor as protected</summary>
		public SameVMODBClient(NeoDatis.Odb.Core.Server.Layers.Layer3.IODBServerExt server
			, string baseIdentifier) : base(new NeoDatis.Odb.Impl.Core.Server.Layers.Layer3.Engine.SameVmClientEngine
			(server, baseIdentifier))
		{
		}

		public override void Close()
		{
			storageEngine.Close();
		}
	}
}

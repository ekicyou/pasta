namespace NeoDatis.Odb.Core.Server.Connection
{
	/// <summary>A class to manage client server connections being executed in the same Vm.
	/// 	</summary>
	/// <remarks>A class to manage client server connections being executed in the same Vm. In this case, we don't use network IO.
	/// 	</remarks>
	/// <author>olivier s</author>
	public class SameVmConnectionThread : NeoDatis.Odb.Core.Server.Connection.ClientServerConnection
	{
		private static readonly string LogId = "SameVmConnectionThread";

		public SameVmConnectionThread(NeoDatis.Odb.Core.Server.Layers.Layer3.IODBServerExt
			 server, bool automaticallyCreateDatabase) : base(server, automaticallyCreateDatabase
			)
		{
		}

		public override string GetName()
		{
			return "Same vm client ";
		}
	}
}

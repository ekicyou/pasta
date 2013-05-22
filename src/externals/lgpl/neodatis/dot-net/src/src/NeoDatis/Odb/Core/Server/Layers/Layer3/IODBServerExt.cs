namespace NeoDatis.Odb.Core.Server.Layers.Layer3
{
	public interface IODBServerExt : NeoDatis.Odb.ODBServer
	{
		void StartServer(bool inThread);

		System.Collections.IDictionary GetConnectionManagers();

		NeoDatis.Odb.Core.Layers.Layer3.IOSocketParameter GetParameters(string baseIdentifier
			, bool clientAndServerRunsInSameVM);
	}
}

namespace NeoDatis.Odb
{
	/// <summary>The ODBFactory to obtain the right ODB implementation.</summary>
	/// <remarks>The ODBFactory to obtain the right ODB implementation.</remarks>
	/// <author>osmadja</author>
	public class ODBFactory
	{
		/// <summary>A private constructor to avoid instantiation</summary>
		private ODBFactory()
		{
		}

		/// <summary>Open an ODB database protected by a user and password</summary>
		/// <param name="fileName">The name of the ODB database</param>
		/// <param name="user">The user of the database</param>
		/// <param name="password">The password of the user</param>
		/// <returns>The ODB database</returns>
		public static NeoDatis.Odb.ODB Open(string fileName, string user, string password
			)
		{
			NeoDatis.Odb.ODB odBase = NeoDatis.Odb.Impl.Main.LocalODB.GetInstance(fileName, user
				, password);
			return odBase;
		}

		/// <summary>Open a non password protected ODB database</summary>
		/// <param name="fileName">The ODB database name</param>
		/// <returns>A local ODB implementation</returns>
		public static NeoDatis.Odb.ODB Open(string fileName)
		{
			NeoDatis.Odb.ODB odBase = NeoDatis.Odb.Impl.Main.LocalODB.GetInstance(fileName);
			return odBase;
		}

		/// <summary>Open an ODB server on the specific port.</summary>
		/// <remarks>
		/// Open an ODB server on the specific port. This will the socketServer on
		/// the specified port. Must call startServer of the ODBServer to actually
		/// start the server
		/// </remarks>
		/// <param name="port">The server port</param>
		/// <returns>The server</returns>
		public static NeoDatis.Odb.ODBServer OpenServer(int port)
		{
			return new NeoDatis.Odb.Impl.Main.ODBDefaultServer(port);
		}

		/// <summary>Open an ODB Client</summary>
		/// <param name="hostName"></param>
		/// <param name="port"></param>
		/// <param name="baseIdentifier">
		/// The base identifier : The alias used by the server to declare
		/// database
		/// </param>
		/// <returns>The ODB</returns>
		public static NeoDatis.Odb.ODB OpenClient(string hostName, int port, string baseIdentifier
			)
		{
			return new NeoDatis.Odb.Impl.Main.RemoteODBClient(hostName, port, baseIdentifier);
		}

		/// <param name="hostName"></param>
		/// <param name="port"></param>
		/// <param name="baseIdentifier">
		/// The base identifier : The alias used by the server to declare
		/// database
		/// </param>
		/// <param name="user">Remote access user</param>
		/// <param name="password">Remote access password</param>
		/// <returns>The ODB</returns>
		public static NeoDatis.Odb.ODB OpenClient(string hostName, int port, string baseIdentifier
			, string user, string password)
		{
			return new NeoDatis.Odb.Impl.Main.RemoteODBClient(hostName, port, baseIdentifier, 
				user, password);
		}
	}
}

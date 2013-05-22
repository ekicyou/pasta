namespace NeoDatis.Odb
{
	public interface ODBServer
	{
		/// <summary>Adds a base to the server.</summary>
		/// <remarks>
		/// Adds a base to the server. If the base does not exist, it will be
		/// created. Can be called after server start.
		/// </remarks>
		/// <param name="baseIdentifier">The name that the client must use to reference this base
		/// 	</param>
		/// <param name="fileName">The physical file name of this base</param>
		void AddBase(string baseIdentifier, string fileName);

		/// <summary>Adds a base to the server.</summary>
		/// <remarks>
		/// Adds a base to the server. If the base does not exist, it will be
		/// created. Can be called after server start.
		/// </remarks>
		/// <param name="baseIdentifier"></param>
		/// <param name="fileName">The name that the client must use to reference this base</param>
		/// <param name="user">The user that will be used to open the database</param>
		/// <param name="password">The password that will be used to open the base</param>
		void AddBase(string baseIdentifier, string fileName, string user, string password
			);

		/// <summary>Not yet implemented</summary>
		/// <param name="baseIdentifier"></param>
		/// <param name="user"></param>
		/// <param name="password"></param>
		void AddUserForBase(string baseIdentifier, string user, string password);

		/// <summary>actually starts the server.</summary>
		/// <remarks>
		/// actually starts the server. Starts listening incoming connections on the
		/// port.
		/// </remarks>
		/// <param name="inThread">
		/// If true, the server is started in an independent thread for
		/// listening incoming connections, else it simply executes the
		/// server (client connection) in the current thread
		/// </param>
		void StartServer(bool inThread);

		/// <summary>Closes the server.</summary>
		/// <remarks>Closes the server. Closes the socket server and all registered databases.
		/// 	</remarks>
		void Close();

		void SetAutomaticallyCreateDatabase(bool yes);

		NeoDatis.Odb.ODB OpenClient(string baseIdentifier);

		/// <summary>Used to add an update trigger callback</summary>
		/// <param name="trigger"></param>
		void AddUpdateTrigger(string baseIdentifier, string className, NeoDatis.Odb.Core.Server.Trigger.ServerUpdateTrigger
			 trigger);

		/// <summary>Used to add an insert trigger callback</summary>
		/// <param name="trigger"></param>
		void AddInsertTrigger(string baseIdentifier, string className, NeoDatis.Odb.Core.Server.Trigger.ServerInsertTrigger
			 trigger);

		/// <summary>USed to add a delete trigger callback</summary>
		/// <param name="trigger"></param>
		void AddDeleteTrigger(string baseIdentifier, string className, NeoDatis.Odb.Core.Server.Trigger.ServerDeleteTrigger
			 trigger);

		/// <summary>Used to add a select trigger callback</summary>
		/// <param name="trigger"></param>
		void AddSelectTrigger(string baseIdentifier, string className, NeoDatis.Odb.Core.Server.Trigger.ServerSelectTrigger
			 trigger);
	}
}

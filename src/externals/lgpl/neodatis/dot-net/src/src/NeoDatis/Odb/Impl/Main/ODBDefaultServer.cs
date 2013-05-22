namespace NeoDatis.Odb.Impl.Main
{
	public class ODBDefaultServer : NeoDatis.Odb.ODBServer
	{
		protected NeoDatis.Odb.ODBServer serverImpl;

		public ODBDefaultServer(int port)
		{
			//this.serverImpl = new NeoDatis.Odb.Impl.Core.Server.Layers.Layer3.Engine.ODBServerImpl
				//(port);
		}

		public virtual void AddBase(string baseIdentifier, string fileName)
		{
			serverImpl.AddBase(baseIdentifier, fileName);
		}

		public virtual void AddBase(string baseIdentifier, string fileName, string user, 
			string password)
		{
			serverImpl.AddBase(baseIdentifier, fileName, user, password);
		}

		public virtual void AddUserForBase(string baseIdentifier, string user, string password
			)
		{
			serverImpl.AddUserForBase(baseIdentifier, user, password);
		}

		public virtual void Close()
		{
			serverImpl.Close();
		}

		public virtual NeoDatis.Odb.ODB OpenClient(string baseIdentifier)
		{
			return serverImpl.OpenClient(baseIdentifier);
		}

		public virtual void SetAutomaticallyCreateDatabase(bool yes)
		{
			serverImpl.SetAutomaticallyCreateDatabase(yes);
		}

		public virtual void StartServer(bool inThread)
		{
			serverImpl.StartServer(inThread);
		}

		public virtual void AddDeleteTrigger(string baseIdentifier, string className, NeoDatis.Odb.Core.Server.Trigger.ServerDeleteTrigger
			 trigger)
		{
			serverImpl.AddDeleteTrigger(baseIdentifier, className, trigger);
		}

		public virtual void AddInsertTrigger(string baseIdentifier, string className, NeoDatis.Odb.Core.Server.Trigger.ServerInsertTrigger
			 trigger)
		{
			serverImpl.AddInsertTrigger(baseIdentifier, className, trigger);
		}

		public virtual void AddSelectTrigger(string baseIdentifier, string className, NeoDatis.Odb.Core.Server.Trigger.ServerSelectTrigger
			 trigger)
		{
			serverImpl.AddSelectTrigger(baseIdentifier, className, trigger);
		}

		public virtual void AddUpdateTrigger(string baseIdentifier, string className, NeoDatis.Odb.Core.Server.Trigger.ServerUpdateTrigger
			 trigger)
		{
			serverImpl.AddUpdateTrigger(baseIdentifier, className, trigger);
		}
	}
}

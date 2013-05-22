namespace NeoDatis.Odb.Core.Layers.Layer3
{
	/// <summary>To express parameters that must be passed to a remote server.</summary>
	/// <remarks>
	/// To express parameters that must be passed to a remote server.
	/// If base id is defined then filename is null. If filename is defined, then baseId is null
	/// </remarks>
	/// <author>osmadja</author>
	public class IOSocketParameter : NeoDatis.Odb.Core.Layers.Layer3.IBaseIdentification
	{
		public const int TypeDatabase = 1;

		public const int TypeTransaction = 2;

		private string destinationHost;

		private int port;

		private string baseIdentifier;

		private int type;

		private string user;

		[System.NonSerialized]
		private string password;

		/// <summary>Used for TYPE_TRANSACTION, to buld the entire transaction file name</summary>
		private long dateTimeCreation;

		/// <summary>To know if client runs on the same vm than the server.</summary>
		/// <remarks>
		/// To know if client runs on the same vm than the server. It is the case, we client / server communication
		/// can be optimized.
		/// </remarks>
		protected bool clientAndServerRunInSameVM;

		public IOSocketParameter(string host, int port, string identifier, int type, string
			 user, string password) : this(host, port, identifier, type, -1, user, password, 
			false)
		{
		}

		public IOSocketParameter(string host, int port, string identifier, int type, long
			 dtCreation, string user, string password) : this(host, port, identifier, type, 
			dtCreation, user, password, false)
		{
		}

		public IOSocketParameter(string host, int port, string identifier, int type, long
			 dtCreation, string user, string password, bool clientAndServerRunOnSameVm)
		{
			this.destinationHost = host;
			if (destinationHost.IndexOf(".") == -1)
			{
				// this is not the IP, get the ip address
				destinationHost = NeoDatis.Tool.Wrappers.Net.NeoDatisIpAddress.Get(destinationHost
					);
			}
			this.port = port;
			this.baseIdentifier = identifier;
			this.type = type;
			this.dateTimeCreation = dtCreation;
			this.user = user;
			this.password = password;
			this.clientAndServerRunInSameVM = clientAndServerRunOnSameVm;
		}

		public virtual string GetDestinationHost()
		{
			return destinationHost;
		}

		public virtual int GetPort()
		{
			return port;
		}

		public virtual string GetBaseIdentifier()
		{
			return baseIdentifier;
		}

		public virtual bool CanWrite()
		{
			return true;
		}

		public virtual int GetType()
		{
			return type;
		}

		public virtual bool IsDatabase()
		{
			return type == TypeDatabase;
		}

		public virtual bool IsTransaction()
		{
			return type == TypeTransaction;
		}

		public virtual long GetDateTimeCreation()
		{
			return dateTimeCreation;
		}

		public virtual string GetPassword()
		{
			return password;
		}

		public virtual void SetPassword(string password)
		{
			this.password = password;
		}

		public virtual string GetUserName()
		{
			return user;
		}

		public virtual void SetUserName(string user)
		{
			this.user = user;
		}

		public override string ToString()
		{
			return baseIdentifier + "@" + destinationHost + ":" + port;
		}

		public virtual string GetIdentification()
		{
			return ToString();
		}

		public virtual bool IsNew()
		{
			return false;
		}

		public virtual bool IsLocal()
		{
			return false;
		}

		public virtual bool ClientAndServerRunInSameVM()
		{
			return clientAndServerRunInSameVM;
		}

		public virtual string GetDirectory()
		{
			return string.Empty;
		}
	}
}

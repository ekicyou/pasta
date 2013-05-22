namespace NeoDatis.Odb.Impl.Core.Server.Transaction
{
	public class SessionManager : NeoDatis.Odb.Core.Server.Transaction.ISessionManager
	{
		public static readonly string LogId = "SessionManager";

		protected System.Collections.Generic.IDictionary<string, NeoDatis.Odb.Core.Transaction.ISession
			> sessions;

		public SessionManager()
		{
			sessions = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string, NeoDatis.Odb.Core.Transaction.ISession
				>();
		}

		public virtual void Init2()
		{
		}

		// TODO Nothing to do 	
		public virtual NeoDatis.Odb.Core.Transaction.ISession GetSession(string baseIdentification
			, bool throwExceptionIfDoesNotExist)
		{
			string threadName = NeoDatis.Tool.Wrappers.OdbThread.GetCurrentThreadName();
			System.Text.StringBuilder id = new System.Text.StringBuilder(threadName).Append(baseIdentification
				);
			NeoDatis.Odb.Core.Transaction.ISession session = sessions[id.ToString()];
			if (session == null && throwExceptionIfDoesNotExist)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.SessionDoesNotExistForConnection
					.AddParameter(threadName).AddParameter(baseIdentification).AddParameter(id));
			}
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug("Getting session for base " + baseIdentification + " and thread "
					 + threadName + " = " + id + " - sid=" + session.GetId());
			}
			return session;
		}

		public virtual void AddSession(NeoDatis.Odb.Core.Transaction.ISession session)
		{
			string id = NeoDatis.Tool.Wrappers.OdbThread.GetCurrentThreadName() + session.GetBaseIdentification
				();
			sessions.Add(id, session);
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug("Associating id = " + id + " to session " + session.GetId
					());
			}
		}

		//DLogger.info(StringUtils.exceptionToString(new Exception()));
		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			System.Collections.Generic.IEnumerator<string> iterator = sessions.Keys.GetEnumerator
				();
			string sid = null;
			NeoDatis.Odb.Core.Transaction.ISession session = null;
			while (iterator.MoveNext())
			{
				sid = iterator.Current;
				session = sessions[sid];
				buffer.Append(sid).Append(":").Append(session.ToString()).Append("\n");
			}
			return buffer.ToString();
		}

		public virtual void RemoveSession(string baseIdentification)
		{
			string id = NeoDatis.Tool.Wrappers.OdbThread.GetCurrentThreadName() + baseIdentification;
			sessions.Remove(id);
		}

		//ISession session = sessions.remove(id);
		//session.close();
		//session = null;
		public virtual System.Collections.Generic.IList<string> GetSessionDescriptions(System.Collections.IDictionary
			 connectionManagers)
		{
			System.Collections.Generic.IList<string> l = new System.Collections.Generic.List<
				string>();
			System.Collections.Generic.IEnumerator<string> iterator = sessions.Keys.GetEnumerator
				();
			string sid = null;
			NeoDatis.Odb.Core.Transaction.ISession session = null;
			NeoDatis.Odb.Core.Server.Connection.ConnectionManager cm = null;
			System.Text.StringBuilder buffer = null;
			while (iterator.MoveNext())
			{
				sid = iterator.Current;
				session = sessions[sid];
				cm = (NeoDatis.Odb.Core.Server.Connection.ConnectionManager)connectionManagers[session
					.GetBaseIdentification()];
				buffer = new System.Text.StringBuilder("Session " + sid + " : " + session.ToString
					());
				if (cm != null)
				{
					buffer.Append(" - Number of connections=" + cm.GetNbConnections());
					buffer.Append(cm.GetConnectionDescriptions());
				}
				l.Add(buffer.ToString());
			}
			return l;
		}

		public virtual long GetNumberOfSessions()
		{
			return sessions.Count;
		}
	}
}

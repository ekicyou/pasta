namespace NeoDatis.Odb.Core.Server.Transaction
{
	/// <summary>The interface for Client server Session Manager</summary>
	/// <author>olivier</author>
	public interface ISessionManager : NeoDatis.Odb.Core.ITwoPhaseInit
	{
		NeoDatis.Odb.Core.Transaction.ISession GetSession(string baseIdentification, bool
			 throwExceptionIfDoesNotExist);

		//ISession getSessionByConnectionId(String connectionId, boolean throwExceptionIfDoesNotExist);
		void AddSession(NeoDatis.Odb.Core.Transaction.ISession session);

		void RemoveSession(string baseIdentification);

		System.Collections.Generic.IList<string> GetSessionDescriptions(System.Collections.IDictionary
			 connectionManagers);

		long GetNumberOfSessions();
	}
}

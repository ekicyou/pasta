namespace NeoDatis.Odb
{
	/// <summary>The exception thrown when the user/password is wrong</summary>
	/// <author>osmadja</author>
	[System.Serializable]
	public class ODBAuthenticationRuntimeException : System.Exception
	{
		public ODBAuthenticationRuntimeException() : base("invalid user/password")
		{
		}
	}
}

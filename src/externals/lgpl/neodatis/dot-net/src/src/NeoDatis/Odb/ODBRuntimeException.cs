namespace NeoDatis.Odb
{
	/// <summary>Generic ODB Runtime exception : Used to report all problems.</summary>
	/// <remarks>Generic ODB Runtime exception : Used to report all problems.</remarks>
	/// <author>osmadja</author>
	[System.Serializable]
	public class ODBRuntimeException : System.Exception
	{
		private static readonly string url = "https://sourceforge.net/tracker/?func=add&group_id=179124&atid=887885";

		private static readonly string message1 = string.Format("\nNeoDatis has thrown an Exception, please help us filling a bug report at {0} with the following error message"
			, url);

		public ODBRuntimeException(NeoDatis.Odb.Core.IError error, System.Exception t) :
            base(string.Format("{0}\nVersion={1} , Build={2}, Date={3}, Thread={4}\nError:{5}"
			, message1, NeoDatis.Odb.Core.Release.ReleaseNumber, NeoDatis.Odb.Core.Release.ReleaseBuild
			, NeoDatis.Odb.Core.Release.ReleaseDate, NeoDatis.Tool.Wrappers.OdbThread.GetCurrentThreadName
			(), error.ToString()), t)
		{
		}

        public ODBRuntimeException(NeoDatis.Odb.Core.IError error)
            : base(string.Format("{0}\nVersion={1} , Build={2}, Date={3}, Thread={4}\nError:{5}"
			, message1, NeoDatis.Odb.Core.Release.ReleaseNumber, NeoDatis.Odb.Core.Release.ReleaseBuild
			, NeoDatis.Odb.Core.Release.ReleaseDate, NeoDatis.Tool.Wrappers.OdbThread.GetCurrentThreadName
			(), error))
		{
		}

		public ODBRuntimeException(NeoDatis.Odb.Core.IError error, string message) : base
            (string.Format("{0}\nVersion={1} , Build={2}, Date={3}, Thread={4}\nError:{5}\nStackTrace:{6}"
			, message1, NeoDatis.Odb.Core.Release.ReleaseNumber, NeoDatis.Odb.Core.Release.ReleaseBuild
			, NeoDatis.Odb.Core.Release.ReleaseDate, NeoDatis.Tool.Wrappers.OdbThread.GetCurrentThreadName
			(), error.ToString(), message))
		{
		}

		// FIXME add a submit a bug link to SF
		public virtual void AddMessageHeader(string @string)
		{
		}

		public ODBRuntimeException(System.Exception e, string message) : base(string.Format
            ("{0}\nVersion={1} , Build={2}, Date={3}, Thread={4}\nError:{5}\nStackTrace:{6}", message1
			, NeoDatis.Odb.Core.Release.ReleaseNumber, NeoDatis.Odb.Core.Release.ReleaseBuild
			, NeoDatis.Odb.Core.Release.ReleaseDate, NeoDatis.Tool.Wrappers.OdbThread.GetCurrentThreadName
			(), message), e)
		{
		}
		// TODO Auto-generated method stub
	}
}

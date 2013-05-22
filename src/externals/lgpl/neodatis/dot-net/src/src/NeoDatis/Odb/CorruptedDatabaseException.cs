namespace NeoDatis.Odb
{
	/// <summary>An exception thrown by ODB when a corrupted block is found</summary>
	/// <author>olivier</author>
	[System.Serializable]
	public class CorruptedDatabaseException : NeoDatis.Odb.ODBRuntimeException
	{
		public CorruptedDatabaseException(NeoDatis.Odb.Core.IError error, string message)
			 : base(error, message)
		{
		}

		public CorruptedDatabaseException(NeoDatis.Odb.Core.IError error, System.Exception
			 t) : base(error, t)
		{
		}

		public CorruptedDatabaseException(NeoDatis.Odb.Core.IError error) : base(error)
		{
		}
	}
}

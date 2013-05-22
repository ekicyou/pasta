namespace NeoDatis.Btree.Exception
{
	[System.Serializable]
	public class DuplicatedKeyException : NeoDatis.Btree.Exception.BTreeException
	{
		public DuplicatedKeyException() : base()
		{
		}

		public DuplicatedKeyException(string message, System.Exception cause) : base(message
			, cause)
		{
		}

		public DuplicatedKeyException(string message) : base(message)
		{
		}
	}
}

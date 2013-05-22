namespace NeoDatis.Btree.Exception
{
	[System.Serializable]
	public class BTreeException : System.Exception
	{
		public BTreeException() : base()
		{
		}

		public BTreeException(string message, System.Exception cause) : base(message, cause
			)
		{
		}

		public BTreeException(string message) : base(message)
		{
		}
	}
}

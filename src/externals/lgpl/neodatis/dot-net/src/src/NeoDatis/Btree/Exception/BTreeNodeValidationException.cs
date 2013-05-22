namespace NeoDatis.Btree.Exception
{
	[System.Serializable]
	public class BTreeNodeValidationException : System.Exception
	{
		public BTreeNodeValidationException() : base()
		{
		}

		public BTreeNodeValidationException(string message, System.Exception cause) : base
			(message, cause)
		{
		}

		public BTreeNodeValidationException(string message) : base(message)
		{
		}
	}
}

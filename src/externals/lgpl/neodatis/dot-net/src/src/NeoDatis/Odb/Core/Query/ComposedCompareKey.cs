namespace NeoDatis.Odb.Core.Query
{
	/// <summary>
	/// A composed key : an object that contains various values used for indexing query result
	/// <p>
	/// This is an implementation that allows compare keys to contain more than one single value to be compared
	/// </p>
	/// </summary>
	[System.Serializable]
	public class ComposedCompareKey : NeoDatis.Odb.Core.Query.CompareKey
	{
		private System.IComparable[] keys;

		public ComposedCompareKey(NeoDatis.Tool.Wrappers.OdbComparable[] keys)
		{
			this.keys = keys;
		}

		public override int CompareTo(object o)
		{
			if (o == null || o.GetType() != typeof(NeoDatis.Odb.Core.Query.ComposedCompareKey
				))
			{
				return -1;
			}
			NeoDatis.Odb.Core.Query.ComposedCompareKey ckey = (NeoDatis.Odb.Core.Query.ComposedCompareKey
				)o;
			int result = 0;
			for (int i = 0; i < keys.Length; i++)
			{
				result = keys[i].CompareTo(ckey.keys[i]);
				if (result != 0)
				{
					return result;
				}
			}
			return 0;
		}

		public override string ToString()
		{
			if (keys == null)
			{
				return "no keys defined";
			}
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			for (int i = 0; i < keys.Length; i++)
			{
				if (i != 0)
				{
					buffer.Append("|");
				}
				buffer.Append(keys[i]);
			}
			return buffer.ToString();
		}
	}
}

namespace NeoDatis.Odb.Core.Query
{
	/// <summary>
	/// A simple compare key : an object that contains various values used for indexing query result
	/// <p>
	/// </p>
	/// </summary>
	[System.Serializable]
	public class SimpleCompareKey : NeoDatis.Odb.Core.Query.CompareKey
	{
		private System.IComparable key;

		public SimpleCompareKey(System.IComparable key)
		{
			this.key = key;
		}

		public override int CompareTo(object o)
		{
			if (o == null || o.GetType() != typeof(NeoDatis.Odb.Core.Query.SimpleCompareKey))
			{
				return -1;
			}
			NeoDatis.Odb.Core.Query.SimpleCompareKey ckey = (NeoDatis.Odb.Core.Query.SimpleCompareKey
				)o;
			return key.CompareTo(ckey.key);
		}

		public override string ToString()
		{
			return key.ToString();
		}

		public override bool Equals(object o)
		{
			if (o == null || !(o is NeoDatis.Odb.Core.Query.SimpleCompareKey))
			{
				return false;
			}
			NeoDatis.Odb.Core.Query.SimpleCompareKey c = (NeoDatis.Odb.Core.Query.SimpleCompareKey
				)o;
			return key.Equals(c.key);
		}

		public override int GetHashCode()
		{
			return key.GetHashCode();
		}
	}
}

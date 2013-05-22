namespace NeoDatis.Odb.Core.Query
{
	[System.Serializable]
	public abstract class CompareKey : NeoDatis.Tool.Wrappers.OdbComparable
	{
		public abstract int CompareTo(object o);
	}
}

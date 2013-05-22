namespace NeoDatis.Tool.Wrappers
{
	/// <summary>o wrapper to the native Comparable interface</summary>
	/// <author>olivier</author>
	public interface OdbComparable : System.IComparable
	{
		int CompareTo(object o);
	}
}

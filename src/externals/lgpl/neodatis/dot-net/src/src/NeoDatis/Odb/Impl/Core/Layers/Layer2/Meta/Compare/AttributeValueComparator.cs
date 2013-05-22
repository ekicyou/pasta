namespace NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare
{
	/// <author>olivier</author>
	public class AttributeValueComparator
	{
		/// <summary>A geenric compare method</summary>
		/// <param name="c1"></param>
		/// <param name="c2"></param>
		/// <returns></returns>
		public static int Compare(System.IComparable c1, System.IComparable c2)
		{
			return c1.CompareTo(c2);
		}
	}
}

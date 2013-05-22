using NUnit.Framework;
namespace NeoDatis.Odb.Test.Index
{
	/// <author>olivier</author>
	public class IndexedObject3
	{
		public IndexedObject3(int i1, int i2, int i3, string s1, string s2, string s3, System.DateTime
			 dt1, System.DateTime dt2, System.DateTime dt3) : base()
		{
			this.i1 = i1;
			this.i2 = i2;
			this.i3 = i3;
			this.s1 = s1;
			this.s2 = s2;
			this.s3 = s3;
			this.dt1 = dt1;
			this.dt2 = dt2;
			this.dt3 = dt3;
		}

		private int i1;

		private int i2;

		private int i3;

		private string s1;

		private string s2;

		private string s3;

		private System.DateTime dt1;

		private System.DateTime dt2;

		private System.DateTime dt3;
	}
}

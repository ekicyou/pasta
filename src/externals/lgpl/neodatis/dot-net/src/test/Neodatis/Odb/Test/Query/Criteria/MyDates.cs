using NUnit.Framework;
namespace NeoDatis.Odb.Test.Query.Criteria
{
	public class MyDates
	{
		private int i;

		private System.DateTime date1;

		private System.DateTime date2;

		public MyDates()
		{
		}

		public virtual System.DateTime GetDate1()
		{
			return date1;
		}

		public virtual void SetDate1(System.DateTime date1)
		{
			this.date1 = date1;
		}

		public virtual System.DateTime GetDate2()
		{
			return date2;
		}

		public virtual void SetDate2(System.DateTime date2)
		{
			this.date2 = date2;
		}

		public virtual int GetI()
		{
			return i;
		}

		public virtual void SetI(int i)
		{
			this.i = i;
		}
	}
}

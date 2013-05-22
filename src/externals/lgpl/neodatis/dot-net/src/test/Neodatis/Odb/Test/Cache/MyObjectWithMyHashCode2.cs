using NUnit.Framework;
namespace NeoDatis.Odb.Test.Cache
{
	public class MyObjectWithMyHashCode2
	{
		private long myLong;

		public MyObjectWithMyHashCode2(long myLong) : base()
		{
			this.myLong = myLong;
		}

		public virtual long GetMyLong()
		{
			return myLong;
		}

		public virtual void SetMyLong(long myLong)
		{
			this.myLong = myLong;
		}

		public override int GetHashCode()
		{
			if (myLong == null)
			{
				return 0;
			}
			return myLong.GetHashCode();
		}
	}
}

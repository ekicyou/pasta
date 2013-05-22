using NUnit.Framework;
namespace NeoDatis.Odb.Test.Query.Criteria
{
	public class ClassWithLong
	{
		private long i;

		private string name;

		public ClassWithLong(long i, string name) : base()
		{
			this.i = i;
			this.name = name;
		}

		public virtual long GetI()
		{
			return i;
		}

		public virtual void SetI(long i)
		{
			this.i = i;
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}
	}
}

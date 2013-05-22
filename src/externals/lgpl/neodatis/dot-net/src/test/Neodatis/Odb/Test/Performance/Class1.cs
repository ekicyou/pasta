using NUnit.Framework;
namespace NeoDatis.Odb.Test.Performance
{
	public class Class1
	{
		private int ww;

		private string ccc;

		public Class1(int ww, string ccc) : base()
		{
			this.ww = ww;
			this.ccc = ccc;
		}

		public virtual int GetWw()
		{
			return ww;
		}

		public virtual void SetWw(int ww)
		{
			this.ww = ww;
		}

		public virtual string GetCcc()
		{
			return ccc;
		}

		public virtual void SetCcc(string ccc)
		{
			this.ccc = ccc;
		}
	}
}

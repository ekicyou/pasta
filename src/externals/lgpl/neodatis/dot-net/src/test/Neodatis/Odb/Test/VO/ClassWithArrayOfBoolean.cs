using NUnit.Framework;
namespace NeoDatis.Odb.Test.VO
{
	/// <author>olivier</author>
	public class ClassWithArrayOfBoolean
	{
		private string name;

		private bool[] bools1;

		private bool[] bools2;

		public ClassWithArrayOfBoolean(string name, bool[] bools1, bool[] bools2) : base(
			)
		{
			this.name = name;
			this.bools1 = bools1;
			this.bools2 = bools2;
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public virtual bool[] GetBools1()
		{
			return bools1;
		}

		public virtual void SetBools1(bool[] bools1)
		{
			this.bools1 = bools1;
		}

		public virtual bool[] GetBools2()
		{
			return bools2;
		}

		public virtual void SetBools2(bool[] bools2)
		{
			this.bools2 = bools2;
		}
	}
}

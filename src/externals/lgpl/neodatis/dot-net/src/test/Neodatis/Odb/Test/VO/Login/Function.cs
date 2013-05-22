using NUnit.Framework;
namespace NeoDatis.Odb.Test.VO.Login
{
	[System.Serializable]
	public class Function
	{
		private string name;

		public Function()
		{
		}

		public Function(string name)
		{
			this.name = name;
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public override string ToString()
		{
			return name;
		}
	}
}

using NUnit.Framework;
namespace NeoDatis.Odb.Test.Oid
{
	public class B
	{
		private string name;

		public B(string name) : base()
		{
			this.name = name;
		}

		public virtual string GetName()
		{
			return name;
		}
	}
}

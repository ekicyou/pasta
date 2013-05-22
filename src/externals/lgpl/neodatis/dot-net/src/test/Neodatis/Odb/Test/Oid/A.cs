using NUnit.Framework;
namespace NeoDatis.Odb.Test.Oid
{
	public class A
	{
		private string name;

		private NeoDatis.Odb.Test.Oid.B b;

		public A(string name, NeoDatis.Odb.Test.Oid.B b) : base()
		{
			this.name = name;
			this.b = b;
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual NeoDatis.Odb.Test.Oid.B GetB()
		{
			return b;
		}
	}
}

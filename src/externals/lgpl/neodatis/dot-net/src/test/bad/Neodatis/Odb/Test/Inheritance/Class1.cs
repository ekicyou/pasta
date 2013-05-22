namespace NeoDatis.Odb.Test.Inheritance
{
	public class Class1 : NeoDatis.Odb.Test.Inheritance.IInterface
	{
		private string name;

		public Class1(string name)
		{
			this.name = name;
		}

		public virtual string GetName()
		{
			return name;
		}
	}
}

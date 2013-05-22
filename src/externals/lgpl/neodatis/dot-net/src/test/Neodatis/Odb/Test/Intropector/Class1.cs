using NUnit.Framework;
namespace NeoDatis.Odb.Test.Intropector
{
	public class Class1
	{
		private string name1;

		private NeoDatis.Odb.Test.Intropector.Class2 class2;

		public Class1(string name1, string name2, string name3)
		{
			this.name1 = name1;
			class2 = new NeoDatis.Odb.Test.Intropector.Class2(name2, name3);
		}

		public virtual string GetName1()
		{
			return name1;
		}

		public virtual void SetName1(string name1)
		{
			this.name1 = name1;
		}

		public virtual NeoDatis.Odb.Test.Intropector.Class2 GetClass2()
		{
			return class2;
		}

		public virtual void SetClass2(NeoDatis.Odb.Test.Intropector.Class2 class2)
		{
			this.class2 = class2;
		}
	}
}

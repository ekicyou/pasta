namespace NeoDatis.Odb.Test.Intropector
{
	public class Class2
	{
		public Class2(string name22, string name3)
		{
			this.name2 = name22;
			class3 = new NeoDatis.Odb.Test.Intropector.Class3(name3);
		}

		private string name2;

		private NeoDatis.Odb.Test.Intropector.Class3 class3;

		public virtual string GetName2()
		{
			return name2;
		}

		public virtual void SetName2(string name2)
		{
			this.name2 = name2;
		}

		public virtual NeoDatis.Odb.Test.Intropector.Class3 GetClass3()
		{
			return class3;
		}

		public virtual void SetClass3(NeoDatis.Odb.Test.Intropector.Class3 class3)
		{
			this.class3 = class3;
		}
	}
}

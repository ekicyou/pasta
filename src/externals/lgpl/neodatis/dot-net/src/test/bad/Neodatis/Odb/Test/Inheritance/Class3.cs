namespace NeoDatis.Odb.Test.Inheritance
{
	public class Class3
	{
		private int nb;

		private NeoDatis.Odb.Test.Inheritance.Class1 class1;

		public Class3(int nb, NeoDatis.Odb.Test.Inheritance.Class1 class1)
		{
			this.nb = nb;
			this.class1 = class1;
		}

		public virtual NeoDatis.Odb.Test.Inheritance.Class1 GetClass1()
		{
			return class1;
		}

		public virtual void SetClass1(NeoDatis.Odb.Test.Inheritance.Class1 class1)
		{
			this.class1 = class1;
		}

		public virtual int GetNb()
		{
			return nb;
		}

		public virtual void SetNb(int nb)
		{
			this.nb = nb;
		}
	}
}

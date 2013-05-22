namespace NeoDatis.Odb.Test.Inheritance
{
	public class Class2
	{
		private int nb;

		private NeoDatis.Odb.Test.Inheritance.IInterface interface1;

		public Class2(int nb, NeoDatis.Odb.Test.Inheritance.IInterface interface1)
		{
			this.nb = nb;
			this.interface1 = interface1;
		}

		public virtual NeoDatis.Odb.Test.Inheritance.IInterface GetInterface1()
		{
			return interface1;
		}

		public virtual void SetInterface1(NeoDatis.Odb.Test.Inheritance.IInterface interface1
			)
		{
			this.interface1 = interface1;
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

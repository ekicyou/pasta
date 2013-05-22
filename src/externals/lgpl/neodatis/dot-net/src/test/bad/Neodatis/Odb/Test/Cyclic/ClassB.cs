namespace NeoDatis.Odb.Test.Cyclic
{
	/// <author>olivier</author>
	public class ClassB
	{
		internal string name;

		internal NeoDatis.Odb.Test.Cyclic.ClassA classA;

		public ClassB() : base()
		{
		}

		public ClassB(NeoDatis.Odb.Test.Cyclic.ClassA classA, string name) : base()
		{
			this.classA = classA;
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

		public virtual NeoDatis.Odb.Test.Cyclic.ClassA GetClassA()
		{
			return classA;
		}

		public virtual void SetClassA(NeoDatis.Odb.Test.Cyclic.ClassA classA)
		{
			this.classA = classA;
		}
	}
}

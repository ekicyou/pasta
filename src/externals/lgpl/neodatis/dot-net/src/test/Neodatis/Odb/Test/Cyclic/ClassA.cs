using NUnit.Framework;
namespace NeoDatis.Odb.Test.Cyclic
{
	/// <author>olivier</author>
	public class ClassA
	{
		internal string name;

		internal NeoDatis.Odb.Test.Cyclic.ClassB classb;

		public ClassA() : base()
		{
		}

		public ClassA(NeoDatis.Odb.Test.Cyclic.ClassB classb, string name) : base()
		{
			this.classb = classb;
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

		public virtual NeoDatis.Odb.Test.Cyclic.ClassB GetClassb()
		{
			return classb;
		}

		public virtual void SetClassb(NeoDatis.Odb.Test.Cyclic.ClassB classb)
		{
			this.classb = classb;
		}
	}
}

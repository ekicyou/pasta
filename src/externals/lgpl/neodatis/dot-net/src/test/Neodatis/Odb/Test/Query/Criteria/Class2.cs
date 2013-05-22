using NUnit.Framework;
namespace NeoDatis.Odb.Test.Query.Criteria
{
	public class Class2
	{
		private NeoDatis.Odb.Test.Query.Criteria.Class1 class1;

		public Class2() : base()
		{
		}

		public Class2(NeoDatis.Odb.Test.Query.Criteria.Class1 class1) : base()
		{
			// TODO Auto-generated constructor stub
			this.class1 = class1;
		}

		public virtual NeoDatis.Odb.Test.Query.Criteria.Class1 GetClass1()
		{
			return class1;
		}

		public virtual void SetClass1(NeoDatis.Odb.Test.Query.Criteria.Class1 class1)
		{
			this.class1 = class1;
		}
	}
}

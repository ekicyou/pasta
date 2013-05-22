namespace NeoDatis.Odb.Test.Inheritance
{
	public class TestInheritance : NeoDatis.Odb.Test.ODBTest
	{
		private static readonly string Name = "inheritance.neodatis";

		/// <summary>Test persistence of attributes declared by an interface</summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void TestInterface()
		{
			DeleteBase(Name);
			NeoDatis.Odb.Test.Inheritance.Class1 class1 = new NeoDatis.Odb.Test.Inheritance.Class1
				("olivier");
			Class2 class2 = new Class2
				(10, class1);
			NeoDatis.Odb.ODB odb = Open(Name);
			odb.Store(class2);
			odb.Close();
			odb = Open(Name);
			Class2 c2 = (Class2)odb.GetObjects<Class2>().GetFirst();
			AssertEquals(class2.GetNb(), c2.GetNb());
			AssertEquals(class2.GetInterface1().GetName(), c2.GetInterface1().GetName());
			odb.Close();
		}

		/// <summary>Test persistence of attributes declared by an interface</summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void TestSubClass()
		{
			DeleteBase(Name);
			NeoDatis.Odb.Test.Inheritance.Class1 class1 = new NeoDatis.Odb.Test.Inheritance.SubClassOfClass1
				("olivier", 78);
			NeoDatis.Odb.Test.Inheritance.Class3 class3 = new NeoDatis.Odb.Test.Inheritance.Class3
				(10, class1);
			NeoDatis.Odb.ODB odb = Open(Name);
			odb.Store(class3);
			odb.Close();
			odb = Open(Name);
			Class3 c3 = odb.GetObjects<Class3>().GetFirst();
			AssertEquals(class3.GetNb(), c3.GetNb());
			AssertEquals(class3.GetClass1().GetName(), c3.GetClass1().GetName());
			odb.Close();
		}
	}
}

using NUnit.Framework;
namespace NeoDatis.Odb.Test.Other
{
	public class Student1 : NeoDatis.Odb.Test.Other.Person1
	{
		private string school;

		public Student1() : base(null, null, 0)
		{
			this.school = null;
		}

		public Student1(string address, string name, string school) : base(address, name, 
			0)
		{
			this.school = school;
		}

		public virtual string GetSchool()
		{
			return school;
		}

		public virtual void SetSchool(string school)
		{
			this.school = school;
		}
	}
}

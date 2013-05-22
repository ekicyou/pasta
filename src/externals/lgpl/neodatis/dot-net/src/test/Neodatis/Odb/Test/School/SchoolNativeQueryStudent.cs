using NUnit.Framework;
namespace NeoDatis.Odb.Test.School
{
	[System.Serializable]
	public class SchoolNativeQueryStudent : NeoDatis.Odb.Core.Query.NQ.NativeQuery
	{
		private string name;

		private int age;

		public SchoolNativeQueryStudent(string name, int age)
		{
			this.name = name;
			this.age = age;
		}

		public override bool Match(object @object)
		{
			NeoDatis.Odb.Test.VO.School.Student s = (NeoDatis.Odb.Test.VO.School.Student)@object;
			return s.GetName().Equals(name);
		}

		public override System.Type GetObjectType()
		{
			return typeof(NeoDatis.Odb.Test.VO.School.Student);
		}

		public override System.Type[] GetObjectTypes()
		{
			// TODO Auto-generated method stub
			return null;
		}

		public override string[] GetIndexFields()
		{
			// TODO Auto-generated method stub
			return null;
		}
	}
}

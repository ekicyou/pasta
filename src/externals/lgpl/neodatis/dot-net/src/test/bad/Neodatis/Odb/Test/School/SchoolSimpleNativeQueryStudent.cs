namespace NeoDatis.Odb.Test.School
{
	[System.Serializable]
	public class SchoolSimpleNativeQueryStudent : NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery
	{
		private string name;

		public SchoolSimpleNativeQueryStudent(string name)
		{
			this.name = name;
		}

		public virtual bool Match(NeoDatis.Odb.Test.VO.School.Student @object)
		{
			return @object.GetName().Equals(name);
		}
	}
}

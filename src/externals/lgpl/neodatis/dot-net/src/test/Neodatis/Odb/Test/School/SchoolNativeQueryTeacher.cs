using NUnit.Framework;
namespace NeoDatis.Odb.Test.School
{
	[System.Serializable]
	public class SchoolNativeQueryTeacher : NeoDatis.Odb.Core.Query.NQ.NativeQuery
	{
		private string name;

		public SchoolNativeQueryTeacher(string name)
		{
			this.name = name;
		}

		public override bool Match(object @object)
		{
			NeoDatis.Odb.Test.VO.School.History s = (NeoDatis.Odb.Test.VO.School.History)@object;
			Java.Util.Calendar c = Java.Util.Calendar.GetInstance();
			c.Set(Java.Util.Calendar.Month, 6);
			c.Set(Java.Util.Calendar.Year, 2005);
			return s.GetTeacher().GetName().Equals(name) && s.GetDate().Millisecond > (c.GetTime
				().Millisecond);
		}

		public override System.Type GetObjectType()
		{
			return typeof(NeoDatis.Odb.Test.VO.School.History);
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

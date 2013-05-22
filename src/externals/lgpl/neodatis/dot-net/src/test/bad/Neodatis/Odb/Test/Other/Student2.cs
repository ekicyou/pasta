namespace NeoDatis.Odb.Test.Other
{
	public class Student2 : NeoDatis.Odb.Test.VO.School.Student
	{
		private NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine;

		private bool isModified;

		public Student2() : base(0, null, new System.DateTime(), null, null)
		{
		}
	}
}

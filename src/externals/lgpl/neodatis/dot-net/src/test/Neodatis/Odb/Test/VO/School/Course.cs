using NUnit.Framework;
namespace NeoDatis.Odb.Test.VO.School
{
	public class Course
	{
		private string name;

		private System.Collections.IList listOfDiscipline;

		public Course()
		{
		}

		public Course(string name)
		{
			this.name = name;
			this.listOfDiscipline = new System.Collections.ArrayList();
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public virtual System.Collections.IList GetListOfDiscipline()
		{
			return listOfDiscipline;
		}

		public virtual void SetListOfDiscipline(System.Collections.IList listOfDiscipline
			)
		{
			this.listOfDiscipline = listOfDiscipline;
		}

		public override string ToString()
		{
			return "name=" + name + " | disciplines=" + listOfDiscipline.ToString();
		}
	}
}

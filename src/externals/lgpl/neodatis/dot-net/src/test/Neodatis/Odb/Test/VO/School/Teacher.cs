using NUnit.Framework;
namespace NeoDatis.Odb.Test.VO.School
{
	public class Teacher
	{
		private string name;

		private string speciality;

		public Teacher()
		{
		}

		public Teacher(string name, string speciality) : base()
		{
			this.name = name;
			this.speciality = speciality;
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual string GetSpeciality()
		{
			return speciality;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public virtual void SetSpeciality(string speciality)
		{
			this.speciality = speciality;
		}

		public override string ToString()
		{
			return "name=" + name + " | speciality=" + speciality;
		}
	}
}

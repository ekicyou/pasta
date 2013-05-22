using NUnit.Framework;
namespace NeoDatis.Odb.Test.VO.School
{
	public class History
	{
		private NeoDatis.Odb.Test.VO.School.Discipline discipline;

		private NeoDatis.Odb.Test.VO.School.Teacher teacher;

		private int score;

		private System.DateTime date;

		private NeoDatis.Odb.Test.VO.School.Student student;

		public History()
		{
		}

		public History(System.DateTime data, NeoDatis.Odb.Test.VO.School.Discipline discipline
			, int score, NeoDatis.Odb.Test.VO.School.Teacher teacher)
		{
			this.date = data;
			this.discipline = discipline;
			this.score = score;
			this.teacher = teacher;
		}

		public virtual System.DateTime GetDate()
		{
			return date;
		}

		public virtual NeoDatis.Odb.Test.VO.School.Discipline GetDiscipline()
		{
			return discipline;
		}

		public virtual int GetScore()
		{
			return score;
		}

		public virtual void SetDate(System.DateTime data)
		{
			this.date = data;
		}

		public virtual void SetDiscipline(NeoDatis.Odb.Test.VO.School.Discipline discipline
			)
		{
			this.discipline = discipline;
		}

		public virtual void SetScore(int score)
		{
			this.score = score;
		}

		public virtual NeoDatis.Odb.Test.VO.School.Teacher GetTeacher()
		{
			return teacher;
		}

		public virtual void SetTeacher(NeoDatis.Odb.Test.VO.School.Teacher teacher)
		{
			this.teacher = teacher;
		}

		public override string ToString()
		{
			return "disc.=" + discipline.GetName() + " | teacher=" + teacher.GetName() + " | student="
				 + student.GetName() + " | date=" + date + " | score=" + score;
		}

		public virtual NeoDatis.Odb.Test.VO.School.Student GetStudent()
		{
			return student;
		}

		public virtual void SetStudent(NeoDatis.Odb.Test.VO.School.Student student)
		{
			this.student = student;
		}
	}
}

using NUnit.Framework;
namespace NeoDatis.Odb.Test.VO.School
{
	public class Student
	{
		private string id;

		private string name;

		private int age;

		private NeoDatis.Odb.Test.VO.School.Course course;

		private System.DateTime firstDate;

		private System.Collections.IList listHistory;

		public Student(int age, NeoDatis.Odb.Test.VO.School.Course course, System.DateTime
			 date, string id, string name)
		{
			this.age = age;
			this.course = course;
			firstDate = date;
			this.id = id;
			this.name = name;
			listHistory = new System.Collections.ArrayList();
		}

		public virtual int GetAge()
		{
			return age;
		}

		public virtual NeoDatis.Odb.Test.VO.School.Course GetCourse()
		{
			return course;
		}

		public virtual System.DateTime GetFirstDate()
		{
			return firstDate;
		}

		public virtual string GetId()
		{
			return id;
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetAge(int age)
		{
			this.age = age;
		}

		public virtual void SetCourse(NeoDatis.Odb.Test.VO.School.Course course)
		{
			this.course = course;
		}

		public virtual void SetFirstDate(System.DateTime firstDate)
		{
			this.firstDate = firstDate;
		}

		public virtual void SetId(string id)
		{
			this.id = id;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public virtual System.Collections.IList GetListHistory()
		{
			return listHistory;
		}

		public virtual void SetListHistory(System.Collections.IList listHistory)
		{
			this.listHistory = listHistory;
		}

		public virtual void AddHistory(NeoDatis.Odb.Test.VO.School.History history)
		{
			history.SetStudent(this);
			listHistory.Add(history);
		}

		public override string ToString()
		{
			return "id=" + id + " | name=" + name + " | age= " + age + " | date=" + firstDate
				 + " | course=" + course.GetName() + " | history=" + listHistory.ToString();
		}
	}
}

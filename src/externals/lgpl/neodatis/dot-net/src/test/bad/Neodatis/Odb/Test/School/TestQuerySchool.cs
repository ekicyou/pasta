using NeoDatis.Odb.Test.VO.School;
namespace NeoDatis.Odb.Test.School
{
	public class TestQuerySchool : NeoDatis.Odb.Test.ODBTest
	{
		// possiveis consultas
		// Listar todos os alunos de determinado professor
		// Listar alunos com nota abaixo de x
		// Listar disciplinas que um professor ministrou no semestre
		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			if (!isLocal)
			{
				return;
			}
			NeoDatis.Odb.ODB odb = Open("t-school.neodatis");
			// List students by name
			SchoolNativeQueryStudent natQuery = new SchoolNativeQueryStudent("Brenna", 23);
			NeoDatis.Odb.Objects<Student> students = odb.GetObjects<Student>(natQuery);
			SchoolSimpleNativeQueryStudent sNatQuery = new SchoolSimpleNativeQueryStudent("Brenna");
			students = odb.GetObjects<Student>(sNatQuery);
			// list disciplines of one teacher by semester
			SchoolNativeQueryTeacher natQuery2 = new SchoolNativeQueryTeacher
				("Jeremias");
			NeoDatis.Odb.Objects<History> historys = odb.GetObjects<History>(natQuery2);
			System.Collections.Hashtable listDiscipline = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string,Discipline>();
			for (System.Collections.IEnumerator iter = historys.GetEnumerator(); iter.MoveNext
				(); )
			{
				History h = (History)iter
					.Current;
				listDiscipline.Add(h.GetDiscipline().GetName(), h.GetDiscipline());
			}
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test12()
		{
			NeoDatis.Odb.ODB odb = null;
			try
			{
				odb = Open("t-school.neodatis");
				NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy
					.GetEngine(odb).GetSession(true).GetMetaModel().GetClassInfo(typeof(Student
					).FullName, true);
				AssertFalse(ci.HasCyclicReference());
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
			}
		}

		/// <exception cref="System.Exception"></exception>
		public override void SetUp()
		{
			base.SetUp();
			DeleteBase("t-school.neodatis");
			NeoDatis.Odb.ODB odb = Open("t-school.neodatis");
			NeoDatis.Odb.Objects<Student> students = odb.GetObjects<Student>(true);
			int numStudents = students.Count;
			Course computerScience = new Course
				("Computer Science");
			Teacher teacher = new Teacher
				("Jeremias", "Java");
			Discipline dw1 = new Discipline
				("Des. Web 1", 3);
			Discipline @is = new Discipline
				("Intranet/Seguran√ßa", 4);
			Student std1 = new Student
				(20, computerScience, new System.DateTime(), "1cs", "Brenna");
			History h1 = new History(
				new System.DateTime(), dw1, 0, teacher);
			History h2 = new History(
				new System.DateTime(), @is, 0, teacher);
			std1.AddHistory(h1);
			std1.AddHistory(h2);
			odb.Store(std1);
			odb.Commit();
			odb.Close();
			odb = Open("t-school.neodatis");
			students = odb.GetObjects<Student>(true);
			odb.Close();
			AssertEquals(numStudents + 1, students.Count);
		}

		/// <exception cref="System.Exception"></exception>
		public override void TearDown()
		{
			DeleteBase("t-school.neodatis");
		}
	}
}

namespace NeoDatis.Odb.Test.Explorer
{
	public class CreateDataToTestOdbExplorer
	{
		/// <summary>
		/// bug found by Julio Jimenez Borreguero When there exist an index on a
		/// numeric field, the criteria query is constructed with a value of type
		/// String instead of numeric
		/// </summary>
		public virtual void Test1()
		{
			NeoDatis.Tool.IOUtil.DeleteFile("base1.neodatis");
			NeoDatis.Odb.ODB odb = NeoDatis.Odb.ODBFactory.Open("base1.neodatis");
			string[] fields = new string[] { "int1" };
			odb.GetClassRepresentation(typeof(NeoDatis.Odb.Test.VO.Attribute.TestClass)).AddUniqueIndexOn
				("index1", fields, true);
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			int size = 50;
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Test.VO.Attribute.TestClass testClass = new NeoDatis.Odb.Test.VO.Attribute.TestClass
					();
				testClass.SetBigDecimal1(new System.Decimal(i));
				testClass.SetBoolean1(i % 3 == 0);
				testClass.SetChar1((char)(i % 5));
				testClass.SetDate1(new System.DateTime(start + i));
				testClass.SetDouble1(((double)(i % 10)) / size);
				testClass.SetInt1(size - i);
				testClass.SetString1("test class " + i);
				odb.Store(testClass);
			}
			// println(testClass.getDouble1() + " | " + testClass.getString1() +
			// " | " + testClass.getInt1());
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public static void Main2(string[] args)
		{
			new NeoDatis.Odb.Test.Explorer.CreateDataToTestOdbExplorer().Test1();
		}
	}
}

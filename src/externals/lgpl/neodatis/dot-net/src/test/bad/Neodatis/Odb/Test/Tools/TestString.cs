namespace NeoDatis.Odb.Test.Tools
{
	/// <author>olivier</author>
	public class TestString : NeoDatis.Odb.Test.ODBTest
	{
		public virtual void Test1()
		{
			string s = "ola $1 ola $2";
			s = NeoDatis.Tool.Wrappers.OdbString.ReplaceToken(s, "$1", "param1");
			AssertEquals("ola param1 ola $2", s);
		}

		public virtual void Test2()
		{
			string s = "ola $1 ola $2";
			s = NeoDatis.Tool.Wrappers.OdbString.ReplaceToken(s, "$1", "param1");
			s = NeoDatis.Tool.Wrappers.OdbString.ReplaceToken(s, "$2", "param2");
			AssertEquals("ola param1 ola param2", s);
		}

		public virtual void Test3()
		{
			string s = "ola $1 ola $2";
			s = NeoDatis.Tool.Wrappers.OdbString.ReplaceToken(s, "$", "param");
			AssertEquals("ola param1 ola param2", s);
		}

		public virtual void Test4()
		{
			string s = "ola $1 ola $2";
			s = NeoDatis.Tool.Wrappers.OdbString.ReplaceToken(s, "$", "param", 1);
			AssertEquals("ola param1 ola $2", s);
		}

		public virtual void Test5()
		{
			string s = "ola $1 ola $2";
			s = NeoDatis.Tool.Wrappers.OdbString.ReplaceToken(s, "$$", "param1");
			AssertEquals("ola $1 ola $2", s);
		}

		public virtual void Test6()
		{
			string s = "ola $1 ola $2 ola $3 ola $4";
			s = NeoDatis.Tool.Wrappers.OdbString.ReplaceToken(s, "$", "param", 2);
			AssertEquals("ola param1 ola param2 ola $3 ola $4", s);
		}

		public virtual void Test7()
		{
			int size = 100;
			System.Text.StringBuilder b = new System.Text.StringBuilder();
			for (int i = 0; i < size; i++)
			{
				b.Append("text").Append(i).Append(" ");
			}
			string s = NeoDatis.Tool.Wrappers.OdbString.ReplaceToken(b.ToString(), "text", string.Empty
				);
			// Check that there is no more "text"in the string
			AssertTrue(s.IndexOf("text") == -1);
		}

		public virtual void Test8subString()
		{
			string s = "NeoDatis ODB - The open source object database";
			for (int i = 0; i < 10; i++)
			{
				string s1 = Sharpen.Runtime.Substring(s, i, i + 15);
				string s2 = NeoDatis.Tool.Wrappers.OdbString.Substring(s, i, i + 15);
				AssertEquals(s1, s2);
			}
		}

		public virtual void Test9subString()
		{
			string s = "NeoDatis ODB - The open source object database";
			string s1 = Sharpen.Runtime.Substring(s, 0, s.Length);
			string s2 = NeoDatis.Tool.Wrappers.OdbString.Substring(s, 0, s.Length);
			AssertEquals(s1, s2);
		}
	}
}

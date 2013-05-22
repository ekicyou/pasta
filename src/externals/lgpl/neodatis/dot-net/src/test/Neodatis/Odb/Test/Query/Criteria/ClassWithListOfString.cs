using NUnit.Framework;
namespace NeoDatis.Odb.Test.Query.Criteria
{
	/// <author>olivier</author>
	public class ClassWithListOfString
	{
		private string name;

		private System.Collections.Generic.IList<string> strings;

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public virtual System.Collections.Generic.IList<string> GetStrings()
		{
			return strings;
		}

		public virtual void SetStrings(System.Collections.Generic.IList<string> strings)
		{
			this.strings = strings;
		}

		public ClassWithListOfString(string name, System.Collections.Generic.IList<string
			> strings) : base()
		{
			this.name = name;
			this.strings = strings;
		}
	}
}

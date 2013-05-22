using NUnit.Framework;
namespace NeoDatis.Odb.Test.Arraycollectionmap
{
	public class ObjectWithListOfInteger
	{
		private string name;

		private System.Collections.IList listOfIntegers;

		public ObjectWithListOfInteger(string name)
		{
			this.name = name;
			listOfIntegers = new System.Collections.ArrayList();
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public virtual System.Collections.IList GetListOfIntegers()
		{
			return listOfIntegers;
		}

		public virtual void SetListOfIntegers(System.Collections.IList listOfIntegers)
		{
			this.listOfIntegers = listOfIntegers;
		}
	}
}

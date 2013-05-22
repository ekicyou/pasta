using NUnit.Framework;
namespace NeoDatis.Odb.Test.Arraycollectionmap
{
	[System.Serializable]
	public class MyList : System.Collections.ArrayList
	{
		public virtual object MyGet(int i)
		{
			return this[i];
		}
	}
}

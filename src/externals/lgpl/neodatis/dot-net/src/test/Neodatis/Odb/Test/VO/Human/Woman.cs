using NUnit.Framework;
namespace NeoDatis.Odb.Test.VO.Human
{
	public class Woman : NeoDatis.Odb.Test.VO.Human.Human
	{
		public Woman(string name) : base("woman", "F", name)
		{
		}
	}
}

namespace NeoDatis.Odb.Impl.Core.Query.Values
{
	public class ValuesUtil
	{
		public static System.Decimal Convert(System.Decimal number)
		{
			System.Decimal bd = NeoDatis.Tool.Wrappers.NeoDatisNumber.CreateDecimalFromString
				(number.ToString());
			return bd;
		}
	}
}

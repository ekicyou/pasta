namespace NeoDatis.Odb.Tool
{
	/// <summary>
	/// An utility class to build a string description from a list of
	/// ObjectInfoHeader
	/// </summary>
	/// <author>osmadja</author>
	public class ObjectInfoHeaderListDisplay
	{
		public static string Build(System.Collections.IList objectInfoHeaderList, bool withDetail
			)
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih = null;
			buffer.Append(objectInfoHeaderList.Count).Append(" objects : ");
			for (int i = 0; i < objectInfoHeaderList.Count; i++)
			{
				oih = (NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader)objectInfoHeaderList
					[i];
				if (withDetail)
				{
					buffer.Append("(").Append(oih.GetPreviousObjectOID()).Append(" <= ").Append(oih.GetOid
						()).Append(" => ").Append(oih.GetNextObjectOID()).Append(") ");
				}
				else
				{
					buffer.Append(oih.GetOid()).Append(" ");
				}
			}
			return buffer.ToString();
		}
	}
}

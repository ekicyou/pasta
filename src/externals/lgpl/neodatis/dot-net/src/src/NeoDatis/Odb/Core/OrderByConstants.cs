namespace NeoDatis.Odb.Core
{
	/// <summary>
	/// Some constants used for ordering queries and creating ordered collection
	/// iterators
	/// </summary>
	/// <author>osmadja</author>
	[System.Serializable]
	public class OrderByConstants
	{
		private const int OrderByNoneType = 0;

		private const int OrderByDescType = 1;

		private const int OrderByAscType = 2;

		public static readonly NeoDatis.Odb.Core.OrderByConstants OrderByNone = new NeoDatis.Odb.Core.OrderByConstants
			(OrderByNoneType);

		public static readonly NeoDatis.Odb.Core.OrderByConstants OrderByDesc = new NeoDatis.Odb.Core.OrderByConstants
			(OrderByDescType);

		public static readonly NeoDatis.Odb.Core.OrderByConstants OrderByAsc = new NeoDatis.Odb.Core.OrderByConstants
			(OrderByAscType);

		private int type;

		private OrderByConstants(int type)
		{
			this.type = type;
		}

		public virtual bool IsOrderByDesc()
		{
			return type == OrderByDescType;
		}

		public virtual bool IsOrderByAsc()
		{
			return type == OrderByAscType;
		}

		public virtual bool IsOrderByNone()
		{
			return type == OrderByNoneType;
		}

		public override string ToString()
		{
			switch (type)
			{
				case OrderByAscType:
				{
					return "order by asc";
				}

				case OrderByDescType:
				{
					return "order by desc";
				}

				case OrderByNoneType:
				{
					return "no order by";
				}
			}
			return "?";
		}
	}
}

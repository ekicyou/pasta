namespace NeoDatis.Odb.Core.Query.Criteria
{
	[System.Serializable]
	public class Operator
	{
		private string name;

		public static readonly NeoDatis.Odb.Core.Query.Criteria.Operator Equal = new NeoDatis.Odb.Core.Query.Criteria.Operator
			("=");

		public static readonly NeoDatis.Odb.Core.Query.Criteria.Operator Contain = new NeoDatis.Odb.Core.Query.Criteria.Operator
			("contain");

		public static readonly NeoDatis.Odb.Core.Query.Criteria.Operator Like = new NeoDatis.Odb.Core.Query.Criteria.Operator
			("like");

		public static readonly NeoDatis.Odb.Core.Query.Criteria.Operator CaseInsentiveLike
			 = new NeoDatis.Odb.Core.Query.Criteria.Operator("ilike");

		public static readonly NeoDatis.Odb.Core.Query.Criteria.Operator GreaterThan = new 
			NeoDatis.Odb.Core.Query.Criteria.Operator(">");

		public static readonly NeoDatis.Odb.Core.Query.Criteria.Operator GreaterOrEqual = 
			new NeoDatis.Odb.Core.Query.Criteria.Operator(">=");

		public static readonly NeoDatis.Odb.Core.Query.Criteria.Operator LessThan = new NeoDatis.Odb.Core.Query.Criteria.Operator
			("<");

		public static readonly NeoDatis.Odb.Core.Query.Criteria.Operator LessOrEqual = new 
			NeoDatis.Odb.Core.Query.Criteria.Operator("<=");

		protected Operator(string name)
		{
			this.name = name;
		}

		public virtual string GetName()
		{
			return name;
		}

		public override string ToString()
		{
			return name;
		}

		public override bool Equals(object obj)
		{
			if (!(obj is NeoDatis.Odb.Core.Query.Criteria.Operator) || obj == null)
			{
				return false;
			}
			NeoDatis.Odb.Core.Query.Criteria.Operator @operator = (NeoDatis.Odb.Core.Query.Criteria.Operator
				)obj;
			return name.Equals(@operator.name);
		}
	}
}

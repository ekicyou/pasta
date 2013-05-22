namespace NeoDatis.Odb.Core.Query.Criteria
{
	[System.Serializable]
	public class Or : NeoDatis.Odb.Core.Query.Criteria.ComposedExpression
	{
		public Or() : base()
		{
		}

		public override bool Match(object @object)
		{
			System.Collections.IEnumerator iterator = criteria.GetEnumerator();
			NeoDatis.Odb.Core.Query.Criteria.ICriterion criterion = null;
			while (iterator.MoveNext())
			{
				criterion = (NeoDatis.Odb.Core.Query.Criteria.ICriterion)iterator.Current;
				// For OR Expression, if one is true, then the whole expression
				// will be true
				if (criterion.Match(@object))
				{
					return true;
				}
			}
			return false;
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			System.Collections.IEnumerator iterator = criteria.GetEnumerator();
			NeoDatis.Odb.Core.Query.Criteria.ICriterion criterion = null;
			buffer.Append("(");
			bool isFirst = true;
			while (iterator.MoveNext())
			{
				criterion = (NeoDatis.Odb.Core.Query.Criteria.ICriterion)iterator.Current;
				if (isFirst)
				{
					buffer.Append(criterion.ToString());
					isFirst = false;
				}
				else
				{
					buffer.Append(" or ").Append(criterion.ToString());
				}
			}
			buffer.Append(")");
			return buffer.ToString();
		}
	}
}

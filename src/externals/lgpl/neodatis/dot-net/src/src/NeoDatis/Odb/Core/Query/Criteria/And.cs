namespace NeoDatis.Odb.Core.Query.Criteria
{
	[System.Serializable]
	public class And : NeoDatis.Odb.Core.Query.Criteria.ComposedExpression
	{
		public And()
		{
		}

		public override bool Match(object @object)
		{
			System.Collections.IEnumerator iterator = criteria.GetEnumerator();
			NeoDatis.Odb.Core.Query.Criteria.ICriterion criterion = null;
			while (iterator.MoveNext())
			{
				criterion = (NeoDatis.Odb.Core.Query.Criteria.ICriterion)iterator.Current;
				// For AND Expression, if one is false, then the whole
				// expression will be false
				if (!criterion.Match(@object))
				{
					return false;
				}
			}
			return true;
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
					buffer.Append(" and ").Append(criterion.ToString());
				}
			}
			buffer.Append(")");
			return buffer.ToString();
		}

		public override bool CanUseIndex()
		{
			System.Collections.IEnumerator iterator = criteria.GetEnumerator();
			NeoDatis.Odb.Core.Query.Criteria.ICriterion criterion = null;
			while (iterator.MoveNext())
			{
				criterion = (NeoDatis.Odb.Core.Query.Criteria.ICriterion)iterator.Current;
				if (!criterion.CanUseIndex())
				{
					return false;
				}
			}
			return true;
		}
	}
}

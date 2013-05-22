namespace NeoDatis.Odb.Core.Query.Criteria
{
	[System.Serializable]
	public class Not : NeoDatis.Odb.Core.Query.Criteria.AbstractExpression
	{
		private NeoDatis.Odb.Core.Query.Criteria.ICriterion criterion;

		public Not(NeoDatis.Odb.Core.Query.Criteria.ICriterion criterion)
		{
			this.criterion = criterion;
		}

		public override bool Match(object @object)
		{
			return !criterion.Match(@object);
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append(" not ").Append(criterion);
			return buffer.ToString();
		}

		public override NeoDatis.Tool.Wrappers.List.IOdbList<string> GetAllInvolvedFields
			()
		{
			return criterion.GetAllInvolvedFields();
		}

		public override NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap GetValues
			()
		{
			return new NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap();
		}

		public override void Ready()
		{
		}
	}
}

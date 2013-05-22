namespace NeoDatis.Odb.Impl.Core.Query.Criteria
{
	[System.Serializable]
	public class IsNotNullCriterion : NeoDatis.Odb.Core.Query.Criteria.AbstractCriterion
	{
		public IsNotNullCriterion(string attributeName) : base(attributeName)
		{
		}

		public override bool Match(object valueToMatch)
		{
			// If it is a AttributeValuesMap, then gets the real value from the map
			if (valueToMatch is NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap)
			{
				NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap attributeValues = (NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap
					)valueToMatch;
				valueToMatch = attributeValues[attributeName];
			}
			return valueToMatch != null;
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
